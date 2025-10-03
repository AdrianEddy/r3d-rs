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

/// The different resolutions and qualities the clip can be
/// decoded at. This list expand over time.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum VideoDecodeMode {
	/// 'DFRP', full resolution, slow, but highest resolution & quality
    FullResPremium   = 0x44465250,
    /// 'DHRP', half resolution, slow, but highest quality
    HalfResPremium   = 0x44485250,
    /// 'DHRG', half resolution, fast, still very good quality
    HalfResGood      = 0x44485247,
    /// 'DQRG', quarter resolution, fast, very good quality
    QuarterResGood   = 0x44515247,
    /// 'DERG', eight resolution, fast, good quality
    EightResGood     = 0x44455247,
    /// 'DSRG', sixteenth resolution, fast, good quality
    SixteenthResGood = 0x44535247,
}

/// The different pixel types to decode images at. The 16-bit RGB
/// planar format has always existed in the SDK but is not supported
/// for RED Rocket decoding. The other two formats are interleaved
/// and are supported for both software and RED Rocket decoding.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum VideoPixelType {
    /// Interleaved RGB decoding in 16-bits per pixel 4K decodes in this 16-bit pixel type using the Rocket will not be real time.
    /// Use the 12- or 10-bit pixel types below for 4K real time needs
	/// Supported by software and RED Rocket decode
    Rgb16bitInterleaved     = 0x52423649,
    /// 16-bit half-float decoding. These ImageProcessingSettings fields are ignored:
	/// - GammaCurve (will always be linear)
	/// - Contrast
	/// - Brightness
	/// - Saturation
	/// - RGB gains
	/// - Shadow
	/// - UserCurve, UserCurveRed, UserCurveGreen & UserCurveBlue
	/// - LggRed, LggGreen & LggBlue
	/// - CustomPDLogBlackPoint, CustomPDLogWhitePoint & CustomPDLogGamma
    RgbHalfFloatInterleaved = 0x52424846,
    /// 16-bit half-float ACES decoding. These ImageProcessingSettings fields are ignored:
	/// - Same fields as PixelType_HalfFloat_RGB_Interleaved pixel type
	/// - ColorSpace (will always be set to ACES AP0)
    RgbHalfFloatAcesInt     = 0x52424841,
    /// Planar RGB decoding in 16-bits per pixel
	/// Supported by software decode only
    Rgb16bitPlanar          = 0x52423650,
    // Interleaved BGR 8-bit
    Bgr8bitInterleaved      = 0x42475238,
    // Interleaved BGRA decoding in 8 bits per pixel, alpha channel = 0xFF
    Bgra8bitInterleaved     = 0x42524138,
    // Interleaved RGB 10-bit DPX Method B
    Dpx10bitMethodB         = 0x44503042,
}

/// HDRx blending algorithm to use when doing HDR blending
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum HdrBlendAlgorithm {
    /// Simple blend, exactly as implemented in 12_SimpleHDRxBlend
	/// On the Rocket this will ALWAYS decode in 16-bit to do the blend
	/// This can result in slow downs at full resolution. If you need a
	/// preview option in 8-bit then decode 2 images in 8-bit and
	/// implement the simple blend as per sample code 12_SimpleHDRxBlend
    HDRxSimpleBlend = 0x48445253,
    /// Magic Motion, more sophisticated blend
    HDRxMagicMotion = 0x4844524D,
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
    /// in microseconds, see RMD_FRAME_TIMESTAMP + Clip::frame_metadata()
    pub timestamp: u64,
    /// in G's
    pub acceleration: Axes,
    /// in deg/s
    pub rotation: Axes,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Axes {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
