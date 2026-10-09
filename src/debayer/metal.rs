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
    #include "R3DSDKMetal.h"
    #include <cstring>

    // Owned (+1) `id<MTLBuffer>` handles passed to Rust as `void *`; see
    // `new_managed_buffer` / `write_managed_buffer` / `release_buffer` below.
    static void * r3drs_new_managed_buffer(id<MTLCommandQueue> queue, size_t size) {
        id<MTLBuffer> buffer = [[queue device] newBufferWithLength:size options:MTLResourceStorageModeManaged];
    #if __has_feature(objc_arc)
        return (__bridge_retained void *)buffer;
    #else
        return (void *)buffer; // `new…` already returns a +1 reference
    #endif
    }
    static void r3drs_write_managed_buffer(void * handle, const void * src, size_t len) {
    #if __has_feature(objc_arc)
        id<MTLBuffer> buffer = (__bridge id<MTLBuffer>)handle;
    #else
        id<MTLBuffer> buffer = (id<MTLBuffer>)handle;
    #endif
        memcpy([buffer contents], src, len);
        [buffer didModifyRange:NSMakeRange(0, len)];
    }
    static void r3drs_release_buffer(void * handle) {
        if (handle) CFRelease(handle);
    }
}}

/// Allocate a `MTLResourceStorageModeManaged` `MTLBuffer` of `size` bytes on
/// the device that owns `queue` (an `id<MTLCommandQueue>`), as RED's Metal
/// samples do for `raw_device_mem`. Returns an owned (+1) `id<MTLBuffer>` to
/// free with [`release_buffer`], or null if Metal could not allocate it.
pub(crate) fn new_managed_buffer(queue: *mut c_void, size: usize) -> *mut c_void {
    cpp!(unsafe [queue as "id<MTLCommandQueue>", size as "size_t"] -> *mut c_void as "void *" {
        return r3drs_new_managed_buffer(queue, size);
    })
}

/// Copy `len` bytes from `src` into a managed `MTLBuffer` and flag the range
/// as modified so Metal synchronises it to the GPU.
///
/// # Safety
/// `buffer` must come from [`new_managed_buffer`] with a size of at least
/// `len`, `src` must be readable for `len` bytes, and no GPU work may be
/// using `buffer`.
pub(crate) unsafe fn write_managed_buffer(buffer: *mut c_void, src: *const c_void, len: usize) {
    cpp!(unsafe [buffer as "void *", src as "const void *", len as "size_t"] {
        r3drs_write_managed_buffer(buffer, src, len);
    })
}

/// Release a buffer from [`new_managed_buffer`].
///
/// # Safety
/// `buffer` must be an owned handle from [`new_managed_buffer`] that no GPU
/// work is still using, and must not be used afterwards.
pub(crate) unsafe fn release_buffer(buffer: *mut c_void) {
    cpp!(unsafe [buffer as "void *"] {
        r3drs_release_buffer(buffer);
    })
}

/// A created `R3DSDK::REDMetal`, shared with its debayer jobs, which release
/// themselves through it.
struct RedMetalInner {
    ptr: *mut c_void,
    sdk: Sdk,
}
// SAFETY: `Send + Sync`. The SDK header documents this lane debayer's
// `process_blocking` as a "Thread safe blocking call" (see below); `ptr` is an
// opaque owned `R3DSDK::REDMetal *` with no thread affinity. (No `#[cfg]`
// needed — this whole module is gated `feature = "metal-debayer"` in
// `debayer/mod.rs`.)
unsafe impl Send for RedMetalInner {}
unsafe impl Sync for RedMetalInner {}
impl Drop for RedMetalInner {
    fn drop(&mut self) {
        let metal = self.ptr;
        cpp!(unsafe [metal as "R3DSDK::REDMetal *"] { delete metal; });
    }
}

/// The Metal debayer of the SDK's GPU decode path.
/// Needs the SDK initialized with [`InitializeFlags::Metal`].
pub struct RedMetal {
    inner: Arc<RedMetalInner>,
}

impl RedMetal {
    /// Create REDMetal wrapper
    pub fn new(sdk: &Sdk) -> RedResult<Self> {
        sdk.require(InitializeFlags::Metal)?;
        let ptr = cpp!(unsafe [] -> *mut c_void as "R3DSDK::REDMetal *" {
            R3DSDK::EXT_METAL_API api;
            return new R3DSDK::REDMetal(api);
        });

        Ok(Self { inner: Arc::new(RedMetalInner { ptr, sdk: sdk.clone() }) })
    }

    /// The SDK this debayer was created with.
    pub(crate) fn sdk(&self) -> &Sdk {
        &self.inner.sdk
    }

    /// Fail unless `job` was created by this debayer.
    fn check_job(&self, job: &DebayerMetalJob) -> RedResult<()> {
        if Arc::ptr_eq(&job.owner, &self.inner) { Ok(()) } else { Err(RedError::InvalidJobParameter) }
    }

    /// Create a debayer job for this debayer.
    pub fn create_debayer_job(&self) -> DebayerMetalJob {
        let metal = self.inner.ptr;
        let ptr = cpp!(unsafe [metal as "R3DSDK::REDMetal *"] -> *mut c_void as "void*" {
            return (void*)metal->createDebayerJob();
        });
        DebayerMetalJob { ptr, owner: self.inner.clone(), settings: None }
    }

    /// checks to see if a context and command queue are compatible with REDMetal
    /// This call may take several seconds on slower cards
    ///
    /// # Safety
    /// `queue` must be a valid `id<MTLCommandQueue>`.
    pub unsafe fn check_compatibility(&self, queue: *mut c_void) -> RedResult<()> {
        let metal = self.inner.ptr;
        let mut error: i32 = 0;
        let error_ptr = &mut error;
        check_status::<MetalStatus>(cpp!(unsafe [metal as "R3DSDK::REDMetal *", queue as "id<MTLCommandQueue>", error_ptr as "int *"] -> i32 as "int" {
            return (int)metal->checkCompatibility(queue, *error_ptr);
        }))?;
        if error == 0 { Ok(()) } else { Err(RedError::MetalError(error)) }
    }

    /// processes a frame.
    /// Thread safe blocking call
    /// This function will enqueue the Metal commands and wait for the processing to complete
    /// to ensure memory buffers are not released prior to completion of kernel
    ///
    /// # Safety
    /// `queue` must be a valid `id<MTLCommandQueue>`.
    /// The memory `debayer_job` points to (see its `unsafe` setters) must be
    /// valid until this returns.
    pub unsafe fn process_blocking(&self, queue: *mut c_void, debayer_job: &mut DebayerMetalJob) -> RedResult<()> {
        self.check_job(debayer_job)?;
        let metal = self.inner.ptr;
        let job = debayer_job.ptr;
        let mut error: i32 = 0;
        let error_ptr = &mut error;
        check_status::<MetalStatus>(cpp!(unsafe [metal as "R3DSDK::REDMetal *", queue as "id<MTLCommandQueue>", job as "R3DSDK::DebayerMetalJob *", error_ptr as "int *"] -> i32 as "int" {
            return (int)metal->process(queue, job, *error_ptr);
        }))?;
        if error == 0 { Ok(()) } else { Err(RedError::MetalError(error)) }
    }

    /// enqueues all red processing on the current command queue. NOTE: All calls to this
    /// should be done using a single MTLCommandQueue to avoid potential MTLCommandBuffer failures
    /// once this has been called you will need to call [`DebayerMetalJob::complete_async`]
    /// This will wait for commands enqueued by this sdk for the current frame to complete and then releases any additional resources.
    /// If the commands have already completed there is no wait involved
    ///
    /// # Safety
    /// `queue` must be a valid `id<MTLCommandQueue>`.
    /// The handles and the memory `debayer_job` points to (see its `unsafe`
    /// setters) must stay valid until [`DebayerMetalJob::complete_async`] returns or the
    /// job is dropped.
    pub unsafe fn process_async(&self, queue: *mut c_void, debayer_job: &mut DebayerMetalJob) -> RedResult<()> {
        self.check_job(debayer_job)?;
        let metal = self.inner.ptr;
        let job_ptr = debayer_job.ptr;
        let mut error: i32 = 0;
        let error_ptr = &mut error;
        check_status::<MetalStatus>(cpp!(unsafe [metal as "R3DSDK::REDMetal *", queue as "id<MTLCommandQueue>", job_ptr as "R3DSDK::DebayerMetalJob *", error_ptr as "int *"] -> i32 as "int" {
            return (int)metal->processAsync(queue, job_ptr, *error_ptr);
        }))?;
        if error == 0 { Ok(()) } else { Err(RedError::MetalError(error)) }
    }

    /// flushes the current batch job that has been queued for the current MTLCommandQueue
    /// after this has been called the jobs in the current batch will be processed in the order they were submitted
    /// [`DebayerMetalJob::complete_async`] will return after each job in the batch completes
    ///
    /// # Safety
    /// `queue` must be a valid `id<MTLCommandQueue>`.
    pub unsafe fn flush(&self, queue: *mut c_void) -> RedResult<()> {
        let metal = self.inner.ptr;
        let mut error: i32 = 0;
        let error_ptr = &mut error;
        check_status::<MetalStatus>(cpp!(unsafe [metal as "R3DSDK::REDMetal *", queue as "id<MTLCommandQueue>", error_ptr as "int *"] -> i32 as "int" {
            return (int)metal->flush(queue, *error_ptr);
        }))?;
        if error == 0 { Ok(()) } else { Err(RedError::MetalError(error)) }
    }
}

/// A debayer job for [`RedMetal`].
pub struct DebayerMetalJob {
    ptr: *mut c_void,
    /// The debayer that created the job, which releases it.
    owner: Arc<RedMetalInner>,
    /// The settings the SDK reads, owned by the job.
    settings: Option<Box<ImageProcessingSettings>>,
}
// SAFETY: `Send` only — single-owner cursor-state mutator consumed once per
// submission. Concurrent `&self` would race the SDK's internal state, so not
// `Sync` (pinned negative in `tests/send_sync_surface.rs`).
unsafe impl Send for DebayerMetalJob {}

impl DebayerMetalJob {
    pub fn as_mut_ptr(&self) -> *mut c_void { self.ptr }

    pub fn result_frame_size(&self) -> usize {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "const R3DSDK::DebayerMetalJob *"] -> usize as "size_t" {
            return R3DSDK::DebayerMetalJob::ResultFrameSize(*ptr);
        })
    }

    /// NOTE: This pointer + MTLBuffer & their contents must remain valid until [`complete_async`](DebayerMetalJob::complete_async) or [`RedMetal::process_blocking`] returns!
    ///
    /// The frame to debayer: the output buffer of a completed decompress
    /// (an [`AsyncDecompressJob`](crate::AsyncDecompressJob) of
    /// [`AsyncDecoder`](crate::AsyncDecoder) or [`GpuDecoder`](crate::GpuDecoder)).
    ///
    /// # Safety
    /// `p` must point to such a buffer and stay valid until processing of the job is complete: until
    /// [`RedMetal::process_blocking`] returns, or after [`RedMetal::process_async`]
    /// until [`complete_async`](Self::complete_async) returns or the job is dropped.
    pub unsafe fn set_raw_host_mem(&mut self, p: *mut c_void) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *", p as "void *"] { ptr->raw_host_mem = p; });
    }
    /// The same data as the raw host memory, already on the GPU (or, for
    /// [`GpuDecoder`](crate::GpuDecoder) output, filled in by the SDK).
    ///
    /// # Safety
    /// `p` must be a Metal buffer on the processing device at least as large
    /// as the raw host memory, and stay valid until processing of the job is complete: until
    /// [`RedMetal::process_blocking`] returns, or after [`RedMetal::process_async`]
    /// until [`complete_async`](Self::complete_async) returns or the job is dropped.
    pub unsafe fn set_raw_device_mem(&mut self, p: *mut c_void) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *", p as "id<MTLBuffer>"] { ptr->raw_device_mem = p; });
    }

    /// result/output buffer on the GPU
    /// object of the size output_device_mem_size
    /// -- do not use output_device_image when using this field
    ///
    /// The Metal buffer to write the debayered image to, of `size` bytes.
    ///
    /// # Safety
    /// `p` must be such a buffer on the processing device, valid for writes of
    /// `size` bytes until processing of the job is complete: until
    /// [`RedMetal::process_blocking`] returns, or after [`RedMetal::process_async`]
    /// until [`complete_async`](Self::complete_async) returns or the job is dropped.
    pub unsafe fn set_output_device_mem(&mut self, p: *mut c_void, size: usize) {
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
    /// The `id<MTLTexture>` to write the debayered image to instead of a
    /// buffer (16-bit uint and float formats are supported), with `size` the
    /// byte size of its storage.
    ///
    /// # Safety
    /// `p` must be such a texture on the processing device, valid until processing of the job is complete: until
    /// [`RedMetal::process_blocking`] returns, or after [`RedMetal::process_async`]
    /// until [`complete_async`](Self::complete_async) returns or the job is dropped.
    pub unsafe fn set_output_device_image(&mut self, p: *mut c_void, size: usize) {
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

    /// Image processing settings to apply. These must be set. The job keeps
    /// its own copy.
    pub fn set_image_processing(&mut self, v: &ImageProcessingSettings) {
        let ptr = self.ptr;
        let v: *const ImageProcessingSettings = &**self.settings.insert(Box::new(*v));
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *", v as "R3DSDK::ImageProcessingSettings *"] {
            ptr->imageProcessingSettings = v;
        });
    }

    /// Supported pixel types are:
    /// * [`VideoPixelType::Rgb16bitInterleaved`]
    /// * [`VideoPixelType::RgbHalfFloatInterleaved`]
    /// * [`VideoPixelType::RgbHalfFloatAcesInt`]
    pub fn set_pixel_type(&mut self, pixel_type: VideoPixelType) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *", pixel_type as "int"] {
            ptr->pixelType = (R3DSDK::VideoPixelType)pixel_type;
        });
    }

    /// This will ensure all objects used for the frame are disposed of.
    /// This call will block until the debayer on the queue executes, if the debayer has already executed no blocking will occur
    /// This function must be called at a point after passing the job into [`RedMetal::process_async`]
    /// This will automatically be called when the job is dropped if it is not manually called earlier
    pub fn complete_async(&mut self) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *"] {
            ptr->completeAsync();
        });
    }

    /// Enables batch processing of frames - frames added to a single batch will all complete together when flush() is called
    /// `true` to add frame to the batch, queuing up the work
    /// `false` to add frame to the batch, and then complete the whole batch
    /// Leaving the field `false` will cause frames to be processed individually
    /// Recommended batch size: 2 for laptops, 4 for iMac Pro, 8 for Mac Pro (2019)
    pub fn set_batch_mode(&mut self, v: bool) {
        let ptr = self.ptr;
        cpp!(unsafe [ptr as "R3DSDK::DebayerMetalJob *", v as "bool"] {
            ptr->batchMode = v;
        });
    }
}

impl Drop for DebayerMetalJob {
    fn drop(&mut self) {
        // Ensure completion
        self.complete_async();
        let owner = self.owner.ptr;
        let ptr = self.ptr;
        cpp!(unsafe [owner as "R3DSDK::REDMetal *", ptr as "R3DSDK::DebayerMetalJob *"] {
            owner->releaseDebayerJob(ptr);
        });
    }
}

sdk_enum! {
    /// Status of a [`RedMetal`] call.
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
        InvalidAPIObject = 13,
    }
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
