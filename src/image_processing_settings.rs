// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use crate::enums::*;
use crate::ffi::enum_from_sdk;
use cpp::*;

cpp_class! {
    /// How a decode processes the image: white balance, exposure, color,
    /// gamma and more. A decode given no settings uses the clip's own.
    ///
    /// [`Default`] gives the SDK's generic defaults, not the clip's; start
    /// from [`Clip::default_image_processing_settings`](crate::Clip::default_image_processing_settings)
    /// to match the clip. Values outside the ranges in
    /// [`ImageProcessingLimits`](crate::ImageProcessingLimits) are clamped.
    ///
    /// Which settings apply depends on [`version`](Self::version). In the
    /// method docs, IPP2 means [`ColorVersion::Version3`], Broadcast Color
    /// [`ColorVersion::VersionBC`], and the legacy pipeline
    /// [`ColorVersion::Version1`] or [`ColorVersion::Version2`].
    #[derive(Clone)]
    pub unsafe struct ImageProcessingSettings as "R3DSDK::ImageProcessingSettings"
}
// SAFETY: `Send + Sync`. Every mutator takes `&mut self` (so it cannot run
// concurrently with shared access); the only `&self` getters are read-only or
// return immutable slices (`user_curve(&self) -> &[f32]`). `Send` impl'd
// explicitly (macro elides it); `Sync` auto-derives from the inline storage and
// is sound under those access rules. Pinned by `assert_impl_all!` in
// `tests/send_sync_surface.rs`.
unsafe impl Send for ImageProcessingSettings {}

impl ImageProcessingSettings {
    // --------------------------------------------------------------------------------
    // Core / global
    // --------------------------------------------------------------------------------

    /// Brightness. IPP2 applies it in converted form; Broadcast Color ignores it.
    pub fn     brightness(&self)     -> f32  { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).Brightness; }) }
    pub fn set_brightness(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).Brightness = v; }) }

    /// ISO. A value missing from [`ImageProcessingLimits::iso_list`](crate::ImageProcessingLimits::iso_list) snaps to the closest listed one.
    pub fn     iso(&self)      -> usize { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> usize as "size_t" { return (*self).ISO; }) }
    pub fn set_iso(&mut self, v: usize) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "size_t"] { (*self).ISO = v; }) }

    /// Color science version, the newest (IPP2) by default. A version older
    /// than the clip supports is raised to one the clip can use.
    pub fn     version(&self)     -> ColorVersion { enum_from_sdk(cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).Version; })) }
    pub fn set_version(&mut self, v: ColorVersion) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).Version = (R3DSDK::ColorVersion)v; }) }

    /// White balance color temperature, in kelvin.
    pub fn     kelvin(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).Kelvin; }) }
    pub fn set_kelvin(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).Kelvin = v; }) }

    /// White balance tint.
    pub fn     tint(&self)       -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).Tint; }) }
    pub fn set_tint(&mut self, v: f32)   { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).Tint = v; }) }

    /// Exposure compensation. IPP2 applies it in converted form.
    pub fn     exposure_compensation(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).ExposureCompensation; }) }
    pub fn set_exposure_compensation(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).ExposureCompensation = v; }) }

    /// Red gain. Legacy pipeline only.
    pub fn     gain_red(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).GainRed; }) }
    pub fn set_gain_red(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).GainRed = v; }) }

    /// Green gain. Legacy pipeline only.
    pub fn     gain_green(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).GainGreen; }) }
    pub fn set_gain_green(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).GainGreen = v; }) }

    /// Blue gain. Legacy pipeline only.
    pub fn     gain_blue(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).GainBlue; }) }
    pub fn set_gain_blue(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).GainBlue = v; }) }

    /// Saturation. Legacy pipeline only.
    pub fn     saturation(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).Saturation; }) }
    pub fn set_saturation(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).Saturation = v; }) }

    /// Contrast. Legacy pipeline only.
    pub fn     contrast(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).Contrast; }) }
    pub fn set_contrast(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).Contrast = v; }) }

    /// DRX highlight extension. Legacy pipeline only, and without effect on Dragon or Helium clips.
    pub fn     drx(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).DRX; }) }
    pub fn set_drx(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).DRX = v; }) }

    /// Output gamma curve, an [`ImageGammaCurve`] value. Broadcast Color only accepts
    /// [`ImageGammaCurve::BT1886`] and [`ImageGammaCurve::HDR2084`].
    pub fn     gamma_curve(&self)     -> i32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).GammaCurve; }) }
    pub fn set_gamma_curve(&mut self, v: i32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).GammaCurve = (R3DSDK::ImageGammaCurve)v; }) }

    /// Output color space, an [`ImageColorSpace`] value. Broadcast Color only accepts
    /// [`ImageColorSpace::Rec709`] and [`ImageColorSpace::Rec2020`].
    pub fn     color_space(&self)     -> i32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).ColorSpace; }) }
    pub fn set_color_space(&mut self, v: i32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).ColorSpace = (R3DSDK::ImageColorSpace)v; }) }

    // --------------------------------------------------------------------------------
    // Custom PDLog (ignored in IPP2/BC)
    // --------------------------------------------------------------------------------

    /// Black point of the custom PDlog curve. Fill in all three PDlog values when choosing [`ImageGammaCurve::CustomPDlog`].
    pub fn     custom_pdlog_black_point(&self)     -> usize { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> usize as "size_t" { return (*self).CustomPDLogBlackPoint; }) }
    pub fn set_custom_pdlog_black_point(&mut self, v: usize) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "size_t"] { (*self).CustomPDLogBlackPoint = v; }) }

    /// White point of the custom PDlog curve, see [`custom_pdlog_black_point`](Self::custom_pdlog_black_point).
    pub fn     custom_pdlog_white_point(&self)     -> usize { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> usize as "size_t" { return (*self).CustomPDLogWhitePoint; }) }
    pub fn set_custom_pdlog_white_point(&mut self, v: usize) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "size_t"] { (*self).CustomPDLogWhitePoint = v; }) }

    /// Gamma of the custom PDlog curve, see [`custom_pdlog_black_point`](Self::custom_pdlog_black_point).
    pub fn     custom_pdlog_gamma(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).CustomPDLogGamma; }) }
    pub fn set_custom_pdlog_gamma(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).CustomPDLogGamma = v; }) }

    // --------------------------------------------------------------------------------
    // User curves
    // --------------------------------------------------------------------------------

    /// Luma curve through five control points from black to white, stored as
    /// `[x0, y0, x1, y1, ..., x4, y4]` with every coordinate in 0..=1. The x
    /// values must strictly increase; otherwise the SDK falls back to a
    /// straight line. Ignored by Broadcast Color.
    pub fn     user_curve(&self) -> &[f32] {
        let ptr = cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> *const f32 as "const float *" { return (*self).UserCurve; });
        unsafe { std::slice::from_raw_parts(ptr, 10) }
    }
    /// Mutable access to luma user curve.
    pub fn user_curve_mut(&mut self) -> &mut [f32] {
        let ptr = cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *"] -> *mut f32 as "float *" { return (*self).UserCurve; });
        unsafe { std::slice::from_raw_parts_mut(ptr, 10) }
    }

    /// Red channel curve, laid out like [`user_curve`](Self::user_curve). Legacy pipeline only.
    pub fn     user_curve_red(&self) -> &[f32] {
        let ptr = cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> *const f32 as "const float *" { return (*self).UserCurveRed; });
        unsafe { std::slice::from_raw_parts(ptr, 10) }
    }
    /// Mutable access to red user curve.
    pub fn user_curve_red_mut(&mut self) -> &mut [f32] {
        let ptr = cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *"] -> *mut f32 as "float *" { return (*self).UserCurveRed; });
        unsafe { std::slice::from_raw_parts_mut(ptr, 10) }
    }

    /// Green channel curve, laid out like [`user_curve`](Self::user_curve). Legacy pipeline only.
    pub fn     user_curve_green(&self) -> &[f32] {
        let ptr = cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> *const f32 as "const float *" { return (*self).UserCurveGreen; });
        unsafe { std::slice::from_raw_parts(ptr, 10) }
    }
    /// Mutable access to green user curve.
    pub fn user_curve_green_mut(&mut self) -> &mut [f32] {
        let ptr = cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *"] -> *mut f32 as "float *" { return (*self).UserCurveGreen; });
        unsafe { std::slice::from_raw_parts_mut(ptr, 10) }
    }

    /// Blue channel curve, laid out like [`user_curve`](Self::user_curve). Legacy pipeline only.
    pub fn     user_curve_blue(&self) -> &[f32] {
        let ptr = cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> *const f32 as "const float *" { return (*self).UserCurveBlue; });
        unsafe { std::slice::from_raw_parts(ptr, 10) }
    }
    /// Mutable access to blue user curve.
    pub fn user_curve_blue_mut(&mut self) -> &mut [f32] {
        let ptr = cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *"] -> *mut f32 as "float *" { return (*self).UserCurveBlue; });
        unsafe { std::slice::from_raw_parts_mut(ptr, 10) }
    }

    // --------------------------------------------------------------------------------
    // Premium decode-only (ignored in IPP2/BC where noted)
    // --------------------------------------------------------------------------------

    /// Detail level. Only used by full-resolution premium decodes with the legacy pipeline.
    pub fn     detail(&self)     -> ImageDetail { enum_from_sdk(cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).Detail; })) }
    pub fn set_detail(&mut self, v: ImageDetail) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).Detail = (R3DSDK::ImageDetail)v; }) }

    /// OLPF compensation. Only used by full- and half-resolution premium decodes with the legacy pipeline.
    pub fn     olpf_compensation(&self)     -> ImageOLPFCompensation { enum_from_sdk(cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).OLPFCompensation; })) }
    pub fn set_olpf_compensation(&mut self, v: ImageOLPFCompensation) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).OLPFCompensation = (R3DSDK::ImageOLPFCompensation)v; }) }

    /// Noise reduction level. Only used by full-resolution premium decodes with the legacy pipeline.
    pub fn     denoise(&self)     -> ImageDenoise { enum_from_sdk(cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).Denoise; })) }
    pub fn set_denoise(&mut self, v: ImageDenoise) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).Denoise = (R3DSDK::ImageDenoise)v; }) }

    /// Dragon Enhanced Blacks, for Dragon and newer sensors in the legacy pipeline. Only used by
    /// full- and half-resolution premium decodes. Stored in the same place as
    /// [`chroma_noise_reduction`](Self::chroma_noise_reduction), so setting one changes the other.
    pub fn     deb(&self)     -> bool { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> bool as "bool" { return (*self).DEB; }) }
    pub fn set_deb(&mut self, v: bool) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "bool"] { (*self).DEB = v; }) }

    /// Chroma noise reduction, for IPP2 and Broadcast Color. Only used by full- and half-resolution
    /// premium decodes. Shares its storage with [`deb`](Self::deb).
    pub fn     chroma_noise_reduction(&self)     -> bool { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> bool as "bool" { return (*self).ChromaNoiseReduction; }) }
    pub fn set_chroma_noise_reduction(&mut self, v: bool) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "bool"] { (*self).ChromaNoiseReduction = v; }) }

    /// Flashing pixel adjustment. Only used by full- and half-resolution premium decodes, and never for N-RAW clips.
    pub fn     flashing_pixel_adjustment(&self)     -> FlashingPixelAdjust { enum_from_sdk(cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).FlashingPixelAdjustment; })) }
    pub fn set_flashing_pixel_adjustment(&mut self, v: FlashingPixelAdjust) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).FlashingPixelAdjustment = (R3DSDK::FlashingPixelAdjust)v; }) }

    // --------------------------------------------------------------------------------
    // ColorVersion2-only (not IPP2/BC)
    // --------------------------------------------------------------------------------

    /// Shadow. Legacy pipeline only.
    pub fn     shadow(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).Shadow; }) }
    pub fn set_shadow(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).Shadow = v; }) }

    /// FLUT, a fine-grained ISO adjustment. Legacy pipeline only.
    pub fn     flut(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).FLUT; }) }
    pub fn set_flut(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).FLUT = v; }) }

    // --------------------------------------------------------------------------------
    // Lift/Gamma/Gain (ignored in IPP2/BC) — references into the C++ struct
    // --------------------------------------------------------------------------------

    /// Lift, gamma and gain for the red channel. Legacy pipeline only.
    pub fn     lgg_red(&self) -> &LiftGammaGain { let p: *const LiftGammaGain = cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> *const LiftGammaGain as "const R3DSDK::LiftGammaGain*" { return &(*self).LggRed; }); unsafe { &*p } }
    pub fn lgg_red_mut(&mut self) -> &mut LiftGammaGain { let p: *mut LiftGammaGain = cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *"] -> *mut LiftGammaGain as "R3DSDK::LiftGammaGain*" { return &(*self).LggRed; }); unsafe { &mut *p } }

    /// Lift, gamma and gain for the green channel, see [`lgg_red`](Self::lgg_red).
    pub fn     lgg_green(&self) -> &LiftGammaGain { let p: *const LiftGammaGain = cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> *const LiftGammaGain as "const R3DSDK::LiftGammaGain*" { return &(*self).LggGreen; }); unsafe { &*p } }
    pub fn lgg_green_mut(&mut self) -> &mut LiftGammaGain { let p: *mut LiftGammaGain = cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *"] -> *mut LiftGammaGain as "R3DSDK::LiftGammaGain*" { return &(*self).LggGreen; }); unsafe { &mut *p } }

    /// Lift, gamma and gain for the blue channel, see [`lgg_red`](Self::lgg_red).
    pub fn     lgg_blue(&self) -> &LiftGammaGain { let p: *const LiftGammaGain = cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> *const LiftGammaGain as "const R3DSDK::LiftGammaGain*" { return &(*self).LggBlue; }); unsafe { &*p } }
    pub fn lgg_blue_mut(&mut self) -> &mut LiftGammaGain { let p: *mut LiftGammaGain = cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *"] -> *mut LiftGammaGain as "R3DSDK::LiftGammaGain*" { return &(*self).LggBlue; }); unsafe { &mut *p } }

    // --------------------------------------------------------------------------------
    // IPP2 (ColorVersion3) — Grading / Pipeline / Output Transform
    // --------------------------------------------------------------------------------

    /// Image Pipeline Mode (IPP2).
    pub fn     image_pipeline_mode(&self)     -> ImagePipeline { enum_from_sdk(cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).ImagePipelineMode; })) }
    pub fn set_image_pipeline_mode(&mut self, v: ImagePipeline) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).ImagePipelineMode = (R3DSDK::ImagePipeline)v; }) }

    /// IPP2 exposure adjustment, also honored by Broadcast Color.
    pub fn     exposure_adjust(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).ExposureAdjust; }) }
    pub fn set_exposure_adjust(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).ExposureAdjust = v; }) }

    /// CDL Saturation.
    pub fn     cdl_saturation(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).CdlSaturation; }) }
    pub fn set_cdl_saturation(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).CdlSaturation = v; }) }

    /// CDL SOP (Red)
    pub fn     cdl_red(&self) -> &SlopeOffsetPower { let p: *const SlopeOffsetPower = cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> *const SlopeOffsetPower as "const R3DSDK::SlopeOffsetPower*" { return &(*self).CdlRed; }); unsafe { &*p } }
    pub fn cdl_red_mut(&mut self) -> &mut SlopeOffsetPower { let p: *mut SlopeOffsetPower = cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *"] -> *mut SlopeOffsetPower as "R3DSDK::SlopeOffsetPower*" { return &(*self).CdlRed; }); unsafe { &mut *p } }

    /// CDL SOP (Green)
    pub fn     cdl_green(&self) -> &SlopeOffsetPower { let p: *const SlopeOffsetPower = cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> *const SlopeOffsetPower as "const R3DSDK::SlopeOffsetPower*" { return &(*self).CdlGreen; }); unsafe { &*p } }
    pub fn cdl_green_mut(&mut self) -> &mut SlopeOffsetPower { let p: *mut SlopeOffsetPower = cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *"] -> *mut SlopeOffsetPower as "R3DSDK::SlopeOffsetPower*" { return &(*self).CdlGreen; }); unsafe { &mut *p } }

    /// CDL SOP (Blue)
    pub fn     cdl_blue(&self) -> &SlopeOffsetPower { let p: *const SlopeOffsetPower = cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> *const SlopeOffsetPower as "const R3DSDK::SlopeOffsetPower*" { return &(*self).CdlBlue; }); unsafe { &*p } }
    pub fn cdl_blue_mut(&mut self) -> &mut SlopeOffsetPower { let p: *mut SlopeOffsetPower = cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *"] -> *mut SlopeOffsetPower as "R3DSDK::SlopeOffsetPower*" { return &(*self).CdlBlue; }); unsafe { &mut *p } }

    /// CDL Enabled toggle.
    pub fn     cdl_enabled(&self)     -> bool { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> bool as "bool" { return (*self).CdlEnabled; }) }
    pub fn set_cdl_enabled(&mut self, v: bool) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "bool"] { (*self).CdlEnabled = v; }) }

    /// Handle of the 3D LUT applied by IPP2, or null when there is none. A handle is only
    /// meaningful inside the process that loaded the LUT and cannot be saved for later.
    pub fn     lut3d(&self)     -> *mut core::ffi::c_void { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> *mut core::ffi::c_void as "void *" { return (*self).Lut3D; }) }

    /// Whether to apply the 3D LUT. Off by default, and has no effect without a LUT.
    pub fn     lut3d_enabled(&self)     -> bool { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> bool as "bool" { return (*self).Lut3DEnabled; }) }
    pub fn set_lut3d_enabled(&mut self, v: bool) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "bool"] { (*self).Lut3DEnabled = v; }) }

    /// Output Tone Map (IPP2).
    pub fn     output_tone_map(&self)     -> ToneMap { enum_from_sdk(cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).OutputToneMap; })) }
    pub fn set_output_tone_map(&mut self, v: ToneMap) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).OutputToneMap = (R3DSDK::ToneMap)v; }) }

    /// Highlight roll-off. Does nothing while a log gamma curve is selected.
    pub fn     highlight_roll_off(&self)     -> RollOff { enum_from_sdk(cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).HighlightRollOff; })) }
    pub fn set_highlight_roll_off(&mut self, v: RollOff) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).HighlightRollOff = (R3DSDK::RollOff)v; }) }

    /// Peak brightness of HDR output, in nits. Only used with [`ImageGammaCurve::HDR2084`] and a
    /// highlight roll-off other than [`RollOff::None`].
    pub fn     hdr_peak_nits(&self)     -> u32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> u32 as "unsigned int" { return (*self).HdrPeakNits; }) }
    pub fn set_hdr_peak_nits(&mut self, v: u32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "unsigned int"] { (*self).HdrPeakNits = v; }) }

    /// Maskline adjustment, also honored by Broadcast Color. Not used for N-RAW clips.
    pub fn     maskline_adjust(&self)     -> bool { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> bool as "bool" { return (*self).MasklineAdjust; }) }
    pub fn set_maskline_adjust(&mut self, v: bool) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "bool"] { (*self).MasklineAdjust = v; }) }

    /// IPP2 lens distortion correction, applied by full- and half-resolution premium decodes. Only
    /// worth offering when the clip's `lens_distortion_correction` metadata item
    /// (`RMD_LENS_DISTORTION_CORRECTION`) is non-zero.
    pub fn     lens_distortion_correction(&self)     -> LensCorrection { enum_from_sdk(cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).LensDistortionCorrection; })) }
    pub fn set_lens_distortion_correction(&mut self, v: LensCorrection) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).LensDistortionCorrection = (R3DSDK::LensCorrection)v; }) }

    /// Clamp every setting into its valid range.
    pub fn check_bounds(&mut self) {
        cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *"] {
            self->CheckBounds();
        })
    }
}

cpp_class! {
    /// How to merge the two tracks of an HDRx clip into one image. Pass it
    /// to a decode to get the blended frame rather than a single track.
    #[derive(Clone)]
    pub unsafe struct HdrProcessingSettings as "R3DSDK::HdrProcessingSettings"
}
// SAFETY: `Send + Sync`, same rationale as `ImageProcessingSettings` — all
// mutators `&mut self`, all `&self` getters read-only. `Send` explicit; `Sync`
// auto-derives from the inline storage.
unsafe impl Send for HdrProcessingSettings {}

impl HdrProcessingSettings {
    /// Algorithm that merges the two HDRx tracks.
    pub fn hdr_blend_algorithm(&self) -> HdrBlendAlgorithm {
        enum_from_sdk(cpp!(unsafe [self as "const R3DSDK::HdrProcessingSettings *"] -> i32 as "int" {
            return (int)(*self).BlendAlgorithm;
        }))
    }
    /// Choose the algorithm that merges the two HDRx tracks.
    pub fn set_hdr_blend_algorithm(&mut self, v: HdrBlendAlgorithm) {
        let v: i32 = v as i32;
        cpp!(unsafe [self as "R3DSDK::HdrProcessingSettings *", v as "int"] {
            (*self).BlendAlgorithm = (R3DSDK::HdrBlendAlgorithm)v;
        })
    }
    /// Blend bias, between -1 and +1. What it controls depends on the
    /// algorithm:
    ///
    /// - [`HdrBlendAlgorithm::HDRxSimpleBlend`]: the mix of the two tracks.
    ///   +1 shows only the normal exposure, -1 only the highlight track, and
    ///   0 weighs them equally.
    /// - [`HdrBlendAlgorithm::HDRxMagicMotion`]: how strongly highlights roll
    ///   off in the result, from not at all at +1 to fully at -1.
    pub fn bias(&self) -> f32 {
        cpp!(unsafe [self as "const R3DSDK::HdrProcessingSettings *"] -> f32 as "float" {
            return (*self).Bias;
        })
    }
    /// Set the blend bias, see [`bias`](Self::bias). Values outside -1..=+1
    /// get clamped.
    pub fn set_bias(&mut self, v: f32) {
        cpp!(unsafe [self as "R3DSDK::HdrProcessingSettings *", v as "float"] {
            (*self).Bias = v;
        })
    }

    /// Clamp every setting into its valid range.
    pub fn check_bounds(&mut self) {
        cpp!(unsafe [self as "R3DSDK::HdrProcessingSettings *"] {
            self->CheckBounds();
        })
    }
}

