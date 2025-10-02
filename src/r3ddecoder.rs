// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use crate::{ RedResult, RedError,  clip::AlignedBuffer, enums::*, future::*, metadata::* };
use core::ffi::c_void;
use std::sync::{ Arc, atomic::Ordering };
use cpp::*;

cpp! {{
    #include "R3DSDKDecoder.h"
}}

pub struct R3dDecoder {
    ptr: *mut core::ffi::c_void
}
impl R3dDecoder {
    pub fn new(options: &R3dDecoderOptions) -> RedResult<Self> {
        unsafe {
            let mut ptr = std::ptr::null_mut();
            let ptrptr = &mut ptr;
            let options_ptr = options.ptr;
            let status: R3DStatus = std::mem::transmute(cpp!([options_ptr as "R3DSDK::R3DDecoderOptions *", ptrptr as "R3DSDK::R3DDecoder **"] -> i32 as "int" { return R3DSDK::R3DDecoder::CreateDecoder(options_ptr, ptrptr); }));
            if status != R3DStatus::Ok {
                Err(RedError::from(status))
            } else {
                Ok(Self { ptr })
            }
        }
    }

    pub fn as_mut_ptr(&self) -> *mut core::ffi::c_void {
        self.ptr
    }

    /// Asynchronously decode a single frame as specified in the job.
    /// The decode will be scheduled and the function will immediately return a future.
    ///
    /// You should await the returned future to get the status of the decode operation.
    /// The future gives you back ownership of the job you passed in, so you can re-use it if needed.
    pub fn decode(&self, job: R3dDecodeJob) -> RedResult<CallbackFuture<R3dDecodeJob>> {
        unsafe {
            let ptr = self.ptr;
            let job_ptr = job.ptr;

            let state = Arc::new(State::new(job));
            let state_ptr = Arc::as_ptr(&state) as *mut c_void;

            let callback_ptr = decode_callback as extern "C" fn(*mut c_void, R3DStatus);

            let status: R3DStatus = std::mem::transmute(cpp!([ptr as "R3DSDK::R3DDecoder *", job_ptr as "R3DSDK::R3DDecodeJob *", state_ptr as "void *", callback_ptr as "void *"] -> i32 as "int" {
                job_ptr->callback = (R3DSDK::R3DDecodeJob::DecodeCallback)callback_ptr;
                job_ptr->privateData = state_ptr;
                return ptr->decode(job_ptr);
            }));
            if status != R3DStatus::Ok {
                Err(RedError::from(status))
            } else {
                Ok(CallbackFuture { state })
            }
        }
    }
}

impl Drop for R3dDecoder {
    fn drop(&mut self) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::R3DDecoder *"] {
            R3DSDK::R3DDecoder::ReleaseDecoder(ptr);
        });
    }
}

/// Class for loading a clip, retrieving information and extracting images.
/// This class is thread-safe as long as no call to LoadFrom() or Close() is in progress.
///
/// The set routines on this class are available to tweak the performance characteristics of the R3DDecoder.
/// The SDK will automatically choose sane defaults based upon which devices are chosen with the useDevice call, however if you need to restrict the SDK for a specific use case you can use the set routines to do so.
/// The defaults the SDK chooses are usually core count - 1 for threads, and min(device memory * 0.75, concurrent GPU Frames * 1GB) for GPUs, for host memory the value is by default set to concurrentImageCount * 512MB,
/// note if the device of the host actually runs out of memory the out of memory status will be returned from either the callback or the decode call, based upon when it occurs.  Memory pools do not pre-allocate the memory.
pub struct R3dDecoderOptions {
    ptr: *mut core::ffi::c_void
}
impl R3dDecoderOptions {
    pub fn new() -> RedResult<Self> {
        unsafe {
            let mut ptr = std::ptr::null_mut();
            let ptrptr = &mut ptr;
            let status: R3DStatus = std::mem::transmute(cpp!([ptrptr as "R3DSDK::R3DDecoderOptions **"] -> i32 as "int" { return R3DSDK::R3DDecoderOptions::CreateOptions(ptrptr); }));
            if status != R3DStatus::Ok {
                Err(RedError::from(status))
            } else {
                Ok(Self { ptr })
            }
        }
    }

    pub fn as_mut_ptr(&self) -> *mut core::ffi::c_void {
        self.ptr
    }

	/// sets the folder for temporary files and compiled kernels.
	/// if set to empty string or not called no temporary folder will be used.
    /// On some older boards the kernel compilation may take upwards of 5 seconds, this is present to mitigate that and make it a one time occurance instaed of per application run.
    /// On modern boards the compilation time is hardly noticable.
    pub fn set_scratch_folder(&mut self, path: &str) -> RedResult<()> {
        let c_path = std::ffi::CString::new(path).unwrap();
        unsafe {
            let self_ptr = self.ptr;
            let c_ptr = c_path.as_ptr();
            let status: R3DStatus = std::mem::transmute(cpp!(unsafe [self_ptr as "R3DSDK::R3DDecoderOptions *", c_ptr as "const char *"] -> i32 as "int" { return self_ptr->setScratchFolder(std::string(c_ptr)); }));
            if status != R3DStatus::Ok {
                Err(RedError::from(status))
            } else {
                Ok(())
            }
        }
    }
    /// This is the number of active CPU threads used for decompression of the footage.
    /// This does not have a 1:1 ratio to the number of frames being processed at a time, a single frame in most cases will be split to operate on multiple of these task threads.
    /// We recommend 1 - core count to prevent full CPU usage keeping you from interacting with the computer while processing.
	///
    /// 0 = use (cores - 1)
    pub fn set_decompression_thread_count(&mut self, count: usize) -> RedResult<()> {
        unsafe {
            let self_ptr = self.ptr;
            let status: R3DStatus = std::mem::transmute(cpp!(unsafe [self_ptr as "R3DSDK::R3DDecoderOptions *", count as "size_t"] -> i32 as "int" { return self_ptr->setDecompressionThreadCount(count); }));
            if status != R3DStatus::Ok {
                Err(RedError::from(status))
            } else {
                Ok(())
            }
        }
    }
    /// This number determines the how many threads will be used to manage the frame(s) lifetime, from your request all the way to the complete callback.
    /// When you call decode, your request gets verified, prepared and then queued until one of these threads is available.
    /// For each of these threads some buffers will need to be allocated in host memory to handle the decompression step.
    ///
	/// 0 = use (cores - 1)
    pub fn set_concurrent_image_count(&mut self, count: usize) -> RedResult<()> {
        unsafe {
            let self_ptr = self.ptr;
            let status: R3DStatus = std::mem::transmute(cpp!(unsafe [self_ptr as "R3DSDK::R3DDecoderOptions *", count as "size_t"] -> i32 as "int" { return self_ptr->setConcurrentImageCount(count); }));
            if status != R3DStatus::Ok {
                Err(RedError::from(status))
            } else {
                Ok(())
            }
        }
    }
	/// minimum 1024 recommended 4096 or higher
    /// This value is used for allocations of Host memory, the value passed into here will need to scale for the value passed to setConcurrentImageCount.
    pub fn set_memory_pool_size(&mut self, size_mbs: usize) -> RedResult<()> {
        unsafe {
            let self_ptr = self.ptr;
            let status: R3DStatus = std::mem::transmute(cpp!(unsafe [self_ptr as "R3DSDK::R3DDecoderOptions *", size_mbs as "size_t"] -> i32 as "int" { return self_ptr->setMemoryPoolSize(size_mbs); }));
            if status != R3DStatus::Ok {
                Err(RedError::from(status))
            } else {
                Ok(())
            }
        }
    }

    /// minimum 1024 recommended 4096 or higher
    /// The memory pool is used to keep larger buffers on the device around to be re-used between frames.
    /// All of our GPU buffer types come from here, pinned, and temp processing buffers, if this is not high enough for the value passed to setGPUConcurrentFrameCount you will not be running as optimal as possible, though it should not be higher than what the graphics card has available.
	pub fn set_gpu_memory_pool_size(&mut self, size_mbs: usize) -> RedResult<()> {
        unsafe {
            let self_ptr = self.ptr;
            let status: R3DStatus = std::mem::transmute(cpp!(unsafe [self_ptr as "R3DSDK::R3DDecoderOptions *", size_mbs as "size_t"] -> i32 as "int" { return self_ptr->setGPUMemoryPoolSize(size_mbs); }));
            if status != R3DStatus::Ok {
                Err(RedError::from(status))
            } else {
                Ok(())
            }
        }
    }
    /// Sets number of cudaStreams or OpenCL cl_command_queue(s).
	/// minimum 1 - recommended 3 (if GPU has enough RAM)
    /// This value determines how many frames we will attempt to process on the GPU at a time, a frame consists of HostToDevice/ run kernels/ DeviceToHost.
    /// By increasing the number of frames you directly 1:1 increase memory requirement to process on the card.
    /// The memory requirement to process a single frame on the GPU is not light as RED deals with very large frame sizes, there are also several other utility buffers needed when processing the larger of them are as follows: upload pinned buffer, temp processing buffer, download pinned buffer.
    /// Due to the limitation of hardware only overlapping upto 3 of these actions, there is not much benefit to this number going above 3.
    /// More about the hardware limitation: Modern GPU Devices support at most processing of 3 actions at once by overlapping actions in different streams/command queues.
    /// The types of actions are: HostToDevice/Kerenel/DeviceToHost.
    /// The high end quadros can do one of each overlapped as they have 2 transfer engines and 1 kernel engine, where the Geforce line can only do 2 of them at a time meaning they cannot upload and download concurrently.
    pub fn set_gpu_concurrent_frame_count(&mut self, count: usize) -> RedResult<()> {
        unsafe {
            let self_ptr = self.ptr;
            let status: R3DStatus = std::mem::transmute(cpp!(unsafe [self_ptr as "R3DSDK::R3DDecoderOptions *", count as "size_t"] -> i32 as "int" { return self_ptr->setGPUConcurrentFrameCount(count); }));
            if status != R3DStatus::Ok {
                Err(RedError::from(status))
            } else {
                Ok(())
            }
        }
    }

	/// returns a list of opencl devices compatible with SDK
    pub fn opencl_device_list() -> RedResult<Vec<OpenCLDeviceInfo>> {
        unsafe {
            let mut list = Vec::<OpenCLDeviceInfo>::new();
            let list_ref = &mut list;
            //let list_ptr = list.as_mut_ptr();
            let status: R3DStatus = std::mem::transmute(cpp!(unsafe [list_ref as "void*"] -> i32 as "int" {
                std::vector<R3DSDK::OpenCLDeviceInfo> list;
                auto status = R3DSDK::R3DDecoderOptions::GetOpenCLDeviceList(list);
                for (const auto &item : list) {
                    rust!(opencl_list_add [list_ref: &mut Vec::<OpenCLDeviceInfo> as "void*", item: OpenCLDeviceInfo as "R3DSDK::OpenCLDeviceInfo"] {
                        list_ref.push(item);
                    });
                }
                return status;
            }));
            dbg!(list.len());
            if status != R3DStatus::Ok {
                Err(RedError::from(status))
            } else {
                Ok(list)
            }
        }
    }
	/// returns a list of opencl devices compatible with SDK
    pub fn cuda_device_list() -> RedResult<Vec<CudaDeviceInfo>> {
        unsafe {
            let mut list = Vec::<CudaDeviceInfo>::new();
            let list_ref = &mut list;
            let status: R3DStatus = std::mem::transmute(cpp!(unsafe [list_ref as "void*"] -> i32 as "int" {
                std::vector<R3DSDK::CudaDeviceInfo> list;
                auto status = R3DSDK::R3DDecoderOptions::GetCudaDeviceList(list);
                for (const auto &item : list) {
                    rust!(cuda_list_add [list_ref: &mut Vec::<CudaDeviceInfo> as "void*", item: CudaDeviceInfo as "R3DSDK::CudaDeviceInfo"] {
                        list_ref.push(item);
                    });
                }
                return status;
            }));
            if status != R3DStatus::Ok {
                Err(RedError::from(status))
            } else {
                Ok(list)
            }
        }
    }

	/// sets the opencl device to be used by the decoder
	/// cuda and opencl are mutually exclusive, you cannot combine usage of cuda and opencl on the same options instance
    pub fn use_opencl_device(&mut self, device: &OpenCLDeviceInfo) -> RedResult<()> {
        unsafe {
            let self_ptr = self.ptr;
            let status: R3DStatus = std::mem::transmute(cpp!(unsafe [self_ptr as "R3DSDK::R3DDecoderOptions *", device as "const R3DSDK::OpenCLDeviceInfo *"] -> i32 as "int" { return self_ptr->useDevice(*device); }));
            if status != R3DStatus::Ok {
                Err(RedError::from(status))
            } else {
                Ok(())
            }
        }
    }
	/// sets the cuda device to be used by the decoder
	/// cuda and opencl are mutually exclusive, you cannot combine usage of cuda and opencl on the same options instance
    pub fn use_cuda_device(&mut self, device: &CudaDeviceInfo) -> RedResult<()> {
        unsafe {
            let self_ptr = self.ptr;
            let status: R3DStatus = std::mem::transmute(cpp!(unsafe [self_ptr as "R3DSDK::R3DDecoderOptions *", device as "const R3DSDK::CudaDeviceInfo *"] -> i32 as "int" { return self_ptr->useDevice(*device); }));
            if status != R3DStatus::Ok {
                Err(RedError::from(status))
            } else {
                Ok(())
            }
        }
    }

}

cpp_class! {
    pub unsafe struct OpenCLDeviceInfo as "R3DSDK::OpenCLDeviceInfo"
}
impl OpenCLDeviceInfo {
    /// platform_id corresponds to a cl_platform_id which is actually _cl_platform_id *
    /// must be set to choose a specific device.
    pub fn platform_id(&self) -> *mut c_void {
        cpp!(unsafe [self as "const R3DSDK::OpenCLDeviceInfo *"] -> *mut c_void as "void *" { return self->platform_id; })
    }

    /// device_id corresponds to a cl_device_id which is actually _cl_device_id *
    /// must be set to choose a specific device.
    pub fn device_id(&self) -> *mut c_void {
        cpp!(unsafe [self as "const R3DSDK::OpenCLDeviceInfo *"] -> *mut c_void as "void *" { return self->device_id; })
    }

    /// used for feedback only; not used for any device selection logic.
    pub fn name(&self) -> String {
        unsafe {
            let c_str = std::ffi::CStr::from_ptr(cpp!(unsafe [self as "const R3DSDK::OpenCLDeviceInfo *"] -> *const core::ffi::c_char as "const char *" {
                return self->name;
            }));
            c_str.to_str().map(|x| x.to_string()).unwrap_or_default()
        }
    }
    /// used for feedback only; not used for any device selection logic.
    pub fn platform_name(&self) -> String {
        unsafe {
            let c_str = std::ffi::CStr::from_ptr(cpp!(unsafe [self as "const R3DSDK::OpenCLDeviceInfo *"] -> *const core::ffi::c_char as "const char *" {
                return self->platform_name;
            }));
            c_str.to_str().map(|x| x.to_string()).unwrap_or_default()
        }
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
    pub unsafe struct CudaDeviceInfo as "R3DSDK::CudaDeviceInfo"
}
impl CudaDeviceInfo {
    /// pci_bus_id value must be set to choose a specific device bus id.
    /// pci_bus_id corresponds to the cudaDeviceProp.pciBusID field
    pub fn pci_bus_id(&self) -> i32 {
        cpp!(unsafe [self as "const R3DSDK::CudaDeviceInfo *"] -> i32 as "int" { return self->pci_bus_id; })
    }

    /// string name is required to ensure the pci bus id matches the proper device
    pub fn name(&self) -> String {
        unsafe {
            let c_str = std::ffi::CStr::from_ptr(cpp!(unsafe [self as "const R3DSDK::CudaDeviceInfo *"] -> *const core::ffi::c_char as "const char *" {
                return self->name;
            }));
            c_str.to_str().map(|x| x.to_string()).unwrap_or_default()
        }
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


pub struct R3dDecodeJob {
    ptr: *mut core::ffi::c_void,
    internal_buffer: Option<AlignedBuffer>,
    metadata_allocated: bool,
}
impl R3dDecodeJob {
    pub fn new() -> RedResult<Self> {
        unsafe {
            let mut ptr = std::ptr::null_mut();
            let ptrptr = &mut ptr;
            let status: R3DStatus = std::mem::transmute(cpp!([ptrptr as "R3DSDK::R3DDecodeJob **"] -> i32 as "int" { return R3DSDK::R3DDecoder::CreateDecodeJob(ptrptr); }));
            if status != R3DStatus::Ok {
                Err(RedError::from(status))
            } else {
                Ok(Self { ptr, internal_buffer: None, metadata_allocated: false })
            }
        }
    }
    pub fn as_mut_ptr(&self) -> *mut core::ffi::c_void {
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
    pub fn set_clip(&mut self, clip: &crate::clip::Clip) {
        let self_ptr = self.ptr;
        let clip_ptr = clip.as_mut_ptr();
		cpp!(unsafe [self_ptr as "R3DSDK::R3DDecodeJob *", clip_ptr as "R3DSDK::Clip *"] {
			self_ptr->clip = clip_ptr;
		})
    }
    /// 0 = main (A) track, 1 = EPIC/Scarlet-X higlight protection track 2 (X track)
	/// ignored when doing HDRx blending
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
	// item is the pointer to the original item as submitted in the Decode() call
	// typedef void (*DecodeCallback)(R3DDecodeJob * item, R3DStatus decodeStatus);
	// DecodeCallback				callback;
	// void *						privateData;			// available for your use as you see fit, R3D SDK will not touch this variable!


	/// Resolution/speed to decode the image at. This will also
	/// influence how much memory is needed for the output buffer
	pub fn mode(&self) -> VideoDecodeMode {
        let self_ptr = self.ptr;
		unsafe { std::mem::transmute(cpp!([self_ptr as "const R3DSDK::R3DDecodeJob *"] -> i32 as "int" { return (int)self_ptr->mode; })) }
	}
	/// Resolution/speed to decode the image at. This will also
	/// influence how much memory is needed for the output buffer
	pub fn set_mode(&mut self, v: VideoDecodeMode) {
		let v: i32 = v as i32;
        let self_ptr = self.ptr;
		cpp!(unsafe [self_ptr as "R3DSDK::R3DDecodeJob *", v as "int"] { self_ptr->mode = (R3DSDK::VideoDecodeMode)v; })
	}
	/// Which pixel type to decode the image in. See the VideoPixelType enum for the available options. To get the image in the
	/// original pixel type supported by earlier SDK's set PixelType_16Bit_RGB_Planar
	pub fn pixel_type(&self) -> VideoPixelType {
        let self_ptr = self.ptr;
		unsafe { std::mem::transmute(cpp!([self_ptr as "const R3DSDK::R3DDecodeJob *"] -> i32 as "int" { return (int)self_ptr->pixelType; })) }
	}

	/// Which pixel type to decode the image in. See the VideoPixelType
	/// enum for the available options.
	/// using an unsupported pixeltype will result in an InvalidPixelType error code being returned.
	/// Supported VideoPixelTypes:
	/// * PixelType_16Bit_RGB_Planar
	/// * PixelType_16Bit_RGB_Interleaved
	/// * PixelType_10Bit_DPX_MethodB
	/// * PixelType_8Bit_BGR_Interleaved
	/// * PixelType_HalfFloat_RGB_Interleaved
	/// * PixelType_HalfFloat_RGB_ACES_Int
	pub fn set_pixel_type(&mut self, v: VideoPixelType) {
		let v: i32 = v as i32;
        let self_ptr = self.ptr;
		cpp!(unsafe [self_ptr as "R3DSDK::R3DDecodeJob *", v as "int"] { self_ptr->pixelType = (R3DSDK::VideoPixelType)v; })
	}
	/// Pointer to the buffer to store the image in. This cannot be
	/// NULL otherwise the decode will fail. The buffer must be aligned
	/// on a 16-byte boundary (see sample code).
	pub fn set_output_buffer(&mut self, buf: *mut core::ffi::c_void, size: usize) {
        let self_ptr = self.ptr;
		cpp!(unsafe [self_ptr as "R3DSDK::R3DDecodeJob *", buf as "void *", size as "size_t"] {
			self_ptr->outputBuffer = buf;
			self_ptr->outputBufferSize = size;
		})
	}
	/// Allocate internal buffer for the output image.
    /// Always aligned to 1024 bytes.
	pub fn allocate_internal_buffer(&mut self, clip: &crate::clip::Clip) -> RedResult<()> {
        self.internal_buffer = Some(clip.allocate_aligned_buffer(&self.mode(), &self.pixel_type(), 1024)?);
        let buf = self.internal_buffer.as_ref().unwrap();
        self.set_output_buffer(buf.ptr, buf.len());
        Ok(())
	}

	/// Image processing settings to apply to the decode. This cannot be NULL.
	pub fn set_image_processing(&mut self, v: &crate::image_processing_settings::ImageProcessingSettings) {
		let ptr = v as *const _;
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
    /// Optionally return the decoded frame's per-frame metadata
    pub fn metadata(&self) -> RedResult<&Metadata> {
        if !self.metadata_allocated {
            return Err(RedError::Other("Metadata was not requested in the job".to_string()));
        }
        let self_ptr = self.ptr;
        let ptr = cpp!(unsafe [self_ptr as "const R3DSDK::R3DDecodeJob *"] -> *const core::ffi::c_void as "void *" {
            return self_ptr->outputFrameMetadata;
        });
        Ok(unsafe { &*(ptr as *const Metadata) })
    }
}

extern "C" fn decode_callback(job: *mut c_void /* R3DDecodeJob * */, decode_status: R3DStatus) {
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

    // Safety: `ud` points to the State inside an Arc held by the Future.
    let state: &mut State<R3dDecodeJob> = unsafe { &mut *(ud as *mut State<R3dDecodeJob>) };

    // Store the result and signal completion
    {
        let mut lock = state.result.lock().unwrap();
        let org_job = state.job.take().unwrap();
        if decode_status == R3DStatus::Ok {
            *lock = Some(Ok(org_job));
        } else {
            *lock = Some(Err(RedError::from(decode_status)));
        }
    }
    state.done.store(true, Ordering::Release);
    state.waker.wake();
}
