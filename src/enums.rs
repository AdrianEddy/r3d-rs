// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use bitflags::bitflags;
bitflags! {
    /// Bit flags that can be passed to InitializeSdk() through the optional_components parameter
    pub struct InitializeFlags: u32 {
        const None = 0;
        const Cuda = 0x01;
        const OpenCL = 0x02;
        /// exclusive, do not combine with OPTION_RED_CUDA, OPTION_RED_OPENCL or OPTION_RED_METAL flags
        const R3DDecoder = 0x04;
        /// macOS only
        const Metal = 0x08;
        /// OpenCL only: don't pre-compile kernels in REDCL::checkCompatibility(), instead compile them when needed
        const DelayGpuCompile = 0x10;
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum FileId {
    /// unknown/can't open or read
	Unknown  = 0,
    /// original .R3D with multi-file/spanning clips (A001_C001_1231XY_001...999.R3D, A001_C002_1231XY_001...999.R3D, etc.)
	R3D      = 1,
    /// Nikon .NEV N-RAW with single file per clip (DSC_0001.NEV, DSC_0002.NEV, etc.)
	NevNraw = 3,
    /// .R3D with single file per clip (DSC_0001.R3D, DSC_0002.R3D, etc.)
	R3dNe   = 4
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum FileType {
    /// fileIdx invalid in FileList() call
	Invalid  = 0,

	// Files that are part of the clip itself, there's always at least 1 of these
    /// .R3D or .NEV
	Clip     = 1,

	// Optional sidecar / external files
    /// .RMD or .cdl
	Metadata = 2,
    /// .cube
	Lut      = 3,
    /// .wav
	Audio    = 5
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum VideoDecodeMode {
    FullResPremium   = 1145459280,
    HalfResPremium   = 1145590352,
    HalfResGood      = 1145590343,
    QuarterResGood   = 1146180167,
    EightResGood     = 1145393735,
    SixteenthResGood = 1146311239,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum VideoPixelType {
    Rgb16bitInterleaved     = 1380070985,
    RgbHalfFloatInterleaved = 1380075590,
    RgbHalfFloatAcesInt     = 1380075585,
    Rgb16bitPlanar          = 1380070992,
    Bgr8bitInterleaved      = 1111970360,
    Bgra8bitInterleaved     = 1112686904,
    Dpx10bitMethodB         = 1146105922,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum HdrBlendAlgorithm {
    HDRxSimpleBlend = 1212437075,
    HDRxMagicMotion = 1212437069,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum HdrMode {
   UseTrackNo = 0,
   DoBlend = 1,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum MetadataType {
    Invalid = 0,
    Int = 1,
    String = 2,
    Float = 3,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum InitializeStatus {
    ISInitializeOK = 0,
    ISLibraryNotLoaded = 1,
    ISR3DSDKLibraryNotFound = 2,
    ISRedCudaLibraryNotFound = 3,
    ISRedOpenCLLibraryNotFound = 4,
    ISR3DDecoderLibraryNotFound = 5,
    ISRedMetalLibraryNotFound = 17,
    ISLibraryVersionMismatch = 6,
    ISInvalidR3DSDKLibrary = 7,
    ISInvalidRedCudaLibrary = 8,
    ISInvalidRedOpenCLLibrary = 9,
    ISInvalidR3DDecoderLibrary = 10,
    ISInvalidRedMetalLibrary = 18,
    ISRedCudaLibraryInitializeFailed = 11,
    ISRedOpenCLLibraryInitializeFailed = 12,
    ISR3DDecoderLibraryInitializeFailed = 13,
    ISR3DSDKLibraryInitializeFailed = 14,
    ISRedMetalLibraryInitializeFailed = 19,
    ISInvalidPath = 15,
    ISInternalError = 16,
    ISMetalNotAvailable = 20,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum LoadStatus {
    LSClipLoaded = 0,
    LSPathNotFound = 1,
    LSFailedToOpenFile = 2,
    LSNotAnR3DFile = 3,
    LSClipIsEmpty = 4,
    LSOutOfMemory = 5,
    LSUnknownError = 6,
    LSNoClipOpen = 7,
    LSNotInitialized = 8,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum CreateStatus {
    CSStarted = 0,
    CSFrameAdded = 1,
    CSDone = 2,
    CSOutOfMemory = 3,
    CSRequestOutOfRange = 4,
    CSInvalidParameter = 5,
    CSFailedToGetSourceFrame = 6,
    CSFailedToCreateDestination = 7,
    CSFailedToWriteToDestination = 8,
    CSUnknownError = 9,
    CSInvalidSourceClip = 10,
    CSInvalidPath = 11,
    CSFailedToGetSourceAudio = 12,
    CSOutOfOrder = 13,
    CSInvalidStream = 14,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum DecodeStatus {
    DSDecodeOK = 0,
    DSOutputBufferInvalid = 1,
    DSRequestOutOfRange = 3,
    DSInvalidParameter = 4,
    DSIsDroppedFrame = 5,
    DSDecodeFailed = 6,
    DSOutOfMemory = 7,
    DSUnknownError = 8,
    DSNoClipOpen = 9,
    DSCannotReadFromFile = 10,
    DSInvalidPixelType = 11,
    DSNotAnHDRxClip = 12,
    DSCancelled = 13,
    DSUnsupportedClipFormat = 14,
    DSParameterUnsupported = 15,
    DSDecoderNotOpened = 16,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum R3DStatus {
    Ok = 0,
    ErrorProcessing = 1,
    InvalidJobParameter = 2,
    InvalidJobParameterMode = 3,
    InvalidJobParameterRawHostMem = 4,
    InvalidJobParameterRawDeviceMem = 5,
    InvalidJobParameterPixelType = 6,
    InvalidJobParameterOutputDeviceMemSize = 7,
    InvalidJobParameterOutputDeviceMem = 8,
    InvalidJobParameterColorVersion1 = 9,
    InvalidJobParameterClip = 10,
    UnableToUseGPUDevice = 11,
    NoGPUDeviceSpecified = 12,
    UnableToLoadLibrary = 13,
    ParameterUnsupported = 14,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum MarkerType {
    SubClip = 0,
    StillFrame = 1,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ImageGammaCurve {
    Linear = 0,
    SRGB = 8,
    HDR2084 = 14,
    BT1886 = 15,
    Log3G12 = 16,
    Log3G10 = 18,
    REDlogFilm = 9,
    HybridLogGamma = 19,
    Gamma2_2 = 20,
    Gamma2_6 = 21,
    Rec709 = 1,
    REDgamma4 = 12,
    PDlog685 = 4,
    PDlog985 = 5,
    CustomPDlog = 6,
    REDspace = 2,
    REDlog = 3,
    REDgamma = 7,
    REDgamma2 = 10,
    REDgamma3 = 11,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ImageColorSpace {
    REDWideGamutRGB = 15,
    Rec2020 = 13,
    Rec709 = 1,
    SRGB = 4,
    Adobe1998 = 5,
    DCIP3 = 16,
    ProPhotoRGB = 17,
    DCIP3D65 = 18,
    DRAGONcolor2 = 12,
    REDcolor4 = 11,
    CameraRGB = 0,
    REDspace = 2,
    REDcolor = 3,
    REDcolor2 = 6,
    REDcolor3 = 8,
    DRAGONcolor = 9,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ImagePipeline {
    PrimaryDevelopmentOnly = 0,
    FullGraded = 1,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum RollOff {
    VerySoft = 4,
    Soft = 3,
    Medium = 2,
    Hard = 1,
    None = 0,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ToneMap {
    Low = 0,
    Medium = 1,
    High = 2,
    None = 3,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ImageDetail {
    Low = 0,
    Medium = 1,
    High = 2,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ImageOLPFCompensation {
    Off = 0,
    Low = 1,
    Medium = 2,
    High = 3,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ImageDenoise {
    Off = 0,
    Minimum = 1,
    Milder = 2,
    Mild = 3,
    Medium = 4,
    Strong = 5,
    Maximum = 6,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum FlashingPixelAdjust {
    Off = 0,
    Mild = 1179667780,
    Medium = 1179667789,
    Strong = 1179669319,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ColorVersion {
    Version1 = 1,
    Version2 = 2,
    Version3 = 3,
    VersionBC = 4,
}
#[repr(C)]
pub struct LiftGammaGain {
    pub lift: f32,
    pub gamma: f32,
    pub gain: f32,
}
#[repr(C)]
pub struct SlopeOffsetPower {
    pub slope: f32,
    pub offset: f32,
    pub power: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct IMUSample {
    pub timestamp: u64,
    pub acceleration: Axes,
    pub rotation: Axes,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Axes {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
