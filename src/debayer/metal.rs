// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[repr(i32)]
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
