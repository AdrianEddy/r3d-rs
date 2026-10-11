// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use bitflags::bitflags;
bitflags! {
    /// Optional SDK components to load in [`Sdk::initialize`](crate::Sdk::initialize).
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub struct InitializeFlags: u32 {
        const None = 0;
        const Cuda = 0x01;
        const OpenCL = 0x02;
        /// Load the R3DDecoder library. This flag is exclusive: pass it
        /// without `Cuda`, `OpenCL` or `Metal`.
        const R3DDecoder = 0x04;
        /// Available on macOS only.
        const Metal = 0x08;
        /// Only affects OpenCL. Skips building the GPU kernels up front in
        /// [`RedOpenCl::check_compatibility`](crate::RedOpenCl::check_compatibility)
        /// and builds each one on demand instead.
        const DelayGpuCompile = 0x10;
    }
}

sdk_enum! {
    /// File format of a clip.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum FileId {
        /// Not a recognized format, or the file could not be opened or read.
        Unknown = 0,
        /// RED's original `.R3D` format. One clip may span several numbered
        /// files, e.g. `A001_C001_1231XY_001.R3D`, `A001_C001_1231XY_002.R3D`.
        R3D = 1,
        /// Nikon N-RAW. Every `.NEV` file is a separate clip, e.g. `DSC_0001.NEV`.
        NevNraw = 3,
        /// Nikon R3D NE. Every `.R3D` file is a separate clip, e.g. `DSC_0001.R3D`.
        R3dNe = 4,
    }
}

sdk_enum! {
    /// Role of a file that belongs to a clip.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum FileType {
        /// The file index is out of range.
        Invalid = 0,

        // Media that makes up the clip. Every clip has one or more of these.
        /// A `.R3D` or `.NEV` media file.
        Clip = 1,

        // Sidecar files stored next to the clip. Any of them may be missing.
        /// A `.RMD` or `.cdl` metadata sidecar.
        Metadata = 2,
        /// A `.cube` LUT.
        Lut = 3,
        /// A `.wav` audio file.
        Audio = 5,
    }
}

sdk_enum! {
    /// Resolution and quality to decode at. Premium modes are slow and give
    /// the best image the SDK can produce. Good modes are fast; their output
    /// is very good at half and quarter resolution and good at smaller sizes.
    /// Later SDK releases may add more modes.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum VideoDecodeMode {
        /// Full resolution, premium (slow). FourCC `DFRP`.
        FullResPremium   = 0x44465250,
        /// Half resolution, premium (slow). FourCC `DHRP`.
        HalfResPremium   = 0x44485250,
        /// Half resolution, good (fast). FourCC `DHRG`.
        HalfResGood      = 0x44485247,
        /// Quarter resolution, good (fast). FourCC `DQRG`.
        QuarterResGood   = 0x44515247,
        /// One-eighth resolution, good (fast). FourCC `DERG`.
        EightResGood     = 0x44455247,
        /// One-sixteenth resolution, good (fast). FourCC `DSRG`.
        SixteenthResGood = 0x44535247,
    }
}

sdk_enum! {
    /// Pixel format to decode into. CPU decodes accept every variant. GPU
    /// decodes accept only [`Rgb16bitInterleaved`](Self::Rgb16bitInterleaved),
    /// [`RgbHalfFloatInterleaved`](Self::RgbHalfFloatInterleaved) and
    /// [`RgbHalfFloatAcesInt`](Self::RgbHalfFloatAcesInt).
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum VideoPixelType {
        /// Interleaved RGB with a 16-bit unsigned integer per channel. CPU and GPU.
        Rgb16bitInterleaved     = 0x52423649,
        /// Interleaved RGB with a 16-bit half float per channel. CPU and GPU.
        /// The output is always linear, and these [`ImageProcessingSettings`]
        /// have no effect:
        /// - [`set_gamma_curve`]
        /// - [`set_contrast`], [`set_brightness`] and [`set_saturation`]
        /// - [`set_gain_red`], [`set_gain_green`] and [`set_gain_blue`]
        /// - [`set_shadow`]
        /// - all user curves ([`user_curve_mut`] and the red, green and blue
        ///   variants)
        /// - lift/gamma/gain ([`lgg_red_mut`] and the green and blue variants)
        /// - the CDL ([`set_cdl_saturation`], [`cdl_red_mut`] and the green and
        ///   blue variants, [`set_cdl_enabled`])
        /// - the custom PDlog parameters ([`set_custom_pdlog_black_point`],
        ///   [`set_custom_pdlog_white_point`], [`set_custom_pdlog_gamma`])
        ///
        /// [`ImageProcessingSettings`]: crate::ImageProcessingSettings
        /// [`set_gamma_curve`]: crate::ImageProcessingSettings::set_gamma_curve
        /// [`set_contrast`]: crate::ImageProcessingSettings::set_contrast
        /// [`set_brightness`]: crate::ImageProcessingSettings::set_brightness
        /// [`set_saturation`]: crate::ImageProcessingSettings::set_saturation
        /// [`set_gain_red`]: crate::ImageProcessingSettings::set_gain_red
        /// [`set_gain_green`]: crate::ImageProcessingSettings::set_gain_green
        /// [`set_gain_blue`]: crate::ImageProcessingSettings::set_gain_blue
        /// [`set_shadow`]: crate::ImageProcessingSettings::set_shadow
        /// [`user_curve_mut`]: crate::ImageProcessingSettings::user_curve_mut
        /// [`lgg_red_mut`]: crate::ImageProcessingSettings::lgg_red_mut
        /// [`set_cdl_saturation`]: crate::ImageProcessingSettings::set_cdl_saturation
        /// [`cdl_red_mut`]: crate::ImageProcessingSettings::cdl_red_mut
        /// [`set_cdl_enabled`]: crate::ImageProcessingSettings::set_cdl_enabled
        /// [`set_custom_pdlog_black_point`]: crate::ImageProcessingSettings::set_custom_pdlog_black_point
        /// [`set_custom_pdlog_white_point`]: crate::ImageProcessingSettings::set_custom_pdlog_white_point
        /// [`set_custom_pdlog_gamma`]: crate::ImageProcessingSettings::set_custom_pdlog_gamma
        RgbHalfFloatInterleaved = 0x52424846,
        /// ACES flavor of [`RgbHalfFloatInterleaved`](Self::RgbHalfFloatInterleaved),
        /// also CPU and GPU. It ignores the same settings, and the color space
        /// is fixed to ACES AP0, so
        /// [`set_color_space`](crate::ImageProcessingSettings::set_color_space)
        /// has no effect either.
        RgbHalfFloatAcesInt     = 0x52424841,
        /// Planar RGB with a 16-bit unsigned integer per channel: one full plane
        /// per channel. CPU only. Older SDK versions decoded to this format.
        Rgb16bitPlanar          = 0x52423650,
        /// Interleaved BGR with 8 bits per channel. CPU only.
        Bgr8bitInterleaved      = 0x42475238,
        /// Interleaved BGRA with 8 bits per channel; alpha is always 0xFF. CPU only.
        Bgra8bitInterleaved     = 0x42524138,
        /// 10-bit RGB in DPX Method B packing. CPU only.
        Dpx10bitMethodB         = 0x44503042,
    }
}

sdk_enum! {
    /// How the two exposures of an HDRx clip are merged. Chosen with
    /// [`HdrProcessingSettings::set_hdr_blend_algorithm`](crate::HdrProcessingSettings::set_hdr_blend_algorithm).
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum HdrBlendAlgorithm {
        /// Per-pixel weighted mix of the two tracks. It is the same algorithm
        /// the SDK sample `CPUSimpleHDRxBlendSample.cpp` implements by hand.
        HDRxSimpleBlend = 0x48445253,
        /// RED's Magic Motion blend, which is more involved than
        /// [`HDRxSimpleBlend`](Self::HDRxSimpleBlend).
        HDRxMagicMotion = 0x4844524D,
    }
}

sdk_enum! {
    /// HDRx handling recorded in, or read from, an RMD sidecar file.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum HdrMode {
        /// Show a single track: 0 is the main A track, 1 is the X track.
        UseTrackNo = 0,
        /// Merge the A and X tracks according to the
        /// [`HdrProcessingSettings`](crate::HdrProcessingSettings).
        DoBlend = 1,
    }
}

sdk_enum! {
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum MetadataType {
        Invalid = 0,
        Int = 1,
        String = 2,
        Float = 3,
    }
}

sdk_enum! {
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
        CudaNotAvailable = 21,
    }
}

sdk_enum! {
    /// Outcome of loading a clip, as reported by [`Clip::status`](crate::Clip::status).
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum LoadStatus {
        ClipLoaded = 0,
        /// Nothing exists at the given path.
        PathNotFound = 1,
        /// The clip file exists but could not be opened.
        FailedToOpenFile = 2,
        /// The file is not an R3D clip the SDK recognizes as valid.
        NotAnR3DFile = 3,
        /// The clip contains no video frames.
        ClipIsEmpty = 4,
        /// A memory allocation failed.
        OutOfMemory = 5,
        /// An unexpected internal error.
        UnknownError = 6,
        /// Nothing has been loaded yet. A new clip starts in this state.
        NoClipOpen = 7,
        /// The SDK libraries are not set up correctly.
        NotInitialized = 8,
    }
}

sdk_enum! {
    /// Progress and result codes for writing a new clip (trim or snapshot).
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum CreateStatus {
        /// The trim is underway.
        Started = 0,
        /// Progress report: another frame has been written and the trim is
        /// still running.
        FrameAdded = 1,
        /// The trim completed.
        Done = 2,
        /// A memory allocation failed.
        OutOfMemory = 3,
        /// The start or end frame lies outside the source clip.
        RequestOutOfRange = 4,
        InvalidParameter = 5,
        /// Reading a source frame failed, for example because it is a dropped
        /// frame.
        FailedToGetSourceFrame = 6,
        /// The output clip could not be created.
        FailedToCreateDestination = 7,
        /// Writing to the output clip failed.
        FailedToWriteToDestination = 8,
        UnknownError = 9,
        /// This source clip can't be trimmed. RED ONE footage shot on
        /// firmware build 15 or earlier is not supported.
        InvalidSourceClip = 10,
        /// The output path doesn't meet the SDK's trim requirements.
        InvalidPath = 11,
        /// Reading the source clip's audio failed.
        FailedToGetSourceAudio = 12,
        /// Stream packets arrived out of order.
        OutOfOrder = 13,
        /// A stream packet is malformed.
        InvalidStream = 14,
    }
}

sdk_enum! {
    /// Outcome of decoding a video frame. Failures surface as the matching
    /// [`RedError`](crate::RedError).
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum DecodeStatus {
        Ok = 0,
        /// The output buffer is missing, smaller than the frame needs, or
        /// misaligned.
        OutputBufferInvalid = 1,
        /// The frame number is outside the clip.
        RequestOutOfRange = 3,
        /// A parameter has an invalid value.
        InvalidParameter = 4,
        /// The frame is marked as dropped, so there is no image to decode.
        /// Only video decodes report this.
        IsDroppedFrame = 5,
        /// The frame is corrupt. [`Clip::check_frame`](crate::Clip::check_frame)
        /// also reports this when a frame's CRC doesn't match.
        DecodeFailed = 6,
        /// A memory allocation failed.
        OutOfMemory = 7,
        /// An unexpected internal error.
        UnknownError = 8,
        /// Decoding was attempted before any clip was loaded.
        NoClipOpen = 9,
        /// An I/O failure prevented the frame from being read.
        CannotReadFromFile = 10,
        /// This decode can't produce the requested [`VideoPixelType`].
        InvalidPixelType = 11,
        /// HDRx blending was requested, but the clip has no HDRx track.
        NotAnHDRxClip = 12,
        /// The decode was aborted on request, for example with
        /// [`AsyncDecompressJob::abort`](crate::AsyncDecompressJob::abort).
        Cancelled = 13,
        /// The clip's format can't be decoded this way.
        /// [`Clip::check_frame`](crate::Clip::check_frame) also reports this
        /// when the clip or frame has no CRC.
        UnsupportedClipFormat = 14,
        /// The loaded SDK libraries don't know one of the parameters. Newer
        /// redistributable libraries may be needed.
        ParameterUnsupported = 15,
        /// Jobs were submitted to a decoder that was never opened.
        DecoderNotOpened = 16,
    }
}

sdk_enum! {
    /// Status codes of the [`R3dDecoder`](crate::R3dDecoder) API. Failures
    /// surface as the matching [`RedError`](crate::RedError).
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum R3DStatus {
        Ok = 0,
        ErrorProcessing = 1,
        InvalidJobParameter = 2,

        /// The decode mode is unsupported by this SDK, or differs from the
        /// mode used when the frame went through
        /// [`AsyncDecoder::decode_for_gpu_sdk`](crate::AsyncDecoder::decode_for_gpu_sdk).
        InvalidJobParameterMode = 3,

        /// The raw host buffer is null, wasn't produced by `decode_for_gpu_sdk`,
        /// doesn't actually live in host memory, or the R3D SDK and GPU SDK
        /// library versions don't match.
        InvalidJobParameterRawHostMem = 4,
        InvalidJobParameterRawDeviceMem = 5,

        /// The pixel type can't be used here.
        InvalidJobParameterPixelType = 6,

        /// The output buffer has a size of zero, or it wasn't allocated before
        /// the job was submitted.
        InvalidJobParameterOutputDeviceMemSize = 7,
        InvalidJobParameterOutputDeviceMem = 8,

        /// The image processing settings request [`ColorVersion::Version1`],
        /// which this decoder can't use.
        InvalidJobParameterColorVersion1 = 9,

        /// The job has no clip, or its clip isn't open.
        InvalidJobParameterClip = 10,

        /// The selected GPU falls short of the SDK's minimum requirements.
        UnableToUseGPUDevice = 11,

        /// No GPU was added to the [`R3dDecoderOptions`](crate::R3dDecoderOptions).
        NoGPUDeviceSpecified = 12,

        /// Loading the R3D SDK dynamic library failed.
        UnableToLoadLibrary = 13,
        ParameterUnsupported = 14,
    }
}

sdk_enum! {
    /// Kind of marker stored in a clip.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum MarkerType {
        /// A span of frames. Its end frame is normally later than its start.
        SubClip = 0,
        /// Points at one frame (start equals end).
        StillFrame = 1,
    }
}

sdk_enum! {
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum ImageGammaCurve {
        Linear = 0,
        SRGB = 8,

        HDR2084 = 14,
        BT1886 = 15,
        /// Log curve that puts mid grey at 1/3 of the output range and keeps
        /// 12 stops of headroom above it.
        Log3G12 = 16,
        /// Log curve that puts mid grey at 1/3 of the output range and keeps
        /// 10 stops of headroom above it.
        Log3G10 = 18,
        /// Needs [`ColorVersion::Version2`] or later. Older versions fall back
        /// to [`REDlog`](Self::REDlog).
        REDlogFilm = 9,

        // IPP2 curves (ColorVersion3).
        /// Needs [`ColorVersion::Version3`] or later.
        HybridLogGamma = 19,
        /// Needs [`ColorVersion::Version3`] or later.
        Gamma2_2 = 20,
        /// Needs [`ColorVersion::Version3`] or later.
        Gamma2_6 = 21,

        // Legacy curves (ColorVersion2). Deprecated, and IPP2 decodes can't use them.
        /// IPP2 decodes substitute [`BT1886`](Self::BT1886).
        Rec709 = 1,
        /// REDgamma version 4. Takes effect only with [`ColorVersion::Version2`].
        REDgamma4 = 12,

        // Also legacy-only (ColorVersion2) and deprecated:
        PDlog685 = 4,
        PDlog985 = 5,
        /// Custom PDlog curve. Requires setting the black point, white point
        /// and gamma too, with
        /// [`set_custom_pdlog_black_point`](crate::ImageProcessingSettings::set_custom_pdlog_black_point),
        /// [`set_custom_pdlog_white_point`](crate::ImageProcessingSettings::set_custom_pdlog_white_point)
        /// and [`set_custom_pdlog_gamma`](crate::ImageProcessingSettings::set_custom_pdlog_gamma).
        CustomPDlog = 6,

        /// Takes effect only with [`ColorVersion::Version2`].
        REDspace = 2,
        /// Takes effect only with [`ColorVersion::Version2`].
        REDlog = 3,
        /// REDgamma version 1. Takes effect only with [`ColorVersion::Version2`].
        REDgamma = 7,
        /// REDgamma version 2. Takes effect only with [`ColorVersion::Version2`].
        REDgamma2 = 10,
        /// REDgamma version 3. Takes effect only with [`ColorVersion::Version2`].
        REDgamma3 = 11,
    }
}

sdk_enum! {
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum ImageColorSpace {
        /// Needs [`ColorVersion::Version2`] or later. Older versions fall back
        /// to [`CameraRGB`](Self::CameraRGB).
        REDWideGamutRGB = 15,

        /// Needs [`ColorVersion::Version2`] or later. Older versions fall back
        /// to [`REDspace`](Self::REDspace).
        Rec2020 = 13,
        Rec709 = 1,
        SRGB = 4,
        Adobe1998 = 5,
        /// Needs [`ColorVersion::Version2`] or later.
        DCIP3 = 16,
        /// Needs [`ColorVersion::Version2`] or later.
        ProPhotoRGB = 17,

        // IPP2 color spaces (ColorVersion3).
        /// Needs [`ColorVersion::Version3`] or later.
        DCIP3D65 = 18,

        // Legacy color spaces (ColorVersion2). Deprecated, and IPP2 decodes can't use them.
        /// DRAGONcolor version 2. Takes effect only with [`ColorVersion::Version2`].
        DRAGONcolor2 = 12,
        /// REDcolor version 4. Takes effect only with [`ColorVersion::Version2`].
        REDcolor4 = 11,

        // Also legacy-only (ColorVersion2) and deprecated:
        CameraRGB = 0,
        REDspace = 2,
        /// REDcolor version 1. Takes effect only with [`ColorVersion::Version2`].
        REDcolor = 3,
        /// REDcolor version 2. Takes effect only with [`ColorVersion::Version2`].
        REDcolor2 = 6,
        /// REDcolor version 3. Takes effect only with [`ColorVersion::Version2`].
        REDcolor3 = 8,
        /// DRAGONcolor version 1. Takes effect only with [`ColorVersion::Version2`].
        DRAGONcolor = 9,
    }
}

sdk_enum! {
    /// Pipeline mode for IPP2 ([`ColorVersion::Version3`]) decodes.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum ImagePipeline {
        /// Stop after primary development. The image comes out as
        /// REDWideGamutRGB with the Log3G10 curve, and only [`set_kelvin`],
        /// [`set_tint`], [`set_iso`] and [`set_exposure_adjust`] have any effect.
        ///
        /// [`set_kelvin`]: crate::ImageProcessingSettings::set_kelvin
        /// [`set_tint`]: crate::ImageProcessingSettings::set_tint
        /// [`set_iso`]: crate::ImageProcessingSettings::set_iso
        /// [`set_exposure_adjust`]: crate::ImageProcessingSettings::set_exposure_adjust
        PrimaryDevelopmentOnly = 0,
        /// The complete IPP2 pipeline, including grading. This is the default.
        FullGraded = 1,
    }
}

sdk_enum! {
    /// Highlight roll-off for IPP2 ([`ColorVersion::Version3`]) decodes.
    /// Ignored while a log gamma curve is in use.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum RollOff {
        VerySoft = 4,
        Soft = 3,
        Medium = 2,
        Hard = 1,
        /// No roll-off. Meant for advanced users.
        None = 0,
    }
}

sdk_enum! {
    /// Lens distortion correction for IPP2 ([`ColorVersion::Version3`]) decodes.
    /// Only full and half resolution premium decodes apply it.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum LensCorrection {
        /// Do what the clip metadata says. This is the state while the user
        /// hasn't overridden it.
        Auto = 0,
        /// Always correct, provided the clip carries correction metadata.
        On = 1,
        /// Never correct.
        Off = 2,
    }
}

sdk_enum! {
    /// Output tone mapping strength for IPP2 ([`ColorVersion::Version3`]) decodes.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum ToneMap {
        Low = 0,
        Medium = 1,
        High = 2,
        None = 3,
    }
}

sdk_enum! {
    /// Level of detail extraction, which is distinct from sharpening. Only
    /// CPU decodes in [`VideoDecodeMode::FullResPremium`] use it, and
    /// [`ColorVersion::Version3`] and [`ColorVersion::VersionBC`] ignore it.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum ImageDetail {
        Low = 0,
        Medium = 1,
        /// The default.
        High = 2,
    }
}

sdk_enum! {
    /// Strength of the correction for the camera's optical low-pass filter
    /// (OLPF). Only CPU decodes in [`VideoDecodeMode::FullResPremium`] and
    /// [`VideoDecodeMode::HalfResPremium`] use it, and
    /// [`ColorVersion::Version3`] and [`ColorVersion::VersionBC`] ignore it.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum ImageOLPFCompensation {
        Off = 0,
        Low = 1,
        Medium = 2,
        High = 3,
    }
}

sdk_enum! {
    /// Noise reduction strength. Only CPU decodes in
    /// [`VideoDecodeMode::FullResPremium`] use it, and
    /// [`ColorVersion::Version3`] and [`ColorVersion::VersionBC`] ignore it.
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
}

sdk_enum! {
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum FlashingPixelAdjust {
        Off = 0,
        Mild = 1179667780,
        Medium = 1179667789,
        Strong = 1179669319,
    }
}

sdk_enum! {
    /// Color science generation used by
    /// [`ImageProcessingSettings`](crate::ImageProcessingSettings). New
    /// settings default to [`Version3`](Self::Version3), the current IPP2
    /// pipeline.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum ColorVersion {
        /// Original legacy color science.
        Version1 = 1,
        /// Newer legacy color science that adds FLUT.
        Version2 = 2,
        /// IPP2.
        Version3 = 3,
        /// Broadcast pipeline. Only Broadcast-mode recordings can use it.
        VersionBC = 4,
    }
}

/// Per-channel lift, gamma and gain. Only the legacy
/// [`ColorVersion::Version2`] pipeline uses it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LiftGammaGain {
    /// Range -1 to 1.
    pub lift: f32,
    /// Range 0 to 4.
    pub gamma: f32,
    /// Range 0 to 2.
    pub gain: f32,
}

/// Slope, offset and power for one channel of a CDL (Color Decision List).
/// Used by IPP2 only.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SlopeOffsetPower {
    pub slope: f32,
    pub offset: f32,
    pub power: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct IMUSample {
    /// Time in microseconds. Comparable with the per-frame
    /// `RMD_FRAME_TIMESTAMP` metadata returned by
    /// [`Clip::metadata_for_frame`](crate::Clip::metadata_for_frame).
    pub timestamp: u64,
    /// Acceleration in g.
    pub acceleration: Axes,
    /// Angular rate in degrees per second.
    pub rotation: Axes,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Axes {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
