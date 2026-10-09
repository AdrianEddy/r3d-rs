// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use crate::{ RedResult, RedError,  clip::AlignedBuffer, enums::*, future::*, metadata::* };
use core::ffi::c_void;
use std::sync::{ Arc, atomic::Ordering };
use cpp::*;

cpp!{{
	#include "R3DSDK.h"
}}

/// Class for asynchronously decompressing a frame for further processing on the GPU.
pub struct AsyncDecoder {
    ptr: *mut core::ffi::c_void,
}
// SAFETY: `Send + Sync`. `ptr` is an opaque owned `R3DSDK::AsyncDecoder *`
// with no thread affinity, and `decode_for_gpu_sdk` is part of the SDK's
// documented thread-safe decode surface (the lane debayers' `process_blocking`
// is documented "Thread safe blocking call"). Pinned by `assert_impl_all!` in
// `tests/send_sync_surface.rs`.
unsafe impl Send for AsyncDecoder {}
unsafe impl Sync for AsyncDecoder {}

impl AsyncDecoder {
	/// Create and open the asynchronous decoder with the specified number of threads
	/// If threads_no is set to 0, threads_available() number of threads will be used.
    pub fn new(threads_no: usize) -> Self {
        let ptr = cpp!(unsafe [threads_no as "size_t"] -> *mut core::ffi::c_void as "R3DSDK::AsyncDecoder *" {
            R3DSDK::AsyncDecoder *ptr = new R3DSDK::AsyncDecoder();
            ptr->Open(threads_no);
            return ptr;
        });
        Self { ptr }
    }
    pub fn as_mut_ptr(&self) -> *mut core::ffi::c_void {
        self.ptr
    }

	/// Returns the number of threads the decoder will use if opened when
	/// noOfThreads in Open() is set to 0 (the default).
    pub fn threads_available() -> usize {
        cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::AsyncDecoder::ThreadsAvailable(); })
    }

	/// Returns the size OutputBufferSize needs to be in the AsyncDecompressJob
	/// so the appropriate size input buffer can be allocated before calling
	/// the DecodeForGpuSdk() function. The following must be set on input:
	///  1) job.Clip must point to an open Clip and cannot be NULL
	///  2) job.Mode must be set appropriately
    ///
	/// returns 0 if these input parameters are invalid or no clip is loaded
    pub fn size_buffer_needed(job: &AsyncDecompressJob) -> usize {
        let job_ptr = job.as_mut_ptr();
        cpp!(unsafe [job_ptr as "const R3DSDK::AsyncDecompressJob *"] -> usize as "size_t" {
            return R3DSDK::AsyncDecoder::GetSizeBufferNeeded(*job_ptr);
        })
    }

	/// Decompress a frame in to the supplied OutputBuffer for further processing on the GPU.
    /// The decompress will be scheduled and the function will immediately return a future.
    ///
    /// You should await the returned future to get the status of the decompress operation.
    /// The future gives you back ownership of the job you passed in, so you can re-use it if needed.
    pub fn decode_for_gpu_sdk(&self, job: AsyncDecompressJob) -> RedResult<CallbackFuture<AsyncDecompressJob>> {
        unsafe {
            let ptr = self.ptr;
            let job_ptr = job.as_mut_ptr();

            let state = Arc::new(State::new(job));
            // Hand the SDK an *owned* refcount (reclaimed in the callback via
            // `Arc::from_raw`). This keeps `State` — and the job + output
            // buffer it holds — alive even if the caller drops the future
            // before the callback fires; otherwise the SDK would write into
            // freed memory. See `State` docs.
            let state_ptr = Arc::into_raw(state.clone()) as *mut c_void;

            let callback_ptr = async_decode_callback as extern "C" fn(*mut c_void, DecodeStatus);

            let status: DecodeStatus = std::mem::transmute(cpp!([ptr as "R3DSDK::AsyncDecoder *", job_ptr as "R3DSDK::AsyncDecompressJob *", state_ptr as "void *", callback_ptr as "void *"] -> i32 as "int" {
                job_ptr->Callback = (R3DSDK::AsyncDecompressJob::AsyncDecompressCallback)callback_ptr;
                job_ptr->PrivateData = state_ptr;
                return ptr->DecodeForGpuSdk(*job_ptr);
            }));
            if status != DecodeStatus::Ok {
                // The job was never queued, so the callback will not fire —
                // reclaim the refcount we handed the SDK to avoid a leak.
                drop(Arc::from_raw(state_ptr as *const State<AsyncDecompressJob>));
                Err(RedError::from(status))
            } else {
                Ok(CallbackFuture { state })
            }
        }
    }
}
impl Drop for AsyncDecoder {
    fn drop(&mut self) {
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "R3DSDK::AsyncDecoder *"] {
            self_ptr->Close();
            delete self_ptr;
        });
    }
}

/// Class for asynchronously processing a frame for later decompression
/// and image processing through CUDA (Windows & Linux) or Metal (macOS).
/// NOTE: 4GB minimum video RAM recommended when using GPU decode
pub struct GpuDecoder {
    ptr: *mut core::ffi::c_void,
}
// SAFETY: `Send + Sync`, same rationale as `AsyncDecoder` above — opaque owned
// handle, `&self` decode entry point on the documented thread-safe surface.
unsafe impl Send for GpuDecoder {}
unsafe impl Sync for GpuDecoder {}

impl GpuDecoder {
	/// Create and open the asynchronous decoder.
    pub fn new() -> Self {
        let ptr = cpp!(unsafe [] -> *mut core::ffi::c_void as "R3DSDK::GpuDecoder *" {
            R3DSDK::GpuDecoder *ptr = new R3DSDK::GpuDecoder();
            ptr->Open();
            return ptr;
        });
        Self { ptr }
    }
    pub fn as_mut_ptr(&self) -> *mut core::ffi::c_void {
        self.ptr
    }

	/// Determines if the current clip is supported for GPU decompression.
	/// All clips except for ones recorded on RED ONE are supported.
    pub fn decode_supported_for_clip(clip: &crate::Clip) -> bool {
        let clip_ptr = clip.as_mut_ptr();
        unsafe {
            let status: DecodeStatus = std::mem::transmute(cpp!([clip_ptr as "const R3DSDK::Clip *"] -> i32 as "int" {
                return (R3DSDK::DecodeStatus)R3DSDK::GpuDecoder::DecodeSupportedForClip(*clip_ptr);
            }));
            status == DecodeStatus::Ok
        }
    }

	/// Returns the size OutputBufferSize needs to be in the AsyncDecompressJob
	/// so the appropriate size input buffer can be allocated before calling
	/// the DecodeForGpuSdk() function. The following must be set on input:
	///  1) job.Clip must point to an open Clip and cannot be NULL
	///  2) job.Mode must be set appropriately
	/// returns 0 if these input parameters are invalid or no clip is loaded
    pub fn size_buffer_needed(job: &AsyncDecompressJob) -> usize {
        let job_ptr = job.as_mut_ptr();
        cpp!(unsafe [job_ptr as "const R3DSDK::AsyncDecompressJob *"] -> usize as "size_t" {
            return R3DSDK::GpuDecoder::GetSizeBufferNeeded(*job_ptr);
        })
    }

	/// Prepare a frame for decompression & further processing on the GPU.
    /// The decompress will be scheduled and the function will immediately return a future.
    ///
    /// You should await the returned future to get the status of the decompress operation.
    /// The future gives you back ownership of the job you passed in, so you can re-use it if needed.
    pub fn decode_for_gpu_sdk(&self, job: AsyncDecompressJob) -> RedResult<CallbackFuture<AsyncDecompressJob>> {
        unsafe {
            let ptr = self.ptr;
            let job_ptr = job.as_mut_ptr();

            let state = Arc::new(State::new(job));
            // Owned refcount handed to the SDK; reclaimed in the callback via
            // `Arc::from_raw`. Keeps `State` + the job/output buffer alive
            // across an early future drop — see `State` docs.
            let state_ptr = Arc::into_raw(state.clone()) as *mut c_void;

            let callback_ptr = async_decode_callback as extern "C" fn(*mut c_void, DecodeStatus);

            let status: DecodeStatus = std::mem::transmute(cpp!([ptr as "R3DSDK::GpuDecoder *", job_ptr as "R3DSDK::AsyncDecompressJob *", state_ptr as "void *", callback_ptr as "void *"] -> i32 as "int" {
                job_ptr->Callback = (R3DSDK::AsyncDecompressJob::AsyncDecompressCallback)callback_ptr;
                job_ptr->PrivateData = state_ptr;
                return ptr->DecodeForGpuSdk(*job_ptr);
            }));
            if status != DecodeStatus::Ok {
                // Job never queued → no callback → reclaim to avoid a leak.
                drop(Arc::from_raw(state_ptr as *const State<AsyncDecompressJob>));
                Err(RedError::from(status))
            } else {
                Ok(CallbackFuture { state })
            }
        }
    }
}
impl Drop for GpuDecoder {
    fn drop(&mut self) {
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "R3DSDK::GpuDecoder *"] {
            self_ptr->Close();
            delete self_ptr;
        });
    }
}

pub struct AsyncDecompressJob {
    ptr: *mut core::ffi::c_void,
    internal_buffer: Option<AlignedBuffer>,
    metadata_allocated: bool,
}
// SAFETY: `Send` only — a single-owner cursor-state mutator, consumed once per
// submission then dropped or recycled by the caller. Concurrent `&self` access
// would race the SDK's internal job state, so it is deliberately not `Sync`
// (pinned negative in `tests/send_sync_surface.rs`).
unsafe impl Send for AsyncDecompressJob {}
impl AsyncDecompressJob {
    pub fn new() -> Self {
        unsafe {
            let mut ptr = std::ptr::null_mut();
            let ptrptr = &mut ptr;
            cpp!([ptrptr as "R3DSDK::AsyncDecompressJob **"] { *ptrptr = new R3DSDK::AsyncDecompressJob(); });
            Self { ptr, internal_buffer: None, metadata_allocated: false }
        }
    }
    pub fn as_mut_ptr(&self) -> *mut core::ffi::c_void {
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
    pub fn set_clip(&mut self, clip: &crate::clip::Clip) {
        let self_ptr = self.ptr;
        let clip_ptr = clip.as_mut_ptr();
		cpp!(unsafe [self_ptr as "R3DSDK::AsyncDecompressJob *", clip_ptr as "R3DSDK::Clip *"] {
			self_ptr->Clip = clip_ptr;
		})
    }

    /// Call this if you want to abort processing this frame as soon as possible
    pub fn abort(&mut self) {
        let self_ptr = self.ptr;
        cpp!(unsafe [self_ptr as "R3DSDK::AsyncDecompressJob *"] { self_ptr->AbortDecode = true; })
    }

    /// 0 = main (A) track, 1 = EPIC/Scarlet-X higlight protection track 2 (X track)
	/// ignored when doing HDRx blending
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
	// item is the pointer to the original item as submitted in the Decode() call
	// typedef void (*DecodeCallback)(AsyncDecompressJob * item, R3DStatus decodeStatus);
	// DecodeCallback				callback;
	// void *						privateData;			// available for your use as you see fit, R3D SDK will not touch this variable!


	/// Resolution/speed to decode the image at. This will also
	/// influence how much memory is needed for the output buffer
	pub fn mode(&self) -> VideoDecodeMode {
        let self_ptr = self.ptr;
		unsafe { std::mem::transmute(cpp!([self_ptr as "const R3DSDK::AsyncDecompressJob *"] -> i32 as "int" { return (int)self_ptr->Mode; })) }
	}
	/// Resolution/speed to decode the image at. This will also
	/// influence how much memory is needed for the output buffer
	pub fn set_mode(&mut self, v: VideoDecodeMode) {
		let v: i32 = v as i32;
        let self_ptr = self.ptr;
		cpp!(unsafe [self_ptr as "R3DSDK::AsyncDecompressJob *", v as "int"] { self_ptr->Mode = (R3DSDK::VideoDecodeMode)v; })
	}

	/// Pointer to the buffer to store the image in. This cannot be
	/// NULL otherwise the decode will fail. The buffer must be aligned
	/// on a 16-byte boundary (see sample code).
	pub fn set_output_buffer(&mut self, buf: *mut core::ffi::c_void, size: usize) {
        let self_ptr = self.ptr;
		cpp!(unsafe [self_ptr as "R3DSDK::AsyncDecompressJob *", buf as "void *", size as "size_t"] {
			self_ptr->OutputBuffer = buf;
			self_ptr->OutputBufferSize = size;
		})
	}
	/// Allocate internal buffer for the output image.
    /// Always aligned to 1024 bytes.
	pub fn allocate_internal_buffer(&mut self) -> RedResult<()> {
        let size = AsyncDecoder::size_buffer_needed(self);
        self.set_internal_buffer(AlignedBuffer::new(size, 16)?);
        Ok(())
	}

    /// Give the job ownership of `buf` and point the output buffer at it,
    /// replacing any previous internal buffer. `buf` must be 16-byte aligned
    /// and at least as large as the decoder's `size_buffer_needed`.
    ///
    /// Unlike [`set_output_buffer`](Self::set_output_buffer), the buffer then
    /// lives exactly as long as the job. A submitted job is co-owned by the
    /// SDK until its completion callback, so the SDK can never write into a
    /// buffer that was freed under it.
    pub fn set_internal_buffer(&mut self, buf: AlignedBuffer) {
        self.set_output_buffer(buf.ptr, buf.len());
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
            self.set_output_buffer(core::ptr::null_mut(), 0);
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
    /// Optionally return the decoded frame's per-frame metadata
    pub fn metadata(&self) -> RedResult<&Metadata> {
        if !self.metadata_allocated {
            return Err(RedError::Other("Metadata was not requested in the job".to_string()));
        }
        let self_ptr = self.ptr;
        let ptr = cpp!(unsafe [self_ptr as "const R3DSDK::AsyncDecompressJob *"] -> *const core::ffi::c_void as "void *" {
            return self_ptr->OutputFrameMetadata;
        });
        Ok(unsafe { &*(ptr as *const Metadata) })
    }
}

extern "C" fn async_decode_callback(job: *mut c_void /* AsyncDecompressJob * */, decode_status: DecodeStatus) {
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
    *state.status.lock().unwrap() = Some(if decode_status == DecodeStatus::Ok {
        Ok(())
    } else {
        Err(RedError::from(decode_status))
    });
    state.done.store(true, Ordering::Release);
    state.waker.wake();
    // If the future is gone this is the last reference, and dropping it here
    // would delete `job` while the SDK is still inside its callback.
    release_from_callback(state);
}

impl CallbackFuture<AsyncDecompressJob> {
    /// Request the SDK abort this in-flight decode as soon as possible
    /// (sets `AsyncDecompressJob::AbortDecode`).
    ///
    /// The SDK still delivers exactly one completion callback per submitted
    /// job — an aborted frame completes with [`DecodeStatus::Cancelled`] —
    /// so after `abort()` you must still drive the future to completion
    /// (await it). **Do not** just drop the future: dropping is memory-safe
    /// (the SDK co-owns the `State`), but the decode keeps running until it
    /// finishes or notices the flag, so awaiting is how you reclaim the slot
    /// promptly. No-op if the result has already been observed.
    pub fn abort(&self) {
        if let Some(job) = self.state.job.lock().unwrap().as_mut() {
            job.abort();
        }
    }
}
