// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use cpp::*;
use crate::{ RedResult, RedError, Sdk, enums::*, image_processing_settings::ImageProcessingSettings };
use std::sync::Arc;
use crate::ffi::check_status;
use core::ffi::c_void;

cpp!{{
    #include "R3DSDKCuda.h"
}}

/// Opaque handle to CUDA stream from cudart
pub type CudaStream = *mut c_void;

/// A created `R3DSDK::REDCuda`, shared with its debayer jobs, which release
/// themselves through it.
struct RedCudaInner {
    ptr: *mut c_void,
    sdk: Sdk,
}
// SAFETY: `Send + Sync`. The SDK header documents this lane debayer's
// `process_blocking` as a "Thread safe blocking call" (see below); `ptr` is an
// opaque owned `R3DSDK::REDCuda *` with no thread affinity. The CUDA "set the
// device on the calling thread" requirement is a call-site contract on
// `process_blocking`, not an object-ownership constraint. (No `#[cfg]` needed —
// this whole module is gated `not(target_os = "macos")` in `debayer/mod.rs`.)
unsafe impl Send for RedCudaInner {}
unsafe impl Sync for RedCudaInner {}
impl Drop for RedCudaInner {
    fn drop(&mut self) {
        let cuda = self.ptr;
        cpp!(unsafe [cuda as "R3DSDK::REDCuda *"] { delete cuda; });
    }
}

/// The CUDA debayer of the SDK's GPU decode path.
/// Needs the SDK initialized with [`InitializeFlags::Cuda`].
pub struct RedCuda {
    inner: Arc<RedCudaInner>,
}

impl RedCuda {
    /// Create REDCuda wrapper and bind cudart functions dynamically
    pub fn new(sdk: &Sdk) -> RedResult<Self> {
        sdk.require(InitializeFlags::Cuda)?;
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

        Ok(Self { inner: Arc::new(RedCudaInner { ptr, sdk: sdk.clone() }) })
    }

    /// The SDK this debayer was created with.
    pub(crate) fn sdk(&self) -> &Sdk {
        &self.inner.sdk
    }

    /// Fail unless `job` was created by this debayer.
    fn check_job(&self, job: &DebayerCudaJob) -> RedResult<()> {
        if Arc::ptr_eq(&job.owner, &self.inner) { Ok(()) } else { Err(RedError::InvalidJobParameter) }
    }

    /// Create a debayer job for this debayer.
    pub fn create_debayer_job(&self) -> DebayerCudaJob {
        let cuda = self.inner.ptr;
        let ptr = cpp!(unsafe [cuda as "R3DSDK::REDCuda *"] -> *mut c_void as "void*" {
            return (void*)cuda->createDebayerJob();
        });
        DebayerCudaJob { ptr, owner: self.inner.clone(), settings: None }
    }

    /// checks to see if a device and stream queue are compatible with REDCuda.
    /// This call may take several seconds on slower cards
    ///
    /// # Safety
    /// `stream` must be a CUDA stream of device `device_id` (or null for its
    /// default stream), and `device_id` must be the current device of the
    /// calling thread.
    pub unsafe fn check_compatibility(&self, device_id: i32, stream: CudaStream) -> RedResult<()> {
        let cuda = self.inner.ptr;
        let mut cuda_error: cudaError_t = 0;
        let cuda_error_ptr = &mut cuda_error;
        check_status::<CudaStatus>(cpp!(unsafe [cuda as "R3DSDK::REDCuda *", device_id as "int", stream as "cudaStream_t", cuda_error_ptr as "cudaError_t *"] -> i32 as "int" {
            return (int)cuda->checkCompatibility(device_id, stream, *cuda_error_ptr);
        }))?;
        if cuda_error == 0 { Ok(()) } else { Err(RedError::CudaError(cuda_error)) }
    }

    /// processes a frame.
    /// Thread safe blocking call
    /// This function will enqueue the cuda kernels and wait for the processing to complete
    ///
    /// To ensure memory buffers are not released prior to completion of kernel deviceId and stream are used as tokens for internal object mapping.
    /// Be sure you have already done setCudaDevice on the thread using the passed in deviceId prior to calling process
    ///
    /// # Safety
    /// `stream` must be a CUDA stream of device `device_id` (or null for its
    /// default stream), and `device_id` must be the current device of the
    /// calling thread.
    /// The memory `debayer_job` points to (see its `unsafe` setters) must be
    /// valid until this returns.
    pub unsafe fn process_blocking(&self, device_id: i32, stream: CudaStream, debayer_job: &mut DebayerCudaJob) -> RedResult<()> {
        self.check_job(debayer_job)?;
        let cuda = self.inner.ptr;
        let job = debayer_job.ptr;
        let mut cuda_error: cudaError_t = 0;
        let cuda_error_ptr = &mut cuda_error;
        check_status::<CudaStatus>(cpp!(unsafe [cuda as "R3DSDK::REDCuda *", device_id as "int", stream as "cudaStream_t", job as "R3DSDK::DebayerCudaJob *", cuda_error_ptr as "cudaError_t *"] -> i32 as "int" {
            return (int)cuda->process(device_id, stream, job, *cuda_error_ptr);
        }))?;
        if cuda_error == 0 { Ok(()) } else { Err(RedError::CudaError(cuda_error)) }
    }
    /// enqueues all red processing on the current stream
    /// once this has been called you will need to call [`DebayerCudaJob::complete_async`]
    ///
    /// This will wait for commands enqueued by this sdk for the current frame to complete and then releases any additional resources.
    /// If the commands have already completed there is no wait involved
    ///
    /// # Safety
    /// `stream` must be a CUDA stream of device `device_id` (or null for its
    /// default stream), and `device_id` must be the current device of the
    /// calling thread.
    /// The handles and the memory `debayer_job` points to (see its `unsafe`
    /// setters) must stay valid until [`DebayerCudaJob::complete_async`] returns or the
    /// job is dropped.
    pub unsafe fn process_async(&self, device_id: i32, stream: CudaStream, debayer_job: &mut DebayerCudaJob) -> RedResult<()> {
        self.check_job(debayer_job)?;
        let cuda = self.inner.ptr;
        let job_ptr = debayer_job.ptr;
        let mut cuda_error: cudaError_t = 0;
        let cuda_error_ptr = &mut cuda_error;
        check_status::<CudaStatus>(cpp!(unsafe [cuda as "R3DSDK::REDCuda *", device_id as "int", stream as "cudaStream_t", job_ptr as "R3DSDK::DebayerCudaJob *", cuda_error_ptr as "cudaError_t *"] -> i32 as "int" {
            return (int)cuda->processAsync(device_id, stream, job_ptr, *cuda_error_ptr);
        }))?;
        if cuda_error == 0 { Ok(()) } else { Err(RedError::CudaError(cuda_error)) }
    }
}

/// A debayer job for [`RedCuda`].
pub struct DebayerCudaJob {
    ptr: *mut c_void,
    /// The debayer that created the job, which releases it.
    owner: Arc<RedCudaInner>,
    /// The settings the SDK reads, owned by the job.
    settings: Option<Box<ImageProcessingSettings>>,
}
// SAFETY: `Send` only — single-owner cursor-state mutator consumed once per
// submission. Concurrent `&self` would race the SDK's internal state, so not
// `Sync` (pinned negative in `tests/send_sync_surface.rs`).
unsafe impl Send for DebayerCudaJob {}

impl DebayerCudaJob {
    pub fn as_mut_ptr(&self) -> *mut c_void { self.ptr }

    pub fn result_frame_size(&self) -> usize {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "const R3DSDK::DebayerCudaJob *"] -> usize as "size_t" {
            return R3DSDK::DebayerCudaJob::ResultFrameSize(ptr);
        })
    }

    /// The frame to debayer: the output buffer of a completed decompress
    /// (an [`AsyncDecompressJob`](crate::AsyncDecompressJob) of
    /// [`AsyncDecoder`](crate::AsyncDecoder) or [`GpuDecoder`](crate::GpuDecoder)).
    ///
    /// # Safety
    /// `p` must point to such a buffer and stay valid until processing of the job is complete: until
    /// [`RedCuda::process_blocking`] returns, or after [`RedCuda::process_async`]
    /// until [`complete_async`](Self::complete_async) returns or the job is dropped.
    pub unsafe fn set_raw_host_mem(&mut self, p: *mut c_void) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerCudaJob *", p as "void *"] {
            ptr->raw_host_mem = p;
        });
    }
    /// The same data as the raw host memory, already on the GPU (or, for
    /// [`GpuDecoder`](crate::GpuDecoder) output, filled in by the SDK).
    ///
    /// # Safety
    /// `p` must be a CUDA buffer on the processing device at least as large
    /// as the raw host memory, and stay valid until processing of the job is complete: until
    /// [`RedCuda::process_blocking`] returns, or after [`RedCuda::process_async`]
    /// until [`complete_async`](Self::complete_async) returns or the job is dropped.
    pub unsafe fn set_raw_device_mem(&mut self, p: *mut c_void) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerCudaJob *", p as "void *"] {
            ptr->raw_device_mem = p;
        });
    }
    /// The CUDA buffer to write the debayered image to, of `size` bytes.
    ///
    /// # Safety
    /// `p` must be such a buffer on the processing device, valid for writes of
    /// `size` bytes until processing of the job is complete: until
    /// [`RedCuda::process_blocking`] returns, or after [`RedCuda::process_async`]
    /// until [`complete_async`](Self::complete_async) returns or the job is dropped.
    pub unsafe fn set_output_device_mem(&mut self, p: *mut c_void, size: usize) {
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

    /// Image processing settings to apply. These must be set. The job keeps
    /// its own copy.
    pub fn set_image_processing(&mut self, v: &ImageProcessingSettings) {
        let ptr = self.ptr;
        let v: *const ImageProcessingSettings = &**self.settings.insert(Box::new(*v));
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
        let owner = self.owner.ptr;
        let ptr = self.ptr;
        cpp!(unsafe [owner as "R3DSDK::REDCuda *", ptr as "R3DSDK::DebayerCudaJob *"] {
            owner->releaseDebayerJob(ptr);
        });
    }
}

sdk_enum! {
    /// Status of a [`RedCuda`] call.
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
        InvalidJobParameter_deviceId = 13,
    }
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

/// The CUDA runtime functions this crate uses, loaded from the CUDA runtime library at runtime.
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
    pub cudaStreamCreate:         dl::Symbol<unsafe extern "C" fn(pStream: *mut *mut ::core::ffi::c_void) -> cudaError_t>,
    pub cudaStreamDestroy:        dl::Symbol<unsafe extern "C" fn(stream: *mut ::core::ffi::c_void) -> cudaError_t>,
    pub cudaMemcpyAsync:          dl::Symbol<unsafe extern "C" fn(dst: *mut ::core::ffi::c_void, src: *const ::core::ffi::c_void, count: usize, kind: cudaMemcpyKind, stream: *mut ::core::ffi::c_void) -> cudaError_t>,
    pub cudaStreamSynchronize:    dl::Symbol<unsafe extern "C" fn(stream: *mut ::core::ffi::c_void) -> cudaError_t>,
}

pub type cudaMemcpyKind = i32;
/// `cudaMemcpyHostToDevice`
pub const CUDA_MEMCPY_HOST_TO_DEVICE: cudaMemcpyKind = 1;

impl CudaFunctions {
    /// Load the CUDA runtime library and look up its functions.
    ///
    /// # Safety
    /// Loading a library runs its initialization code; see [`libloading::Library::new`].
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
            ]
        };
        let mut cudart = None;
        for filename in candidates {
            if let Ok(l) = unsafe { dl::Library::new(filename) } {
                cudart = Some(l);
                log::debug!("Loaded {filename}");
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
                cudaStreamCreate:         cudart.get(b"cudaStreamCreate")?,
                cudaStreamDestroy:        cudart.get(b"cudaStreamDestroy")?,
                cudaMemcpyAsync:          cudart.get(b"cudaMemcpyAsync")?,
                cudaStreamSynchronize:    cudart.get(b"cudaStreamSynchronize")?,

                _cudart: cudart,
            })
        }
    }
}

/// The CUDA runtime, loaded on first use.
pub static CUDA: LazyLock<Result<CudaFunctions, libloading::Error>> = LazyLock::new(|| unsafe { CudaFunctions::new() });
