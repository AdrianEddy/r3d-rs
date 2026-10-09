// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2026 Adrian <adrian.eddy at gmail>

//! Hardware test for `gpu::GpuDecodePipeline`: decodes real frames on the
//! CUDA and OpenCL lanes and checks each against the CPU decode of the same
//! frame.
//!
//! It needs a GPU, the SDK redistributables and a clip, so it is ignored by
//! default:
//!
//! ```text
//! R3D_TEST_CLIP=/path/to/clip.R3D cargo test --test gpu_pipeline_hw -- --ignored
//! ```
//!
//! The redistributable folder is `$R3D_TEST_REDIST_DIR`, or else
//! `$R3DSDK_DIR/Redistributable/<os>` (with `R3DSDK_DIR` defaulting to `sdk`
//! in the crate root, as in `build.rs`). The comparison assumes the GPU
//! output is tightly packed, which holds when the scaled width is a
//! multiple of 16.
#![cfg(not(target_os = "macos"))]

use core::ffi::c_void;
use std::future::Future;
use r3d_rs::gpu::{GpuDecodePipeline, GpuLane, GpuOutput};
use r3d_rs::*;

const MODE: VideoDecodeMode = VideoDecodeMode::HalfResGood;
const PIXEL: VideoPixelType = VideoPixelType::Rgb16bitInterleaved;

#[test]
#[ignore = "needs a GPU, the SDK redistributables and R3D_TEST_CLIP"]
fn gpu_pipeline_matches_cpu_decode() {
    let Ok(clip_path) = std::env::var("R3D_TEST_CLIP") else {
        panic!("set R3D_TEST_CLIP to an R3D clip to run this test");
    };
    let _sdk = Sdk::initialize(&redist_dir(), InitializeFlags::Cuda | InitializeFlags::OpenCL)
        .expect("R3D SDK init");
    let clip = Clip::from_path(&clip_path).expect("open R3D_TEST_CLIP");
    assert!(clip.video_frame_count() >= 8, "test clip needs at least 8 frames");
    let settings = clip.default_image_processing_settings();
    let cpu = |frame: usize, mode: VideoDecodeMode| -> Vec<u16> {
        let buf = clip.decode_video_frame(frame, mode, PIXEL, Some(&settings), None, None)
            .expect("CPU decode").expect("CPU decode buffer");
        buf.as_slice::<u16>().to_vec()
    };

    let cuda = Cuda::load();
    cuda.set_device(0);

    // AsyncDecoder route — the SDK leaves the upload to `raw_device_mem` to
    // the caller, and skipping it yields a black frame.
    {
        let pipe = cuda.pipeline(false);
        let out = cuda.alloc(&pipe, &clip, MODE);
        pipe.decode_to_gpu_blocking(&clip, 3, MODE, PIXEL, &settings, out.output()).unwrap();
        assert_matches_cpu("CUDA AsyncDecoder", &cuda.read(&out), &cpu(3, MODE));

        // A smaller mode on the same pipeline, then back: the pools hand out
        // differently sized buffers.
        let small = VideoDecodeMode::SixteenthResGood;
        let out_small = cuda.alloc(&pipe, &clip, small);
        pipe.decode_to_gpu_blocking(&clip, 4, small, PIXEL, &settings, out_small.output()).unwrap();
        assert_matches_cpu("CUDA AsyncDecoder 1/16", &cuda.read(&out_small), &cpu(4, small));
        pipe.decode_to_gpu_blocking(&clip, 5, MODE, PIXEL, &settings, out.output()).unwrap();
        assert_matches_cpu("CUDA AsyncDecoder after mode switch", &cuda.read(&out), &cpu(5, MODE));
    }

    // GpuDecoder enabled: whichever route the clip takes must decode correctly.
    {
        let pipe = cuda.pipeline(true);
        eprintln!("GpuDecoder route taken for this clip: {}", pipe.will_use_gpu_decoder(&clip));
        let out = cuda.alloc(&pipe, &clip, MODE);
        pipe.decode_to_gpu_blocking(&clip, 2, MODE, PIXEL, &settings, out.output()).unwrap();
        assert_matches_cpu("CUDA with GpuDecoder enabled", &cuda.read(&out), &cpu(2, MODE));
    }

    // Concurrent blocking decodes from several threads share the lane stream.
    {
        let pipe = cuda.pipeline(false);
        let frames = [0, 1, 6, 7];
        let outs: Vec<_> = frames.iter().map(|_| cuda.alloc(&pipe, &clip, MODE)).collect();
        std::thread::scope(|s| {
            for (frame, out) in frames.iter().zip(&outs) {
                let (pipe, clip, settings, cuda) = (&pipe, &clip, &settings, &cuda);
                s.spawn(move || {
                    cuda.set_device(0);
                    for _ in 0..3 {
                        pipe.decode_to_gpu_blocking(clip, *frame, MODE, PIXEL, settings, out.output()).unwrap();
                    }
                });
            }
        });
        for (frame, out) in frames.iter().zip(&outs) {
            assert_matches_cpu(&format!("CUDA concurrent frame {frame}"), &cuda.read(out), &cpu(*frame, MODE));
        }
    }

    // Split stages: overlapped async debayers on distinct stream slots.
    {
        let pipe = cuda.pipeline(false);
        let frames = [1, 2, 3];
        let decoded: Vec<_> = frames.iter()
            .map(|&f| pipe.submit_decompress(&clip, f, MODE).unwrap())
            .collect();
        let decoded = pollster::block_on(futures_util::future::try_join_all(decoded)).unwrap();
        let outs: Vec<_> = frames.iter().map(|_| cuda.alloc(&pipe, &clip, MODE)).collect();
        let pending: Vec<_> = decoded.into_iter().zip(&outs).enumerate()
            .map(|(slot, (frame, out))| pipe.submit_debayer(frame, MODE, PIXEL, &settings, out.output(), slot).unwrap())
            .collect();

        // A slot whose debayer is still in flight is refused.
        let busy = pollster::block_on(pipe.submit_decompress(&clip, 4, MODE).unwrap()).unwrap();
        let spare = cuda.alloc(&pipe, &clip, MODE);
        assert!(pipe.submit_debayer(busy, MODE, PIXEL, &settings, spare.output(), 0).is_err());

        for p in pending {
            pipe.complete_debayer(p).unwrap();
        }
        for (frame, out) in frames.iter().zip(&outs) {
            assert_matches_cpu(&format!("CUDA submit_debayer frame {frame}"), &cuda.read(out), &cpu(*frame, MODE));
        }

        // The slot is free again once its debayer completed.
        let again = pollster::block_on(pipe.submit_decompress(&clip, 4, MODE).unwrap()).unwrap();
        pipe.complete_debayer(pipe.submit_debayer(again, MODE, PIXEL, &settings, spare.output(), 0).unwrap()).unwrap();
        assert_matches_cpu("CUDA reused slot", &cuda.read(&spare), &cpu(4, MODE));
    }

    // A PendingDebayer outliving its pipeline keeps the lane alive.
    {
        let pipe = cuda.pipeline(false);
        let out = cuda.alloc(&pipe, &clip, MODE);
        let frame = pollster::block_on(pipe.submit_decompress(&clip, 6, MODE).unwrap()).unwrap();
        let pending = pipe.submit_debayer(frame, MODE, PIXEL, &settings, out.output(), 0).unwrap();
        drop(pipe);
        drop(pending);
        assert_matches_cpu("CUDA PendingDebayer after pipeline drop", &cuda.read(&out), &cpu(6, MODE));
    }

    // Decodes abandoned mid-flight: the SDK keeps writing into the scratch
    // buffers, which must stay alive until its callbacks.
    {
        let pipe = cuda.pipeline(false);
        for f in 0..8 {
            drop(pipe.submit_decompress(&clip, f, MODE).unwrap());
        }
        let out = cuda.alloc(&pipe, &clip, MODE);
        let mut fut = Box::pin(pipe.decode_to_gpu(&clip, 7, MODE, PIXEL, &settings, out.output()));
        let mut cx = std::task::Context::from_waker(futures_util::task::noop_waker_ref());
        let _ = fut.as_mut().poll(&mut cx);
        drop(fut);
        // Still usable afterwards.
        pipe.decode_to_gpu_blocking(&clip, 7, MODE, PIXEL, &settings, out.output()).unwrap();
        assert_matches_cpu("CUDA after abandoned decodes", &cuda.read(&out), &cpu(7, MODE));
    }

    // OpenCL lane: uploads only the bytes the decompressor wrote.
    if let Some(ocl) = OpenCl::create() {
        let pipe = ocl.pipeline();
        for (frame, mode) in [(1, MODE), (2, VideoDecodeMode::SixteenthResGood), (3, MODE)] {
            let out = ocl.alloc(&pipe, &clip, mode);
            pipe.decode_to_gpu_blocking(&clip, frame, mode, PIXEL, &settings, out.output()).unwrap();
            assert_matches_cpu(&format!("OpenCL frame {frame} {mode:?}"), &ocl.read(&out), &cpu(frame, mode));
        }
    } else {
        eprintln!("No OpenCL GPU found; skipping the OpenCL lane");
    }
}

fn redist_dir() -> String {
    if let Ok(dir) = std::env::var("R3D_TEST_REDIST_DIR") {
        return dir;
    }
    let sdk = std::env::var("R3DSDK_DIR")
        .unwrap_or_else(|_| format!("{}/sdk", env!("CARGO_MANIFEST_DIR")));
    let os = if cfg!(windows) { "win" } else { "linux" };
    format!("{sdk}/Redistributable/{os}")
}

/// The GPU frame must be the CPU frame up to the GPU kernels' own
/// differences. Measured against the CPU decode: an N-RAW clip matches to ±1
/// on CUDA and within 0.2% outliers on OpenCL; RED clips reach a mean of
/// ~23 with ~0.3% outliers on every GPU path, and GPU decompression
/// (`GpuDecoder`) up to ~0.6%. A wrong, stale or black frame is far off:
/// neighbouring frames of those clips differ by a mean of 190–1160, in
/// 72–91% of samples.
fn assert_matches_cpu(label: &str, gpu: &[u16], cpu: &[u16]) {
    assert!(gpu.len() >= cpu.len(), "{label}: GPU output smaller than the CPU frame");
    let gpu = &gpu[..cpu.len()];
    let (mut sum, mut outliers) = (0u64, 0usize);
    for (g, c) in gpu.iter().zip(cpu) {
        let d = (*g as i32 - *c as i32).unsigned_abs();
        sum += d as u64;
        outliers += (d > 64) as usize;
    }
    let mean = sum as f64 / cpu.len() as f64;
    let outlier_pct = outliers as f64 * 100.0 / cpu.len() as f64;
    assert!(gpu.iter().any(|&v| v != 0), "{label}: GPU frame is all zeros");
    assert!(mean < 50.0 && outlier_pct < 2.0,
        "{label}: GPU frame differs from CPU (mean {mean:.2}, {outlier_pct:.2}% of samples off by > 64)");
}

/// A device output buffer plus its size.
struct DeviceBuf { ptr: *mut c_void, size: usize }
// SAFETY: an opaque device pointer; the test only hands it to the pipeline.
unsafe impl Sync for DeviceBuf {}
impl DeviceBuf {
    fn output(&self) -> GpuOutput<'static> {
        GpuOutput::DeviceMem { ptr: self.ptr, size: self.size }
    }
}

struct Cuda {
    _lib:      libloading::Library,
    memcpy:    unsafe extern "C" fn(*mut c_void, *const c_void, usize, i32) -> i32,
    set_dev:   unsafe extern "C" fn(i32) -> i32,
    streams:   std::sync::Mutex<Vec<usize>>,
    allocs:    std::sync::Mutex<Vec<usize>>,
}

impl Cuda {
    fn load() -> Self {
        let names: &[&str] = if cfg!(windows) {
            &["cudart64_12.dll", "cudart64_110.dll"]
        } else {
            &["libcudart.so", "/usr/local/cuda/lib64/libcudart.so"]
        };
        let lib = names.iter()
            .find_map(|&n| unsafe { libloading::Library::new(n) }.ok())
            .expect("CUDA runtime library");
        let memcpy = unsafe { *lib.get(b"cudaMemcpy").unwrap() };
        let set_dev = unsafe { *lib.get(b"cudaSetDevice").unwrap() };
        Self { _lib: lib, memcpy, set_dev, streams: Default::default(), allocs: Default::default() }
    }

    fn set_device(&self, id: i32) {
        assert_eq!(unsafe { (self.set_dev)(id) }, 0, "cudaSetDevice");
    }

    fn pipeline(&self, gpu_decompress: bool) -> GpuDecodePipeline {
        let cudart = CUDA.as_ref().expect("cudart");
        let mut stream: *mut c_void = core::ptr::null_mut();
        assert_eq!(unsafe { (cudart.cudaStreamCreate)(&mut stream) }, 0);
        self.streams.lock().unwrap().push(stream as usize);
        let red = RedCuda::new().expect("RedCuda");
        red.check_compatibility(0, stream).expect("REDCuda compatibility");
        GpuDecodePipeline::new(GpuLane::Cuda { red, stream, device_id: 0 }, 0, gpu_decompress).unwrap()
    }

    fn alloc(&self, pipe: &GpuDecodePipeline, clip: &Clip, mode: VideoDecodeMode) -> DeviceBuf {
        let size = pipe.result_frame_size(clip, mode, PIXEL).unwrap();
        let mut ptr: *mut c_void = core::ptr::null_mut();
        assert_eq!(unsafe { (CUDA.as_ref().unwrap().cudaMalloc)(&mut ptr, size) }, 0);
        self.allocs.lock().unwrap().push(ptr as usize);
        DeviceBuf { ptr, size }
    }

    fn read(&self, buf: &DeviceBuf) -> Vec<u16> {
        let mut host = vec![0u16; buf.size / 2];
        // cudaMemcpyDeviceToHost = 2
        assert_eq!(unsafe { (self.memcpy)(host.as_mut_ptr().cast(), buf.ptr, buf.size, 2) }, 0);
        host
    }
}

impl Drop for Cuda {
    fn drop(&mut self) {
        let cudart = CUDA.as_ref().unwrap();
        for p in self.allocs.lock().unwrap().drain(..) {
            unsafe { (cudart.cudaFree)(p as *mut c_void) };
        }
        for s in self.streams.lock().unwrap().drain(..) {
            unsafe { (cudart.cudaStreamDestroy)(s as *mut c_void) };
        }
    }
}

struct OpenCl {
    ctx:    cl_context,
    queue:  cl_command_queue,
    allocs: std::cell::RefCell<Vec<cl_mem>>,
}

impl OpenCl {
    /// A context and queue on the first OpenCL GPU, if any.
    fn create() -> Option<Self> {
        let ocl = OPENCL.as_ref().ok()?;
        let mut platforms = [core::ptr::null_mut(); 16];
        let mut n = 0u32;
        unsafe { (ocl.clGetPlatformIDs)(16, platforms.as_mut_ptr(), &mut n) };
        let device = platforms[..n as usize].iter().find_map(|&p| {
            let mut device = core::ptr::null_mut();
            let mut found = 0u32;
            // CL_DEVICE_TYPE_GPU = 1 << 2
            let rc = unsafe { (ocl.clGetDeviceIDs)(p, 1 << 2, 1, &mut device, &mut found) };
            (rc == 0 && found > 0).then_some(device)
        })?;
        let mut err = 0;
        let ctx = unsafe { (ocl.clCreateContext)(core::ptr::null(), 1, &device, None, core::ptr::null_mut(), &mut err) };
        assert_eq!(err, 0, "clCreateContext");
        let queue = unsafe { (ocl.clCreateCommandQueue)(ctx, device, 0, &mut err) };
        assert_eq!(err, 0, "clCreateCommandQueue");
        Some(Self { ctx, queue, allocs: Default::default() })
    }

    fn pipeline(&self) -> GpuDecodePipeline {
        let cache = std::env::temp_dir().join("r3d-rs-opencl-kernels");
        std::fs::create_dir_all(&cache).unwrap();
        let red = RedOpenCl::new(cache.to_str().unwrap()).expect("RedOpenCl");
        red.check_compatibility(self.ctx, self.queue).expect("REDCL compatibility");
        GpuDecodePipeline::new(GpuLane::OpenCl { red, ctx: self.ctx, queue: self.queue }, 0, false).unwrap()
    }

    fn alloc(&self, pipe: &GpuDecodePipeline, clip: &Clip, mode: VideoDecodeMode) -> DeviceBuf {
        let ocl = OPENCL.as_ref().unwrap();
        let size = pipe.result_frame_size(clip, mode, PIXEL).unwrap();
        let mut err = 0;
        // CL_MEM_READ_WRITE = 1
        let mem = unsafe { (ocl.clCreateBuffer)(self.ctx, 1, size, core::ptr::null_mut(), &mut err) };
        assert_eq!(err, 0, "clCreateBuffer");
        self.allocs.borrow_mut().push(mem);
        DeviceBuf { ptr: mem, size }
    }

    fn read(&self, buf: &DeviceBuf) -> Vec<u16> {
        let ocl = OPENCL.as_ref().unwrap();
        let mut host = vec![0u16; buf.size / 2];
        let rc = unsafe {
            (ocl.clEnqueueReadBuffer)(self.queue, buf.ptr, 1, 0, buf.size, host.as_mut_ptr().cast(),
                0, core::ptr::null(), core::ptr::null_mut())
        };
        assert_eq!(rc, 0, "clEnqueueReadBuffer");
        host
    }
}

impl Drop for OpenCl {
    fn drop(&mut self) {
        let ocl = OPENCL.as_ref().unwrap();
        for mem in self.allocs.borrow_mut().drain(..) {
            unsafe { (ocl.clReleaseMemObject)(mem) };
        }
        unsafe {
            (ocl.clReleaseCommandQueue)(self.queue);
            (ocl.clReleaseContext)(self.ctx);
        }
    }
}
