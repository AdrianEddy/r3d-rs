// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use cpp::*;
use crate::{ RedResult, RedError, enums::*, image_processing_settings::ImageProcessingSettings };
use core::ffi::c_void;

cpp!{{
    #include "R3DSDKMetal.h"
}}

pub struct RedMetal {
    ptr: *mut c_void,
}

impl RedMetal {
    /// Create REDMetal wrapper
    pub fn new() -> RedResult<Self> {
        let ptr = cpp!(unsafe [] -> *mut c_void as "R3DSDK::REDMetal *" {
            R3DSDK::EXT_METAL_API api;
            return new R3DSDK::REDMetal(api);
        });

        Ok(Self { ptr })
    }

    pub fn create_debayer_job(&self) -> DebayerMetalJob {
        let metal = self.ptr;
        let ptr = cpp!(unsafe [metal as "R3DSDK::REDMetal *"] -> *mut c_void as "void*" {
            return (void*)metal->createDebayerJob();
        });
        DebayerMetalJob { ptr, owner: self.ptr }
    }

    /// checks to see if a context and command queue are compatible with REDMetal
    /// This call may take several seconds on slower cards
    pub fn check_compatibility(&self, queue: *mut c_void) -> RedResult<()> {
        let metal = self.ptr;
        let mut error: i32 = 0;
        let error_ptr = &mut error;
        let status: MetalStatus = unsafe { std::mem::transmute(cpp!([metal as "R3DSDK::REDMetal *", queue as "id<MTLCommandQueue>", error_ptr as "int *"] -> i32 as "int" {
            return (int)metal->checkCompatibility(queue, *error_ptr);
        })) };
        if status == MetalStatus::Ok {
            if error == 0 { Ok(()) } else { Err(RedError::MetalError(error)) }
        } else {
            Err(status.into())
        }
    }

    /// processes a frame.
    /// Thread safe blocking call
    /// This function will enqueue the Metal commands and wait for the processing to complete
    /// to ensure memory buffers are not released prior to completion of kernel
    pub fn process_blocking(&self, queue: *mut c_void, debayer_job: &mut DebayerMetalJob) -> RedResult<()> {
        let metal = self.ptr;
        let job = debayer_job.ptr;
        let mut error: i32 = 0;
        let error_ptr = &mut error;
        let status: MetalStatus = unsafe { std::mem::transmute(cpp!([metal as "R3DSDK::REDMetal *", queue as "id<MTLCommandQueue>", job as "R3DSDK::DebayerMetalJob *", error_ptr as "int *"] -> i32 as "int" {
            return (int)metal->process(queue, job, *error_ptr);
        })) };
        if status == MetalStatus::Ok {
            if error == 0 { Ok(()) } else { Err(RedError::MetalError(error)) }
        } else {
            Err(status.into())
        }
    }

    /// enqueues all red processing on the current command queue. NOTE: All calls to processAsync
	/// should be done using a single MTLCommandQueue to avoid potential MTLCommandBuffer failures
    /// once processAsync has been called you will need to call debayerJob->completeAsync
    /// This will wait for commands enqueued by this sdk for the current frame to complete and then releases any additional resources.
    /// If the commands have already completed there is no wait involved
    pub fn process_async(&self, queue: *mut c_void, debayer_job: &mut DebayerMetalJob) -> RedResult<()> {
        let metal = self.ptr;
        let job_ptr = debayer_job.ptr;
        let mut error: i32 = 0;
        let error_ptr = &mut error;
        let status: MetalStatus = unsafe { std::mem::transmute(cpp!([metal as "R3DSDK::REDMetal *", queue as "id<MTLCommandQueue>", job_ptr as "R3DSDK::DebayerMetalJob *", error_ptr as "int *"] -> i32 as "int" {
            return (int)metal->processAsync(queue, job_ptr, *error_ptr);
        })) };
        if status == MetalStatus::Ok {
            if error == 0 { Ok(()) } else { Err(RedError::MetalError(error)) }
        } else {
            Err(status.into())
        }
    }

    /// flushes the current batch job that has been queued for the current MTLCommandQueue
    /// after this has been called the jobs in the current batch will be processed in the order they were submitted
    /// completeAsync() will return after each job in the batch completes
    /// if debayerJobCallback has been set, the callback function will be called after each job completes
    pub fn flush(&self, queue: *mut c_void) -> RedResult<()> {
        let metal = self.ptr;
        let mut error: i32 = 0;
        let error_ptr = &mut error;
        let status: MetalStatus = unsafe { std::mem::transmute(cpp!([metal as "R3DSDK::REDMetal *", queue as "id<MTLCommandQueue>", error_ptr as "int *"] -> i32 as "int" {
            return (int)metal->flush(queue, *error_ptr);
        })) };
        if status == MetalStatus::Ok {
            if error == 0 { Ok(()) } else { Err(RedError::MetalError(error)) }
        } else {
            Err(status.into())
        }
    }
}

impl Drop for RedMetal {
    fn drop(&mut self) {
        let metal = self.ptr;
        cpp!(unsafe [metal as "R3DSDK::REDMetal *"] { delete metal; });
    }
}

pub struct DebayerMetalJob {
    ptr: *mut c_void,   // R3DSDK::DebayerMetalJob *
    owner: *mut c_void, // R3DSDK::REDMetal *
}

impl DebayerMetalJob {
    pub fn as_mut_ptr(&self) -> *mut c_void { self.ptr }

    pub fn result_frame_size(&self) -> usize {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "const R3DSDK::DebayerMetalJob *"] -> usize as "size_t" {
            return R3DSDK::DebayerMetalJob::ResultFrameSize(*ptr);
        })
    }

    /// NOTE: This pointer + MTLBuffer & their contents must remain valid until completeAsync() or process() returns!
    ///
    /// sized from r3dsdk buffer
    pub fn set_raw_host_mem(&mut self, p: *mut c_void) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *", p as "void *"] { ptr->raw_host_mem = p; });
    }
    /// Same data as raw_host_mem but already on the GPU
    pub fn set_raw_device_mem(&mut self, p: *mut c_void) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *", p as "id<MTLBuffer>"] { ptr->raw_device_mem = p; });
    }

    /// result/output buffer on the GPU
	/// object of the size output_device_mem_size
	/// -- do not use output_device_image when using this field
    ///
    /// Size must be a minimum of ResultFrameSize(job)
    pub fn set_output_device_mem(&mut self, p: *mut c_void, size: usize) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *", p as "id<MTLBuffer>", size as "size_t"] {
            ptr->output_device_mem_size = size;
            ptr->output_device_mem = p;
        });
    }

    /// alternate result/output texture on the GPU (16-bit uint and float supported)
    /// use this field if you want the result image written to an MTLTexture
    /// -- do not use output_device_mem when using this field
    ///
    /// Size must be a minimum of ResultFrameSize(job)
    pub fn set_output_device_image(&mut self, p: *mut c_void, size: usize) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *", p as "id<MTLTexture>", size as "size_t"] {
            ptr->output_device_mem_size = size;
            ptr->output_device_image = p;
        });
    }

    pub fn raw_host_mem(&self) -> *mut c_void {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *"] -> *mut c_void as "void *" { return ptr->raw_host_mem; })
    }
    pub fn raw_device_mem(&self) -> *mut c_void {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *"] -> *mut c_void as "id<MTLBuffer>" { return ptr->raw_device_mem; })
    }
    pub fn output_device_mem(&self) -> *mut c_void {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *"] -> *mut c_void as "id<MTLBuffer>" { return ptr->output_device_mem; })
    }
    pub fn output_device_image(&self) -> *mut c_void {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *"] -> *mut c_void as "id<MTLTexture>" { return ptr->output_device_image; })
    }
    pub fn output_device_size(&self) -> usize {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *"] -> usize as "size_t" { return ptr->output_device_mem_size; })
    }

    pub fn set_mode(&mut self, mode: VideoDecodeMode) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *", mode as "int"] {
            ptr->mode = (R3DSDK::VideoDecodeMode)mode;
        });
    }

	/// Image Processing Settings. This cannot be NULL.
    pub fn set_image_processing(&mut self, v: &ImageProcessingSettings) {
        let ptr = self.ptr;
        let v = v as *const _;
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *", v as "R3DSDK::ImageProcessingSettings *"] {
            ptr->imageProcessingSettings = v;
        });
    }

    /// Supported Types are:
    ///  PixelType_16Bit_RGB_Interleaved
    ///  PixelType_HalfFloat_RGB_Interleaved
    ///  PixelType_HalfFloat_RGB_ACES_Int
    pub fn set_pixel_type(&mut self, pixel_type: VideoPixelType) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *", pixel_type as "int"] {
            ptr->pixelType = (R3DSDK::VideoPixelType)pixel_type;
        });
    }

    /// This will ensure all objects used for the frame are disposed of.
    /// This call will block until the debayer on the queue executes, if the debayer has already executed no blocking will occur
    /// This function must be called at a point after passing the DebayerMetalJob into REDMetal::processAsync
    /// This will automatically be called upon destruction the DebayerMetalJob if it is not manually called earlier
    pub fn complete_async(&mut self) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *"] {
            ptr->completeAsync();
        });
    }

    /// Enables batch processing of frames - frames added to a single batch will all complete together when flush() is called
	/// True to add frame to the batch, queuing up the work
	/// False to add frame to the batch, and then complete the whole batch
    /// Leaving the field False will cause frames to be processed individually
	/// Recommended batch size: 2 for laptops, 4 for iMac Pro, 8 for Mac Pro (2019)
    pub fn set_batch_mode(&mut self, v: bool) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *", v as "bool"] {
            ptr->batchMode = v;
        });
    }

    // Provides a callback that will be called when the job completes
    // This can be used instead of calling completeAsync
    // Can be NULL
    //void (* debayerJobCallback)(DebayerMetalJob * job);

}

impl Drop for DebayerMetalJob {
    fn drop(&mut self) {
        // Ensure completion
        self.complete_async();
        let owner = self.owner;
        let ptr = self.ptr;
        cpp!(unsafe [owner as "R3DSDK::REDMetal *", ptr as "R3DSDK::DebayerMetalJob *"] {
            owner->releaseDebayerJob(ptr);
        });
    }
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MetalStatus {
    Ok = 0,
    ErrorProcessing = 1,
    InvalidJobParameter = 2,
    /// mode value passed in is not compatible with this SDK or the mode used with the DecodeForGpuSdk call is not compatible
    InvalidJobParameter_mode = 3,
    /// pointer is NULL, data is not from DecodeForGpuSdk, R3DSDK and GPUSDK versions are incompatible or buffer is not actually in host memory.
    InvalidJobParameter_raw_host_mem = 4,
    /// Raw device mem was NULL
    InvalidJobParameter_raw_device_mem = 5,
    /// unsupported pixel type
    InvalidJobParameter_pixelType = 6,
    /// Output buffer Size must be non zero.
    /// Output buffer must be allocated prior to passing it into the sdk
    InvalidJobParameter_output_device_mem_size = 7,
    InvalidJobParameter_output_device_mem = 8,
    /// Image processing settings ColorVersion was set to ColorVersion1 which is not supported by this SDK
    InvalidJobParameter_ColorVersion1 = 9,
    /// GPU Device did not meet minimum requirements.
    UnableToUseGPUDevice = 10,
    /// Error loading R3DSDK dynamic library
    UnableToLoadLibrary = 11,
    ParameterUnsupported = 12,
	InvalidAPIObject = 13
}

impl From<MetalStatus> for RedError {
    fn from(value: MetalStatus) -> Self {
        match value {
            MetalStatus::Ok => panic!("Cannot convert Ok to RedError"),
            MetalStatus::ErrorProcessing => RedError::ErrorProcessing,
            MetalStatus::InvalidJobParameter => RedError::InvalidJobParameter,
            MetalStatus::InvalidJobParameter_mode => RedError::InvalidJobParameterMode,
            MetalStatus::InvalidJobParameter_raw_host_mem => RedError::InvalidJobParameterRawHostMem,
            MetalStatus::InvalidJobParameter_raw_device_mem => RedError::InvalidJobParameterRawDeviceMem,
            MetalStatus::InvalidJobParameter_pixelType => RedError::InvalidPixelType,
            MetalStatus::InvalidJobParameter_output_device_mem_size => RedError::InvalidJobParameterOutputDeviceMemSize,
            MetalStatus::InvalidJobParameter_output_device_mem => RedError::InvalidJobParameterOutputDeviceMem,
            MetalStatus::InvalidJobParameter_ColorVersion1 => RedError::InvalidJobParameterColorVersion1,
            MetalStatus::UnableToUseGPUDevice => RedError::UnableToUseGPUDevice,
            MetalStatus::UnableToLoadLibrary => RedError::UnableToLoadLibrary,
            MetalStatus::ParameterUnsupported => RedError::ParameterUnsupported,
            MetalStatus::InvalidAPIObject => RedError::InvalidAPIObject,
        }
    }
}
