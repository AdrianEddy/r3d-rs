// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use crate::{ RedResult, RedError, Sdk, clip::{ AlignedBuffer, Clip }, enums::*, future::*, metadata::* };
use crate::image_processing_settings::ImageProcessingSettings;
use crate::ffi::{ check_status, enum_from_sdk, string_from_c_array };
use core::ffi::c_void;
use std::sync::{ Arc, atomic::Ordering };
use cpp::*;

cpp! {{
    #include "R3DSDKDecoder.h"
}}

/// A created `R3DSDK::R3DDecoder`. Shared with the jobs in flight on it, so it
/// is only released once none is left.
struct R3dDecoderInner {
    ptr: *mut c_void,
    _sdk: Sdk,
}
// SAFETY: `ptr` is an opaque owned `R3DSDK::R3DDecoder *`; it is only used for
// the thread-safe `decode` and on drop.
unsafe impl Send for R3dDecoderInner {}
unsafe impl Sync for R3dDecoderInner {}
impl Drop for R3dDecoderInner {
    fn drop(&mut self) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::R3DDecoder *"] {
            R3DSDK::R3DDecoder::ReleaseDecoder(ptr);
        });
    }
}

/// Decodes frames on the GPU (CUDA or OpenCL) with full image processing,
/// asynchronously. Needs the SDK initialized with [`InitializeFlags::R3DDecoder`].
pub struct R3dDecoder {
    inner: Arc<R3dDecoderInner>,
}
impl R3dDecoder {
    /// Create a decoder with the given options. The decoder does not use
    /// `options` afterwards.
    pub fn new(options: &R3dDecoderOptions) -> RedResult<Self> {
        let mut ptr = std::ptr::null_mut();
        let ptrptr = &mut ptr;
        let options_ptr = options.ptr;
        check_status::<R3DStatus>(cpp!(unsafe [options_ptr as "R3DSDK::R3DDecoderOptions *", ptrptr as "R3DSDK::R3DDecoder **"] -> i32 as "int" { return R3DSDK::R3DDecoder::CreateDecoder(options_ptr, ptrptr); }))?;
        Ok(Self { inner: Arc::new(R3dDecoderInner { ptr, _sdk: options.sdk.clone() }) })
    }

    /// Pointer to the underlying `R3DSDK::R3DDecoder`.
    pub fn as_mut_ptr(&self) -> *mut c_void {
        self.inner.ptr
    }

    /// Asynchronously decode a single frame as specified in the job, which
    /// needs a clip, image processing settings and an output buffer.
    /// The decode will be scheduled and the function will immediately return a future.
    ///
    /// You should await the returned future to get the status of the decode operation.
    /// The future gives you back ownership of the job you passed in, so you can re-use it if needed.
    pub fn decode(&self, job: R3dDecodeJob) -> RedResult<CallbackFuture<R3dDecodeJob>> {
        if job.clip.is_none() {
            return Err(RedError::InvalidJobParameterClip);
        }
        if job.settings.is_none() {
            return Err(RedError::InvalidJobParameter);
        }
        if !job.has_output_buffer() {
            return Err(RedError::InvalidJobParameterOutputDeviceMem);
        }
        let ptr = self.inner.ptr;
        let job_ptr = job.ptr;

        let state = Arc::new(State::new(job, Some(self.inner.clone() as Arc<dyn Send + Sync>)));
        // Owned refcount handed to the SDK; reclaimed in `decode_callback`
        // via `Arc::from_raw`. Keeps `State` + the job alive across an
        // early future drop (no use-after-free). See `State` docs.
        let state_ptr = Arc::into_raw(state.clone()) as *mut c_void;

        let callback_ptr = decode_callback as extern "C" fn(*mut c_void, i32);

        let status = check_status::<R3DStatus>(cpp!(unsafe [ptr as "R3DSDK::R3DDecoder *", job_ptr as "R3DSDK::R3DDecodeJob *", state_ptr as "void *", callback_ptr as "void *"] -> i32 as "int" {
            job_ptr->callback = (R3DSDK::R3DDecodeJob::DecodeCallback)callback_ptr;
            job_ptr->privateData = state_ptr;
            return ptr->decode(job_ptr);
        }));
        if let Err(e) = status {
            // Job never queued → no callback → reclaim to avoid a leak.
            drop(unsafe { Arc::from_raw(state_ptr as *const State<R3dDecodeJob>) });
            Err(e)
        } else {
            Ok(CallbackFuture { state })
        }
    }
}

/// Options for an [`R3dDecoder`].
///
/// Only a device is required, picked with
/// [`use_cuda_device`](Self::use_cuda_device) or
/// [`use_opencl_device`](Self::use_opencl_device). The SDK derives reasonable
/// resource settings from that device; the other setters exist to cap or tune
/// them for a particular workload. Typical derived values are:
/// * thread counts: the number of CPU cores minus one;
/// * host memory pool: 512 MB per concurrent image;
/// * GPU memory pool: 1 GB per concurrent GPU frame, but at most 75% of the
///   device's memory.
///
/// Pools grow as needed instead of reserving their full size up front. If the
/// host or the device does run out of memory, the failure comes either from
/// [`R3dDecoder::decode`] itself or from the future it returns, depending on
/// when it happens.
pub struct R3dDecoderOptions {
    ptr: *mut c_void,
    sdk: Sdk,
}
// SAFETY: `Send` only — owns lane-specific device handles whose concurrent
// `&self` shareability is not documented by the SDK, so no `Sync`.
unsafe impl Send for R3dDecoderOptions {}
impl R3dDecoderOptions {
    /// Create options with the SDK defaults. Needs the SDK initialized with
    /// [`InitializeFlags::R3DDecoder`].
    pub fn new(sdk: &Sdk) -> RedResult<Self> {
        sdk.require(InitializeFlags::R3DDecoder)?;
        let mut ptr = std::ptr::null_mut();
        let ptrptr = &mut ptr;
        check_status::<R3DStatus>(cpp!(unsafe [ptrptr as "R3DSDK::R3DDecoderOptions **"] -> i32 as "int" { return R3DSDK::R3DDecoderOptions::CreateOptions(ptrptr); }))?;
        Ok(Self { ptr, sdk: sdk.clone() })
    }

    /// Pointer to the underlying `R3DSDK::R3DDecoderOptions`.
    pub fn as_mut_ptr(&self) -> *mut c_void {
        self.ptr
    }

    /// Directory where the SDK keeps temporary files and its compiled GPU
    /// kernels. With an empty path, or when this is never called, the SDK
    /// works without a scratch directory.
    /// Caching the kernels means they are compiled once rather than on every
    /// application start, which matters on some older graphics cards where
    /// compiling can take more than 5 seconds; on current cards it is barely
    /// noticeable.
    ///
    /// Fails with [`RedError::InvalidPath`] if `path` contains a NUL byte.
    pub fn set_scratch_folder(&mut self, path: &str) -> RedResult<()> {
        let c_path = std::ffi::CString::new(path).map_err(|_| RedError::InvalidPath)?;
        let self_ptr = self.ptr;
        let c_ptr = c_path.as_ptr();
        check_status::<R3DStatus>(cpp!(unsafe [self_ptr as "R3DSDK::R3DDecoderOptions *", c_ptr as "const char *"] -> i32 as "int" { return self_ptr->setScratchFolder(std::string(c_ptr)); }))?;
        Ok(())
    }
    /// Number of CPU worker threads that decompress the footage.
    /// They work as a shared pool: one frame is usually spread over several
    /// of them, so this is not the number of frames decoded at once.
    /// RED suggests one fewer than the number of CPU cores, which leaves the
    /// machine responsive while decoding.
    ///
    /// `0` selects that value (cores minus one).
    pub fn set_decompression_thread_count(&mut self, count: usize) -> RedResult<()> {
        let self_ptr = self.ptr;
        check_status::<R3DStatus>(cpp!(unsafe [self_ptr as "R3DSDK::R3DDecoderOptions *", count as "size_t"] -> i32 as "int" { return self_ptr->setDecompressionThreadCount(count); }))?;
        Ok(())
    }
    /// Number of threads that each carry one frame through the whole pipeline,
    /// from the [`R3dDecoder::decode`] call until its future resolves.
    /// A submitted job is checked and prepared, then waits for one of these
    /// threads to become free.
    /// Each thread needs its own host buffers for decompression, so this also
    /// scales host memory use, see [`set_memory_pool_size`](Self::set_memory_pool_size).
    ///
    /// `0` selects the number of CPU cores minus one.
    pub fn set_concurrent_image_count(&mut self, count: usize) -> RedResult<()> {
        let self_ptr = self.ptr;
        check_status::<R3DStatus>(cpp!(unsafe [self_ptr as "R3DSDK::R3DDecoderOptions *", count as "size_t"] -> i32 as "int" { return self_ptr->setConcurrentImageCount(count); }))?;
        Ok(())
    }
    /// Size of the host memory pool, in megabytes. The minimum is 1024; 4096
    /// or more is recommended.
    /// Every concurrent image draws its buffers from this pool, so grow it
    /// along with [`set_concurrent_image_count`](R3dDecoderOptions::set_concurrent_image_count).
    pub fn set_memory_pool_size(&mut self, size_mbs: usize) -> RedResult<()> {
        let self_ptr = self.ptr;
        check_status::<R3DStatus>(cpp!(unsafe [self_ptr as "R3DSDK::R3DDecoderOptions *", size_mbs as "size_t"] -> i32 as "int" { return self_ptr->setMemoryPoolSize(size_mbs); }))?;
        Ok(())
    }

    /// Size of the GPU memory pool, in megabytes. The minimum is 1024; 4096
    /// or more is recommended.
    /// The decoder takes all of its GPU-side buffers from this pool, the
    /// pinned transfer buffers and the temporary processing buffers included,
    /// and keeps them between frames for reuse.
    /// A pool too small for the [`set_gpu_concurrent_frame_count`](R3dDecoderOptions::set_gpu_concurrent_frame_count)
    /// value costs performance. Do not make it larger than the memory the
    /// graphics card actually has.
    pub fn set_gpu_memory_pool_size(&mut self, size_mbs: usize) -> RedResult<()> {
        let self_ptr = self.ptr;
        check_status::<R3DStatus>(cpp!(unsafe [self_ptr as "R3DSDK::R3DDecoderOptions *", size_mbs as "size_t"] -> i32 as "int" { return self_ptr->setGPUMemoryPoolSize(size_mbs); }))?;
        Ok(())
    }
    /// Number of frames in flight on the GPU at once, which is also the
    /// number of CUDA streams or OpenCL command queues the decoder creates.
    /// The minimum is 1; 3 is recommended when the card has the memory for it.
    ///
    /// Each frame is copied to the device, run through the processing kernels
    /// and copied back. GPU memory use grows linearly with this value, and
    /// one frame alone needs a lot, since RED frames are large and processing
    /// them also takes helper buffers, chiefly pinned staging buffers for the
    /// upload and the download and a temporary processing buffer.
    ///
    /// Going beyond 3 rarely helps: a GPU overlaps at most three operations
    /// from different streams or queues, one host-to-device copy, one kernel
    /// and one device-to-host copy. High-end Quadro cards, with two copy
    /// engines and one compute engine, run all three at once. GeForce cards
    /// manage only two, so their uploads and downloads never overlap.
    pub fn set_gpu_concurrent_frame_count(&mut self, count: usize) -> RedResult<()> {
        let self_ptr = self.ptr;
        check_status::<R3DStatus>(cpp!(unsafe [self_ptr as "R3DSDK::R3DDecoderOptions *", count as "size_t"] -> i32 as "int" { return self_ptr->setGPUConcurrentFrameCount(count); }))?;
        Ok(())
    }

    /// The OpenCL devices the decoder can run on. Needs the SDK initialized
    /// with [`InitializeFlags::R3DDecoder`].
    pub fn opencl_device_list(sdk: &Sdk) -> RedResult<Vec<OpenCLDeviceInfo>> {
        sdk.require(InitializeFlags::R3DDecoder)?;
        let mut list = Vec::<OpenCLDeviceInfo>::new();
        let list_ref = &mut list;
        check_status::<R3DStatus>(cpp!(unsafe [list_ref as "void*"] -> i32 as "int" {
            std::vector<R3DSDK::OpenCLDeviceInfo> list;
            auto status = R3DSDK::R3DDecoderOptions::GetOpenCLDeviceList(list);
            for (const auto &item : list) {
                rust!(opencl_list_add [list_ref: &mut Vec::<OpenCLDeviceInfo> as "void*", item: OpenCLDeviceInfo as "R3DSDK::OpenCLDeviceInfo"] {
                    list_ref.push(item);
                });
            }
            return status;
        }))?;
        Ok(list)
    }
    /// The CUDA devices the decoder can run on. Needs the SDK initialized
    /// with [`InitializeFlags::R3DDecoder`].
    pub fn cuda_device_list(sdk: &Sdk) -> RedResult<Vec<CudaDeviceInfo>> {
        sdk.require(InitializeFlags::R3DDecoder)?;
        let mut list = Vec::<CudaDeviceInfo>::new();
        let list_ref = &mut list;
        check_status::<R3DStatus>(cpp!(unsafe [list_ref as "void*"] -> i32 as "int" {
            std::vector<R3DSDK::CudaDeviceInfo> list;
            auto status = R3DSDK::R3DDecoderOptions::GetCudaDeviceList(list);
            for (const auto &item : list) {
                rust!(cuda_list_add [list_ref: &mut Vec::<CudaDeviceInfo> as "void*", item: CudaDeviceInfo as "R3DSDK::CudaDeviceInfo"] {
                    list_ref.push(item);
                });
            }
            return status;
        }))?;
        Ok(list)
    }

    /// Decode on `device`, one of the entries of
    /// [`opencl_device_list`](Self::opencl_device_list).
    /// One options object drives either OpenCL or CUDA, never both, so do not
    /// also call [`use_cuda_device`](Self::use_cuda_device) on it.
    pub fn use_opencl_device(&mut self, device: &OpenCLDeviceInfo) -> RedResult<()> {
        let self_ptr = self.ptr;
        check_status::<R3DStatus>(cpp!(unsafe [self_ptr as "R3DSDK::R3DDecoderOptions *", device as "const R3DSDK::OpenCLDeviceInfo *"] -> i32 as "int" { return self_ptr->useDevice(*device); }))?;
        Ok(())
    }
    /// Decode on `device`, one of the entries of
    /// [`cuda_device_list`](Self::cuda_device_list).
    /// One options object drives either CUDA or OpenCL, never both, so do not
    /// also call [`use_opencl_device`](Self::use_opencl_device) on it.
    pub fn use_cuda_device(&mut self, device: &CudaDeviceInfo) -> RedResult<()> {
        let self_ptr = self.ptr;
        check_status::<R3DStatus>(cpp!(unsafe [self_ptr as "R3DSDK::R3DDecoderOptions *", device as "const R3DSDK::CudaDeviceInfo *"] -> i32 as "int" { return self_ptr->useDevice(*device); }))?;
        Ok(())
    }

}

cpp_class! {
    /// An OpenCL device, see [`R3dDecoderOptions::opencl_device_list`].
    pub unsafe struct OpenCLDeviceInfo as "R3DSDK::OpenCLDeviceInfo"
}
// SAFETY: SDK-emitted device-enumeration entry — opaque, read-only at the
// public surface. The `cpp_class!` macro elides auto-`Send`, so add it
// explicitly; `Sync` falls out of the `[u8; N]` inline storage.
unsafe impl Send for OpenCLDeviceInfo {}
impl OpenCLDeviceInfo {
    /// The device's OpenCL platform, as a raw `cl_platform_id`. Together with
    /// [`device_id`](Self::device_id) it is what identifies the device to
    /// [`R3dDecoderOptions::use_opencl_device`].
    pub fn platform_id(&self) -> *mut c_void {
        cpp!(unsafe [self as "const R3DSDK::OpenCLDeviceInfo *"] -> *mut c_void as "void *" { return self->platform_id; })
    }

    /// The OpenCL device, as a raw `cl_device_id`. See
    /// [`platform_id`](Self::platform_id).
    pub fn device_id(&self) -> *mut c_void {
        cpp!(unsafe [self as "const R3DSDK::OpenCLDeviceInfo *"] -> *mut c_void as "void *" { return self->device_id; })
    }

    /// Device name, for display. Device selection ignores it.
    pub fn name(&self) -> String {
        let ptr = cpp!(unsafe [self as "const R3DSDK::OpenCLDeviceInfo *"] -> *const u8 as "const char *" {
            return self->name;
        });
        // SAFETY: the field is a `char[256]` array.
        string_from_c_array(unsafe { std::slice::from_raw_parts(ptr, 256) })
    }
    /// Platform name, for display. Device selection ignores it.
    pub fn platform_name(&self) -> String {
        let ptr = cpp!(unsafe [self as "const R3DSDK::OpenCLDeviceInfo *"] -> *const u8 as "const char *" {
            return self->platform_name;
        });
        // SAFETY: the field is a `char[256]` array.
        string_from_c_array(unsafe { std::slice::from_raw_parts(ptr, 256) })
    }
}
impl std::fmt::Debug for OpenCLDeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenCLDeviceInfo")
            .field("platform_id", &self.platform_id())
            .field("device_id", &self.device_id())
            .field("platform_name", &self.platform_name())
            .field("name", &self.name())
            .finish()
    }
}

cpp_class! {
    /// A CUDA device, see [`R3dDecoderOptions::cuda_device_list`].
    pub unsafe struct CudaDeviceInfo as "R3DSDK::CudaDeviceInfo"
}
// SAFETY: SDK-emitted device-enumeration entry — opaque, read-only at the
// public surface. `Send` added explicitly (macro elides it); `Sync` auto.
unsafe impl Send for CudaDeviceInfo {}
impl CudaDeviceInfo {
    /// PCI bus of the device, the same number CUDA reports in
    /// `cudaDeviceProp::pciBusID`. This is how
    /// [`R3dDecoderOptions::use_cuda_device`] finds the device.
    pub fn pci_bus_id(&self) -> i32 {
        cpp!(unsafe [self as "const R3DSDK::CudaDeviceInfo *"] -> i32 as "int" { return self->pci_bus_id; })
    }

    /// Device name. The SDK checks it against the PCI bus ID, so that the bus
    /// ID really refers to the intended device.
    pub fn name(&self) -> String {
        let ptr = cpp!(unsafe [self as "const R3DSDK::CudaDeviceInfo *"] -> *const u8 as "const char *" {
            return self->name;
        });
        // SAFETY: the field is a `char[256]` array.
        string_from_c_array(unsafe { std::slice::from_raw_parts(ptr, 256) })
    }
}
impl std::fmt::Debug for CudaDeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CudaDeviceInfo")
            .field("pci_bus_id", &self.pci_bus_id())
            .field("name", &self.name())
            .finish()
    }
}
impl Drop for R3dDecoderOptions {
    fn drop(&mut self) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::R3DDecoderOptions *"] {
            R3DSDK::R3DDecoderOptions::ReleaseOptions(ptr);
        });
    }
}


/// A frame to decode with [`R3dDecoder::decode`].
pub struct R3dDecodeJob {
    ptr: *mut c_void,
    /// The clip the SDK reads from, kept alive with the job.
    clip: Option<Arc<Clip>>,
    /// The settings the SDK reads, owned by the job.
    settings: Option<Box<ImageProcessingSettings>>,
    internal_buffer: Option<AlignedBuffer>,
    metadata_allocated: bool,
    _sdk: Sdk,
}
// SAFETY: `Send` only — single-owner cursor-state mutator, consumed once per
// submission then dropped or recycled. Concurrent `&self` would race the SDK's
// internal state, so not `Sync` (pinned negative in `tests/send_sync_surface.rs`).
unsafe impl Send for R3dDecodeJob {}
impl R3dDecodeJob {
    /// Create an empty job. Needs the SDK initialized with
    /// [`InitializeFlags::R3DDecoder`].
    pub fn new(sdk: &Sdk) -> RedResult<Self> {
        sdk.require(InitializeFlags::R3DDecoder)?;
        let mut ptr = std::ptr::null_mut();
        let ptrptr = &mut ptr;
        check_status::<R3DStatus>(cpp!(unsafe [ptrptr as "R3DSDK::R3DDecodeJob **"] -> i32 as "int" { return R3DSDK::R3DDecoder::CreateDecodeJob(ptrptr); }))?;
        Ok(Self { ptr, clip: None, settings: None, internal_buffer: None, metadata_allocated: false, _sdk: sdk.clone() })
    }
    /// Pointer to the underlying `R3DSDK::R3DDecodeJob`.
    pub fn as_mut_ptr(&self) -> *mut c_void {
        self.ptr
    }
}
impl Drop for R3dDecodeJob {
    fn drop(&mut self) {
        let self_ptr = self.ptr;
        let md = self.metadata_allocated;
        cpp!(unsafe [self_ptr as "R3DSDK::R3DDecodeJob *", md as "bool"] {
            if (md) {
                delete self_ptr->outputFrameMetadata;
                self_ptr->outputFrameMetadata = nullptr;
            }
            R3DSDK::R3DDecoder::ReleaseDecodeJob(self_ptr);
        });
    }
}

impl R3dDecodeJob {
    /// The clip to decode from. It must be set before submitting the job.
    pub fn set_clip(&mut self, clip: &Arc<Clip>) {
        let self_ptr = self.ptr;
        let clip_ptr = clip.as_mut_ptr();
        cpp!(unsafe [self_ptr as "R3DSDK::R3DDecodeJob *", clip_ptr as "R3DSDK::Clip *"] {
            self_ptr->clip = clip_ptr;
        });
        self.clip = Some(clip.clone());
    }
    /// The clip set by [`set_clip`](Self::set_clip).
    pub fn clip(&self) -> Option<&Arc<Clip>> {
        self.clip.as_ref()
    }
    /// Video track to decode: `0` for the main (A) track, `1` for the X track,
    /// the second, highlight-protection track that EPIC and Scarlet-X cameras
    /// can record. Has no effect when the two tracks are blended (HDRx).
    pub fn set_video_track_no(&mut self, v: usize) {
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "R3DSDK::R3DDecodeJob *", v as "size_t"] { self_ptr->videoTrackNo = v; })
    }
    pub fn set_video_frame_no(&mut self, v: usize) {
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "R3DSDK::R3DDecodeJob *", v as "size_t"] { self_ptr->videoFrameNo = v; })
    }
    pub fn video_frame_no(&self) -> usize {
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "const R3DSDK::R3DDecodeJob *"] -> usize as "size_t" { return self_ptr->videoFrameNo; })
    }
    /// The decode mode, see [`set_mode`](Self::set_mode).
    pub fn mode(&self) -> VideoDecodeMode {
        enum_from_sdk(self.raw_mode())
    }
    fn raw_mode(&self) -> i32 {
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "const R3DSDK::R3DDecodeJob *"] -> i32 as "int" { return (int)self_ptr->mode; })
    }
    /// Choose the decode mode, which trades output resolution for speed. The
    /// mode also sets the size of the decoded image, and with it how large the
    /// output buffer must be.
    pub fn set_mode(&mut self, v: VideoDecodeMode) {
        let v: i32 = v as i32;
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "R3DSDK::R3DDecodeJob *", v as "int"] { self_ptr->mode = (R3DSDK::VideoDecodeMode)v; })
    }
    /// The output pixel format, see [`set_pixel_type`](Self::set_pixel_type).
    pub fn pixel_type(&self) -> VideoPixelType {
        enum_from_sdk(self.raw_pixel_type())
    }
    fn raw_pixel_type(&self) -> i32 {
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "const R3DSDK::R3DDecodeJob *"] -> i32 as "int" { return (int)self_ptr->pixelType; })
    }

    /// Choose the pixel format of the decoded image. The decoder handles only
    /// the [`VideoPixelType`] values below; with any other the decode fails
    /// with [`RedError::InvalidJobParameterPixelType`].
    /// * [`VideoPixelType::Rgb16bitPlanar`]
    /// * [`VideoPixelType::Rgb16bitInterleaved`]
    /// * [`VideoPixelType::Dpx10bitMethodB`]
    /// * [`VideoPixelType::Bgr8bitInterleaved`]
    /// * [`VideoPixelType::RgbHalfFloatInterleaved`]
    /// * [`VideoPixelType::RgbHalfFloatAcesInt`]
    pub fn set_pixel_type(&mut self, v: VideoPixelType) {
        let v: i32 = v as i32;
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "R3DSDK::R3DDecodeJob *", v as "int"] { self_ptr->pixelType = (R3DSDK::VideoPixelType)v; })
    }
    /// Point the output at a caller-owned buffer. Prefer
    /// [`set_internal_buffer`](Self::set_internal_buffer) or
    /// [`allocate_internal_buffer`](Self::allocate_internal_buffer), which keep
    /// the buffer alive for as long as the SDK may write to it.
    ///
    /// # Safety
    /// `buf` must be 16-byte aligned and stay valid for writes of `size` bytes
    /// until every decode of this job has completed: until the future
    /// returned by [`R3dDecoder::decode`] resolves, or, if it is dropped first,
    /// until the SDK is done with the job, which is only known once the job
    /// itself is dropped.
    pub unsafe fn set_output_buffer(&mut self, buf: *mut c_void, size: usize) {
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "R3DSDK::R3DDecodeJob *", buf as "void *", size as "size_t"] {
            self_ptr->outputBuffer = buf;
            self_ptr->outputBufferSize = size;
        })
    }
    fn has_output_buffer(&self) -> bool {
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "const R3DSDK::R3DDecodeJob *"] -> bool as "bool" {
            return self_ptr->outputBuffer != nullptr && self_ptr->outputBufferSize != 0;
        })
    }
    /// Allocate an internal buffer for the output image, sized for the job's
    /// clip, mode and pixel type (set those first), aligned to 1024 bytes.
    pub fn allocate_internal_buffer(&mut self) -> RedResult<()> {
        let clip = self.clip.as_ref().ok_or(RedError::InvalidJobParameterClip)?;
        let mode = VideoDecodeMode::try_from(self.raw_mode()).map_err(|_| RedError::InvalidJobParameterMode)?;
        let pixel_type = VideoPixelType::try_from(self.raw_pixel_type()).map_err(|_| RedError::InvalidJobParameterPixelType)?;
        let buf = clip.allocate_aligned_buffer(&mode, &pixel_type, 1024)?;
        self.set_internal_buffer(buf);
        Ok(())
    }
    /// Give the job ownership of `buf` and point the output buffer at it,
    /// replacing any previous internal buffer. `buf` must be 16-byte aligned
    /// and large enough for the job's clip, mode and pixel type.
    ///
    /// The buffer then lives exactly as long as the job. A submitted job is
    /// co-owned by the SDK until its completion callback, so the SDK can never
    /// write into a buffer that was freed under it.
    pub fn set_internal_buffer(&mut self, mut buf: AlignedBuffer) {
        // SAFETY: the job owns `buf` from here on, see above.
        unsafe { self.set_output_buffer(buf.as_mut_ptr(), buf.len()) };
        self.internal_buffer = Some(buf);
    }
    /// The buffer set by [`set_internal_buffer`](Self::set_internal_buffer)
    /// or [`allocate_internal_buffer`](Self::allocate_internal_buffer).
    pub fn internal_buffer(&self) -> Option<&AlignedBuffer> {
        self.internal_buffer.as_ref()
    }
    /// Take the internal buffer back out of the job (e.g. to recycle it) and
    /// clear the job's output buffer pointer.
    pub fn take_internal_buffer(&mut self) -> Option<AlignedBuffer> {
        let buf = self.internal_buffer.take();
        if buf.is_some() {
            // SAFETY: a null buffer is never written to.
            unsafe { self.set_output_buffer(core::ptr::null_mut(), 0) };
        }
        buf
    }

    /// Set how the decoded image is processed (color, exposure and so on).
    /// Required: without it [`R3dDecoder::decode`] fails with
    /// [`RedError::InvalidJobParameter`]. The job stores its own copy of
    /// `settings`.
    pub fn set_image_processing(&mut self, settings: &ImageProcessingSettings) {
        let settings: &ImageProcessingSettings = self.settings.insert(Box::new(*settings));
        let ptr = settings as *const ImageProcessingSettings;
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "R3DSDK::R3DDecodeJob *", ptr as "R3DSDK::ImageProcessingSettings *"] {
            self_ptr->imageProcessingSettings = ptr;
        })
    }

    /// Allocate internal metadata object to receive per-frame metadata.
    /// Call this before submitting the job if you want to receive metadata.
    pub fn allocate_frame_metadata(&mut self) {
        if self.metadata_allocated {
            // Already allocated
            return;
        }
        let self_ptr = self.ptr;
        self.metadata_allocated = true;
        cpp!(unsafe [self_ptr as "R3DSDK::R3DDecodeJob *"] {
            self_ptr->outputFrameMetadata = new R3DSDK::Metadata();
        })
    }
    /// Metadata of the decoded frame, filled in by the decode. Fails with
    /// [`RedError::Other`] unless
    /// [`allocate_frame_metadata`](Self::allocate_frame_metadata) was called
    /// first.
    pub fn metadata(&self) -> RedResult<&Metadata> {
        if !self.metadata_allocated {
            return Err(RedError::Other("Metadata was not requested in the job".to_string()));
        }
        let self_ptr = self.ptr;
        let ptr: *const Metadata = cpp!(unsafe [self_ptr as "const R3DSDK::R3DDecodeJob *"] -> *const Metadata as "void *" {
            return self_ptr->outputFrameMetadata;
        });
        // SAFETY: the `Metadata` allocated above, owned by this job.
        Ok(unsafe { &*ptr })
    }
}

extern "C" fn decode_callback(job: *mut c_void /* R3DDecodeJob * */, decode_status: i32) {
    if job.is_null() {
        log::error!("Job pointer is null in callback.");
        return;
    }
    let ud: *mut c_void = cpp!(unsafe [job as "R3DSDK::R3DDecodeJob *"] -> *mut c_void as "void *" {
        return job->privateData;
    });
    if ud.is_null() {
        log::error!("No user data in job.");
        return;
    }

    // Reclaim the owned refcount handed to the SDK at submit time (balances
    // the `Arc::into_raw` in `decode`). `State` outlives an early future drop,
    // so the callback never writes into freed memory. The job stays in the
    // slot for `poll` to return.
    let state = unsafe { Arc::from_raw(ud as *const State<R3dDecodeJob>) };
    *state.status.lock().unwrap() = Some(check_status::<R3DStatus>(decode_status));
    state.done.store(true, Ordering::Release);
    state.waker.wake();
    // If the future is gone this is the last reference, and dropping it here
    // would call `ReleaseDecodeJob` on `job` from inside its own callback.
    release_from_callback(state);
}
