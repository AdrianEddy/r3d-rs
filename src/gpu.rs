// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2026 Adrian <adrian.eddy at gmail>

//! Bundled GPU decode pipeline — collapses the SDK's two-stage GPU
//! output sequence (`AsyncDecoder::decode_for_gpu_sdk` to a CPU-side
//! raw-Bayer scratch buffer, then `Red{Cuda,Metal,OpenCl}::process_blocking`
//! GPU debayer) into a single async (or blocking) call. Manages the
//! raw-Bayer scratch pool transparently.
//!
//! See [`GpuDecodePipeline`] for the public surface.

use core::ffi::c_void;
use std::future::Future;
use std::pin::Pin;
use std::sync::{ Arc, Mutex, MutexGuard, PoisonError };
use std::sync::atomic::{ AtomicBool, AtomicUsize, Ordering };
use std::task::{ Context, Poll };

use crate::{
    AlignedBuffer, AsyncDecoder, AsyncDecompressJob, CallbackFuture, Clip, GpuDecoder,
    ImageProcessingSettings, RedError, RedResult, Sdk, VideoDecodeMode, VideoPixelType,
};

#[cfg(not(target_os = "macos"))]
use crate::{ RedCuda, CudaStream, DebayerCudaJob };

#[cfg(feature = "metal-debayer")]
use crate::RedMetal;

use crate::{ RedOpenCl, cl_context, cl_command_queue };

/// The GPU a [`GpuDecodePipeline`] debayers on: the lane-specific debayer
/// object (`RedCuda` / `RedMetal` / [`RedOpenCl`]) plus the stream, queue or
/// context handles it runs on. Created with the `unsafe` constructors, which
/// take the raw handles.
///
/// # Thread-affinity
///
/// The lanes carry different concurrency contracts:
///
/// - **CUDA**: the CUDA stream + device id are thread-affine for
///   *enqueue* purposes — any `cuLaunchKernel` / `cudaMemcpyAsync`
///   submission requires the calling thread to have `device_id` as
///   its current CUDA device (`cuCtxSetCurrent` /
///   `cudaSetDevice`). Moving the handle across threads is sound
///   iff the destination thread re-establishes the device-current.
///   We treat the device-current requirement as a call-site contract
///   on `process_blocking` / `decode_to_gpu`, documented on those
///   methods. The handle itself has no apartment-model thread
///   affinity, so `Send` across threads is sound. The SDK keys its
///   per-frame state on `(device_id, stream)` (see `R3DSDKCuda.h`),
///   so [`GpuDecodePipeline`] serializes blocking debayers on this
///   stream; its async path runs each in-flight debayer on its own
///   pipeline-owned stream. Do not share `stream` with another
///   pipeline.
///
/// - **Metal**: `id<MTLCommandQueue>` is Apple-documented thread-
///   safe — multiple host threads may submit `MTLCommandBuffer`s
///   concurrently. The opaque `*mut c_void` is a refcounted
///   Objective-C object.
///
/// - **OpenCL**: `cl_context` and `cl_command_queue` are OpenCL-spec
///   thread-safe (all OpenCL API calls except `clSetKernelArg` are
///   thread-safe per OpenCL 1.2 §A.2).
pub struct GpuLane(Lane);

enum Lane {
    #[cfg(not(target_os = "macos"))]
    Cuda  { red: RedCuda,    stream: CudaStream, device_id: i32 },
    #[cfg(feature = "metal-debayer")]
    Metal { red: RedMetal,   queue:  *mut c_void },
    OpenCl{ red: RedOpenCl,  ctx: cl_context, queue: cl_command_queue },
}

// SAFETY: see the per-lane audit on `GpuLane` above. Rust does not
// permit per-variant `unsafe impl`; the blanket is intentional and the
// audit comment on the struct is the source of truth for reviewers.
unsafe impl Send for GpuLane {}
unsafe impl Sync for GpuLane {}

impl GpuLane {
    /// Debayer with CUDA on `stream` of device `device_id`.
    ///
    /// # Safety
    /// `stream` must be a CUDA stream of device `device_id` (or null for its
    /// default stream), valid for as long as this lane, the pipeline built on
    /// it and every [`PendingDebayer`] of that pipeline live.
    #[cfg(not(target_os = "macos"))]
    pub unsafe fn cuda(red: RedCuda, stream: CudaStream, device_id: i32) -> Self {
        Self(Lane::Cuda { red, stream, device_id })
    }

    /// Debayer with OpenCL on `queue` of `ctx`.
    ///
    /// # Safety
    /// `ctx` must be a valid `cl_context` and `queue` a valid
    /// `cl_command_queue` of it, both valid for as long as this lane and the
    /// pipeline built on it live.
    pub unsafe fn opencl(red: RedOpenCl, ctx: cl_context, queue: cl_command_queue) -> Self {
        Self(Lane::OpenCl { red, ctx, queue })
    }

    /// Debayer with Metal on `queue`.
    ///
    /// # Safety
    /// `queue` must be a valid `id<MTLCommandQueue>`, valid for as long as
    /// this lane and the pipeline built on it live.
    #[cfg(feature = "metal-debayer")]
    pub unsafe fn metal(red: RedMetal, queue: *mut c_void) -> Self {
        Self(Lane::Metal { red, queue })
    }

    fn sdk(&self) -> &Sdk {
        match &self.0 {
            #[cfg(not(target_os = "macos"))]
            Lane::Cuda { red, .. } => red.sdk(),
            #[cfg(feature = "metal-debayer")]
            Lane::Metal { red, .. } => red.sdk(),
            Lane::OpenCl { red, .. } => red.sdk(),
        }
    }
}

/// GPU output destination handed to `decode_to_gpu` / `decode_to_gpu_blocking`,
/// created with the `unsafe` constructors, which take the raw handles.
///
/// A device buffer covers CUDA `CUdeviceptr`, OpenCL `cl_mem` BUFFER, and
/// Metal `id<MTLBuffer>`. A Metal texture is the Metal-only `cl_image`-
/// equivalent path: caller-allocated `MTLTexture` written by the SDK
/// debayer via `set_output_device_image`, no expand pass needed on
/// consume because the SDK writes 4-channel RGBA with α = 1.
///
/// An OpenCL image is the OpenCL equivalent — a `cl_image` typed as
/// `{CL_RGBA, CL_UNORM_INT16}` (or `CL_HALF_FLOAT`) the SDK debayer
/// writes via `write_imagef()`. **4-channel RGBA output with α = 1.0
/// implicit**; consumers can skip the RGB→RGBA expand pass that the
/// device buffer path requires. See [`ClImageDataType`] for the
/// pixel_type ↔ data_type pairing.
///
/// Every constructor requires the output to stay valid until the decode it is
/// passed to has completed: until `decode_to_gpu*` or `debayer_blocking`
/// returns, or until the [`PendingDebayer`] of `submit_debayer` is completed
/// or dropped.
pub struct GpuOutput(Output);

enum Output {
    DeviceMem { ptr: *mut c_void, size: usize },
    #[cfg(feature = "metal-debayer")]
    MetalTexture { texture: *mut c_void, size: usize },
    OpenClImage {
        image: *mut c_void,
        /// Image dimensions, in pixels (un-rounded). The SDK reads the
        /// dimensions from the cl_image object directly; these only
        /// size the output passed to the SDK (with
        /// [`ClImageDataType::bytes_per_pixel`]).
        width: usize,
        height: usize,
        data_type: ClImageDataType,
    },
}

impl GpuOutput {
    /// A device buffer of `size` bytes on the pipeline's lane: a CUDA device
    /// pointer, an OpenCL `cl_mem` buffer or a Metal `id<MTLBuffer>`.
    ///
    /// # Safety
    /// `ptr` must be such a buffer, valid for writes of `size` bytes until the
    /// decode it is passed to has completed (see [`GpuOutput`]).
    pub unsafe fn device_mem(ptr: *mut c_void, size: usize) -> Self {
        Self(Output::DeviceMem { ptr, size })
    }

    /// A Metal `id<MTLTexture>`, with `size` the byte size of its storage.
    ///
    /// # Safety
    /// `texture` must be such a texture on the pipeline's device, valid until
    /// the decode it is passed to has completed (see [`GpuOutput`]).
    #[cfg(feature = "metal-debayer")]
    pub unsafe fn metal_texture(texture: *mut c_void, size: usize) -> Self {
        Self(Output::MetalTexture { texture, size })
    }

    /// An OpenCL 2D `cl_image` of `width × height` pixels whose
    /// `cl_image_format` channel order is `CL_RGBA` and whose data type is
    /// `data_type`, which pairs with the decode's `pixel_type`. Only valid on
    /// the OpenCL lane.
    ///
    /// # Safety
    /// `image` must be such an image of the pipeline's context, valid until
    /// the decode it is passed to has completed (see [`GpuOutput`]).
    pub unsafe fn opencl_image(image: *mut c_void, width: usize, height: usize, data_type: ClImageDataType) -> Self {
        Self(Output::OpenClImage { image, width, height, data_type })
    }
}

/// OpenCL `cl_image` data type, in the strict subset the R3D SDK's
/// OpenCL debayer accepts. Encoded in the type system so a caller
/// cannot construct an OpenCL image [`GpuOutput`] whose pixel_type ↔
/// image-format pairing the SDK would reject.
///
/// The pairings are documented in the SDK's `REDOpenCLGpuDecodeSample.cpp`
/// sample: the cl_image must have `channel_order = CL_RGBA` and a data type
/// paired with the SDK `VideoPixelType` the kernel was set up for —
/// `CL_UNORM_INT16` with `Rgb16bitInterleaved`, `CL_HALF_FLOAT` with
/// `RgbHalfFloatInterleaved`. The sample also warns that
/// `CL_UNSIGNED_INT{8,16,32}` channel types are **not** accepted by
/// the cl_image output path (the SDK kernel writes via
/// `write_imagef()` only).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ClImageDataType {
    /// `CL_UNORM_INT16` — pairs with `VideoPixelType::Rgb16bitInterleaved`.
    /// wgpu equivalent: `Rgba16Unorm`. Mandatory minimum-image-format
    /// support per OpenCL 1.2 §5.3.2.2 (works on every conformant
    /// CL 1.2+ driver).
    UnormInt16,
    /// `CL_HALF_FLOAT` — pairs with `VideoPixelType::RgbHalfFloatInterleaved`.
    /// wgpu equivalent: `Rgba16Float`. Also mandatory minimum-image-
    /// format support.
    HalfFloat,
}

impl ClImageDataType {
    /// The `VideoPixelType` the SDK expects when this cl_image is the
    /// output. `run_debayer` cross-checks the `pixel_type` argument
    /// against this and returns `RedError::Other` on mismatch — a
    /// pairing mismatch would otherwise surface as
    /// `Status_InvalidJobParameter_pixelType` deep inside the SDK,
    /// long after the caller's bookkeeping is done.
    pub fn expected_pixel_type(self) -> VideoPixelType {
        match self {
            Self::UnormInt16 => VideoPixelType::Rgb16bitInterleaved,
            Self::HalfFloat  => VideoPixelType::RgbHalfFloatInterleaved,
        }
    }

    /// Per-pixel byte size of the cl_image format. The SDK's
    /// `output_device_mem_size` field is **destination-aware** for
    /// cl_image outputs: per the SDK's OpenCL sample
    /// `REDOpenCLGpuDecodeSample.cpp`, the size is
    /// `width × height × 4 × 2` (4 channels × 2 B) for `UnormInt16`
    /// and `HalfFloat`. Both data types are 2 B / channel here.
    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            Self::UnormInt16 | Self::HalfFloat => 4 * 2,
        }
    }
}

/// Bundled GPU decode pipeline.
///
/// Owns an `AsyncDecoder`, an optional `GpuDecoder` (GPU wavelet
/// decompress, host-output Bayer), a `GpuLane`, and a small
/// pool of raw-Bayer scratch buffers (`AlignedBuffer`s, 16-byte
/// aligned per SDK contract). The pool hands out the smallest idle
/// buffer that fits and steady-states to one slot per `(clip, mode)`
/// combination.
///
/// One call (`decode_to_gpu` / `decode_to_gpu_blocking`) performs the
/// full sequence: pick the decompressor (per-clip via
/// `GpuDecoder::decode_supported_for_clip`), schedule the decompress
/// into a scratch buffer, await its completion, run the lane debayer,
/// and wait for GPU completion. On success, the GPU output buffer is
/// GPU-visible on the queue/stream this pipeline owns.
///
/// # CUDA call-site contract
///
/// The CUDA lane requires the calling thread to have the same CUDA
/// device set as the CUDA lane's `device_id` (via `cuCtxSetCurrent` or
/// the runtime API equivalent) before calling `decode_to_gpu*`. The
/// SDK reads the current device from the calling thread — see
/// `RedCuda::process_blocking`.
pub struct GpuDecodePipeline {
    async_decoder: AsyncDecoder,
    /// `GpuDecoder` performs the wavelet decompression on the GPU
    /// (writing the raw Bayer to a host scratch buffer — same shape
    /// as `AsyncDecoder`, just moves the decompress kernel off the
    /// CPU). `None` when `enable_gpu_decompress = false` at
    /// construction. The per-clip
    /// `GpuDecoder::decode_supported_for_clip` check picks the active
    /// decompressor per frame; RED ONE clips, clips on platforms
    /// without GPU decompression (e.g. Windows ARM64, per the SDK
    /// 9.2.0 release notes) and any future clip with codec features
    /// `GpuDecoder` hasn't been updated for fall back to
    /// `AsyncDecoder`.
    gpu_decoder:   Option<GpuDecoder>,
    /// Whether frames may still be routed to `gpu_decoder`. Cleared for
    /// good the first time the `GpuDecoder` rejects a frame at submit
    /// that the `AsyncDecoder` then accepts: `GpuDecoder::Open()`
    /// reports no status and `DecodeSupportedForClip()` only inspects
    /// the clip, so a `GpuDecoder` this SDK setup cannot run first
    /// shows up there. See [`submit_decompress`](Self::submit_decompress).
    gpu_decoder_usable: AtomicBool,
    /// The lane plus every pool an in-flight [`PendingDebayer`] still
    /// uses. Shared so a `PendingDebayer` keeps them alive even if the
    /// pipeline is dropped first.
    shared:        Arc<LaneShared>,
}

/// The parts of a [`GpuDecodePipeline`] that outlive it while a
/// [`PendingDebayer`] is in flight.
struct LaneShared {
    lane:          GpuLane,
    bayer_pool:    Mutex<Vec<AlignedBuffer>>,
    /// Pooled device-side `raw_device_mem` upload buffers (one per
    /// in-flight debayer). Reusing these is **load-bearing for
    /// throughput**, not just an allocation tidy-up: the per-frame
    /// `cudaMalloc`/`cudaFree` they replace are *device-wide
    /// synchronizing* calls — every `cudaFree` drains all concurrent
    /// `DecodeForGpuSdk` decompress jobs, collapsing pipeline depth back
    /// to one. RED's own GPU-decode sample pools exactly these via
    /// `SimpleMemoryPool` (`REDCudaGpuDecodeSample.cpp`). Each buffer
    /// frees itself with its lane's allocator on drop. Idle count capped
    /// at `pool_cap`.
    raw_device_pool: Mutex<Vec<RawDeviceBuf>>,
    /// CUDA debayer streams for the async (`process_async`) path — one
    /// per concurrently in-flight debayer, indexed by the caller's
    /// `stream_slot`. The SDK keys its per-frame internal object mapping
    /// on `(deviceId, stream)` (`R3DSDKCuda.h`: "deviceId and
    /// stream are used as tokens for internal object mapping"), so two
    /// concurrent `processAsync` jobs MUST run on distinct streams or
    /// they collide. RED's own sample round-robins `NUM_STREAMS`
    /// (`REDCudaGpuDecodeSample.cpp`). Each slot is marked busy
    /// while a `PendingDebayer` uses it. Created lazily on first async
    /// submit (under the caller's current CUDA context) and destroyed in
    /// `Drop`.
    #[cfg(not(target_os = "macos"))]
    debayer_streams: Mutex<Vec<DebayerStream>>,
    /// Held across each blocking CUDA debayer: they all run on the lane
    /// stream, which the SDK uses as a per-frame state key, so they must
    /// not overlap.
    #[cfg(not(target_os = "macos"))]
    lane_stream:   Mutex<()>,
    /// Idle-buffer high-water mark for `bayer_pool` **and**
    /// `raw_device_pool`. The pipelined caller keeps up to `depth`
    /// decompress/debayer jobs in flight, each holding one scratch +
    /// one raw-device buffer, so a cap below `depth` would force a
    /// re-allocation every cycle for the buffers past the cap. Defaults
    /// to 4 (the one-frame-at-a-time blocking path never needs more
    /// than one); raise it to the pipeline depth via
    /// [`GpuDecodePipeline::set_scratch_pool_capacity`].
    pool_cap:      AtomicUsize,
}

/// A pooled device-side upload buffer for the debayer's `raw_device_mem`:
/// a `cudaMalloc`'d CUDA device pointer, an OpenCL `clCreateBuffer`
/// `cl_mem`, or an owned Metal `id<MTLBuffer>`. Freed with the allocator
/// that made it when dropped, so no exit path can leak it.
struct RawDeviceBuf {
    ptr:   *mut c_void,
    size:  usize,
    alloc: DeviceAlloc,
}

#[derive(Clone, Copy)]
enum DeviceAlloc {
    #[cfg(not(target_os = "macos"))]
    Cuda,
    OpenCl,
    #[cfg(feature = "metal-debayer")]
    Metal,
}

// SAFETY: `ptr` is an opaque device-memory handle (CUDA device pointer,
// OpenCL `cl_mem` or `id<MTLBuffer>`); it carries no host-thread affinity.
// Moving it between threads is sound under the same call-site
// device-current contract that already governs `GpuLane` (see the audit on
// that enum).
unsafe impl Send for RawDeviceBuf {}

impl RawDeviceBuf {
    #[cfg(not(target_os = "macos"))]
    fn cuda(size: usize) -> RedResult<Self> {
        let cudart = crate::CUDA.as_ref().map_err(|_| RedError::RedCudaLibraryNotFound)?;
        let mut ptr: *mut c_void = core::ptr::null_mut();
        let rc = unsafe { (cudart.cudaMalloc)(&mut ptr, size) };
        if rc != 0 { return Err(RedError::CudaError(rc)); }
        Ok(Self { ptr, size, alloc: DeviceAlloc::Cuda })
    }

    fn opencl(ctx: cl_context, size: usize) -> RedResult<Self> {
        let ocl = crate::OPENCL.as_ref().map_err(|_| RedError::RedOpenCLLibraryNotFound)?;
        let mut err: i32 = 0;
        // CL_MEM_READ_ONLY = (1 << 2)
        let ptr = unsafe { (ocl.clCreateBuffer)(ctx, 1 << 2, size, core::ptr::null_mut(), &mut err) };
        if err != 0 || ptr.is_null() { return Err(RedError::OpenCLError(err)); }
        Ok(Self { ptr, size, alloc: DeviceAlloc::OpenCl })
    }

    #[cfg(feature = "metal-debayer")]
    fn metal(queue: *mut c_void, size: usize) -> RedResult<Self> {
        let ptr = crate::debayer::new_managed_buffer(queue, size);
        if ptr.is_null() {
            return Err(RedError::Other(format!("Metal failed to allocate a {size}-byte raw_device_mem buffer")));
        }
        Ok(Self { ptr, size, alloc: DeviceAlloc::Metal })
    }
}

impl Drop for RawDeviceBuf {
    fn drop(&mut self) {
        match self.alloc {
            #[cfg(not(target_os = "macos"))]
            DeviceAlloc::Cuda => {
                if let Ok(cudart) = crate::CUDA.as_ref() {
                    let _ = unsafe { (cudart.cudaFree)(self.ptr) };
                }
            }
            DeviceAlloc::OpenCl => {
                if let Ok(ocl) = crate::OPENCL.as_ref() {
                    let _ = unsafe { (ocl.clReleaseMemObject)(self.ptr) };
                }
            }
            #[cfg(feature = "metal-debayer")]
            DeviceAlloc::Metal => unsafe { crate::debayer::release_buffer(self.ptr) },
        }
    }
}

/// A pooled CUDA debayer stream (`cudaStream_t`). Newtype so the pool can
/// live in a `Send` pipeline — a raw stream handle is `!Send`, but the
/// handle has no host-thread affinity (same call-site device-current
/// contract as [`GpuLane`]).
#[cfg(not(target_os = "macos"))]
struct DebayerStream {
    stream: *mut c_void,
    /// A `PendingDebayer` is running on this stream.
    busy:   bool,
}
// SAFETY: opaque CUDA stream handle; no host-thread affinity (see above).
#[cfg(not(target_os = "macos"))]
unsafe impl Send for DebayerStream {}

/// A completed decompress, ready for the debayer: the decompress job and
/// the host buffer the SDK wrote the frame into (`raw_host_mem`), which
/// the job owns. Produced by awaiting a [`PendingDecompress`]; consumed by
/// [`GpuDecodePipeline::debayer_blocking`] or
/// [`GpuDecodePipeline::submit_debayer`].
pub struct DecompressedFrame {
    /// Owns the host buffer through its internal buffer
    /// ([`AsyncDecompressJob::set_internal_buffer`]).
    job:         AsyncDecompressJob,
    /// Bytes of that buffer the decompressor asked for
    /// (`GetSizeBufferNeeded`); a pooled buffer can be larger.
    raw_len:     usize,
    /// `GpuDecoder` (rather than `AsyncDecoder`) produced the buffer.
    gpu_decoded: bool,
}

impl DecompressedFrame {
    /// The completed decompress job (frame number, mode, per-frame metadata).
    pub fn job(&self) -> &AsyncDecompressJob {
        &self.job
    }

    /// `true` if the `GpuDecoder` produced this frame, `false` for the
    /// `AsyncDecoder`.
    pub fn used_gpu_decoder(&self) -> bool {
        self.gpu_decoded
    }

    fn raw_host(&self) -> *mut c_void {
        self.job.internal_buffer().map_or(core::ptr::null_mut(), |b| b.as_ptr().cast_mut())
    }
}

/// An in-flight decompress from [`GpuDecodePipeline::submit_decompress`];
/// resolves to its [`DecompressedFrame`].
///
/// The decompress job owns its host buffer, and the SDK co-owns the job
/// until its completion callback, so dropping this before it resolves is
/// memory-safe — the buffer is freed once the SDK is done with it. To stop
/// a decode early (e.g. on seek), call [`abort`](Self::abort) and keep
/// awaiting; the decode then completes promptly with
/// [`RedError::Cancelled`].
pub struct PendingDecompress {
    fut:         CallbackFuture<AsyncDecompressJob>,
    raw_len:     usize,
    gpu_decoded: bool,
}

impl PendingDecompress {
    /// Ask the SDK to abort this decode as soon as possible. See
    /// [`CallbackFuture::abort`].
    pub fn abort(&self) {
        self.fut.abort();
    }

    /// `true` if this frame was submitted to the `GpuDecoder`.
    pub fn used_gpu_decoder(&self) -> bool {
        self.gpu_decoded
    }
}

impl Future for PendingDecompress {
    type Output = RedResult<DecompressedFrame>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        Pin::new(&mut this.fut).poll(cx).map(|result| result.map(|job| DecompressedFrame {
            job,
            raw_len:     this.raw_len,
            gpu_decoded: this.gpu_decoded,
        }))
    }
}

/// An in-flight async debayer on the CUDA lane — see
/// [`GpuDecodePipeline::submit_debayer`]. Owns everything the SDK reads
/// until `completeAsync`: the decompressed frame (the SDK reads frame
/// metadata from its host buffer, `raw_host_mem`), the pooled
/// `raw_device_mem` upload buffer, and (through the job) a copy of the image
/// processing settings. It also holds the pipeline's lane, so the `RedCuda`
/// and the debayer stream outlive it even if the pipeline is dropped first.
///
/// Finish it with [`GpuDecodePipeline::complete_debayer`] or by dropping
/// it: both wait for the GPU work (`completeAsync`), then return the
/// buffers and the stream slot to the pipeline.
#[cfg(not(target_os = "macos"))]
pub struct PendingDebayer {
    job:        Option<DebayerCudaJob>,
    raw_device: Option<RawDeviceBuf>,
    frame:      Option<DecompressedFrame>,
    /// Claimed debayer stream slot.
    slot:       Option<usize>,
    shared:     Arc<LaneShared>,
}

#[cfg(not(target_os = "macos"))]
impl Drop for PendingDebayer {
    fn drop(&mut self) {
        // `DebayerCudaJob::drop` runs `completeAsync`, which waits for the
        // GPU work reading `raw_device` and the frame's host buffer, so it
        // must go before either is recycled.
        drop(self.job.take());
        if let Some(slot) = self.slot.take() {
            self.shared.release_debayer_stream(slot);
        }
        if let Some(raw_device) = self.raw_device.take() {
            self.shared.release_raw_device(raw_device);
        }
        if let Some(frame) = self.frame.take() {
            self.shared.recycle_frame(frame);
        }
        // `shared` drops after this, keeping the `RedCuda` that owns `job`
        // and the stream alive until here.
    }
}

impl GpuDecodePipeline {
    /// Create a new pipeline. `threads = 0` selects the SDK default
    /// ([`AsyncDecoder::threads_available`]). `enable_gpu_decompress`
    /// controls GPU decompression — when `true`, the pipeline owns a paired
    /// [`GpuDecoder`] and per-frame picks it via
    /// [`GpuDecoder::decode_supported_for_clip`] (RED ONE clips and any
    /// future codec feature `GpuDecoder` hasn't been updated for fall
    /// back to the `AsyncDecoder`). When `false`, only the
    /// `AsyncDecoder` runs.
    pub fn new(lane: GpuLane, threads: usize, enable_gpu_decompress: bool) -> RedResult<Self> {
        Ok(Self {
            async_decoder:      AsyncDecoder::new(lane.sdk(), threads),
            gpu_decoder:        enable_gpu_decompress.then(|| GpuDecoder::new(lane.sdk())),
            gpu_decoder_usable: AtomicBool::new(enable_gpu_decompress),
            shared: Arc::new(LaneShared {
                lane,
                bayer_pool:      Mutex::new(Vec::with_capacity(4)),
                raw_device_pool: Mutex::new(Vec::with_capacity(4)),
                #[cfg(not(target_os = "macos"))]
                debayer_streams: Mutex::new(Vec::new()),
                #[cfg(not(target_os = "macos"))]
                lane_stream:     Mutex::new(()),
                pool_cap:        AtomicUsize::new(4),
            }),
        })
    }

    /// Set the idle-buffer high-water mark of the scratch and
    /// `raw_device_mem` pools (4 by default). A pipelined caller running
    /// `depth` concurrent decompress jobs sets this to `depth` so the
    /// per-frame buffers steady-state in the pools instead of being
    /// re-allocated each cycle. No effect on the one-frame blocking path.
    pub fn set_scratch_pool_capacity(&mut self, cap: usize) {
        self.shared.pool_cap.store(cap.max(1), Ordering::Relaxed);
    }

    /// Per-clip predicate — `true` iff the pipeline will route this
    /// clip's decompress through the SDK `GpuDecoder` rather than the
    /// `AsyncDecoder`. Returns `false` when:
    /// - the pipeline was constructed with `enable_gpu_decompress = false`,
    /// - or `GpuDecoder::decode_supported_for_clip(clip)` returns false
    ///   (e.g. RED ONE clips),
    /// - or the `GpuDecoder` already failed on this setup (see
    ///   [`submit_decompress`](Self::submit_decompress)).
    ///
    /// Exposed for callers that want to surface the active decompressor
    /// to telemetry / UI without paying the full decode pass.
    pub fn will_use_gpu_decoder(&self, clip: &Clip) -> bool {
        self.gpu_decoder_for(clip).is_some()
    }

    /// Required output buffer size, in bytes, for a frame of `clip`
    /// decoded at `(mode, pixel_type)`.
    ///
    /// **Why not the SDK's `DebayerXxxJob::ResultFrameSize`?** Both
    /// `R3DSDK::DebayerCudaJob::ResultFrameSize` and the OpenCL / Metal
    /// equivalents read the *raw_host_mem* frame header to discover the
    /// scaled dimensions — they return `0` if `raw_host_mem` isn't set
    /// yet, as the SDK GPU-decode samples make clear (see the SDK's
    /// `REDCudaGpuDecodeSample.cpp` and `REDOpenCLGpuDecodeSample.cpp`
    /// samples). This pipeline
    /// allocates the GPU output buffer BEFORE the AsyncDecoder
    /// decompress, so we cannot call the SDK helper at that point.
    ///
    /// The computation is deterministic from `(clip.width(),
    /// clip.height(), mode, pixel_type)`:
    /// - mode scales both dimensions by 1 / 2 / 4 / 8 / 16
    /// - pixel_type sets the per-pixel byte count
    /// - a 16-byte row/page alignment slack is added to absorb the
    ///   SDK's internal row-pitch padding.
    pub fn result_frame_size(
        &self,
        clip:       &Clip,
        mode:       VideoDecodeMode,
        pixel_type: VideoPixelType,
    ) -> RedResult<usize> {
        Ok(compute_output_size(clip.width(), clip.height(), mode, pixel_type))
    }

    /// Async path. Decompress raw Bayer into a pooled CPU scratch buffer,
    /// then debayer into `out` synchronously on the lane queue/stream.
    ///
    /// On `Ok(())` the GPU output buffer is GPU-visible on the
    /// queue/stream this pipeline owns. The scratch buffer is returned
    /// to the pool when this function returns. Dropping the future
    /// part-way is memory-safe (see [`PendingDecompress`]).
    pub async fn decode_to_gpu(
        &self,
        clip:       &Arc<Clip>,
        frame_no:   usize,
        mode:       VideoDecodeMode,
        pixel_type: VideoPixelType,
        settings:   &ImageProcessingSettings,
        out:        GpuOutput,
    ) -> RedResult<()> {
        // The one-call path is the two split stages run back-to-back: a
        // pipelined caller drives `submit_decompress` for several frames
        // before calling `debayer_blocking` on each as it completes.
        let frame = self.submit_decompress(clip, frame_no, mode)?.await?;
        self.debayer_blocking(frame, mode, pixel_type, settings, out)
    }

    /// **Decompress — submit.** Pick the per-clip
    /// decompressor, acquire a CPU Bayer scratch buffer from the pool,
    /// and submit the SDK decompress, returning immediately with a
    /// [`PendingDecompress`] that resolves to the [`DecompressedFrame`].
    ///
    /// `GpuDecoder` and `AsyncDecoder` produce identical scratch
    /// contents per the SDK contract; the route is chosen per-clip (see
    /// [`will_use_gpu_decoder`](Self::will_use_gpu_decoder)).
    /// `pixel_type` is **not** an input of the decompress — it only
    /// affects the debayer.
    ///
    /// If the `GpuDecoder` rejects the frame, it is resubmitted to the
    /// `AsyncDecoder`. If that succeeds, the `GpuDecoder` is the problem
    /// (e.g. the SDK was initialized without the GPU component it needs),
    /// and the pipeline stops routing frames to it.
    ///
    /// Await the returned future (several can be in flight at once),
    /// then hand the frame
    /// to [`debayer_blocking`](Self::debayer_blocking) or
    /// [`submit_debayer`](Self::submit_debayer). The frame owns its
    /// scratch, so out-of-order completion is sound.
    pub fn submit_decompress(
        &self,
        clip:     &Arc<Clip>,
        frame_no: usize,
        mode:     VideoDecodeMode,
    ) -> RedResult<PendingDecompress> {
        let via_async = Decompressor::Async(&self.async_decoder);
        let Some(gpu) = self.gpu_decoder_for(clip) else {
            return self.submit_with(via_async, clip, frame_no, mode);
        };
        match self.submit_with(Decompressor::Gpu(gpu), clip, frame_no, mode) {
            Ok(pending) => Ok(pending),
            Err(gpu_err) => {
                // A frame-specific failure (out of range, dropped frame,
                // unreadable file) fails here too, and leaves the
                // GpuDecoder enabled.
                let pending = self.submit_with(via_async, clip, frame_no, mode)?;
                if self.gpu_decoder_usable.swap(false, Ordering::Relaxed) {
                    log::warn!(
                        "GpuDecoder rejected frame {frame_no} ({gpu_err}) that the AsyncDecoder accepted; \
                         using the AsyncDecoder for all further frames"
                    );
                }
                Ok(pending)
            }
        }
    }

    /// Submit one decompress through `decompressor`, into a pooled
    /// scratch buffer that the job owns from here on.
    fn submit_with(
        &self,
        decompressor: Decompressor<'_>,
        clip:         &Arc<Clip>,
        frame_no:     usize,
        mode:         VideoDecodeMode,
    ) -> RedResult<PendingDecompress> {
        // **`size_buffer_needed` is decompressor-specific** — the SDK
        // exposes separate static helpers on `AsyncDecoder` and
        // `GpuDecoder`. Both consult the same per-clip metadata but the
        // underlying plugin call is different (`REDR3D-x64.dll` vs
        // `REDGpuDecoder-x64.dll`).
        let mut job = AsyncDecompressJob::new(clip.sdk());
        job.set_clip(clip);
        job.set_mode(mode);
        job.set_video_frame_no(frame_no);

        let raw_len = decompressor.size_buffer_needed(&job);
        if raw_len == 0 {
            return Err(RedError::Other(format!(
                "{} reported zero buffer size — invalid clip/mode pairing",
                decompressor.name(),
            )));
        }
        // The job owns the scratch: the SDK co-owns the job until its
        // completion callback, so the buffer cannot be freed while the
        // SDK is still writing into it.
        job.set_internal_buffer(self.shared.acquire_scratch(raw_len)?);

        let fut = decompressor.decode_for_gpu_sdk(job)?;
        Ok(PendingDecompress { fut, raw_len, gpu_decoded: decompressor.is_gpu() })
    }

    /// **Debayer — blocking.** Consume a completed decompress
    /// `frame` and debayer it into `out` on the lane queue/stream.
    ///
    /// `process_blocking` runs the SDK `processAsync` *and*
    /// `completeAsync`, so on `Ok(())` the GPU output is fully realized
    /// and visible on the queue this pipeline owns — there is no GPU
    /// fence left to wait on. The scratch is returned to the pool on
    /// return, success or failure.
    ///
    /// On the CUDA lane, concurrent calls take turns: they share the lane
    /// stream, which the SDK uses as a per-frame state key. For
    /// overlapping debayers use [`submit_debayer`](Self::submit_debayer).
    pub fn debayer_blocking(
        &self,
        frame:      DecompressedFrame,
        mode:       VideoDecodeMode,
        pixel_type: VideoPixelType,
        settings:   &ImageProcessingSettings,
        out:        GpuOutput,
    ) -> RedResult<()> {
        let result = self.run_debayer(&frame, mode, pixel_type, settings, out);
        self.shared.recycle_frame(frame);
        result
    }

    /// **Debayer — non-blocking submit. CUDA lane only.**
    ///
    /// Enqueues the SDK `processAsync` for a completed decompress on the
    /// debayer stream selected by `stream_slot`, **deferring**
    /// `completeAsync` (unlike [`debayer_blocking`](Self::debayer_blocking),
    /// which runs both back-to-back). Returns immediately with a
    /// [`PendingDebayer`] that owns the frame, the pooled
    /// `raw_device_mem` buffer and a copy of `settings` — drive it to
    /// completion with [`complete_debayer`](Self::complete_debayer)
    /// (which blocks on `completeAsync`), or drop it.
    ///
    /// `stream_slot` selects which debayer stream the job runs on.
    /// Concurrently in-flight debayers need **distinct** slots — the SDK
    /// keys per-frame internal state on `(deviceId, stream)`
    /// (`R3DSDKCuda.h`), so two concurrent jobs on the same
    /// stream collide. A slot whose previous `PendingDebayer` is still
    /// alive is rejected with an error. Streams are created lazily.
    ///
    /// This lets frame N's debayer GPU work run while the CPU sets up +
    /// enqueues frame N+1, instead of the host blocking on each
    /// `completeAsync`.
    ///
    /// The caller MUST have the lane's CUDA device current (call-site
    /// contract, same as `process_blocking`).
    #[cfg(not(target_os = "macos"))]
    pub fn submit_debayer(
        &self,
        frame:       DecompressedFrame,
        mode:        VideoDecodeMode,
        pixel_type:  VideoPixelType,
        settings:    &ImageProcessingSettings,
        out:         GpuOutput,
        stream_slot: usize,
    ) -> RedResult<PendingDebayer> {
        // From here on every exit path, error or not, goes through
        // `PendingDebayer::drop`, which recycles what has been acquired.
        let mut pending = PendingDebayer {
            job:        None,
            raw_device: None,
            frame:      Some(frame),
            slot:       None,
            shared:     Arc::clone(&self.shared),
        };
        let Lane::Cuda { red, device_id, .. } = &self.shared.lane.0 else {
            return Err(RedError::Other("submit_debayer requires the CUDA lane".into()));
        };
        let Output::DeviceMem { ptr: out_ptr, size: out_size } = out.0 else {
            return Err(RedError::Other("CUDA submit_debayer requires a device buffer output".into()));
        };
        let stream = self.shared.claim_debayer_stream(stream_slot)?;
        pending.slot = Some(stream_slot);

        let Some(frame) = pending.frame.as_ref() else { unreachable!() };
        let raw_device = self.shared.acquire_raw_device(frame.raw_len, RawDeviceBuf::cuda)?;
        let raw_device_ptr = raw_device.ptr;
        pending.raw_device = Some(raw_device);
        if !frame.gpu_decoded {
            cuda_upload(raw_device_ptr, frame.raw_host(), frame.raw_len, stream)?;
        }
        // SAFETY: `pending` owns the frame's host buffer and `raw_device` until
        // the job completes in `PendingDebayer::drop`, and `out` stays valid
        // until then per `GpuOutput`'s contract.
        let job = pending.job.insert(unsafe { Self::build_cuda_debayer_job(
            red, mode, pixel_type, settings, frame.raw_host(), raw_device_ptr, out_ptr, out_size) });
        // Enqueue on `stream` and return WITHOUT completeAsync.
        // SAFETY: `stream` is the pipeline's own debayer stream, alive until
        // `LaneShared` drops after `pending`; the job's memory is kept alive as
        // above.
        unsafe { red.process_async(*device_id, stream, job) }?;
        Ok(pending)
    }

    /// Finish an in-flight async debayer: `completeAsync` (blocks until
    /// the GPU work completes and disposes the SDK's per-frame internal
    /// objects), then recycle the scratch + raw-device buffer into the
    /// pools and free the stream slot. The output buffer is GPU-visible
    /// on return. Equivalent to dropping `pending`.
    #[cfg(not(target_os = "macos"))]
    pub fn complete_debayer(&self, pending: PendingDebayer) -> RedResult<()> {
        drop(pending);
        Ok(())
    }

    /// Blocking convenience for sync callers.
    pub fn decode_to_gpu_blocking(
        &self,
        clip:       &Arc<Clip>,
        frame_no:   usize,
        mode:       VideoDecodeMode,
        pixel_type: VideoPixelType,
        settings:   &ImageProcessingSettings,
        out:        GpuOutput,
    ) -> RedResult<()> {
        pollster::block_on(self.decode_to_gpu(clip, frame_no, mode, pixel_type, settings, out))
    }

    /// The `GpuDecoder`, if this pipeline routes `clip` through it. The
    /// check is a metadata read on the clip; no SDK side-effects.
    fn gpu_decoder_for(&self, clip: &Clip) -> Option<&GpuDecoder> {
        self.gpu_decoder.as_ref().filter(|_| {
            self.gpu_decoder_usable.load(Ordering::Relaxed)
                && GpuDecoder::decode_supported_for_clip(clip)
        })
    }

    /// Build the CUDA `DebayerCudaJob` shared by the blocking
    /// ([`run_debayer`](Self::run_debayer)) and async
    /// ([`submit_debayer`](Self::submit_debayer)) paths: mode / pixel /
    /// settings + host/device-mem wiring.
    ///
    /// # Safety
    /// `host`, `raw_device` and `out_ptr` must stay valid until the job has
    /// completed, see the `DebayerCudaJob` setters.
    #[cfg(not(target_os = "macos"))]
    #[allow(clippy::too_many_arguments)]
    unsafe fn build_cuda_debayer_job(
        red:        &RedCuda,
        mode:       VideoDecodeMode,
        pixel_type: VideoPixelType,
        settings:   &ImageProcessingSettings,
        host:       *mut c_void,
        raw_device: *mut c_void,
        out_ptr:    *mut c_void,
        out_size:   usize,
    ) -> DebayerCudaJob {
        let mut job = red.create_debayer_job();
        job.set_mode(mode);
        job.set_pixel_type(pixel_type);
        job.set_image_processing(settings);
        // SAFETY: the caller keeps the memory valid, see above.
        unsafe {
            job.set_raw_host_mem(host);
            job.set_raw_device_mem(raw_device);
            job.set_output_device_mem(out_ptr, out_size);
        }
        job
    }

    // `raw_device_mem` must hold "the same data as raw_host_mem except
    // already on the GPU" (`R3DSDKCuda.h`), and "Upload the result buffer
    // to the GPU" is step 2 of both GPU modes in "How to use the SDK.txt".
    // For the AsyncDecoder every lane uploads it, as RED's
    // `RED{Cuda,Metal,OpenCL}Sample` do — without the upload the CUDA
    // debayer silently returns a black frame. RED's CUDA and Metal
    // *GpuDecoder* samples only allocate `raw_device_mem` and let the SDK
    // fill it, so those two lanes skip the upload for GpuDecoder output;
    // the OpenCL GpuDecoder sample uploads, so that lane always does.
    fn run_debayer(
        &self,
        frame:      &DecompressedFrame,
        mode:       VideoDecodeMode,
        pixel_type: VideoPixelType,
        settings:   &ImageProcessingSettings,
        out:        GpuOutput,
    ) -> RedResult<()> {
        let shared = &*self.shared;
        match &shared.lane.0 {
            #[cfg(not(target_os = "macos"))]
            Lane::Cuda { red, stream, device_id } => {
                let (out_ptr, out_size) = match out.0 {
                    Output::DeviceMem { ptr, size } => (ptr, size),
                    #[cfg(feature = "metal-debayer")]
                    Output::MetalTexture { .. } =>
                        return Err(RedError::Other("CUDA lane received a Metal texture output".into())),
                    Output::OpenClImage { .. } =>
                        return Err(RedError::Other("CUDA lane received an OpenCL image output".into())),
                };

                let _lane_stream = lock(&shared.lane_stream);
                // Pooled — no per-frame device-wide `cudaMalloc`/`cudaFree`
                // sync.
                let raw_device = shared.acquire_raw_device(frame.raw_len, RawDeviceBuf::cuda)?;
                let result = (|| {
                    if !frame.gpu_decoded {
                        cuda_upload(raw_device.ptr, frame.raw_host(), frame.raw_len, *stream)?;
                    }
                    // SAFETY: the frame, `raw_device` and `out` (per
                    // `GpuOutput`'s contract) outlive this blocking debayer,
                    // and the lane's stream is valid per `GpuLane::cuda`.
                    unsafe {
                        let mut deb = Self::build_cuda_debayer_job(
                            red, mode, pixel_type, settings, frame.raw_host(), raw_device.ptr, out_ptr, out_size);
                        red.process_blocking(*device_id, *stream, &mut deb)
                    }
                })();
                if result.is_err() {
                    // The upload may still be landing in `raw_device`.
                    cuda_sync(*stream);
                }
                shared.release_raw_device(raw_device);
                result
            }
            #[cfg(feature = "metal-debayer")]
            Lane::Metal { red, queue } => {
                let raw_device = shared.acquire_raw_device(frame.raw_len, |size| RawDeviceBuf::metal(*queue, size))?;
                let result = (|| {
                    if !frame.gpu_decoded {
                        // SAFETY: the buffer holds at least `raw_len`
                        // bytes, the scratch is readable for `raw_len`
                        // bytes, and a pooled buffer is idle on the GPU.
                        unsafe { crate::debayer::write_managed_buffer(raw_device.ptr, frame.raw_host(), frame.raw_len) };
                    }
                    let mut deb = red.create_debayer_job();
                    deb.set_mode(mode);
                    deb.set_pixel_type(pixel_type);
                    deb.set_image_processing(settings);
                    // SAFETY: the frame, `raw_device` and `out` (per
                    // `GpuOutput`'s contract) outlive this blocking debayer,
                    // and the lane's queue is valid per `GpuLane::metal`.
                    unsafe {
                        deb.set_raw_host_mem(frame.raw_host());
                        deb.set_raw_device_mem(raw_device.ptr);
                        match out.0 {
                            Output::DeviceMem  { ptr, size } => deb.set_output_device_mem(ptr, size),
                            Output::MetalTexture { texture, size } => deb.set_output_device_image(texture, size),
                            Output::OpenClImage { .. } =>
                                return Err(RedError::Other("Metal lane received an OpenCL image output".into())),
                        }
                        red.process_blocking(*queue, &mut deb)
                    }
                })();
                shared.release_raw_device(raw_device);
                result
            }
            Lane::OpenCl { red, ctx, queue } => {
                // Resolve the destination shape. Two cases:
                //   1. Device buffer — cl_mem BUFFER, size = output bytes.
                //      Tight-packed 6 B/px (`Rgb16bitInterleaved`) or
                //      4 B/px (`Bgr8bit*`); SDK kernel writes via
                //      `write_only` pointer stores.
                //   2. OpenCL image — cl_mem IMAGE2D of channel order
                //      `CL_RGBA` paired with `pixel_type` per
                //      [`ClImageDataType::expected_pixel_type`]. SDK
                //      kernel writes via `write_imagef()` — 4-channel
                //      output with α = 1.0 implicit, no consumer-side
                //      expand pass needed. Per the OpenCL sample
                //      (`REDOpenCLGpuDecodeSample.cpp`), the SDK's
                //      `output_device_mem_size` is `width × height × 4
                //      × 2` for both `UnormInt16` and `HalfFloat`.
                let (out_mem, out_size) = match out.0 {
                    Output::DeviceMem { ptr, size } => (ptr, size),
                    #[cfg(feature = "metal-debayer")]
                    Output::MetalTexture { .. } =>
                        return Err(RedError::Other("OpenCL lane received a Metal texture output".into())),
                    Output::OpenClImage { image, width, height, data_type } => {
                        if data_type.expected_pixel_type() != pixel_type {
                            return Err(RedError::Other(format!(
                                "OpenCL cl_image data_type / pixel_type mismatch: \
                                 data_type = {data_type:?} (expects {:?}), got {pixel_type:?}",
                                data_type.expected_pixel_type(),
                            )));
                        }
                        let size = width
                            .saturating_mul(height)
                            .saturating_mul(data_type.bytes_per_pixel());
                        (image, size)
                    }
                };

                // `REDOpenCLGpuDecodeSample.cpp`: clCreateBuffer +
                // clEnqueueWriteBuffer. The buffer is pooled (no per-frame
                // clCreateBuffer/clReleaseMemObject churn); the upload stays.
                let ocl = crate::OPENCL.as_ref()
                    .map_err(|_| RedError::RedOpenCLLibraryNotFound)?;
                let raw_device = shared.acquire_raw_device(frame.raw_len, |size| RawDeviceBuf::opencl(*ctx, size))?;
                let result = (|| {
                    // CL_TRUE = 1 (blocking write — keeps lifetimes simple).
                    // Only the bytes the decompressor wrote, not the whole
                    // (possibly larger) pooled scratch.
                    let upload_err = unsafe {
                        (ocl.clEnqueueWriteBuffer)(
                            *queue, raw_device.ptr, 1 /* blocking */, 0, frame.raw_len,
                            frame.raw_host(), 0, core::ptr::null(), core::ptr::null_mut(),
                        )
                    };
                    if upload_err != 0 {
                        return Err(RedError::OpenCLError(upload_err));
                    }
                    let mut deb = red.create_debayer_job();
                    deb.set_mode(mode);
                    deb.set_pixel_type(pixel_type);
                    deb.set_image_processing(settings);
                    // SAFETY: the frame, `raw_device` and `out` (per
                    // `GpuOutput`'s contract) outlive this blocking debayer,
                    // and the lane's handles are valid per `GpuLane::opencl`.
                    unsafe {
                        deb.set_raw_host_mem(frame.raw_host());
                        deb.set_raw_device_mem(raw_device.ptr);
                        // Both buffer and cl_image outputs are wired through
                        // `set_output_device_mem` — the SDK auto-detects the
                        // cl_mem type via `clGetMemObjectInfo(MEM_TYPE)` and
                        // routes IMAGE2D writes through `write_imagef()`.
                        deb.set_output_device_mem(out_mem, out_size);
                        red.process_blocking(*ctx, *queue, &mut deb)
                    }
                })();
                shared.release_raw_device(raw_device);
                result
            }
        }
    }
}

impl LaneShared {
    fn pool_cap(&self) -> usize {
        self.pool_cap.load(Ordering::Relaxed)
    }

    /// The smallest idle scratch buffer of at least `size` bytes, or a new one.
    fn acquire_scratch(&self, size: usize) -> RedResult<AlignedBuffer> {
        let reused = take_best_fit(&mut lock(&self.bayer_pool), size, AlignedBuffer::len);
        match reused {
            Some(buf) => Ok(buf),
            None => AlignedBuffer::new(size, 16),
        }
    }

    fn release_scratch(&self, buf: AlignedBuffer) {
        // Keep up to `pool_cap` idle buffers (pipeline depth for a
        // pipelined caller; 4 by default) — beyond that we let buffers drop.
        let excess = push_capped(&mut lock(&self.bayer_pool), buf, self.pool_cap());
        drop(excess);
    }

    /// Return a frame's scratch to the pool; the decompress job is dropped.
    fn recycle_frame(&self, mut frame: DecompressedFrame) {
        if let Some(buf) = frame.job.take_internal_buffer() {
            self.release_scratch(buf);
        }
    }

    /// The smallest idle `raw_device_mem` buffer of at least `size` bytes,
    /// or a new one from `alloc`. See
    /// [`raw_device_pool`](Self::raw_device_pool) for why pooling these is
    /// load-bearing (avoids per-frame device-wide `cudaMalloc`/`cudaFree`).
    fn acquire_raw_device(
        &self,
        size:  usize,
        alloc: impl FnOnce(usize) -> RedResult<RawDeviceBuf>,
    ) -> RedResult<RawDeviceBuf> {
        let reused = take_best_fit(&mut lock(&self.raw_device_pool), size, |b| b.size);
        match reused {
            Some(buf) => Ok(buf),
            None => alloc(size),
        }
    }

    /// Return a `raw_device_mem` buffer to the pool, or free it when the
    /// pool is already at `pool_cap` idle buffers. The free happens outside
    /// the pool lock (`cudaFree` synchronizes the whole device).
    fn release_raw_device(&self, buf: RawDeviceBuf) {
        let excess = push_capped(&mut lock(&self.raw_device_pool), buf, self.pool_cap());
        drop(excess);
    }

    /// Lazily create the CUDA debayer stream for `slot`, mark it busy and
    /// return it. Fails if a `PendingDebayer` is still using the slot. See
    /// [`debayer_streams`](Self::debayer_streams).
    #[cfg(not(target_os = "macos"))]
    fn claim_debayer_stream(&self, slot: usize) -> RedResult<CudaStream> {
        let cudart = crate::CUDA.as_ref().map_err(|_| RedError::RedCudaLibraryNotFound)?;
        let mut streams = lock(&self.debayer_streams);
        while streams.len() <= slot {
            let mut stream: *mut c_void = core::ptr::null_mut();
            let rc = unsafe { (cudart.cudaStreamCreate)(&mut stream) };
            if rc != 0 { return Err(RedError::CudaError(rc)); }
            streams.push(DebayerStream { stream, busy: false });
        }
        let entry = &mut streams[slot];
        if entry.busy {
            return Err(RedError::Other(format!(
                "debayer stream slot {slot} already has a debayer in flight")));
        }
        entry.busy = true;
        Ok(entry.stream as CudaStream)
    }

    /// Free a slot claimed by [`claim_debayer_stream`](Self::claim_debayer_stream)
    /// once nothing is left running on its stream.
    #[cfg(not(target_os = "macos"))]
    fn release_debayer_stream(&self, slot: usize) {
        let Some(stream) = lock(&self.debayer_streams).get(slot).map(|s| s.stream) else { return };
        // If `processAsync` never ran, the raw-frame upload may still be in
        // flight on the stream, writing a buffer about to be recycled.
        cuda_sync(stream);
        if let Some(entry) = lock(&self.debayer_streams).get_mut(slot) {
            entry.busy = false;
        }
    }
}

impl Drop for LaneShared {
    fn drop(&mut self) {
        // Every `PendingDebayer` holds an `Arc` to this, so no debayer is in
        // flight on the streams. Pooled buffers free themselves
        // (`AlignedBuffer` / `RawDeviceBuf: Drop`).
        #[cfg(not(target_os = "macos"))]
        {
            let streams = std::mem::take(self.debayer_streams.get_mut().unwrap_or_else(PoisonError::into_inner));
            if let (false, Ok(cudart)) = (streams.is_empty(), crate::CUDA.as_ref()) {
                for s in streams {
                    let _ = unsafe { (cudart.cudaStreamDestroy)(s.stream) };
                }
            }
        }
    }
}

/// Lock a pool mutex, ignoring poisoning: every critical section here
/// leaves its `Vec` or flag consistent.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Remove and return the smallest pooled item of at least `size` bytes.
fn take_best_fit<T>(pool: &mut Vec<T>, size: usize, len: impl Fn(&T) -> usize) -> Option<T> {
    let (idx, _) = pool.iter()
        .enumerate()
        .filter(|(_, b)| len(b) >= size)
        .min_by_key(|(_, b)| len(b))?;
    Some(pool.swap_remove(idx))
}

/// Push `item` unless the pool already holds `cap` items; returns the item
/// that did not fit, so the caller can drop it outside the lock.
fn push_capped<T>(pool: &mut Vec<T>, item: T, cap: usize) -> Option<T> {
    if pool.len() < cap {
        pool.push(item);
        None
    } else {
        Some(item)
    }
}

/// Enqueue the `len`-byte host→device upload of a decompressed frame into
/// `raw_device_mem` on `stream`, so it lands before the debayer kernels
/// enqueued after it, without a device-wide sync. From pageable memory the
/// call returns once the bytes are staged, so the host buffer is reusable.
#[cfg(not(target_os = "macos"))]
fn cuda_upload(dst: *mut c_void, src: *const c_void, len: usize, stream: CudaStream) -> RedResult<()> {
    let cudart = crate::CUDA.as_ref().map_err(|_| RedError::RedCudaLibraryNotFound)?;
    let rc = unsafe { (cudart.cudaMemcpyAsync)(dst, src, len, crate::CUDA_MEMCPY_HOST_TO_DEVICE, stream) };
    if rc != 0 { Err(RedError::CudaError(rc)) } else { Ok(()) }
}

/// Wait for all work queued on `stream`. Errors are ignored: this only runs
/// on cleanup paths, before a buffer the stream may write is reused.
#[cfg(not(target_os = "macos"))]
fn cuda_sync(stream: CudaStream) {
    if let Ok(cudart) = crate::CUDA.as_ref() {
        let _ = unsafe { (cudart.cudaStreamSynchronize)(stream) };
    }
}

/// Required **flat-buffer** output size for `(width, height, mode,
/// pixel_type)`, in bytes — analytical equivalent of the SDK's
/// `DebayerXxxJob::ResultFrameSize` for cases where the raw_host_mem
/// header isn't available yet (pre-decompress allocation path).
///
/// **Scope — buffer outputs only.** This function applies the SDK's
/// 16-pixel dimension rounding which exists *because* flat-buffer
/// outputs include the SDK's internal row-pitch padding. The
/// destination-typed outputs ([`GpuOutput::opencl_image`] and
/// `GpuOutput::metal_texture`) carry their own row-pitch internally
/// and are sized at the un-rounded clip dimensions — those paths
/// **must not** go through `compute_output_size`. The OpenCL cl_image
/// arm of [`GpuDecodePipeline::run_debayer`] computes its size as
/// `width × height × data_type.bytes_per_pixel()`
/// ([`ClImageDataType::bytes_per_pixel`]); the Metal MTLTexture arm
/// takes the size from its caller, which must account for the
/// texture's rounded storage (`round16(w) × round16(h) × 8`).
fn compute_output_size(
    width:      usize,
    height:     usize,
    mode:       VideoDecodeMode,
    pixel_type: VideoPixelType,
) -> usize {
    let (w, h) = scaled_dims(width, height, mode);
    // Each lane's debayer rounds the output dimensions up to a
    // 16-pixel multiple (matches the 16-byte alignment requirement
    // documented on the SDK's `allocate_aligned_buffer` helpers + the
    // GPU-decode samples). Round before bpp multiplication.
    let w = (w + 15) & !15;
    let h = (h + 15) & !15;
    w.saturating_mul(h).saturating_mul(crate::clip::bytes_per_pixel(pixel_type))
}

/// Per-clip choice of decompressor for
/// [`GpuDecodePipeline::submit_decompress`]. The two variants share
/// the same observable API surface
/// (`size_buffer_needed` / `decode_for_gpu_sdk`), so the submit code
/// stays generic over them.
///
/// Per-frame allocation churn: zero. The variant carries borrowed
/// references; nothing on this enum touches the heap.
#[derive(Clone, Copy)]
enum Decompressor<'a> {
    Async(&'a AsyncDecoder),
    Gpu(&'a GpuDecoder),
}

impl<'a> Decompressor<'a> {
    /// Short label for diagnostics — matches the SDK plugin name.
    fn name(&self) -> &'static str {
        match self {
            Self::Async(_) => "AsyncDecoder",
            Self::Gpu(_)   => "GpuDecoder",
        }
    }

    fn is_gpu(&self) -> bool {
        matches!(self, Self::Gpu(_))
    }

    /// Decompressor-specific `OutputBuffer` sizing. The two SDK
    /// helpers (`AsyncDecoder::GetSizeBufferNeeded` /
    /// `GpuDecoder::GetSizeBufferNeeded`) consult the same per-clip
    /// metadata but the underlying plugin is different — they MUST
    /// be called against the matching plugin.
    fn size_buffer_needed(&self, job: &AsyncDecompressJob) -> usize {
        match self {
            Self::Async(_) => AsyncDecoder::size_buffer_needed(job),
            Self::Gpu(_)   => GpuDecoder::size_buffer_needed(job),
        }
    }

    /// Dispatch the chosen decompressor on `job`. Returns the same
    /// callback-future shape on both arms — the caller awaits it the
    /// same way.
    fn decode_for_gpu_sdk(
        &self,
        job: AsyncDecompressJob,
    ) -> RedResult<CallbackFuture<AsyncDecompressJob>> {
        match self {
            Self::Async(a) => a.decode_for_gpu_sdk(job),
            Self::Gpu(g)   => g.decode_for_gpu_sdk(job),
        }
    }
}

/// **Scaled** clip dimensions after applying `VideoDecodeMode`'s
/// resolution divisor, un-rounded — the dimensions the SDK debayer
/// writes through a `cl_image` / `MTLTexture` output. Use in callers
/// that allocate destination-typed outputs (cl_image, MTLTexture)
/// when they need to size the texture itself.
///
/// For flat-buffer outputs, callers want
/// [`GpuDecodePipeline::result_frame_size`] instead — it applies the
/// SDK's 16-pixel rounding **and** multiplies by the bytes per pixel.
pub fn scaled_dims(width: usize, height: usize, mode: VideoDecodeMode) -> (usize, usize) {
    crate::clip::scaled_dims(width, height, mode)
}
