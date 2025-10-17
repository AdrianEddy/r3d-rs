// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use bitflags::bitflags;
bitflags! {
    /// Bit flags that can be passed to InitializeSdk() through the optional_components parameter
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
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
    /// Interleaved BGR 8-bit
    Bgr8bitInterleaved      = 0x42475238,
    /// Interleaved BGRA decoding in 8 bits per pixel, alpha channel = 0xFF
    Bgra8bitInterleaved     = 0x42524138,
    /// Interleaved RGB 10-bit DPX Method B
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

/// HDR read status & write setting when reading & writing RMD sidecar files
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum HdrMode {
   /// Use A / main (#0) or X (#1) track
   UseTrackNo = 0,
   /// Blend A and X tracks using specified settings
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
    Ok = 0,
    LibraryNotLoaded = 1,

    R3DSDKLibraryNotFound = 2,
    RedCudaLibraryNotFound = 3,
    RedOpenCLLibraryNotFound = 4,
    R3DDecoderLibraryNotFound = 5,
    RedMetalLibraryNotFound = 17,

    LibraryVersionMismatch = 6,

    InvalidR3DSDKLibrary = 7,
    InvalidRedCudaLibrary = 8,
    InvalidRedOpenCLLibrary = 9,
    InvalidR3DDecoderLibrary = 10,
    InvalidRedMetalLibrary = 18,

    RedCudaLibraryInitializeFailed = 11,
    RedOpenCLLibraryInitializeFailed = 12,
    R3DDecoderLibraryInitializeFailed = 13,
    R3DSDKLibraryInitializeFailed = 14,
    RedMetalLibraryInitializeFailed = 19,

    InvalidPath = 15,
    InternalError = 16,

    MetalNotAvailable = 20,
}

/// Clip load status
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum LoadStatus {
    ClipLoaded = 0,
    /// could not find the clip
    PathNotFound = 1,
    /// could not open the clip
    FailedToOpenFile = 2,
    /// clip does not appear to be a(n) (valid) R3D file
    NotAnR3DFile = 3,
    /// clip doesn't have any video frames in it
    ClipIsEmpty = 4,
    /// no more memory could be allocated
    OutOfMemory = 5,
    /// unknown error (shouldn't happen)
    UnknownError = 6,
    /// initial status, no clip has been loaded yet
    NoClipOpen = 7,
    /// library wasn't loaded properly (if obj-c, try new() instead)
    NotInitialized = 8,
}

/// Clip create status
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum CreateStatus {
    /// trim has started but has not finished yet
    Started = 0,
    /// a frame has been added to the output clip, but not finished yet
    FrameAdded = 1,
    /// trim is done
    Done = 2,
    /// no more memory could be allocated
    OutOfMemory = 3,
    /// the requested start or end frame does not exist in the clip
    RequestOutOfRange = 4,
    InvalidParameter = 5,
    /// unable to load a frame from the source clip
    FailedToGetSourceFrame = 6,
    /// unable to create output clip
    FailedToCreateDestination = 7,
    /// unable to write to output clip
    FailedToWriteToDestination = 8,
    UnknownError = 9,
    /// source clip cannot be used for trim (RED ONE firmware build 15 and below not supported)
    InvalidSourceClip = 10,
    /// output path is invalid (see trim requirements)
    InvalidPath = 11,
    /// unable to load needed audio samples form source clip
    FailedToGetSourceAudio = 12,
    /// streaming packets need to be added in order
    OutOfOrder = 13,
    /// streaming packet is invalid
    InvalidStream = 14,
}

/// Video decode status
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum DecodeStatus {
    Ok = 0,
    /// null pointer, too small or not aligned
    OutputBufferInvalid = 1,
    /// the requested frame does not exist in the clip
    RequestOutOfRange = 3,
    /// one of the supplied parameters is not valid
    InvalidParameter = 4,
    /// the requested frame is a dropped frame and can't be decoded (video decode only)
    IsDroppedFrame = 5,
    /// decode failed because of a corrupt frame
    DecodeFailed = 6,
    /// no more memory could be allocated
    OutOfMemory = 7,
    /// unknown error (shouldn't happen)
    UnknownError = 8,
    /// open a clip first before trying to decode a frame
    NoClipOpen = 9,
    /// I/O error reading the frame
    CannotReadFromFile = 10,
    /// cannot decode to specified pixel type
    InvalidPixelType = 11,
    /// cannot decode HDRx as this is not an HDRx clip
    NotAnHDRxClip = 12,
    /// user requested decode to be cancelled
    Cancelled = 13,
    /// this clip format is not supported for the requested decode
    UnsupportedClipFormat = 14,
    /// one of the parameters supplied is not supported by this API. Upgrade to newer dynamic libraries.
    ParameterUnsupported = 15,
    /// open async decoder first before submitting decodes
    DecoderNotOpened = 16,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum R3DStatus {
    Ok = 0,
    ErrorProcessing = 1,
    InvalidJobParameter = 2,

    /// mode value passed in is not compatible with this SDK or the mode used with the DecodeForGpuSdk call is not compatible
    InvalidJobParameterMode = 3,

    /// pointer is NULL, data is not from DecodeForGpuSdk, R3DSDK and GPUSDK versions are incompatible or buffer is not actually in host memory.
    InvalidJobParameterRawHostMem = 4,
    InvalidJobParameterRawDeviceMem = 5,

    /// unsupported pixel type
    InvalidJobParameterPixelType = 6,

    /// Output buffer Size must be non zero.
    /// Output buffer must be allocated prior to passing it into the sdk
    InvalidJobParameterOutputDeviceMemSize = 7,
    InvalidJobParameterOutputDeviceMem = 8,

    /// Image processing settings ColorVersion was set to ColorVersion1 which is not supported by this SDK
    InvalidJobParameterColorVersion1 = 9,

    /// null, or unopened clip.
    InvalidJobParameterClip = 10,

    /// GPU Device did not meet minimum requirements.
    UnableToUseGPUDevice = 11,

    /// No GPU Devices were setup on the R3DDecoderOptions class
    NoGPUDeviceSpecified = 12,

    /// Error loading R3DSDK dynamic library
    UnableToLoadLibrary = 13,
    ParameterUnsupported = 14,
}

/// Possible marker types
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum MarkerType {
    /// region, most likely has different start & end frame numbers
    SubClip = 0,
    /// single frame marker, start & end frame number will be the same
    StillFrame = 1,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ImageGammaCurve {
    Linear = 0,
    SRGB = 8,

    HDR2084 = 14,
    BT1886 = 15,
    /// mid grey point maps to 1/3, encodes 12 stops above mid grey
    Log3G12 = 16,
    /// mid grey point maps to 1/3, encodes 10 stops above mid grey
    Log3G10 = 18,
    /// only available with color version 2 or newer, otherwise ignored (REDlog will be used)
    REDlogFilm = 9,

    // Only for IPP2 (ColorVersion3)
    /// color version 3+
    HybridLogGamma = 19,
    /// color version 3+
    Gamma2_2 = 20,
    /// color version 3+
    Gamma2_6 = 21,

    // Only for Legacy (ColorVersion2), deprecated and not available in IPP2 decode mode.
    /// BT1886 will be used in IPP2 mode.
    Rec709 = 1,
    /// v4: only available with color version 2, otherwise ignored
    REDgamma4 = 12,

    // Below gamma curves are deprecated and not available in IPP2 decode mode, only for ColorVersion2.
    PDlog685 = 4,
    PDlog985 = 5,
    /// if used CustomPDLogBlackPoint, WhitePoint & Gamma must be set!
    CustomPDlog = 6,

    /// only available with color version 2, otherwise ignored
    REDspace = 2,
    /// only available with color version 2, otherwise ignored
    REDlog = 3,
    /// v1: only available with color version 2, otherwise ignored
    REDgamma = 7,
    /// v2: only available with color version 2, otherwise ignored
    REDgamma2 = 10,
    /// v3: only available with color version 2, otherwise ignored
    REDgamma3 = 11,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ImageColorSpace {
    /// only available with color version 2 or newer, otherwise ignored (CameraRGB will be used)
    REDWideGamutRGB = 15,

    /// only available with color version 2 or newer, otherwise ignored (REDspace will be used)
    Rec2020 = 13,
    Rec709 = 1,
    SRGB = 4,
    Adobe1998 = 5,
    /// only available with color version 2 or newer
    DCIP3 = 16,
    /// only available with color version 2 or newer
    ProPhotoRGB = 17,

    // Only for IPP2 (ColorVersion3)
    /// only available with color version 3 or newer
    DCIP3D65 = 18,

    // Only for Legacy (ColorVersion2), deprecated and not available in IPP2 decode mode.
    /// v2: only available with color version 2, otherwise ignored
    DRAGONcolor2 = 12,
    /// v4: only available with color version 2, otherwise ignored
    REDcolor4 = 11,

    // Below color spaces are deprecated and not available in IPP2 decode mode, only for ColorVersion2.
    CameraRGB = 0,
    REDspace = 2,
    /// v1: only available with color version 2, otherwise ignored
    REDcolor = 3,
    /// v2: only available with color version 2, otherwise ignored
    REDcolor2 = 6,
    /// v3: only available with color version 2, otherwise ignored
    REDcolor3 = 8,
    /// v1: only available with color version 2, otherwise ignored
    DRAGONcolor = 9,
}

/// IPP2 (ColorVersion3) only
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ImagePipeline {
    /// output in REDWideGamutRGB & Log3G10. Only Kelvin, Tint, ISO and ExposureAdjust available.
    PrimaryDevelopmentOnly = 0,
    /// default
    FullGraded = 1,
}

/// IPP2 (ColorVersion3) only
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum RollOff {
    VerySoft = 4,
    Soft = 3,
    Medium = 2,
    Hard = 1,
    /// advanced option
    None = 0,
}

/// IPP2 (ColorVersion3) only
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ToneMap {
    Low = 0,
    Medium = 1,
    High = 2,
    None = 3,
}

/// Amount of detail extraction (not sharpening!) for
/// the full resolution premium software decode only!
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ImageDetail {
    Low = 0,
    Medium = 1,
    /// default
    High = 2,
}

/// Compensation for the OLPF (low-pass) filter for
/// the half & full resolution premium software decodes
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ImageOLPFCompensation {
    Off = 0,
    Low = 1,
    Medium = 2,
    High = 3,
}

/// Amount of noise reduction to do for the
/// full resolution premium software decode
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

/// Which color version to use with the image processing settings.
/// Default is ColorVersion3, which is the new IPP2 color science.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ColorVersion {
    /// Legacy
    Version1 = 1,
    /// Legacy (FLUT)
    Version2 = 2,
    /// IPP2
    Version3 = 3,
    VersionBC = 4,
}

/// Legacy (FLUT) only. Lift, Gamma and Gain settings.
#[repr(C)]
pub struct LiftGammaGain {
    /// -1 -- 1
    pub lift: f32,
    /// 0 -- 4
    pub gamma: f32,
    /// 0 -- 2
    pub gain: f32,
}

/// IPP2 only, Color Decision List settings.
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
