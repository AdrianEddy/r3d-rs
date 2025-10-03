// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use cpp::*;
use crate::{ RedResult, RedError, enums::*, image_processing_settings::ImageProcessingSettings };
use core::ffi::c_void;

cpp!{{
    #include "R3DSDKCuda.h"
}}

/// Opaque handle to CUDA stream from cudart
pub type CudaStream = *mut c_void;

pub struct RedCuda {
    ptr: *mut c_void,
}

impl RedCuda {
    /// Create REDCuda wrapper and bind cudart functions dynamically
    pub fn new() -> RedResult<Self> {
        let funcs = CUDA.as_ref().map_err(|_| RedError::RedCudaLibraryNotFound)?;

        let p_cudaFree                 = *funcs.cudaFree                 as *const () as usize;
        let p_cudaFreeArray            = *funcs.cudaFreeArray            as *const () as usize;
        let p_cudaFreeHost             = *funcs.cudaFreeHost             as *const () as usize;
        let p_cudaFreeMipmappedArray   = *funcs.cudaFreeMipmappedArray   as *const () as usize;
        let p_cudaHostAlloc            = *funcs.cudaHostAlloc            as *const () as usize;
        let p_cudaMalloc               = *funcs.cudaMalloc               as *const () as usize;
        let p_cudaMalloc3D             = *funcs.cudaMalloc3D             as *const () as usize;
        let p_cudaMalloc3DArray        = *funcs.cudaMalloc3DArray        as *const () as usize;
        let p_cudaMallocArray          = *funcs.cudaMallocArray          as *const () as usize;
        let p_cudaMallocHost           = *funcs.cudaMallocHost           as *const () as usize;
        let p_cudaMallocMipmappedArray = *funcs.cudaMallocMipmappedArray as *const () as usize;
        let p_cudaMallocPitch          = *funcs.cudaMallocPitch          as *const () as usize;
        let ptr = cpp!(unsafe [
            p_cudaFree as "uintptr_t",
            p_cudaFreeArray as "uintptr_t",
            p_cudaFreeHost as "uintptr_t",
            p_cudaFreeMipmappedArray as "uintptr_t",
            p_cudaHostAlloc as "uintptr_t",
            p_cudaMalloc as "uintptr_t",
            p_cudaMalloc3D as "uintptr_t",
            p_cudaMalloc3DArray as "uintptr_t",
            p_cudaMallocArray as "uintptr_t",
            p_cudaMallocHost as "uintptr_t",
            p_cudaMallocMipmappedArray as "uintptr_t",
            p_cudaMallocPitch as "uintptr_t"
        ] -> *mut c_void as "R3DSDK::REDCuda *" {
            R3DSDK::EXT_CUDA_API api;
            api.cudaFree                 = reinterpret_cast<decltype(api.cudaFree)>(p_cudaFree);
            api.cudaFreeArray            = reinterpret_cast<decltype(api.cudaFreeArray)>(p_cudaFreeArray);
            api.cudaFreeHost             = reinterpret_cast<decltype(api.cudaFreeHost)>(p_cudaFreeHost);
            api.cudaFreeMipmappedArray   = reinterpret_cast<decltype(api.cudaFreeMipmappedArray)>(p_cudaFreeMipmappedArray);
            api.cudaHostAlloc            = reinterpret_cast<decltype(api.cudaHostAlloc)>(p_cudaHostAlloc);
            api.cudaMalloc               = reinterpret_cast<decltype(api.cudaMalloc)>(p_cudaMalloc);
            api.cudaMalloc3D             = reinterpret_cast<decltype(api.cudaMalloc3D)>(p_cudaMalloc3D);
            api.cudaMalloc3DArray        = reinterpret_cast<decltype(api.cudaMalloc3DArray)>(p_cudaMalloc3DArray);
            api.cudaMallocArray          = reinterpret_cast<decltype(api.cudaMallocArray)>(p_cudaMallocArray);
            api.cudaMallocHost           = reinterpret_cast<decltype(api.cudaMallocHost)>(p_cudaMallocHost);
            api.cudaMallocMipmappedArray = reinterpret_cast<decltype(api.cudaMallocMipmappedArray)>(p_cudaMallocMipmappedArray);
            api.cudaMallocPitch          = reinterpret_cast<decltype(api.cudaMallocPitch)>(p_cudaMallocPitch);
            return new R3DSDK::REDCuda(api);
        });

        Ok(Self { ptr })
    }

    pub fn create_debayer_job(&self) -> DebayerCudaJob {
        let cuda = self.ptr;
        let ptr = cpp!(unsafe [cuda as "R3DSDK::REDCuda *"] -> *mut c_void as "void*" {
            return (void*)cuda->createDebayerJob();
        });
        DebayerCudaJob { ptr, owner: self.ptr }
    }

	/// checks to see if a device and stream queue are compatible with REDCuda
	/// This call may take several seconds on slower cards
    pub fn check_compatibility(&self, device_id: i32, stream: CudaStream) -> RedResult<()> {
        let cuda = self.ptr;
        let mut cuda_error: cudaError_t = 0;
        let cuda_error_ptr = &mut cuda_error;
        let status: CudaStatus = unsafe { std::mem::transmute(cpp!([cuda as "R3DSDK::REDCuda *", device_id as "int", stream as "cudaStream_t", cuda_error_ptr as "cudaError_t *"] -> i32 as "int" {
            return (int)cuda->checkCompatibility(device_id, stream, *cuda_error_ptr);
        })) };
        if status == CudaStatus::Ok {
            if cuda_error == 0 { Ok(()) } else { Err(RedError::CudaError(cuda_error)) }
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
    pub fn process_blocking(&self, device_id: i32, stream: CudaStream, debayer_job: &mut DebayerCudaJob) -> RedResult<()> {
        let cuda = self.ptr;
        let job = debayer_job.ptr;
        let mut cuda_error: cudaError_t = 0;
        let cuda_error_ptr = &mut cuda_error;
        let status: CudaStatus = unsafe { std::mem::transmute(cpp!([cuda as "R3DSDK::REDCuda *", device_id as "int", stream as "cudaStream_t", job as "R3DSDK::DebayerCudaJob *", cuda_error_ptr as "cudaError_t *"] -> i32 as "int" {
            return (int)cuda->process(device_id, stream, job, *cuda_error_ptr);
        })) };
        if status == CudaStatus::Ok {
            if cuda_error == 0 { Ok(()) } else { Err(RedError::CudaError(cuda_error)) }
        } else {
            Err(status.into())
        }
    }
	/// enqueues all red processing on the current stream
	/// once processAsync has been called you will need to call debayerJob->completeAsync
    ///
	///	This will wait for commands enqueued by this sdk for the current frame to complete and then releases any additional resources.
	///	If the commands have already completed there is no wait involved
	// Status processAsync(int deviceId, cudaStream_t stream, DebayerCudaJob *debayerJobAsync, cudaError_t &cuda_error);
    pub fn process_async(&self, device_id: i32, stream: CudaStream, debayer_job: &mut DebayerCudaJob) -> RedResult<()> {
        let cuda = self.ptr;
        let job_ptr = debayer_job.ptr;
        let mut cuda_error: cudaError_t = 0;
        let cuda_error_ptr = &mut cuda_error;
        let status: CudaStatus = unsafe { std::mem::transmute(cpp!([cuda as "R3DSDK::REDCuda *", device_id as "int", stream as "cudaStream_t", job_ptr as "R3DSDK::DebayerCudaJob *", cuda_error_ptr as "cudaError_t *"] -> i32 as "int" {
            return (int)cuda->processAsync(device_id, stream, job_ptr, *cuda_error_ptr);
        })) };
        if status == CudaStatus::Ok {
            if cuda_error == 0 { Ok(()) } else { Err(RedError::CudaError(cuda_error)) }
        } else {
            Err(status.into())
        }
    }
}

impl Drop for RedCuda {
    fn drop(&mut self) {
        let cuda = self.ptr;
        cpp!(unsafe [cuda as "R3DSDK::REDCuda *"] { delete cuda; });
    }
}

pub struct DebayerCudaJob {
    ptr: *mut c_void,   // R3DSDK::DebayerCudaJob *
    owner: *mut c_void, // R3DSDK::REDCuda *
}

impl DebayerCudaJob {
    pub fn as_mut_ptr(&self) -> *mut c_void { self.ptr }

    pub fn result_frame_size(&self) -> usize {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "const R3DSDK::DebayerCudaJob *"] -> usize as "size_t" {
            return R3DSDK::DebayerCudaJob::ResultFrameSize(ptr);
        })
    }

    /// NOTE: Must remain valid until process/processAsync/completeAsync sequence completes
    pub fn set_raw_host_mem(&mut self, p: *mut c_void) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerCudaJob *", p as "void *"] {
            ptr->raw_host_mem = p;
        });
    }
    /// Same data as raw_host_mem but already on the GPU
    pub fn set_raw_device_mem(&mut self, p: *mut c_void) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerCudaJob *", p as "void *"] {
            ptr->raw_device_mem = p;
        });
    }
    pub fn set_output_device_mem(&mut self, p: *mut c_void, size: usize) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerCudaJob *", p as "void *", size as "size_t"] {
            ptr->output_device_mem_size = size;
            ptr->output_device_mem = p;
        });
    }
    pub fn raw_host_mem(&self) -> *mut c_void {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerCudaJob *"] -> *mut c_void as "void *" { return ptr->raw_host_mem; })
    }
    pub fn raw_device_mem(&self) -> *mut c_void {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerCudaJob *"] -> *mut c_void as "void *" { return ptr->raw_device_mem; })
    }
    pub fn output_device_mem(&self) -> *mut c_void {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerCudaJob *"] -> *mut c_void as "void *" { return ptr->output_device_mem; })
    }
    pub fn output_device_size(&self) -> usize {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerCudaJob *"] -> usize as "size_t" { return ptr->output_device_mem_size; })
    }

    pub fn set_mode(&mut self, mode: VideoDecodeMode) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerCudaJob *", mode as "int"] {
            ptr->mode = (R3DSDK::VideoDecodeMode)mode;
        });
    }

    pub fn set_image_processing(&mut self, v: &ImageProcessingSettings) {
        let ptr = self.ptr;
        let v = v as *const _;
        cpp!(unsafe [ptr as "R3DSDK::DebayerCudaJob *", v as "R3DSDK::ImageProcessingSettings *"] {
            ptr->imageProcessingSettings = v;
        });
    }

    pub fn set_pixel_type(&mut self, pixel_type: VideoPixelType) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerCudaJob *", pixel_type as "int"] {
            ptr->pixelType = (R3DSDK::VideoPixelType)pixel_type;
        });
    }

    /// Wait for async kernels completion and clean up frame resources
    pub fn complete_async(&mut self) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerCudaJob *"] {
            ptr->completeAsync();
        });
    }
}

impl Drop for DebayerCudaJob {
    fn drop(&mut self) {
        // Ensure completion
        self.complete_async();
        let owner = self.owner;
        let ptr = self.ptr;
        cpp!(unsafe [owner as "R3DSDK::REDCuda *", ptr as "R3DSDK::DebayerCudaJob *"] {
            owner->releaseDebayerJob(ptr);
        });
    }
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CudaStatus {
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
    /// deviceId must be an index between 0 and the number of devices on the system minus one
    InvalidJobParameter_deviceId = 13
}

impl From<CudaStatus> for RedError {
    fn from(value: CudaStatus) -> Self {
        match value {
            CudaStatus::Ok => panic!("Cannot convert Ok to RedError"),
            CudaStatus::ErrorProcessing => RedError::ErrorProcessing,
            CudaStatus::InvalidJobParameter => RedError::InvalidJobParameter,
            CudaStatus::InvalidJobParameter_mode => RedError::InvalidJobParameterMode,
            CudaStatus::InvalidJobParameter_raw_host_mem => RedError::InvalidJobParameterRawHostMem,
            CudaStatus::InvalidJobParameter_raw_device_mem => RedError::InvalidJobParameterRawDeviceMem,
            CudaStatus::InvalidJobParameter_pixelType => RedError::InvalidPixelType,
            CudaStatus::InvalidJobParameter_output_device_mem_size => RedError::InvalidJobParameterOutputDeviceMemSize,
            CudaStatus::InvalidJobParameter_output_device_mem => RedError::InvalidJobParameterOutputDeviceMem,
            CudaStatus::InvalidJobParameter_ColorVersion1 => RedError::InvalidJobParameterColorVersion1,
            CudaStatus::UnableToUseGPUDevice => RedError::UnableToUseGPUDevice,
            CudaStatus::UnableToLoadLibrary => RedError::UnableToLoadLibrary,
            CudaStatus::ParameterUnsupported => RedError::ParameterUnsupported,
            CudaStatus::InvalidJobParameter_deviceId => RedError::InvalidJobParameterDeviceId,
        }
    }
}

//////////////////////////////////////// CUDA FFI ////////////////////////////////////////

#[cfg(target_os = "windows")]
use libloading::os::windows as dl;
#[cfg(target_os = "linux")]
use libloading::os::unix as dl;

use std::sync::LazyLock;

pub type cudaError_t = i32;

#[repr(C)]
#[derive(Debug, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct cudaChannelFormatDesc {
    pub x: ::core::ffi::c_int,
    pub y: ::core::ffi::c_int,
    pub z: ::core::ffi::c_int,
    pub w: ::core::ffi::c_int,
    pub f: i32, /* cudaChannelFormatKind */
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct cudaArray {
    _unused: [u8; 0],
}
pub type cudaArray_t = *mut cudaArray;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct cudaMipmappedArray {
    _unused: [u8; 0],
}
pub type cudaMipmappedArray_t = *mut cudaMipmappedArray;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct cudaExtent {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
}
#[repr(C)]
#[derive(Debug, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct cudaPitchedPtr {
    pub ptr: *mut ::core::ffi::c_void,
    pub pitch: usize,
    pub xsize: usize,
    pub ysize: usize,
}

pub struct CudaFunctions {
    _cudart: dl::Library,

    pub cudaFree:                 dl::Symbol<unsafe extern "C" fn(ptr: *mut ::core::ffi::c_void) -> cudaError_t>,
    pub cudaFreeArray:            dl::Symbol<unsafe extern "C" fn(array: cudaArray_t) -> cudaError_t>,
    pub cudaFreeHost:             dl::Symbol<unsafe extern "C" fn(ptr: *mut ::core::ffi::c_void) -> cudaError_t>,
    pub cudaFreeMipmappedArray:   dl::Symbol<unsafe extern "C" fn(mipmappedArray: cudaMipmappedArray_t) -> cudaError_t>,
    pub cudaHostAlloc:            dl::Symbol<unsafe extern "C" fn(pHost: *mut *mut ::core::ffi::c_void, size: usize, flags: ::core::ffi::c_uint) -> cudaError_t>,
    pub cudaMalloc:               dl::Symbol<unsafe extern "C" fn(devPtr: *mut *mut ::core::ffi::c_void, size: usize) -> cudaError_t>,
    pub cudaMalloc3D:             dl::Symbol<unsafe extern "C" fn(pitchedDevPtr: *mut cudaPitchedPtr, extent: cudaExtent) -> cudaError_t>,
    pub cudaMalloc3DArray:        dl::Symbol<unsafe extern "C" fn(array: *mut cudaArray_t, desc: *const cudaChannelFormatDesc, extent: cudaExtent, flags: ::core::ffi::c_uint) -> cudaError_t>,
    pub cudaMallocArray:          dl::Symbol<unsafe extern "C" fn(array: *mut cudaArray_t, desc: *const cudaChannelFormatDesc, width: usize, height: usize, flags: ::core::ffi::c_uint) -> cudaError_t>,
    pub cudaMallocHost:           dl::Symbol<unsafe extern "C" fn(ptr: *mut *mut ::core::ffi::c_void, size: usize) -> cudaError_t>,
    pub cudaMallocMipmappedArray: dl::Symbol<unsafe extern "C" fn(mipmappedArray: *mut cudaMipmappedArray_t, desc: *const cudaChannelFormatDesc, extent: cudaExtent, numLevels: ::core::ffi::c_uint, flags: ::core::ffi::c_uint) -> cudaError_t>,
    pub cudaMallocPitch:          dl::Symbol<unsafe extern "C" fn(devPtr: *mut *mut ::core::ffi::c_void, pitch: *mut usize, width: usize, height: usize) -> cudaError_t>,
}

impl CudaFunctions {
    pub unsafe fn new() -> Result<Self, libloading::Error> {
        let candidates = if cfg!(target_os = "windows") {
            vec![
                "cudart64_121.dll",
                "cudart64_120.dll",
                "cudart64_12.dll",
                "cudart64_110.dll",
                "cudart64_101.dll",
                "cudart64_91.dll",
                "cudart64_90.dll",
                "cudart64_80.dll",
                "cudart64_75.dll",
                "cudart64_65.dll",
            ]
        } else {
            vec![
                "libcudart.so",
                "/usr/local/cuda/lib64/libcudart.so",
                // "/usr/local/cuda-10.0/targets/amd64-linux/lib/libcudart.so",
            ]
        };
        let mut cudart = None;
        for filename in candidates {
            if let Ok(l) = unsafe { dl::Library::new(filename) } {
                cudart = Some(l);
                log::debug!("Loaded {}", &filename);
                break;
            }
        }
        if cudart.is_none() { return Err(libloading::Error::DlOpenUnknown); }
        let cudart = cudart.unwrap();

        unsafe {
            Ok(Self {
                cudaFree:                 cudart.get(b"cudaFree")?,
                cudaFreeArray:            cudart.get(b"cudaFreeArray")?,
                cudaFreeHost:             cudart.get(b"cudaFreeHost")?,
                cudaFreeMipmappedArray:   cudart.get(b"cudaFreeMipmappedArray")?,
                cudaHostAlloc:            cudart.get(b"cudaHostAlloc")?,
                cudaMalloc:               cudart.get(b"cudaMalloc")?,
                cudaMalloc3D:             cudart.get(b"cudaMalloc3D")?,
                cudaMalloc3DArray:        cudart.get(b"cudaMalloc3DArray")?,
                cudaMallocArray:          cudart.get(b"cudaMallocArray")?,
                cudaMallocHost:           cudart.get(b"cudaMallocHost")?,
                cudaMallocMipmappedArray: cudart.get(b"cudaMallocMipmappedArray")?,
                cudaMallocPitch:          cudart.get(b"cudaMallocPitch")?,

                _cudart: cudart,
            })
        }
    }
}

pub static CUDA: LazyLock<Result<CudaFunctions, libloading::Error>> = LazyLock::new(|| unsafe { CudaFunctions::new() });
