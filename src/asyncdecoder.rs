// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use crate::{ RedResult, RedError, Sdk, clip::{ AlignedBuffer, Clip }, enums::*, future::*, metadata::* };
use crate::ffi::{ check_status, enum_from_sdk };
use core::ffi::c_void;
use std::sync::{ Arc, atomic::Ordering };
use cpp::*;

cpp!{{
    #include "R3DSDK.h"
}}

/// An opened `R3DSDK::AsyncDecoder`. Shared with the jobs in flight on it, so
/// it is only closed once none is left.
struct AsyncDecoderInner {
    ptr: *mut c_void,
    _sdk: Sdk,
}
// SAFETY: `ptr` is an opaque owned `R3DSDK::AsyncDecoder *` with no thread
// affinity; it is only used for the thread-safe `DecodeForGpuSdk` and on drop.
unsafe impl Send for AsyncDecoderInner {}
unsafe impl Sync for AsyncDecoderInner {}
impl Drop for AsyncDecoderInner {
    fn drop(&mut self) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::AsyncDecoder *"] {
            ptr->Close();
            delete ptr;
        });
    }
}

/// Decompresses frames in the background on a pool of CPU threads. The
/// output is raw sensor data, which a GPU debayer then turns into an image.
pub struct AsyncDecoder {
    inner: Arc<AsyncDecoderInner>,
}

impl AsyncDecoder {
    /// Start a decoder with `threads_no` worker threads, or with
    /// [`threads_available`](AsyncDecoder::threads_available) of them when `threads_no` is 0.
    pub fn new(sdk: &Sdk, threads_no: usize) -> Self {
        let ptr = cpp!(unsafe [threads_no as "size_t"] -> *mut c_void as "R3DSDK::AsyncDecoder *" {
            R3DSDK::AsyncDecoder *ptr = new R3DSDK::AsyncDecoder();
            ptr->Open(threads_no);
            return ptr;
        });
        Self { inner: Arc::new(AsyncDecoderInner { ptr, _sdk: sdk.clone() }) }
    }
    /// Pointer to the underlying `R3DSDK::AsyncDecoder`.
    pub fn as_mut_ptr(&self) -> *mut c_void {
        self.inner.ptr
    }

    /// How many worker threads [`new`](Self::new) starts when given a
    /// `threads_no` of 0.
    pub fn threads_available(_sdk: &Sdk) -> usize {
        cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::AsyncDecoder::ThreadsAvailable(); })
    }

    /// How many bytes the output buffer of `job` must hold for this decoder.
    /// [`decode_for_gpu_sdk`](AsyncDecoder::decode_for_gpu_sdk) sizes the
    /// job's internal buffer with it; call it to pre-allocate or pool buffers.
    /// [`GpuDecoder`] needs a different size.
    ///
    /// The result depends on the job's clip and mode, so set both first.
    /// It is 0 if the job has no clip or its [`Clip`] has nothing loaded.
    pub fn size_buffer_needed(job: &AsyncDecompressJob) -> usize {
        // The SDK requires the job's clip to be set.
        if job.clip.is_none() {
            return 0;
        }
        let job_ptr = job.as_mut_ptr();
        cpp!(unsafe [job_ptr as "const R3DSDK::AsyncDecompressJob *"] -> usize as "size_t" {
            return R3DSDK::AsyncDecoder::GetSizeBufferNeeded(*job_ptr);
        })
    }

    /// Queue `job`'s frame to be decompressed into the job's output buffer,
    /// ready to be handed to a GPU debayer as its raw host memory.
    /// The decompress will be scheduled and the function will immediately return a future.
    ///
    /// You should await the returned future to get the status of the decompress operation.
    /// On success, the future gives you back ownership of the job you passed in, so you can re-use it if needed.
    ///
    /// The job's internal buffer is sized for this decoder, see
    /// [`AsyncDecompressJob::set_internal_buffer`].
    ///
    /// # Errors
    /// Fails right away with [`RedError::NoClipOpen`] if the job has no
    /// loaded clip, or with the error the SDK reports if it refuses the job,
    /// e.g. [`RedError::OutputBufferInvalid`] for a buffer passed to
    /// [`set_output_buffer`](AsyncDecompressJob::set_output_buffer) that is
    /// too small. If the decode itself fails, the future resolves to that
    /// error and the job is dropped.
    pub fn decode_for_gpu_sdk(&self, job: AsyncDecompressJob) -> RedResult<CallbackFuture<AsyncDecompressJob>> {
        let ptr = self.inner.ptr;
        submit_decompress(job, self.inner.clone(), Self::size_buffer_needed, |job_ptr, state_ptr, callback_ptr| {
            cpp!(unsafe [ptr as "R3DSDK::AsyncDecoder *", job_ptr as "R3DSDK::AsyncDecompressJob *", state_ptr as "void *", callback_ptr as "void *"] -> i32 as "int" {
                job_ptr->Callback = (R3DSDK::AsyncDecompressJob::AsyncDecompressCallback)callback_ptr;
                job_ptr->PrivateData = state_ptr;
                return ptr->DecodeForGpuSdk(*job_ptr);
            })
        })
    }
}

/// An opened `R3DSDK::GpuDecoder`, shared like [`AsyncDecoderInner`].
struct GpuDecoderInner {
    ptr: *mut c_void,
    _sdk: Sdk,
}
// SAFETY: same as `AsyncDecoderInner`.
unsafe impl Send for GpuDecoderInner {}
unsafe impl Sync for GpuDecoderInner {}
impl Drop for GpuDecoderInner {
    fn drop(&mut self) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::GpuDecoder *"] {
            ptr->Close();
            delete ptr;
        });
    }
}

/// Like [`AsyncDecoder`], but moves the decompression itself to the GPU.
///
/// Its output buffer holds a frame prepared in the background, which the GPU
/// debayer then both decompresses and processes. This works with the CUDA
/// (Windows and Linux), Metal (macOS) and OpenCL debayers. Not every clip is
/// supported, see [`decode_supported_for_clip`](Self::decode_supported_for_clip).
///
/// Plan for 4 GB or more of video memory when decoding on the GPU.
pub struct GpuDecoder {
    inner: Arc<GpuDecoderInner>,
}

impl GpuDecoder {
    /// Create and open the GPU decoder.
    pub fn new(sdk: &Sdk) -> Self {
        let ptr = cpp!(unsafe [] -> *mut c_void as "R3DSDK::GpuDecoder *" {
            R3DSDK::GpuDecoder *ptr = new R3DSDK::GpuDecoder();
            ptr->Open();
            return ptr;
        });
        Self { inner: Arc::new(GpuDecoderInner { ptr, _sdk: sdk.clone() }) }
    }
    /// Pointer to the underlying `R3DSDK::GpuDecoder`.
    pub fn as_mut_ptr(&self) -> *mut c_void {
        self.inner.ptr
    }

    /// Returns `true` if frames of `clip` can be decompressed on the GPU.
    ///
    /// Clips shot on a RED ONE are not supported, nor are clips in the
    /// Nikon N-RAW and R3D NE formats, and SDK 9.2 has no GPU decompression
    /// on Windows ARM64.
    /// Decode unsupported clips with [`AsyncDecoder`] instead.
    pub fn decode_supported_for_clip(clip: &Clip) -> bool {
        let clip_ptr = clip.as_mut_ptr();
        let status: i32 = cpp!(unsafe [clip_ptr as "const R3DSDK::Clip *"] -> i32 as "int" {
            return (int)R3DSDK::GpuDecoder::DecodeSupportedForClip(*clip_ptr);
        });
        status == DecodeStatus::Ok as i32
    }

    /// How many bytes the output buffer of `job` must hold for this decoder.
    /// [`decode_for_gpu_sdk`](GpuDecoder::decode_for_gpu_sdk) sizes the
    /// job's internal buffer with it; call it to pre-allocate or pool buffers.
    ///
    /// This is not the [`AsyncDecoder`]'s size: the frame stays compressed
    /// until the GPU debayer, so the size can be smaller or larger than the
    /// [`AsyncDecoder`]'s, depending on the mode.
    ///
    /// The result depends on the job's clip and mode, so set both first.
    /// It is 0 if the job has no clip or its [`Clip`] has nothing loaded.
    pub fn size_buffer_needed(job: &AsyncDecompressJob) -> usize {
        // The SDK requires the job's clip to be set.
        if job.clip.is_none() {
            return 0;
        }
        let job_ptr = job.as_mut_ptr();
        cpp!(unsafe [job_ptr as "const R3DSDK::AsyncDecompressJob *"] -> usize as "size_t" {
            return R3DSDK::GpuDecoder::GetSizeBufferNeeded(*job_ptr);
        })
    }

    /// Queue `job`'s frame to be prepared so that the GPU debayer can both
    /// decompress and process it.
    /// The decompress will be scheduled and the function will immediately return a future.
    ///
    /// You should await the returned future to get the status of the decompress operation.
    /// On success, the future gives you back ownership of the job you passed in, so you can re-use it if needed.
    ///
    /// The job's internal buffer is sized for this decoder, see
    /// [`AsyncDecompressJob::set_internal_buffer`].
    ///
    /// # Errors
    /// Fails right away with [`RedError::NoClipOpen`] if the job has no
    /// loaded clip, or with the error the SDK reports if it refuses the job,
    /// e.g. [`RedError::OutputBufferInvalid`] for a buffer passed to
    /// [`set_output_buffer`](AsyncDecompressJob::set_output_buffer) that is
    /// too small. If the decode itself fails, the future resolves to that
    /// error and the job is dropped.
    pub fn decode_for_gpu_sdk(&self, job: AsyncDecompressJob) -> RedResult<CallbackFuture<AsyncDecompressJob>> {
        let ptr = self.inner.ptr;
        submit_decompress(job, self.inner.clone(), Self::size_buffer_needed, |job_ptr, state_ptr, callback_ptr| {
            cpp!(unsafe [ptr as "R3DSDK::GpuDecoder *", job_ptr as "R3DSDK::AsyncDecompressJob *", state_ptr as "void *", callback_ptr as "void *"] -> i32 as "int" {
                job_ptr->Callback = (R3DSDK::AsyncDecompressJob::AsyncDecompressCallback)callback_ptr;
                job_ptr->PrivateData = state_ptr;
                return ptr->DecodeForGpuSdk(*job_ptr);
            })
        })
    }
}

/// Hand `job` to the SDK through `submit`, which installs the callback and
/// private data on the job and returns the SDK status. `decoder` is kept alive
/// until the job completes, and `size_buffer_needed` is its buffer size helper.
fn submit_decompress(
    mut job: AsyncDecompressJob,
    decoder: Arc<dyn Send + Sync>,
    size_buffer_needed: fn(&AsyncDecompressJob) -> usize,
    submit: impl FnOnce(*mut c_void, *mut c_void, *mut c_void) -> i32,
) -> RedResult<CallbackFuture<AsyncDecompressJob>> {
    if job.clip.as_ref().is_none_or(|clip| clip.status() != LoadStatus::ClipLoaded) {
        return Err(RedError::NoClipOpen);
    }
    // The two decoders need different sizes, so the size is only known now.
    job.fit_internal_buffer(size_buffer_needed(&job))?;
    if !job.has_output_buffer() {
        return Err(RedError::OutputBufferInvalid);
    }
    // The SDK never clears `AbortDecode`, and fails a job submitted with it
    // set, so a job handed back after an abort request would fail when
    // reused. Every submission starts without one, as in RED's samples.
    job.set_abort_decode(false);
    let job_ptr = job.as_mut_ptr();
    let state = Arc::new(State::new(job, Some(decoder)));
    // Hand the SDK an *owned* refcount (reclaimed in the callback via
    // `Arc::from_raw`). This keeps `State` — and the job + output
    // buffer it holds — alive even if the caller drops the future
    // before the callback fires; otherwise the SDK would write into
    // freed memory. See `State` docs.
    let state_ptr = Arc::into_raw(state.clone()) as *mut c_void;
    let callback_ptr = async_decode_callback as extern "C" fn(*mut c_void, i32) as *mut c_void;

    if let Err(e) = check_status::<DecodeStatus>(submit(job_ptr, state_ptr, callback_ptr)) {
        // The job was never queued, so the callback will not fire —
        // reclaim the refcount we handed the SDK to avoid a leak.
        drop(unsafe { Arc::from_raw(state_ptr as *const State<AsyncDecompressJob>) });
        return Err(e);
    }
    Ok(CallbackFuture { state })
}

/// A frame to decompress with [`AsyncDecoder::decode_for_gpu_sdk`] or
/// [`GpuDecoder::decode_for_gpu_sdk`].
pub struct AsyncDecompressJob {
    ptr: *mut c_void,
    /// The clip the SDK reads from, kept alive with the job.
    clip: Option<Arc<Clip>>,
    internal_buffer: Option<AlignedBuffer>,
    metadata_allocated: bool,
    _sdk: Sdk,
}
// SAFETY: `Send` only — a single-owner cursor-state mutator, consumed once per
// submission then dropped or recycled by the caller. Concurrent `&self` access
// would race the SDK's internal job state, so it is deliberately not `Sync`
// (pinned negative in `tests/send_sync_surface.rs`).
unsafe impl Send for AsyncDecompressJob {}
impl AsyncDecompressJob {
    /// Create a job for frame 0 of the main track, in
    /// [`VideoDecodeMode::FullResPremium`] like a new
    /// [`R3dDecodeJob`](crate::R3dDecodeJob). Set its clip before submitting it.
    pub fn new(sdk: &Sdk) -> Self {
        let ptr = cpp!(unsafe [] -> *mut c_void as "R3DSDK::AsyncDecompressJob *" {
            R3DSDK::AsyncDecompressJob *job = new R3DSDK::AsyncDecompressJob();
            // The SDK leaves `Mode` at 0, which is not a decode mode.
            job->Mode = R3DSDK::DECODE_FULL_RES_PREMIUM;
            return job;
        });
        Self { ptr, clip: None, internal_buffer: None, metadata_allocated: false, _sdk: sdk.clone() }
    }
    /// Pointer to the underlying `R3DSDK::AsyncDecompressJob`.
    pub fn as_mut_ptr(&self) -> *mut c_void {
        self.ptr
    }
}
impl Drop for AsyncDecompressJob {
    fn drop(&mut self) {
        let self_ptr = self.ptr;
        let md = self.metadata_allocated;
        cpp!(unsafe [self_ptr as "R3DSDK::AsyncDecompressJob *", md as "bool"] {
            if (md) {
                delete self_ptr->OutputFrameMetadata;
                self_ptr->OutputFrameMetadata = nullptr;
            }
            delete self_ptr;
        });
    }
}

impl AsyncDecompressJob {
    /// The clip to decode from. It must be set before submitting the job.
    pub fn set_clip(&mut self, clip: &Arc<Clip>) {
        let self_ptr = self.ptr;
        let clip_ptr = clip.as_mut_ptr();
        cpp!(unsafe [self_ptr as "R3DSDK::AsyncDecompressJob *", clip_ptr as "R3DSDK::Clip *"] {
            self_ptr->Clip = clip_ptr;
        });
        self.clip = Some(clip.clone());
    }
    /// The clip set by [`set_clip`](Self::set_clip).
    pub fn clip(&self) -> Option<&Arc<Clip>> {
        self.clip.as_ref()
    }

    /// Set the SDK's request to stop decoding this job early. Only meaningful
    /// while the job is in flight, so it is reached through
    /// [`CallbackFuture::abort`].
    fn set_abort_decode(&mut self, abort: bool) {
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "R3DSDK::AsyncDecompressJob *", abort as "bool"] { self_ptr->AbortDecode = abort; })
    }

    /// The video track to decode: 0 for the main (A) track, or 1 for the
    /// X track, the extra highlight-protection track of HDRx clips shot on
    /// EPIC/Scarlet-X. There is no HDRx blending on the GPU path.
    pub fn set_video_track_no(&mut self, v: usize) {
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "R3DSDK::AsyncDecompressJob *", v as "size_t"] { self_ptr->VideoTrackNo = v; })
    }
    pub fn set_video_frame_no(&mut self, v: usize) {
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "R3DSDK::AsyncDecompressJob *", v as "size_t"] { self_ptr->VideoFrameNo = v; })
    }
    pub fn video_frame_no(&self) -> usize {
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "const R3DSDK::AsyncDecompressJob *"] -> usize as "size_t" { return self_ptr->VideoFrameNo; })
    }
    /// The decode mode set by [`set_mode`](Self::set_mode).
    pub fn mode(&self) -> VideoDecodeMode {
        let self_ptr = self.ptr;
        enum_from_sdk(cpp!(unsafe [self_ptr as "const R3DSDK::AsyncDecompressJob *"] -> i32 as "int" { return (int)self_ptr->Mode; }))
    }
    /// Pick the resolution and quality, and so the speed, of the decode.
    /// The mode changes the output buffer size, so set it before sizing or
    /// allocating the buffer.
    pub fn set_mode(&mut self, v: VideoDecodeMode) {
        let v: i32 = v as i32;
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "R3DSDK::AsyncDecompressJob *", v as "int"] { self_ptr->Mode = (R3DSDK::VideoDecodeMode)v; })
    }

    /// Point the output at a caller-owned buffer. Prefer
    /// [`set_internal_buffer`](Self::set_internal_buffer), which keeps the
    /// buffer alive for as long as the SDK may write to it.
    ///
    /// # Safety
    /// `buf` must be 16-byte aligned and valid for writes of `size` bytes
    /// until every decode of this job has completed: until the future
    /// returned by `decode_for_gpu_sdk` resolves, or, if it is dropped first,
    /// until the SDK is done with the job, which is only known once the job
    /// itself is dropped.
    pub unsafe fn set_output_buffer(&mut self, buf: *mut c_void, size: usize) {
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "R3DSDK::AsyncDecompressJob *", buf as "void *", size as "size_t"] {
            self_ptr->OutputBuffer = buf;
            self_ptr->OutputBufferSize = size;
        })
    }
    fn has_output_buffer(&self) -> bool {
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "const R3DSDK::AsyncDecompressJob *"] -> bool as "bool" {
            return self_ptr->OutputBuffer != nullptr && self_ptr->OutputBufferSize != 0;
        })
    }
    /// Give the job ownership of `buf` and point the output buffer at it,
    /// replacing any previous internal buffer. Use this to recycle buffers;
    /// a job without a buffer gets one when it is submitted.
    ///
    /// The buffer then lives exactly as long as the job. A submitted job is
    /// co-owned by the SDK until its completion callback, so the SDK can never
    /// write into a buffer that was freed under it.
    ///
    /// Submitting the job checks the internal buffer against the
    /// `size_buffer_needed` of the decoder it goes to
    /// ([`AsyncDecoder::size_buffer_needed`] or
    /// [`GpuDecoder::size_buffer_needed`], which differ), and replaces a
    /// missing, smaller or not 16-byte aligned one with a new buffer of
    /// that size.
    pub fn set_internal_buffer(&mut self, mut buf: AlignedBuffer) {
        // SAFETY: the job owns `buf` from here on, see above.
        unsafe { self.set_output_buffer(buf.as_mut_ptr(), buf.len()) };
        self.internal_buffer = Some(buf);
    }

    /// Before submitting, give the job an internal buffer of `needed` bytes
    /// unless its current one is large enough and aligned, or the caller set
    /// its own with [`set_output_buffer`](Self::set_output_buffer), which the
    /// SDK checks itself. A `needed` of 0 (no loaded clip) changes nothing.
    fn fit_internal_buffer(&mut self, needed: usize) -> RedResult<()> {
        let self_ptr = self.ptr;
        let output: *const c_void = cpp!(unsafe [self_ptr as "const R3DSDK::AsyncDecompressJob *"] -> *const c_void as "const void *" {
            return self_ptr->OutputBuffer;
        });
        let internal = self.internal_buffer.as_ref();
        let caller_owned = !output.is_null() && internal.is_none_or(|buf| buf.as_ptr() != output);
        let fits = internal.is_some_and(|buf| buf.len() >= needed && (buf.as_ptr() as usize).is_multiple_of(16));
        if needed != 0 && !caller_owned && !fits {
            self.set_internal_buffer(AlignedBuffer::new(needed, 16)?);
        }
        Ok(())
    }

    /// The buffer set by [`set_internal_buffer`](Self::set_internal_buffer)
    /// or allocated when the job was submitted.
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

    /// Allocate internal metadata object to receive per-frame metadata.
    /// Call this before submitting the job if you want to receive metadata.
    pub fn allocate_frame_metadata(&mut self) {
        if self.metadata_allocated {
            // Already allocated
            return;
        }
        let self_ptr = self.ptr;
        self.metadata_allocated = true;
        cpp!(unsafe [self_ptr as "R3DSDK::AsyncDecompressJob *"] {
            self_ptr->OutputFrameMetadata = new R3DSDK::Metadata();
        })
    }
    /// The metadata the SDK recorded for the frame this job decoded. Fails
    /// with [`RedError::Other`] unless
    /// [`allocate_frame_metadata`](Self::allocate_frame_metadata) was called.
    pub fn metadata(&self) -> RedResult<&Metadata> {
        if !self.metadata_allocated {
            return Err(RedError::Other("Metadata was not requested in the job".to_string()));
        }
        let self_ptr = self.ptr;
        let ptr: *const Metadata = cpp!(unsafe [self_ptr as "const R3DSDK::AsyncDecompressJob *"] -> *const Metadata as "void *" {
            return self_ptr->OutputFrameMetadata;
        });
        // SAFETY: the `Metadata` allocated above, owned by this job.
        Ok(unsafe { &*ptr })
    }
}

extern "C" fn async_decode_callback(job: *mut c_void /* AsyncDecompressJob * */, decode_status: i32) {
    if job.is_null() {
        log::error!("Job pointer is null in callback.");
        return;
    }
    let ud: *mut c_void = cpp!(unsafe [job as "R3DSDK::AsyncDecompressJob *"] -> *mut c_void as "void *" {
        return job->PrivateData;
    });
    if ud.is_null() {
        log::error!("No user data in job.");
        return;
    }

    // Reclaim the owned refcount handed to the SDK at submit time. `State` is
    // freed only once the future's clone is also gone, so a future dropped
    // before this point does not cause a use-after-free. The job stays in the
    // slot — `poll` hands it back; we only record the status here.
    let state = unsafe { Arc::from_raw(ud as *const State<AsyncDecompressJob>) };
    *state.status.lock().unwrap() = Some(check_status::<DecodeStatus>(decode_status));
    state.done.store(true, Ordering::Release);
    state.waker.wake();
    // If the future is gone this is the last reference, and dropping it here
    // would delete `job` while the SDK is still inside its callback.
    release_from_callback(state);
}

impl CallbackFuture<AsyncDecompressJob> {
    /// Ask the SDK to stop this in-flight decode early, by setting the job's
    /// `AbortDecode` flag. No-op once the result has been observed.
    ///
    /// The request is only a hint, and the decode may finish anyway: SDK 9.2
    /// ignores it for every job it has already accepted, so the future then
    /// resolves to the complete frame as usual. A decode the SDK does stop
    /// resolves to an error. Either way the SDK completes every job, so keep
    /// awaiting the future: dropping it is memory-safe (the SDK co-owns the
    /// job), but the job and its buffer stay in use until the SDK is done.
    ///
    /// A job handed back after an abort request can be reused: every
    /// submission clears the request.
    pub fn abort(&self) {
        if let Some(job) = self.state.job.lock().unwrap().as_mut() {
            job.set_abort_decode(true);
        }
    }
}
