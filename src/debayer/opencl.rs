// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[repr(i32)]
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

//////////////////////////////////////// OpenCL FFI ////////////////////////////////////////

#[cfg(target_os = "windows")]
use libloading::os::windows as dl;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use libloading::os::unix as dl;

use std::sync::LazyLock;

use core::ffi::{ c_char, c_uchar, c_void };
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
