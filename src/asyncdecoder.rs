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
            let state_ptr = Arc::as_ptr(&state) as *mut c_void;

            let callback_ptr = async_decode_callback as extern "C" fn(*mut c_void, DecodeStatus);

            let status: DecodeStatus = std::mem::transmute(cpp!([ptr as "R3DSDK::AsyncDecoder *", job_ptr as "R3DSDK::AsyncDecompressJob *", state_ptr as "void *", callback_ptr as "void *"] -> i32 as "int" {
                job_ptr->Callback = (R3DSDK::AsyncDecompressJob::AsyncDecompressCallback)callback_ptr;
                job_ptr->PrivateData = state_ptr;
                return ptr->DecodeForGpuSdk(*job_ptr);
            }));
            if status != DecodeStatus::DSDecodeOK {
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
            status == DecodeStatus::DSDecodeOK
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
            let state_ptr = Arc::as_ptr(&state) as *mut c_void;

            let callback_ptr = async_decode_callback as extern "C" fn(*mut c_void, DecodeStatus);

            let status: DecodeStatus = std::mem::transmute(cpp!([ptr as "R3DSDK::GpuDecoder *", job_ptr as "R3DSDK::AsyncDecompressJob *", state_ptr as "void *", callback_ptr as "void *"] -> i32 as "int" {
                job_ptr->Callback = (R3DSDK::AsyncDecompressJob::AsyncDecompressCallback)callback_ptr;
                job_ptr->PrivateData = state_ptr;
                return ptr->DecodeForGpuSdk(*job_ptr);
            }));
            if status != DecodeStatus::DSDecodeOK {
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
        self.internal_buffer = Some(AlignedBuffer::new(size, 16)?);
        let buf = self.internal_buffer.as_ref().unwrap();
        self.set_output_buffer(buf.ptr, buf.len());
        Ok(())
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

    // Safety: `ud` points to the State inside an Arc held by the Future.
    let state: &mut State<AsyncDecompressJob> = unsafe { &mut *(ud as *mut State<AsyncDecompressJob>) };

    // Store the result and signal completion
    {
        let mut lock = state.result.lock().unwrap();
        let org_job = state.job.take().unwrap();
        if decode_status == DecodeStatus::DSDecodeOK {
            *lock = Some(Ok(org_job));
        } else {
            *lock = Some(Err(RedError::from(decode_status)));
        }
    }
    state.done.store(true, Ordering::Release);
    state.waker.wake();
}
