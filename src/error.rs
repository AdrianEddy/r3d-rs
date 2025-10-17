// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use crate::enums::*;

#[derive(Clone, Debug)]
pub enum RedError {

    // Initialization errors
    LibraryNotLoaded,
    R3DSDKLibraryNotFound,
    RedCudaLibraryNotFound,
    RedOpenCLLibraryNotFound,
    R3DDecoderLibraryNotFound,
    RedMetalLibraryNotFound,
    LibraryVersionMismatch,
    InvalidR3DSDKLibrary,
    InvalidRedCudaLibrary,
    InvalidRedOpenCLLibrary,
    InvalidR3DDecoderLibrary,
    InvalidRedMetalLibrary,
    RedCudaLibraryInitializeFailed,
    RedOpenCLLibraryInitializeFailed,
    R3DDecoderLibraryInitializeFailed,
    R3DSDKLibraryInitializeFailed,
    RedMetalLibraryInitializeFailed,
    InvalidPath,
    InternalError,
    MetalNotAvailable,

    // Clip errors
    ClipPathNotFound,
    ClipFailedToOpenFile,
    ClipNotAnR3DFile,
    ClipIsEmpty,
    ClipOutOfMemory,
    ClipUnknownError,
    ClipNoClipOpen,
    ClipNotInitialized,

    // Decode errors
    OutputBufferInvalid,
    RequestOutOfRange,
    InvalidParameter,
    IsDroppedFrame,
    DecodeFailed,
    OutOfMemory,
    UnknownError,
    NoClipOpen,
    CannotReadFromFile,
    InvalidPixelType,
    NotAnHDRxClip,
    Cancelled,
    UnsupportedClipFormat,
    ParameterUnsupported,
    DecoderNotOpened,

    // R3D status
	ErrorProcessing,
	InvalidJobParameter,
    /// mode value passed in is not compatible with this SDK or the mode used with the DecodeForGpuSdk call is not compatible
	InvalidJobParameterMode,
    /// pointer is NULL, data is not from DecodeForGpuSdk, R3DSDK and GPUSDK versions are incompatible or buffer is not actually in host memory.
	InvalidJobParameterRawHostMem,
	InvalidJobParameterRawDeviceMem,
    /// unsupported pixel type
	InvalidJobParameterPixelType,
    /// Output buffer Size must be non zero.
    /// Output buffer must be allocated prior to passing it into the sdk
	InvalidJobParameterOutputDeviceMemSize,
	InvalidJobParameterOutputDeviceMem,
    /// Image processing settings ColorVersion was set to ColorVersion1 which is not supported by this SDK
	InvalidJobParameterColorVersion1,
    /// null, or unopened clip.
	InvalidJobParameterClip,
    /// GPU Device did not meet minimum requirements.
	UnableToUseGPUDevice,
    /// No GPU Devices were setup on the R3DDecoderOptions class
    NoGPUDeviceSpecified,
    /// Error loading R3DSDK dynamic library
    UnableToLoadLibrary,

    InvalidJobParameterDeviceId,

    BufferNotAligned,
    InvalidAPIObject,

    CudaError(i32),
    OpenCLError(i32),
    MetalError(i32),

    BufferTooSmall { needed: usize, provided: usize },
    Alloc(std::alloc::LayoutError),
    Other(String),
}

impl std::fmt::Display for RedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LibraryNotLoaded                  => write!(f, "Library not loaded"),
            Self::R3DSDKLibraryNotFound             => write!(f, "R3DSDK library not found"),
            Self::RedCudaLibraryNotFound            => write!(f, "RED CUDA library not found"),
            Self::RedOpenCLLibraryNotFound          => write!(f, "RED OpenCL library not found"),
            Self::R3DDecoderLibraryNotFound         => write!(f, "R3D Decoder library not found"),
            Self::RedMetalLibraryNotFound           => write!(f, "RED Metal library not found"),
            Self::LibraryVersionMismatch            => write!(f, "Library version mismatch"),
            Self::InvalidR3DSDKLibrary              => write!(f, "Invalid R3DSDK library"),
            Self::InvalidRedCudaLibrary             => write!(f, "Invalid RED CUDA library"),
            Self::InvalidRedOpenCLLibrary           => write!(f, "Invalid RED OpenCL library"),
            Self::InvalidR3DDecoderLibrary          => write!(f, "Invalid R3D Decoder library"),
            Self::InvalidRedMetalLibrary            => write!(f, "Invalid RED Metal library"),
            Self::RedCudaLibraryInitializeFailed    => write!(f, "RED CUDA library initialize failed"),
            Self::RedOpenCLLibraryInitializeFailed  => write!(f, "RED OpenCL library initialize failed"),
            Self::R3DDecoderLibraryInitializeFailed => write!(f, "R3D Decoder library initialize failed"),
            Self::R3DSDKLibraryInitializeFailed     => write!(f, "R3DSDK library initialize failed"),
            Self::RedMetalLibraryInitializeFailed   => write!(f, "RED Metal library initialize failed"),
            Self::InvalidPath                       => write!(f, "Invalid path"),
            Self::InternalError                     => write!(f, "Internal error"),
            Self::MetalNotAvailable                 => write!(f, "Metal not available"),

            Self::ClipPathNotFound                  => write!(f, "Clip path not found"),
            Self::ClipFailedToOpenFile              => write!(f, "Clip failed to open file"),
            Self::ClipNotAnR3DFile                  => write!(f, "Clip is not an R3D file"),
            Self::ClipIsEmpty                       => write!(f, "Clip is empty"),
            Self::ClipOutOfMemory                   => write!(f, "Clip out of memory"),
            Self::ClipUnknownError                  => write!(f, "Clip unknown error"),
            Self::ClipNoClipOpen                    => write!(f, "No clip open"),
            Self::ClipNotInitialized                => write!(f, "Clip not initialized"),

            Self::OutputBufferInvalid               => write!(f, "Output buffer invalid"),
            Self::RequestOutOfRange                 => write!(f, "Request out of range"),
            Self::InvalidParameter                  => write!(f, "Invalid parameter"),
            Self::IsDroppedFrame                    => write!(f, "Is dropped frame"),
            Self::DecodeFailed                      => write!(f, "Decode failed"),
            Self::OutOfMemory                       => write!(f, "Out of memory"),
            Self::UnknownError                      => write!(f, "Unknown error"),
            Self::NoClipOpen                        => write!(f, "No clip open"),
            Self::CannotReadFromFile                => write!(f, "Cannot read from file"),
            Self::InvalidPixelType                  => write!(f, "Invalid pixel type"),
            Self::NotAnHDRxClip                     => write!(f, "Not an HDRx clip"),
            Self::Cancelled                         => write!(f, "Cancelled"),
            Self::UnsupportedClipFormat             => write!(f, "Unsupported clip format"),
            Self::ParameterUnsupported              => write!(f, "Parameter unsupported"),
            Self::DecoderNotOpened                  => write!(f, "Decoder not opened"),

            Self::ErrorProcessing                   => write!(f, "Error processing"),
            Self::InvalidJobParameter               => write!(f, "Invalid job parameter"),
            Self::InvalidJobParameterMode           => write!(f, "Invalid job parameter: mode"),
            Self::InvalidJobParameterRawHostMem     => write!(f, "Invalid job parameter: raw host memory"),
            Self::InvalidJobParameterRawDeviceMem   => write!(f, "Invalid job parameter: raw device memory"),
            Self::InvalidJobParameterPixelType      => write!(f, "Invalid job parameter: pixel type"),
            Self::InvalidJobParameterOutputDeviceMemSize => write!(f, "Invalid job parameter: output device memory size"),
            Self::InvalidJobParameterOutputDeviceMem => write!(f, "Invalid job parameter: output device memory"),
            Self::InvalidJobParameterColorVersion1  => write!(f, "Invalid job parameter: ColorVersion1"),
            Self::InvalidJobParameterClip           => write!(f, "Invalid job parameter: clip"),
            Self::UnableToUseGPUDevice              => write!(f, "Unable to use GPU device"),
            Self::NoGPUDeviceSpecified              => write!(f, "No GPU device specified"),
            Self::UnableToLoadLibrary               => write!(f, "Unable to load library"),
            Self::InvalidJobParameterDeviceId       => write!(f, "Invalid job parameter: device ID"),

            Self::BufferNotAligned                  => write!(f, "Buffer not aligned"),
            Self::InvalidAPIObject                  => write!(f, "Invalid API object"),
            Self::BufferTooSmall { needed, provided } => write!(f, "Buffer too small: needed {needed} bytes, provided {provided} bytes"),

            Self::CudaError(cuda_error)             => write!(f, "CUDA error: {cuda_error}"),
            Self::OpenCLError(opencl_error)         => write!(f, "OpenCL error: {opencl_error}"),
            Self::MetalError(metal_error)           => write!(f, "Metal error: {metal_error}"),
            Self::Alloc(e)                          => write!(f, "Allocation error: {e}"),

            Self::Other(s)                          => write!(f, "RED error: {s}"),
        }
    }
}
impl std::error::Error for RedError { }

impl From<InitializeStatus> for RedError {
    fn from(value: InitializeStatus) -> Self {
        match value {
            InitializeStatus::LibraryNotLoaded                  => Self::LibraryNotLoaded,
            InitializeStatus::R3DSDKLibraryNotFound             => Self::R3DSDKLibraryNotFound,
            InitializeStatus::RedCudaLibraryNotFound            => Self::RedCudaLibraryNotFound,
            InitializeStatus::RedOpenCLLibraryNotFound          => Self::RedOpenCLLibraryNotFound,
            InitializeStatus::R3DDecoderLibraryNotFound         => Self::R3DDecoderLibraryNotFound,
            InitializeStatus::RedMetalLibraryNotFound           => Self::RedMetalLibraryNotFound,
            InitializeStatus::LibraryVersionMismatch            => Self::LibraryVersionMismatch,
            InitializeStatus::InvalidR3DSDKLibrary              => Self::InvalidR3DSDKLibrary,
            InitializeStatus::InvalidRedCudaLibrary             => Self::InvalidRedCudaLibrary,
            InitializeStatus::InvalidRedOpenCLLibrary           => Self::InvalidRedOpenCLLibrary,
            InitializeStatus::InvalidR3DDecoderLibrary          => Self::InvalidR3DDecoderLibrary,
            InitializeStatus::InvalidRedMetalLibrary            => Self::InvalidRedMetalLibrary,
            InitializeStatus::RedCudaLibraryInitializeFailed    => Self::RedCudaLibraryInitializeFailed,
            InitializeStatus::RedOpenCLLibraryInitializeFailed  => Self::RedOpenCLLibraryInitializeFailed,
            InitializeStatus::R3DDecoderLibraryInitializeFailed => Self::R3DDecoderLibraryInitializeFailed,
            InitializeStatus::R3DSDKLibraryInitializeFailed     => Self::R3DSDKLibraryInitializeFailed,
            InitializeStatus::RedMetalLibraryInitializeFailed   => Self::RedMetalLibraryInitializeFailed,
            InitializeStatus::InvalidPath                       => Self::InvalidPath,
            InitializeStatus::InternalError                     => Self::InternalError,
            InitializeStatus::MetalNotAvailable                 => Self::MetalNotAvailable,
            InitializeStatus::Ok                                => panic!("Cannot convert ISInitializeOK to RedError"),
        }
    }
}
impl From<LoadStatus> for RedError {
    fn from(value: LoadStatus) -> Self {
        match value {
            LoadStatus::ClipLoaded        => panic!("Cannot convert LSClipLoaded to RedError"),
            LoadStatus::PathNotFound      => Self::ClipPathNotFound,
            LoadStatus::FailedToOpenFile  => Self::ClipFailedToOpenFile,
            LoadStatus::NotAnR3DFile      => Self::ClipNotAnR3DFile,
            LoadStatus::ClipIsEmpty       => Self::ClipIsEmpty,
            LoadStatus::OutOfMemory       => Self::ClipOutOfMemory,
            LoadStatus::UnknownError      => Self::ClipUnknownError,
            LoadStatus::NoClipOpen        => Self::ClipNoClipOpen,
            LoadStatus::NotInitialized    => Self::ClipNotInitialized,
        }
    }
}

impl From<DecodeStatus> for RedError {
    fn from(value: DecodeStatus) -> Self {
        match value {
            DecodeStatus::Ok               => panic!("Cannot convert DSDecodeOK to RedError"),
            DecodeStatus::OutputBufferInvalid    => Self::OutputBufferInvalid,
            DecodeStatus::RequestOutOfRange      => Self::RequestOutOfRange,
            DecodeStatus::InvalidParameter       => Self::InvalidParameter,
            DecodeStatus::IsDroppedFrame         => Self::IsDroppedFrame,
            DecodeStatus::DecodeFailed           => Self::DecodeFailed,
            DecodeStatus::OutOfMemory            => Self::OutOfMemory,
            DecodeStatus::UnknownError           => Self::UnknownError,
            DecodeStatus::NoClipOpen             => Self::NoClipOpen,
            DecodeStatus::CannotReadFromFile     => Self::CannotReadFromFile,
            DecodeStatus::InvalidPixelType       => Self::InvalidPixelType,
            DecodeStatus::NotAnHDRxClip          => Self::NotAnHDRxClip,
            DecodeStatus::Cancelled              => Self::Cancelled,
            DecodeStatus::UnsupportedClipFormat  => Self::UnsupportedClipFormat,
            DecodeStatus::ParameterUnsupported   => Self::ParameterUnsupported,
            DecodeStatus::DecoderNotOpened       => Self::DecoderNotOpened,
        }
    }
}

impl From<R3DStatus> for RedError {
    fn from(value: R3DStatus) -> Self {
        match value {
            R3DStatus::Ok                              => panic!("Cannot convert R3DStatus_Ok to RedError"),
            R3DStatus::ErrorProcessing                 => Self::ErrorProcessing,
            R3DStatus::InvalidJobParameter             => Self::InvalidJobParameter,
            R3DStatus::InvalidJobParameterMode         => Self::InvalidJobParameterMode,
            R3DStatus::InvalidJobParameterRawHostMem   => Self::InvalidJobParameterRawHostMem,
            R3DStatus::InvalidJobParameterRawDeviceMem => Self::InvalidJobParameterRawDeviceMem,
            R3DStatus::InvalidJobParameterPixelType    => Self::InvalidJobParameterPixelType,
            R3DStatus::InvalidJobParameterOutputDeviceMemSize => Self::InvalidJobParameterOutputDeviceMemSize,
            R3DStatus::InvalidJobParameterOutputDeviceMem => Self::InvalidJobParameterOutputDeviceMem,
            R3DStatus::InvalidJobParameterColorVersion1 => Self::InvalidJobParameterColorVersion1,
            R3DStatus::InvalidJobParameterClip         => Self::InvalidJobParameterClip,
            R3DStatus::UnableToUseGPUDevice            => Self::UnableToUseGPUDevice,
            R3DStatus::NoGPUDeviceSpecified            => Self::NoGPUDeviceSpecified,
            R3DStatus::UnableToLoadLibrary             => Self::UnableToLoadLibrary,
            R3DStatus::ParameterUnsupported            => Self::ParameterUnsupported,
        }
    }
}
impl From<std::alloc::LayoutError> for RedError {
    fn from(value: std::alloc::LayoutError) -> Self {
        Self::Alloc(value)
    }
}

pub type RedResult<T> = Result<T, RedError>;
