// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use cpp::*;
use crate::{ RedResult, RedError, enums::*, image_processing_settings::ImageProcessingSettings };
use core::ffi::c_void;

cpp!{{
    #include "R3DSDKOpenCL.h"
}}

pub struct RedOpenCl {
    ptr: *mut c_void, // R3DSDK::REDCL *
}

impl RedOpenCl {
    /// compiled_kernel_cache_folder is a folder path to where you want us to store and
	/// load our compiled kernel files on disk. If you provide "" caching of compiled
	/// kernels will be disabled which is NOT recommended as compile times can be long!
	/// (see "OpenCL kernel caching.txt" for more information)
    pub fn new(compiled_kernel_cache_folder: &str) -> RedResult<Self> {
        let funcs = OPENCL.as_ref().map_err(|_| RedError::RedOpenCLLibraryNotFound)?;

        let folder_cstr = std::ffi::CString::new(compiled_kernel_cache_folder).unwrap();
        let folder_ptr = folder_cstr.as_ptr();

        let p_clSetKernelArg                   = *funcs.clSetKernelArg                    as *const () as usize;
        let p_clFlush                          = *funcs.clFlush                           as *const () as usize;
        let p_clFinish                         = *funcs.clFinish                          as *const () as usize;
        let p_clEnqueueCopyImage               = *funcs.clEnqueueCopyImage                as *const () as usize;
        let p_clCreateContext                  = *funcs.clCreateContext                   as *const () as usize;
        let p_clCreateCommandQueue             = *funcs.clCreateCommandQueue              as *const () as usize;
        let p_clCreateSampler                  = *funcs.clCreateSampler                   as *const () as usize;
        let p_clCreateKernel                   = *funcs.clCreateKernel                    as *const () as usize;
        let p_clCreateBuffer                   = *funcs.clCreateBuffer                    as *const () as usize;
        let p_clCreateProgramWithSource        = *funcs.clCreateProgramWithSource         as *const () as usize;
        let p_clCreateProgramWithBinary        = *funcs.clCreateProgramWithBinary         as *const () as usize;
        let p_clReleaseEvent                   = *funcs.clReleaseEvent                    as *const () as usize;
        let p_clReleaseSampler                 = *funcs.clReleaseSampler                  as *const () as usize;
        let p_clReleaseKernel                  = *funcs.clReleaseKernel                   as *const () as usize;
        let p_clReleaseMemObject               = *funcs.clReleaseMemObject                as *const () as usize;
        let p_clReleaseProgram                 = *funcs.clReleaseProgram                  as *const () as usize;
        let p_clReleaseContext                 = *funcs.clReleaseContext                  as *const () as usize;
        let p_clReleaseCommandQueue            = *funcs.clReleaseCommandQueue             as *const () as usize;
        let p_clGetPlatformInfo                = *funcs.clGetPlatformInfo                 as *const () as usize;
        let p_clGetDeviceIDs                   = *funcs.clGetDeviceIDs                    as *const () as usize;
        let p_clGetPlatformIDs                 = *funcs.clGetPlatformIDs                  as *const () as usize;
        let p_clGetDeviceInfo                  = *funcs.clGetDeviceInfo                   as *const () as usize;
        let p_clGetContextInfo                 = *funcs.clGetContextInfo                  as *const () as usize;
        let p_clGetImageInfo                   = *funcs.clGetImageInfo                    as *const () as usize;
        let p_clGetProgramBuildInfo            = *funcs.clGetProgramBuildInfo             as *const () as usize;
        let p_clGetProgramInfo                 = *funcs.clGetProgramInfo                  as *const () as usize;
        let p_clGetKernelWorkGroupInfo         = *funcs.clGetKernelWorkGroupInfo          as *const () as usize;
        let p_clBuildProgram                   = *funcs.clBuildProgram                    as *const () as usize;
        let p_clEnqueueWriteBuffer             = *funcs.clEnqueueWriteBuffer              as *const () as usize;
        let p_clEnqueueReadBuffer              = *funcs.clEnqueueReadBuffer               as *const () as usize;
        let p_clEnqueueCopyBuffer              = *funcs.clEnqueueCopyBuffer               as *const () as usize;
        let p_clEnqueueCopyBufferToImage       = *funcs.clEnqueueCopyBufferToImage        as *const () as usize;
        let p_clEnqueueWriteImage              = *funcs.clEnqueueWriteImage               as *const () as usize;
        let p_clEnqueueNDRangeKernel           = *funcs.clEnqueueNDRangeKernel            as *const () as usize;
        let p_clEnqueueMapBuffer               = *funcs.clEnqueueMapBuffer                as *const () as usize;
        let p_clEnqueueUnmapMemObject          = *funcs.clEnqueueUnmapMemObject           as *const () as usize;
        let p_clWaitForEvents                  = *funcs.clWaitForEvents                   as *const () as usize;
        let p_clEnqueueBarrier                 = *funcs.clEnqueueBarrier                  as *const () as usize;
        let p_clEnqueueMarker                  = *funcs.clEnqueueMarker                   as *const () as usize;
        let p_clCreateImage2D                  = *funcs.clCreateImage2D                   as *const () as usize;
        let p_clSetMemObjectDestructorCallback = *funcs.clSetMemObjectDestructorCallback  as *const () as usize;
        let p_clCreateSubBuffer                = *funcs.clCreateSubBuffer                 as *const () as usize;
        let p_clGetMemObjectInfo               = *funcs.clGetMemObjectInfo                as *const () as usize;
        let p_clCreateImage3D                  = *funcs.clCreateImage3D                   as *const () as usize;
        let ptr = cpp!(unsafe [
            p_clSetKernelArg                    as "uintptr_t",
            p_clFlush                           as "uintptr_t",
            p_clFinish                          as "uintptr_t",
            p_clEnqueueCopyImage                as "uintptr_t",
            p_clCreateContext                   as "uintptr_t",
            p_clCreateCommandQueue              as "uintptr_t",
            p_clCreateSampler                   as "uintptr_t",
            p_clCreateKernel                    as "uintptr_t",
            p_clCreateBuffer                    as "uintptr_t",
            p_clCreateProgramWithSource         as "uintptr_t",
            p_clCreateProgramWithBinary         as "uintptr_t",
            p_clReleaseEvent                    as "uintptr_t",
            p_clReleaseSampler                  as "uintptr_t",
            p_clReleaseKernel                   as "uintptr_t",
            p_clReleaseMemObject                as "uintptr_t",
            p_clReleaseProgram                  as "uintptr_t",
            p_clReleaseContext                  as "uintptr_t",
            p_clReleaseCommandQueue             as "uintptr_t",
            p_clGetPlatformInfo                 as "uintptr_t",
            p_clGetDeviceIDs                    as "uintptr_t",
            p_clGetPlatformIDs                  as "uintptr_t",
            p_clGetDeviceInfo                   as "uintptr_t",
            p_clGetContextInfo                  as "uintptr_t",
            p_clGetImageInfo                    as "uintptr_t",
            p_clGetProgramBuildInfo             as "uintptr_t",
            p_clGetProgramInfo                  as "uintptr_t",
            p_clGetKernelWorkGroupInfo          as "uintptr_t",
            p_clBuildProgram                    as "uintptr_t",
            p_clEnqueueWriteBuffer              as "uintptr_t",
            p_clEnqueueReadBuffer               as "uintptr_t",
            p_clEnqueueCopyBuffer               as "uintptr_t",
            p_clEnqueueCopyBufferToImage        as "uintptr_t",
            p_clEnqueueWriteImage               as "uintptr_t",
            p_clEnqueueNDRangeKernel            as "uintptr_t",
            p_clEnqueueMapBuffer                as "uintptr_t",
            p_clEnqueueUnmapMemObject           as "uintptr_t",
            p_clWaitForEvents                   as "uintptr_t",
            p_clEnqueueBarrier                  as "uintptr_t",
            p_clEnqueueMarker                   as "uintptr_t",
            p_clCreateImage2D                   as "uintptr_t",
            p_clSetMemObjectDestructorCallback  as "uintptr_t",
            p_clCreateSubBuffer                 as "uintptr_t",
            p_clGetMemObjectInfo                as "uintptr_t",
            p_clCreateImage3D                   as "uintptr_t",
            folder_ptr as "const char *"
        ] -> *mut c_void as "R3DSDK::REDCL *" {
            R3DSDK::EXT_OCLAPI_1_1 api;
            api.clSetKernelArg                   = reinterpret_cast<decltype(api.clSetKernelArg                  )>(p_clSetKernelArg                  );
            api.clFlush                          = reinterpret_cast<decltype(api.clFlush                         )>(p_clFlush                         );
            api.clFinish                         = reinterpret_cast<decltype(api.clFinish                        )>(p_clFinish                        );
            api.clEnqueueCopyImage               = reinterpret_cast<decltype(api.clEnqueueCopyImage              )>(p_clEnqueueCopyImage              );
            api.clCreateContext                  = reinterpret_cast<decltype(api.clCreateContext                 )>(p_clCreateContext                 );
            api.clCreateCommandQueue             = reinterpret_cast<decltype(api.clCreateCommandQueue            )>(p_clCreateCommandQueue            );
            api.clCreateSampler                  = reinterpret_cast<decltype(api.clCreateSampler                 )>(p_clCreateSampler                 );
            api.clCreateKernel                   = reinterpret_cast<decltype(api.clCreateKernel                  )>(p_clCreateKernel                  );
            api.clCreateBuffer                   = reinterpret_cast<decltype(api.clCreateBuffer                  )>(p_clCreateBuffer                  );
            api.clCreateProgramWithSource        = reinterpret_cast<decltype(api.clCreateProgramWithSource       )>(p_clCreateProgramWithSource       );
            api.clCreateProgramWithBinary        = reinterpret_cast<decltype(api.clCreateProgramWithBinary       )>(p_clCreateProgramWithBinary       );
            api.clReleaseEvent                   = reinterpret_cast<decltype(api.clReleaseEvent                  )>(p_clReleaseEvent                  );
            api.clReleaseSampler                 = reinterpret_cast<decltype(api.clReleaseSampler                )>(p_clReleaseSampler                );
            api.clReleaseKernel                  = reinterpret_cast<decltype(api.clReleaseKernel                 )>(p_clReleaseKernel                 );
            api.clReleaseMemObject               = reinterpret_cast<decltype(api.clReleaseMemObject              )>(p_clReleaseMemObject              );
            api.clReleaseProgram                 = reinterpret_cast<decltype(api.clReleaseProgram                )>(p_clReleaseProgram                );
            api.clReleaseContext                 = reinterpret_cast<decltype(api.clReleaseContext                )>(p_clReleaseContext                );
            api.clReleaseCommandQueue            = reinterpret_cast<decltype(api.clReleaseCommandQueue           )>(p_clReleaseCommandQueue           );
            api.clGetPlatformInfo                = reinterpret_cast<decltype(api.clGetPlatformInfo               )>(p_clGetPlatformInfo               );
            api.clGetDeviceIDs                   = reinterpret_cast<decltype(api.clGetDeviceIDs                  )>(p_clGetDeviceIDs                  );
            api.clGetPlatformIDs                 = reinterpret_cast<decltype(api.clGetPlatformIDs                )>(p_clGetPlatformIDs                );
            api.clGetDeviceInfo                  = reinterpret_cast<decltype(api.clGetDeviceInfo                 )>(p_clGetDeviceInfo                 );
            api.clGetContextInfo                 = reinterpret_cast<decltype(api.clGetContextInfo                )>(p_clGetContextInfo                );
            api.clGetImageInfo                   = reinterpret_cast<decltype(api.clGetImageInfo                  )>(p_clGetImageInfo                  );
            api.clGetProgramBuildInfo            = reinterpret_cast<decltype(api.clGetProgramBuildInfo           )>(p_clGetProgramBuildInfo           );
            api.clGetProgramInfo                 = reinterpret_cast<decltype(api.clGetProgramInfo                )>(p_clGetProgramInfo                );
            api.clGetKernelWorkGroupInfo         = reinterpret_cast<decltype(api.clGetKernelWorkGroupInfo        )>(p_clGetKernelWorkGroupInfo        );
            api.clBuildProgram                   = reinterpret_cast<decltype(api.clBuildProgram                  )>(p_clBuildProgram                  );
            api.clEnqueueWriteBuffer             = reinterpret_cast<decltype(api.clEnqueueWriteBuffer            )>(p_clEnqueueWriteBuffer            );
            api.clEnqueueReadBuffer              = reinterpret_cast<decltype(api.clEnqueueReadBuffer             )>(p_clEnqueueReadBuffer             );
            api.clEnqueueCopyBuffer              = reinterpret_cast<decltype(api.clEnqueueCopyBuffer             )>(p_clEnqueueCopyBuffer             );
            api.clEnqueueCopyBufferToImage       = reinterpret_cast<decltype(api.clEnqueueCopyBufferToImage      )>(p_clEnqueueCopyBufferToImage      );
            api.clEnqueueWriteImage              = reinterpret_cast<decltype(api.clEnqueueWriteImage             )>(p_clEnqueueWriteImage             );
            api.clEnqueueNDRangeKernel           = reinterpret_cast<decltype(api.clEnqueueNDRangeKernel          )>(p_clEnqueueNDRangeKernel          );
            api.clEnqueueMapBuffer               = reinterpret_cast<decltype(api.clEnqueueMapBuffer              )>(p_clEnqueueMapBuffer              );
            api.clEnqueueUnmapMemObject          = reinterpret_cast<decltype(api.clEnqueueUnmapMemObject         )>(p_clEnqueueUnmapMemObject         );
            api.clWaitForEvents                  = reinterpret_cast<decltype(api.clWaitForEvents                 )>(p_clWaitForEvents                 );
            api.clEnqueueBarrier                 = reinterpret_cast<decltype(api.clEnqueueBarrier                )>(p_clEnqueueBarrier                );
            api.clEnqueueMarker                  = reinterpret_cast<decltype(api.clEnqueueMarker                 )>(p_clEnqueueMarker                 );
            api.clCreateImage2D                  = reinterpret_cast<decltype(api.clCreateImage2D                 )>(p_clCreateImage2D                 );
            api.clSetMemObjectDestructorCallback = reinterpret_cast<decltype(api.clSetMemObjectDestructorCallback)>(p_clSetMemObjectDestructorCallback);
            api.clCreateSubBuffer                = reinterpret_cast<decltype(api.clCreateSubBuffer               )>(p_clCreateSubBuffer               );
            api.clGetMemObjectInfo               = reinterpret_cast<decltype(api.clGetMemObjectInfo              )>(p_clGetMemObjectInfo              );
            api.clCreateImage3D                  = reinterpret_cast<decltype(api.clCreateImage3D                 )>(p_clCreateImage3D                 );
            return new R3DSDK::REDCL(api, folder_ptr);
        });

        Ok(Self { ptr })
    }

    pub fn create_debayer_job(&self) -> DebayerOpenCLJob {
        let opencl = self.ptr;
        let ptr = cpp!(unsafe [opencl as "R3DSDK::REDCL *"] -> *mut c_void as "void*" {
            return (void*)opencl->createDebayerJob();
        });
        DebayerOpenCLJob { ptr, owner: self.ptr }
    }

    /// checks to see if a context and command queue are compatible with REDCL
    /// This call may take several seconds on slower cards as kernels get compiled
	/// Decode kernels are compiled asynchronously. All OpenCL processing will be blocked
	/// until that is complete. See "OpenCL kernel caching.txt" for more information
    pub fn check_compatibility(&self, context: cl_context, queue: cl_command_queue) -> RedResult<()> {
        let opencl = self.ptr;
        let mut opencl_error: cl_int = 0;
        let opencl_error_ptr = &mut opencl_error;
        let status: OpenClStatus = unsafe { std::mem::transmute(cpp!([opencl as "R3DSDK::REDCL *", context as "cl_context", queue as "cl_command_queue", opencl_error_ptr as "cl_int *"] -> i32 as "int" {
            return (int)opencl->checkCompatibility(context, queue, *opencl_error_ptr);
        })) };
        if status == OpenClStatus::Ok {
            if opencl_error == 0 { Ok(()) } else { Err(RedError::OpenCLError(opencl_error)) }
        } else {
            Err(status.into())
        }
    }

	/// processes a frame.
	/// Thread safe blocking call
	/// This function will enqueue the cuda kernels and wait for the processing to complete
    ///
	///	To ensure memory buffers are not released prior to completion of kernel deviceId and stream are used as tokens for internal object mapping.
	///	Be sure you have already done setCudaDevice on the thread using the passed in deviceId prior to calling process
    pub fn process_blocking(&self, context: cl_context, queue: cl_command_queue, debayer_job: &mut DebayerOpenCLJob) -> RedResult<()> {
        let opencl = self.ptr;
        let job = debayer_job.ptr;
        let mut opencl_error: cl_int = 0;
        let opencl_error_ptr = &mut opencl_error;
        let status: OpenClStatus = unsafe { std::mem::transmute(cpp!([opencl as "R3DSDK::REDCL *", context as "cl_context", queue as "cl_command_queue", job as "R3DSDK::DebayerOpenCLJob *", opencl_error_ptr as "cl_int *"] -> i32 as "int" {
            return (int)opencl->process(context, queue, job, *opencl_error_ptr);
        })) };
        if status == OpenClStatus::Ok {
            if opencl_error == 0 { Ok(()) } else { Err(RedError::OpenCLError(opencl_error)) }
        } else {
            Err(status.into())
        }
    }
	/// enqueues all red processing on the current stream
	/// once processAsync has been called you will need to call debayerJob->completeAsync
    ///
	///	This will wait for commands enqueued by this sdk for the current frame to complete and then releases any additional resources.
	///	If the commands have already completed there is no wait involved
    pub fn process_async(&self, context: cl_context, queue: cl_command_queue, debayer_job: &mut DebayerOpenCLJob) -> RedResult<()> {
        let opencl = self.ptr;
        let job_ptr = debayer_job.ptr;
        let mut opencl_error: cl_int = 0;
        let opencl_error_ptr = &mut opencl_error;
        let status: OpenClStatus = unsafe { std::mem::transmute(cpp!([opencl as "R3DSDK::REDCL *", context as "cl_context", queue as "cl_command_queue", job_ptr as "R3DSDK::DebayerOpenCLJob *", opencl_error_ptr as "cl_int *"] -> i32 as "int" {
            return (int)opencl->processAsync(context, queue, job_ptr, *opencl_error_ptr);
        })) };
        if status == OpenClStatus::Ok {
            if opencl_error == 0 { Ok(()) } else { Err(RedError::OpenCLError(opencl_error)) }
        } else {
            Err(status.into())
        }
    }
}

impl Drop for RedOpenCl {
    fn drop(&mut self) {
        let opencl = self.ptr;
        cpp!(unsafe [opencl as "R3DSDK::REDCL *"] { delete opencl; });
    }
}

pub struct DebayerOpenCLJob {
    ptr: *mut c_void,   // R3DSDK::DebayerOpenCLJob *
    owner: *mut c_void, // R3DSDK::REDCL *
}

impl DebayerOpenCLJob {
    pub fn as_mut_ptr(&self) -> *mut c_void { self.ptr }

    pub fn result_frame_size(&self) -> usize {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "const R3DSDK::DebayerOpenCLJob *"] -> usize as "size_t" {
            return R3DSDK::DebayerOpenCLJob::ResultFrameSize(*ptr);
        })
    }

    /// NOTE: Must remain valid until process/processAsync/completeAsync sequence completes
    pub fn set_raw_host_mem(&mut self, p: *mut c_void) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerOpenCLJob *", p as "void *"] {
            ptr->raw_host_mem = p;
        });
    }
    /// Same data as raw_host_mem but already on the GPU
    pub fn set_raw_device_mem(&mut self, p: *mut c_void) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerOpenCLJob *", p as "void *"] {
            ptr->raw_device_mem = (cl_mem)p;
        });
    }
    pub fn set_output_device_mem(&mut self, p: *mut c_void, size: usize) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerOpenCLJob *", p as "void *", size as "size_t"] {
            ptr->output_device_mem_size = size;
            ptr->output_device_mem = (cl_mem)p;
        });
    }

    pub fn set_mode(&mut self, mode: VideoDecodeMode) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerOpenCLJob *", mode as "int"] {
            ptr->mode = (R3DSDK::VideoDecodeMode)mode;
        });
    }

    pub fn set_image_processing(&mut self, v: &ImageProcessingSettings) {
        let ptr = self.ptr;
        let v = v as *const _;
        cpp!(unsafe [ptr as "R3DSDK::DebayerOpenCLJob *", v as "R3DSDK::ImageProcessingSettings *"] {
            ptr->imageProcessingSettings = v;
        });
    }

    pub fn set_pixel_type(&mut self, pixel_type: VideoPixelType) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerOpenCLJob *", pixel_type as "int"] {
            ptr->pixelType = (R3DSDK::VideoPixelType)pixel_type;
        });
    }

    /// Wait for async kernels completion and clean up frame resources
    pub fn complete_async(&mut self) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerOpenCLJob *"] {
            ptr->completeAsync();
        });
    }
}

impl Drop for DebayerOpenCLJob {
    fn drop(&mut self) {
        // Ensure completion
        self.complete_async();
        let owner = self.owner;
        let ptr = self.ptr;
        cpp!(unsafe [owner as "R3DSDK::REDCL *", ptr as "R3DSDK::DebayerOpenCLJob *"] {
            owner->releaseDebayerJob(ptr);
        });
    }
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum OpenClStatus {
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

impl From<OpenClStatus> for RedError {
    fn from(value: OpenClStatus) -> Self {
        match value {
            OpenClStatus::Ok => panic!("Cannot convert Ok to RedError"),
            OpenClStatus::ErrorProcessing => RedError::ErrorProcessing,
            OpenClStatus::InvalidJobParameter => RedError::InvalidJobParameter,
            OpenClStatus::InvalidJobParameter_mode => RedError::InvalidJobParameterMode,
            OpenClStatus::InvalidJobParameter_raw_host_mem => RedError::InvalidJobParameterRawHostMem,
            OpenClStatus::InvalidJobParameter_raw_device_mem => RedError::InvalidJobParameterRawDeviceMem,
            OpenClStatus::InvalidJobParameter_pixelType => RedError::InvalidPixelType,
            OpenClStatus::InvalidJobParameter_output_device_mem_size => RedError::InvalidJobParameterOutputDeviceMemSize,
            OpenClStatus::InvalidJobParameter_output_device_mem => RedError::InvalidJobParameterOutputDeviceMem,
            OpenClStatus::InvalidJobParameter_ColorVersion1 => RedError::InvalidJobParameterColorVersion1,
            OpenClStatus::UnableToUseGPUDevice => RedError::UnableToUseGPUDevice,
            OpenClStatus::UnableToLoadLibrary => RedError::UnableToLoadLibrary,
            OpenClStatus::ParameterUnsupported => RedError::ParameterUnsupported,
            OpenClStatus::InvalidAPIObject => RedError::InvalidAPIObject,
        }
    }
}

//////////////////////////////////////// OpenCL FFI ////////////////////////////////////////

#[cfg(target_os = "windows")]
use libloading::os::windows as dl;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use libloading::os::unix as dl;

use std::sync::LazyLock;

use core::ffi::{ c_char, c_uchar };
type size_t = usize;
type intptr_t = isize;

pub type cl_platform_id = *mut c_void;
pub type cl_device_id = *mut c_void;
pub type cl_context = *mut c_void;
pub type cl_command_queue = *mut c_void;
pub type cl_mem = *mut c_void;
pub type cl_program = *mut c_void;
pub type cl_kernel = *mut c_void;
pub type cl_event = *mut c_void;
pub type cl_sampler = *mut c_void;
pub type cl_int = i32;
pub type cl_uint = u32;
pub type cl_ulong = u64;
pub type cl_bool = cl_uint;
pub type cl_bitfield = cl_ulong;
pub type cl_device_type = cl_bitfield;
pub type cl_platform_info = cl_uint;
pub type cl_device_info = cl_uint;
pub type cl_command_queue_properties = cl_bitfield;
pub type cl_context_properties = intptr_t;
pub type cl_context_info = cl_uint;
pub type cl_channel_order = cl_uint;
pub type cl_channel_type = cl_uint;
pub type cl_mem_flags = cl_bitfield;
pub type cl_mem_info = cl_uint;
pub type cl_image_info = cl_uint;
pub type cl_buffer_create_type = cl_uint;
pub type cl_addressing_mode = cl_uint;
pub type cl_filter_mode = cl_uint;
pub type cl_map_flags = cl_bitfield;
pub type cl_program_info = cl_uint;
pub type cl_program_build_info = cl_uint;
pub type cl_kernel_work_group_info = cl_uint;

#[repr(C)]
pub struct cl_image_format {
    pub image_channel_order: cl_channel_order,
    pub image_channel_data_type: cl_channel_type,
}

pub struct OpenCLFunctions {
    _opencl: dl::Library,

    pub clSetKernelArg:                   dl::Symbol<unsafe extern "C" fn(kernel: cl_kernel, arg_index: cl_uint, arg_size: size_t, arg_value: *const c_void) -> cl_int>,
    pub clFlush:                          dl::Symbol<unsafe extern "C" fn(command_queue: cl_command_queue) -> cl_int>,
    pub clFinish:                         dl::Symbol<unsafe extern "C" fn(command_queue: cl_command_queue) -> cl_int>,
    pub clEnqueueCopyImage:               dl::Symbol<unsafe extern "C" fn(command_queue: cl_command_queue, src_image: cl_mem, dst_image: cl_mem, src_origin: *const size_t, dst_origin: *const size_t, region: *const size_t, num_events_in_wait_list: cl_uint, event_wait_list: *const cl_event, event: *mut cl_event) -> cl_int>,
    pub clCreateContext:                  dl::Symbol<unsafe extern "C" fn(properties: *const cl_context_properties, num_devices: cl_uint, devices: *const cl_device_id, pfn_notify: Option<extern "C" fn(*const c_char, *const c_void, size_t, *mut c_void)>, user_data: *mut c_void, errcode_ret: *mut cl_int) -> cl_context>,
    pub clCreateCommandQueue:             dl::Symbol<unsafe extern "C" fn(context: cl_context, device: cl_device_id, properties: cl_command_queue_properties, errcode_ret: *mut cl_int) -> cl_command_queue>,
    pub clCreateSampler:                  dl::Symbol<unsafe extern "C" fn(context: cl_context, normalize_coords: cl_bool, addressing_mode: cl_addressing_mode, filter_mode: cl_filter_mode, errcode_ret: *mut cl_int) -> cl_sampler>,
    pub clCreateKernel:                   dl::Symbol<unsafe extern "C" fn(program: cl_program, kernel_name: *const c_char, errcode_ret: *mut cl_int) -> cl_kernel>,
    pub clCreateBuffer:                   dl::Symbol<unsafe extern "C" fn(context: cl_context, flags: cl_mem_flags, size: size_t, host_ptr: *mut c_void, errcode_ret: *mut cl_int) -> cl_mem>,
    pub clCreateProgramWithSource:        dl::Symbol<unsafe extern "C" fn(context: cl_context, count: cl_uint, strings: *const *const c_char, lengths: *const size_t, errcode_ret: *mut cl_int) -> cl_program>,
    pub clCreateProgramWithBinary:        dl::Symbol<unsafe extern "C" fn(context: cl_context, num_devices: cl_uint, device_list: *const cl_device_id, lengths: *const size_t, binaries: *const *const c_uchar, binary_status: *mut cl_int, errcode_ret: *mut cl_int) -> cl_program>,
    pub clReleaseEvent:                   dl::Symbol<unsafe extern "C" fn(event: cl_event) -> cl_int>,
    pub clReleaseSampler:                 dl::Symbol<unsafe extern "C" fn(sampler: cl_sampler) -> cl_int>,
    pub clReleaseKernel:                  dl::Symbol<unsafe extern "C" fn(kernel: cl_kernel) -> cl_int>,
    pub clReleaseMemObject:               dl::Symbol<unsafe extern "C" fn(memobj: cl_mem) -> cl_int>,
    pub clReleaseProgram:                 dl::Symbol<unsafe extern "C" fn(program: cl_program) -> cl_int>,
    pub clReleaseContext:                 dl::Symbol<unsafe extern "C" fn(context: cl_context) -> cl_int>,
    pub clReleaseCommandQueue:            dl::Symbol<unsafe extern "C" fn(command_queue: cl_command_queue) -> cl_int>,
    pub clGetPlatformInfo:                dl::Symbol<unsafe extern "C" fn(platform: cl_platform_id, param_name: cl_platform_info, param_value_size: size_t, param_value: *mut c_void, param_value_size_ret: *mut size_t) -> cl_int>,
    pub clGetDeviceIDs:                   dl::Symbol<unsafe extern "C" fn(platform: cl_platform_id, device_type: cl_device_type, num_entries: cl_uint, devices: *mut cl_device_id, num_devices: *mut cl_uint) -> cl_int>,
    pub clGetPlatformIDs:                 dl::Symbol<unsafe extern "C" fn(num_entries: cl_uint, platforms: *mut cl_platform_id, num_platforms: *mut cl_uint) -> cl_int>,
    pub clGetDeviceInfo:                  dl::Symbol<unsafe extern "C" fn(device: cl_device_id, param_name: cl_device_info, param_value_size: size_t, param_value: *mut c_void, param_value_size_ret: *mut size_t) -> cl_int>,
    pub clGetContextInfo:                 dl::Symbol<unsafe extern "C" fn(context: cl_context, param_name: cl_context_info, param_value_size: size_t, param_value: *mut c_void, param_value_size_ret: *mut size_t) -> cl_int>,
    pub clGetImageInfo:                   dl::Symbol<unsafe extern "C" fn(image: cl_mem, param_name: cl_image_info, param_value_size: size_t, param_value: *mut c_void, param_value_size_ret: *mut size_t) -> cl_int>,
    pub clGetProgramBuildInfo:            dl::Symbol<unsafe extern "C" fn(program: cl_program, device: cl_device_id, param_name: cl_program_build_info, param_value_size: size_t, param_value: *mut c_void, param_value_size_ret: *mut size_t) -> cl_int>,
    pub clGetProgramInfo:                 dl::Symbol<unsafe extern "C" fn(program: cl_program, param_name: cl_program_info, param_value_size: size_t, param_value: *mut c_void, param_value_size_ret: *mut size_t) -> cl_int>,
    pub clGetKernelWorkGroupInfo:         dl::Symbol<unsafe extern "C" fn(kernel: cl_kernel, device: cl_device_id, param_name: cl_kernel_work_group_info, param_value_size: size_t, param_value: *mut c_void, param_value_size_ret: *mut size_t) -> cl_int>,
    pub clBuildProgram:                   dl::Symbol<unsafe extern "C" fn(program: cl_program, num_devices: cl_uint, device_list: *const cl_device_id, options: *const c_char, pfn_notify: Option<extern "C" fn(cl_program, *mut c_void)>, user_data: *mut c_void) -> cl_int>,
    pub clEnqueueWriteBuffer:             dl::Symbol<unsafe extern "C" fn(command_queue: cl_command_queue, buffer: cl_mem, blocking_write: cl_bool, offset: size_t, cb: size_t, ptr: *const c_void, num_events_in_wait_list: cl_uint, event_wait_list: *const cl_event, event: *mut cl_event) -> cl_int>,
    pub clEnqueueReadBuffer:              dl::Symbol<unsafe extern "C" fn(command_queue: cl_command_queue, buffer: cl_mem, blocking_read: cl_bool, offset: size_t, cb: size_t, ptr: *mut c_void, num_events_in_wait_list: cl_uint, event_wait_list: *const cl_event, event: *mut cl_event) -> cl_int>,
    pub clEnqueueCopyBuffer:              dl::Symbol<unsafe extern "C" fn(command_queue: cl_command_queue, src_buffer: cl_mem, dst_buffer: cl_mem, src_offset: size_t, dst_offset: size_t, cb: size_t, num_events_in_wait_list: cl_uint, event_wait_list: *const cl_event, event: *mut cl_event) -> cl_int>,
    pub clEnqueueCopyBufferToImage:       dl::Symbol<unsafe extern "C" fn(command_queue: cl_command_queue, src_buffer: cl_mem, dst_image: cl_mem, src_offset: size_t, dst_origin: *const size_t, region: *const size_t, num_events_in_wait_list: cl_uint, event_wait_list: *const cl_event, event: *mut cl_event) -> cl_int>,
    pub clEnqueueWriteImage:              dl::Symbol<unsafe extern "C" fn(command_queue: cl_command_queue, image: cl_mem, blocking_write: cl_bool, origin: *const size_t, region: *const size_t, input_row_pitch: size_t, input_slc_pitch: size_t, ptr: *const c_void, num_events_in_wait_list: cl_uint, event_wait_list: *const cl_event, event: *mut cl_event) -> cl_int>,
    pub clEnqueueNDRangeKernel:           dl::Symbol<unsafe extern "C" fn(command_queue: cl_command_queue, kernel: cl_kernel, work_dim: cl_uint, global_work_offset: *const size_t, global_work_dims: *const size_t, local_work_dims: *const size_t, num_events_in_wait_list: cl_uint, event_wait_list: *const cl_event, event: *mut cl_event) -> cl_int>,
    pub clEnqueueMapBuffer:               dl::Symbol<unsafe extern "C" fn(command_queue: cl_command_queue, buffer: cl_mem, blocking_map: cl_bool, map_flags: cl_map_flags, offset: size_t, size: size_t, num_events_in_wait_list: cl_uint, event_wait_list: *const cl_event, event: *mut cl_event, errorcode_ret: *mut cl_int) -> *mut c_void>,
    pub clEnqueueUnmapMemObject:          dl::Symbol<unsafe extern "C" fn(command_queue: cl_command_queue, memobj: cl_mem, mapped_ptr: *mut c_void, num_events_in_wait_list: cl_uint, event_wait_list: *const cl_event, event: *mut cl_event) -> cl_int>,
    pub clWaitForEvents:                  dl::Symbol<unsafe extern "C" fn(num_events: cl_uint, event_list: *const cl_event) -> cl_int>,
    pub clEnqueueBarrier:                 dl::Symbol<unsafe extern "C" fn(command_queue: cl_command_queue) -> cl_int>,
    pub clEnqueueMarker:                  dl::Symbol<unsafe extern "C" fn(command_queue: cl_command_queue, event: *mut cl_event) -> cl_int>,
    pub clCreateImage2D:                  dl::Symbol<unsafe extern "C" fn(context: cl_context, flags: cl_mem_flags, image_format: *mut cl_image_format, image_width: size_t, image_depth: size_t, image_slc_pitch: size_t, host_ptr: *mut c_void, errcode_ret: *mut cl_int) -> cl_mem>,
    pub clSetMemObjectDestructorCallback: dl::Symbol<unsafe extern "C" fn(memobj: cl_mem, pfn_notify: Option<extern "C" fn(cl_mem, *mut c_void)>, user_data: *mut c_void) -> cl_int>,
    pub clCreateSubBuffer:                dl::Symbol<unsafe extern "C" fn(buffer: cl_mem, flags: cl_mem_flags, buffer_create_type: cl_buffer_create_type, buffer_create_info: *const c_void, errcode_ret: *mut cl_int) -> cl_mem>,
    pub clGetMemObjectInfo:               dl::Symbol<unsafe extern "C" fn(memobj: cl_mem, param_name: cl_mem_info, param_value_size: size_t, param_value: *mut c_void, param_value_size_ret: *mut size_t) -> cl_int>,
    pub clCreateImage3D:                  dl::Symbol<unsafe extern "C" fn(context: cl_context, flags: cl_mem_flags, image_format: *mut cl_image_format, image_width: size_t, image_height: size_t, image_depth: size_t, image_row_pitch: size_t, image_slc_pitch: size_t, host_ptr: *mut c_void, errcode_ret: *mut cl_int) -> cl_mem>,
}

impl OpenCLFunctions {
    pub unsafe fn new() -> Result<Self, libloading::Error> {
        let candidates = if cfg!(target_os = "windows") {
            vec![ "OpenCL.dll" ]
        } else if cfg!(target_os = "macos") {
            vec![ "OpenCL.framework/OpenCL", "/System/Library/Frameworks/OpenCL.framework/OpenCL", "libOpenCL.dylib" ]
        } else {
            vec![ "libOpenCL.so" ]
        };
        let mut opencl = None;
        for filename in candidates {
            if let Ok(l) = unsafe { dl::Library::new(filename) } {
                opencl = Some(l);
                log::debug!("Loaded {}", &filename);
                break;
            }
        }
        if opencl.is_none() { return Err(libloading::Error::DlOpenUnknown); }
        let opencl = opencl.unwrap();

        unsafe {
            Ok(Self {
                clSetKernelArg:                   opencl.get(b"clSetKernelArg")?,
                clFlush:                          opencl.get(b"clFlush")?,
                clFinish:                         opencl.get(b"clFinish")?,
                clEnqueueCopyImage:               opencl.get(b"clEnqueueCopyImage")?,
                clCreateContext:                  opencl.get(b"clCreateContext")?,
                clCreateCommandQueue:             opencl.get(b"clCreateCommandQueue")?,
                clCreateSampler:                  opencl.get(b"clCreateSampler")?,
                clCreateKernel:                   opencl.get(b"clCreateKernel")?,
                clCreateBuffer:                   opencl.get(b"clCreateBuffer")?,
                clCreateProgramWithSource:        opencl.get(b"clCreateProgramWithSource")?,
                clCreateProgramWithBinary:        opencl.get(b"clCreateProgramWithBinary")?,
                clReleaseEvent:                   opencl.get(b"clReleaseEvent")?,
                clReleaseSampler:                 opencl.get(b"clReleaseSampler")?,
                clReleaseKernel:                  opencl.get(b"clReleaseKernel")?,
                clReleaseMemObject:               opencl.get(b"clReleaseMemObject")?,
                clReleaseProgram:                 opencl.get(b"clReleaseProgram")?,
                clReleaseContext:                 opencl.get(b"clReleaseContext")?,
                clReleaseCommandQueue:            opencl.get(b"clReleaseCommandQueue")?,
                clGetPlatformInfo:                opencl.get(b"clGetPlatformInfo")?,
                clGetDeviceIDs:                   opencl.get(b"clGetDeviceIDs")?,
                clGetPlatformIDs:                 opencl.get(b"clGetPlatformIDs")?,
                clGetDeviceInfo:                  opencl.get(b"clGetDeviceInfo")?,
                clGetContextInfo:                 opencl.get(b"clGetContextInfo")?,
                clGetImageInfo:                   opencl.get(b"clGetImageInfo")?,
                clGetProgramBuildInfo:            opencl.get(b"clGetProgramBuildInfo")?,
                clGetProgramInfo:                 opencl.get(b"clGetProgramInfo")?,
                clGetKernelWorkGroupInfo:         opencl.get(b"clGetKernelWorkGroupInfo")?,
                clBuildProgram:                   opencl.get(b"clBuildProgram")?,
                clEnqueueWriteBuffer:             opencl.get(b"clEnqueueWriteBuffer")?,
                clEnqueueReadBuffer:              opencl.get(b"clEnqueueReadBuffer")?,
                clEnqueueCopyBuffer:              opencl.get(b"clEnqueueCopyBuffer")?,
                clEnqueueCopyBufferToImage:       opencl.get(b"clEnqueueCopyBufferToImage")?,
                clEnqueueWriteImage:              opencl.get(b"clEnqueueWriteImage")?,
                clEnqueueNDRangeKernel:           opencl.get(b"clEnqueueNDRangeKernel")?,
                clEnqueueMapBuffer:               opencl.get(b"clEnqueueMapBuffer")?,
                clEnqueueUnmapMemObject:          opencl.get(b"clEnqueueUnmapMemObject")?,
                clWaitForEvents:                  opencl.get(b"clWaitForEvents")?,
                clEnqueueBarrier:                 opencl.get(b"clEnqueueBarrier")?,
                clEnqueueMarker:                  opencl.get(b"clEnqueueMarker")?,
                clCreateImage2D:                  opencl.get(b"clCreateImage2D")?,
                clSetMemObjectDestructorCallback: opencl.get(b"clSetMemObjectDestructorCallback")?,
                clCreateSubBuffer:                opencl.get(b"clCreateSubBuffer")?,
                clGetMemObjectInfo:               opencl.get(b"clGetMemObjectInfo")?,
                clCreateImage3D:                  opencl.get(b"clCreateImage3D")?,

                _opencl: opencl,
            })
        }
    }
}

pub static OPENCL: LazyLock<Result<OpenCLFunctions, libloading::Error>> = LazyLock::new(|| unsafe { OpenCLFunctions::new() });
