// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[repr(i32)]
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








//////////////////////////////////////// CUDA FFI ////////////////////////////////////////

#[cfg(target_os = "windows")]
use libloading::os::windows as dl;
#[cfg(target_os = "linux")]
use libloading::os::unix as dl;

use std::sync::LazyLock;

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

    pub cudaFree:                 dl::Symbol<unsafe extern "C" fn(ptr: *mut ::core::ffi::c_void) -> i32>,
    pub cudaFreeArray:            dl::Symbol<unsafe extern "C" fn(array: cudaArray_t) -> i32>,
    pub cudaFreeHost:             dl::Symbol<unsafe extern "C" fn(ptr: *mut ::core::ffi::c_void) -> i32>,
    pub cudaFreeMipmappedArray:   dl::Symbol<unsafe extern "C" fn(mipmappedArray: cudaMipmappedArray_t) -> i32>,
    pub cudaHostAlloc:            dl::Symbol<unsafe extern "C" fn(pHost: *mut *mut ::core::ffi::c_void, size: usize, flags: ::core::ffi::c_uint) -> i32>,
    pub cudaMalloc:               dl::Symbol<unsafe extern "C" fn(devPtr: *mut *mut ::core::ffi::c_void, size: usize) -> i32>,
    pub cudaMalloc3D:             dl::Symbol<unsafe extern "C" fn(pitchedDevPtr: *mut cudaPitchedPtr, extent: cudaExtent) -> i32>,
    pub cudaMalloc3DArray:        dl::Symbol<unsafe extern "C" fn(array: *mut cudaArray_t, desc: *const cudaChannelFormatDesc, extent: cudaExtent, flags: ::core::ffi::c_uint) -> i32>,
    pub cudaMallocArray:          dl::Symbol<unsafe extern "C" fn(array: *mut cudaArray_t, desc: *const cudaChannelFormatDesc, width: usize, height: usize, flags: ::core::ffi::c_uint) -> i32>,
    pub cudaMallocHost:           dl::Symbol<unsafe extern "C" fn(ptr: *mut *mut ::core::ffi::c_void, size: usize) -> i32>,
    pub cudaMallocMipmappedArray: dl::Symbol<unsafe extern "C" fn(mipmappedArray: *mut cudaMipmappedArray_t, desc: *const cudaChannelFormatDesc, extent: cudaExtent, numLevels: ::core::ffi::c_uint, flags: ::core::ffi::c_uint) -> i32>,
    pub cudaMallocPitch:          dl::Symbol<unsafe extern "C" fn(devPtr: *mut *mut ::core::ffi::c_void, pitch: *mut usize, width: usize, height: usize) -> i32>,
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
