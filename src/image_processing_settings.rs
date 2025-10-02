// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use crate::enums::*;
use cpp::*;

cpp_class! {
	/// Settings to process the image with. If these settings are not
	/// supplied the decoder will use the clip's default settings.
    #[derive(Clone)]
	pub unsafe struct ImageProcessingSettings as "R3DSDK::ImageProcessingSettings"
}

impl ImageProcessingSettings {
	// --------------------------------------------------------------------------------
	// Core / global
	// --------------------------------------------------------------------------------

	/// Brightness — Converted in ColorVersion3 mode. Ignored with ColorVersionBC.
	pub fn     brightness(&self)     -> f32  { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).Brightness; }) }
	pub fn set_brightness(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).Brightness = v; }) }

	/// ISO — will clamp to the nearest valid setting. See ImageProcessingLimits::ISOList.
	pub fn     iso(&self)      -> usize { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> usize as "size_t" { return (*self).ISO; }) }
	pub fn set_iso(&mut self, v: usize) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "size_t"] { (*self).ISO = v; }) }

	/// Color version — defaults to latest (V3). Settings out of bounds will be clipped.
	/// If a lower version is set for a newer clip it will be forced up.
	pub fn     version(&self)     -> ColorVersion { unsafe { std::mem::transmute(cpp!([self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).Version; })) } }
	pub fn set_version(&mut self, v: ColorVersion) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).Version = (R3DSDK::ColorVersion)v; }) }

	/// Kelvin color temperature for white balance.
	pub fn     kelvin(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).Kelvin; }) }
	pub fn set_kelvin(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).Kelvin = v; }) }

	/// Tint for white balance.
	pub fn     tint(&self)       -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).Tint; }) }
	pub fn set_tint(&mut self, v: f32)   { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).Tint = v; }) }

	/// Exposure Compensation — converted in ColorVersion3 mode.
	pub fn     exposure_compensation(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).ExposureCompensation; }) }
	pub fn set_exposure_compensation(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).ExposureCompensation = v; }) }

	/// Gain Red — ignored with ColorVersion3 or ColorVersionBC.
	pub fn     gain_red(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).GainRed; }) }
	pub fn set_gain_red(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).GainRed = v; }) }

	/// Gain Green — ignored with ColorVersion3 or ColorVersionBC.
	pub fn     gain_green(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).GainGreen; }) }
	pub fn set_gain_green(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).GainGreen = v; }) }

	/// Gain Blue — ignored with ColorVersion3 or ColorVersionBC.
	pub fn     gain_blue(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).GainBlue; }) }
	pub fn set_gain_blue(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).GainBlue = v; }) }

	/// Saturation — ignored with ColorVersion3 or ColorVersionBC.
	pub fn     saturation(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).Saturation; }) }
	pub fn set_saturation(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).Saturation = v; }) }

	/// Contrast — ignored with ColorVersion3 or ColorVersionBC.
	pub fn     contrast(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).Contrast; }) }
	pub fn set_contrast(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).Contrast = v; }) }

	/// DRX — highlight extension; ignored for Dragon/Helium or in ColorVersion3/BC.
	pub fn     drx(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).DRX; }) }
	pub fn set_drx(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).DRX = v; }) }

	/// Image Gamma Curve (limited in Broadcast Color).
	pub fn     gamma_curve(&self)     -> i32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).GammaCurve; }) }
	pub fn set_gamma_curve(&mut self, v: i32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).GammaCurve = (R3DSDK::ImageGammaCurve)v; }) }

	/// Image Color Space (limited in Broadcast Color).
	pub fn     color_space(&self)     -> i32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).ColorSpace; }) }
	pub fn set_color_space(&mut self, v: i32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).ColorSpace = (R3DSDK::ImageColorSpace)v; }) }

	// --------------------------------------------------------------------------------
	// Custom PDLog (ignored in IPP2/BC)
	// --------------------------------------------------------------------------------

	/// Custom PDLog Black Point — set when GammaCurve == ImageGammaCustomPDlog.
	pub fn     custom_pdlog_black_point(&self)     -> usize { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> usize as "size_t" { return (*self).CustomPDLogBlackPoint; }) }
	pub fn set_custom_pdlog_black_point(&mut self, v: usize) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "size_t"] { (*self).CustomPDLogBlackPoint = v; }) }

	/// Custom PDLog White Point — set when GammaCurve == ImageGammaCustomPDlog.
	pub fn     custom_pdlog_white_point(&self)     -> usize { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> usize as "size_t" { return (*self).CustomPDLogWhitePoint; }) }
	pub fn set_custom_pdlog_white_point(&mut self, v: usize) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "size_t"] { (*self).CustomPDLogWhitePoint = v; }) }

	/// Custom PDLog Gamma — set when GammaCurve == ImageGammaCustomPDlog.
	pub fn     custom_pdlog_gamma(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).CustomPDLogGamma; }) }
	pub fn set_custom_pdlog_gamma(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).CustomPDLogGamma = v; }) }

	// --------------------------------------------------------------------------------
	// User curves
	// --------------------------------------------------------------------------------

	/// Luma user curve — 5-point spline (black..white) as 10 floats in [0,1].
	/// X’s must satisfy: blackX < lowX < midX < highX < whiteX.
	pub fn     user_curve(&self) -> &[f32] {
		let ptr = cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> *const f32 as "const float *" { return (*self).UserCurve; });
		unsafe { std::slice::from_raw_parts(ptr, 10) }
	}
	/// Mutable access to luma user curve.
	pub fn user_curve_mut(&mut self) -> &mut [f32] {
		let ptr = cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *"] -> *mut f32 as "float *" { return (*self).UserCurve; });
		unsafe { std::slice::from_raw_parts_mut(ptr, 10) }
	}

	/// Red user curve — ignored in Broadcast Color.
	pub fn     user_curve_red(&self) -> &[f32] {
		let ptr = cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> *const f32 as "const float *" { return (*self).UserCurveRed; });
		unsafe { std::slice::from_raw_parts(ptr, 10) }
	}
	/// Mutable access to red user curve.
	pub fn user_curve_red_mut(&mut self) -> &mut [f32] {
		let ptr = cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *"] -> *mut f32 as "float *" { return (*self).UserCurveRed; });
		unsafe { std::slice::from_raw_parts_mut(ptr, 10) }
	}

	/// Green user curve — ignored in Broadcast Color.
	pub fn     user_curve_green(&self) -> &[f32] {
		let ptr = cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> *const f32 as "const float *" { return (*self).UserCurveGreen; });
		unsafe { std::slice::from_raw_parts(ptr, 10) }
	}
	/// Mutable access to green user curve.
	pub fn user_curve_green_mut(&mut self) -> &mut [f32] {
		let ptr = cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *"] -> *mut f32 as "float *" { return (*self).UserCurveGreen; });
		unsafe { std::slice::from_raw_parts_mut(ptr, 10) }
	}

	/// Blue user curve — ignored in Broadcast Color.
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

	/// Detail — only used for full res premium decode; ignored in IPP2/BC.
	pub fn     detail(&self)     -> ImageDetail { unsafe { std::mem::transmute(cpp!([self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).Detail; })) } }
	pub fn set_detail(&mut self, v: ImageDetail) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).Detail = (R3DSDK::ImageDetail)v; }) }

	/// OLPF Compensation — premium full/half res; ignored in IPP2/BC.
	pub fn     olpf_compensation(&self)     -> ImageOLPFCompensation { unsafe { std::mem::transmute(cpp!([self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).OLPFCompensation; })) } }
	pub fn set_olpf_compensation(&mut self, v: ImageOLPFCompensation) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).OLPFCompensation = (R3DSDK::ImageOLPFCompensation)v; }) }

	/// Denoise — only used for full res premium decode; ignored in IPP2/BC.
	pub fn     denoise(&self)     -> ImageDenoise { unsafe { std::mem::transmute(cpp!([self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).Denoise; })) } }
	pub fn set_denoise(&mut self, v: ImageDenoise) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).Denoise = (R3DSDK::ImageDenoise)v; }) }

	/// Dragon Enhanced Blacks (legacy pipe, Dragon+ sensors) — union with ChromaNoiseReduction.
	pub fn     deb(&self)     -> bool { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> bool as "bool" { return (*self).DEB; }) }
	pub fn set_deb(&mut self, v: bool) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "bool"] { (*self).DEB = v; }) }

	/// Chroma Noise Reduction (IPP2 & Broadcast) — union with DEB.
	pub fn     chroma_noise_reduction(&self)     -> bool { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> bool as "bool" { return (*self).ChromaNoiseReduction; }) }
	pub fn set_chroma_noise_reduction(&mut self, v: bool) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "bool"] { (*self).ChromaNoiseReduction = v; }) }

	/// Flashing Pixel Adjustment — premium full/half res; ignored for N-RAW.
	pub fn     flashing_pixel_adjustment(&self)     -> FlashingPixelAdjust { unsafe { std::mem::transmute(cpp!([self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).FlashingPixelAdjustment; })) } }
	pub fn set_flashing_pixel_adjustment(&mut self, v: FlashingPixelAdjust) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).FlashingPixelAdjustment = (R3DSDK::FlashingPixelAdjust)v; }) }

	// --------------------------------------------------------------------------------
	// ColorVersion2-only (not IPP2/BC)
	// --------------------------------------------------------------------------------

	/// Shadow — ignored with ColorVersion3 & Broadcast Color.
	pub fn     shadow(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).Shadow; }) }
	pub fn set_shadow(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).Shadow = v; }) }

	/// FLUT (fine ISO control) — ignored with ColorVersion3 & Broadcast Color.
	pub fn     flut(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).FLUT; }) }
	pub fn set_flut(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).FLUT = v; }) }

	// --------------------------------------------------------------------------------
	// Lift/Gamma/Gain (ignored in IPP2/BC) — expose raw pointers for FFI helpers
	// --------------------------------------------------------------------------------

	/// LggRed (post lift/gamma/gain). Returns const ptr to underlying C++ struct.
	pub fn     lgg_red(&self) -> &LiftGammaGain { unsafe { &*cpp!( [self as "const R3DSDK::ImageProcessingSettings *"] -> *const LiftGammaGain as "const R3DSDK::LiftGammaGain*" { return &(*self).LggRed; }) } }
	pub fn lgg_red_mut(&mut self) -> &mut LiftGammaGain { unsafe { &mut *cpp!([self as "R3DSDK::ImageProcessingSettings *"] -> *mut LiftGammaGain as "R3DSDK::LiftGammaGain*" { return &(*self).LggRed; }) } }

	/// LggGreen pointer access.
	pub fn     lgg_green(&self) -> &LiftGammaGain { unsafe { &*cpp!( [self as "const R3DSDK::ImageProcessingSettings *"] -> *const LiftGammaGain as "const R3DSDK::LiftGammaGain*" { return &(*self).LggGreen; }) } }
    pub fn lgg_green_mut(&mut self) -> &mut LiftGammaGain { unsafe { &mut *cpp!([self as "R3DSDK::ImageProcessingSettings *"] -> *mut LiftGammaGain as "R3DSDK::LiftGammaGain*" { return &(*self).LggGreen; }) } }

	/// LggBlue pointer access.
	pub fn     lgg_blue(&self) -> &LiftGammaGain { unsafe { &*cpp!( [self as "const R3DSDK::ImageProcessingSettings *"] -> *const LiftGammaGain as "const R3DSDK::LiftGammaGain*" { return &(*self).LggBlue; }) } }
    pub fn lgg_blue_mut(&mut self) -> &mut LiftGammaGain { unsafe { &mut *cpp!([self as "R3DSDK::ImageProcessingSettings *"] -> *mut LiftGammaGain as "R3DSDK::LiftGammaGain*" { return &(*self).LggBlue; }) } }

	// --------------------------------------------------------------------------------
	// IPP2 (ColorVersion3) — Grading / Pipeline / Output Transform
	// --------------------------------------------------------------------------------

	/// Image Pipeline Mode (IPP2).
	pub fn     image_pipeline_mode(&self)     -> ImagePipeline { unsafe { std::mem::transmute(cpp!([self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).ImagePipelineMode; })) } }
	pub fn set_image_pipeline_mode(&mut self, v: ImagePipeline) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).ImagePipelineMode = (R3DSDK::ImagePipeline)v; }) }

	/// Exposure Adjust — also available with Broadcast Color.
	pub fn     exposure_adjust(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).ExposureAdjust; }) }
	pub fn set_exposure_adjust(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).ExposureAdjust = v; }) }

	/// CDL Saturation.
	pub fn     cdl_saturation(&self)     -> f32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> f32 as "float" { return (*self).CdlSaturation; }) }
	pub fn set_cdl_saturation(&mut self, v: f32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "float"] { (*self).CdlSaturation = v; }) }

	/// CDL SOP (Red)
	pub fn     cdl_red(&self) -> &SlopeOffsetPower { unsafe { &*cpp!( [self as "const R3DSDK::ImageProcessingSettings *"] -> *const SlopeOffsetPower as "const R3DSDK::SlopeOffsetPower*" { return &(*self).CdlRed; }) } }
    pub fn cdl_red_mut(&mut self) -> &mut SlopeOffsetPower { unsafe { &mut *cpp!([self as "R3DSDK::ImageProcessingSettings *"] -> *mut SlopeOffsetPower as "R3DSDK::SlopeOffsetPower*" { return &(*self).CdlRed; }) } }

	/// CDL SOP (Green)
	pub fn     cdl_green(&self) -> &SlopeOffsetPower { unsafe { &*cpp!( [self as "const R3DSDK::ImageProcessingSettings *"] -> *const SlopeOffsetPower as "const R3DSDK::SlopeOffsetPower*" { return &(*self).CdlGreen; }) } }
	pub fn cdl_green_mut(&mut self) -> &mut SlopeOffsetPower { unsafe { &mut *cpp!([self as "R3DSDK::ImageProcessingSettings *"] -> *mut SlopeOffsetPower as "R3DSDK::SlopeOffsetPower*" { return &(*self).CdlGreen; }) } }

	/// CDL SOP (Blue)
	pub fn     cdl_blue(&self) -> &SlopeOffsetPower { unsafe { &*cpp!( [self as "const R3DSDK::ImageProcessingSettings *"] -> *const SlopeOffsetPower as "const R3DSDK::SlopeOffsetPower*" { return &(*self).CdlBlue; }) } }
	pub fn cdl_blue_mut(&mut self) -> &mut SlopeOffsetPower { unsafe { &mut *cpp!([self as "R3DSDK::ImageProcessingSettings *"] -> *mut SlopeOffsetPower as "R3DSDK::SlopeOffsetPower*" { return &(*self).CdlBlue; }) } }

	/// CDL Enabled toggle.
	pub fn     cdl_enabled(&self)     -> bool { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> bool as "bool" { return (*self).CdlEnabled; }) }
	pub fn set_cdl_enabled(&mut self, v: bool) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "bool"] { (*self).CdlEnabled = v; }) }

	/// 3D LUT handle — NULL if disabled/not set. Value unique per process.
	pub fn     lut3d(&self)     -> *mut core::ffi::c_void { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> *mut core::ffi::c_void as "void *" { return (*self).Lut3D; }) }

	/// 3D LUT enabled — defaults to false; only used if a LUT is set.
	pub fn     lut3d_enabled(&self)     -> bool { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> bool as "bool" { return (*self).Lut3DEnabled; }) }
	pub fn set_lut3d_enabled(&mut self, v: bool) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "bool"] { (*self).Lut3DEnabled = v; }) }

	/// Output Tone Map (IPP2).
	pub fn     output_tone_map(&self)     -> ToneMap { unsafe { std::mem::transmute(cpp!([self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).OutputToneMap; })) } }
	pub fn set_output_tone_map(&mut self, v: ToneMap) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).OutputToneMap = (R3DSDK::ToneMap)v; }) }

	/// Highlight Roll-off — no effect with LOG curve applied.
	pub fn     highlight_roll_off(&self)     -> RollOff { unsafe { std::mem::transmute(cpp!([self as "const R3DSDK::ImageProcessingSettings *"] -> i32 as "int" { return (int)(*self).HighlightRollOff; })) } }
	pub fn set_highlight_roll_off(&mut self, v: RollOff) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "int"] { (*self).HighlightRollOff = (R3DSDK::RollOff)v; }) }

	/// HDR Peak Nits — HDR2084 only and HighlightRollOff != None.
	pub fn     hdr_peak_nits(&self)     -> u32 { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> u32 as "unsigned int" { return (*self).HdrPeakNits; }) }
	pub fn set_hdr_peak_nits(&mut self, v: u32) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "unsigned int"] { (*self).HdrPeakNits = v; }) }

	/// Maskline Adjust — also available with Broadcast Color; ignored for N-RAW.
	pub fn     maskline_adjust(&self)     -> bool { cpp!(unsafe [self as "const R3DSDK::ImageProcessingSettings *"] -> bool as "bool" { return (*self).MasklineAdjust; }) }
	pub fn set_maskline_adjust(&mut self, v: bool) { cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *", v as "bool"] { (*self).MasklineAdjust = v; }) }

	/// Check to see if all parameters are within range and clip any that aren't.
	pub fn check_bounds(&mut self) {
		cpp!(unsafe [self as "R3DSDK::ImageProcessingSettings *"] {
			self->CheckBounds();
		})
	}
}

cpp_class! {
    /// HDR blending settings. This structure needs to be constructed
    /// to submit a HDR blending job to the decoder. Every field must
    /// be set.
    #[derive(Clone)]
	pub unsafe struct HdrProcessingSettings as "R3DSDK::HdrProcessingSettings"
}

impl HdrProcessingSettings {
    /// HDRx blending algorithm to use
    pub fn hdr_blend_algorithm(&self) -> HdrBlendAlgorithm {
        unsafe { std::mem::transmute(cpp!([self as "const R3DSDK::HdrProcessingSettings *"] -> i32 as "int" {
            return (int)(*self).BlendAlgorithm;
        })) }
    }
    /// Set HDRx blending algorithm to use
    pub fn set_hdr_blend_algorithm(&mut self, v: HdrBlendAlgorithm) {
        let v: i32 = v as i32;
        cpp!(unsafe [self as "R3DSDK::HdrProcessingSettings *", v as "int"] {
            (*self).BlendAlgorithm = (R3DSDK::HdrBlendAlgorithm)v;
        })
    }
    /// Amount of blending for the modes, must be in range of -1 -- +1
    /// (will be clipped if it's not!)
    ///
    /// Bias is mode dependent. For Simple Blend it controls the blend
    /// between the two images, +1 being full normal exposure, -1 being
    /// full highlight track exposure and 0 being an equal blend of the two.
    /// For Magic Motion, it adjusts the amount of highlight rolloff in the
    /// blended HDR image. +1 is no highlight rolloff and -1 is full rolloff
    pub fn bias(&self) -> f32 {
        cpp!(unsafe [self as "const R3DSDK::HdrProcessingSettings *"] -> f32 as "float" {
            return (*self).Bias;
        })
    }
    /// Set amount of blending for the modes, must be in range of -1 -- +1
    /// (will be clipped if it's not!)
    pub fn set_bias(&mut self, v: f32) {
        cpp!(unsafe [self as "R3DSDK::HdrProcessingSettings *", v as "float"] {
            (*self).Bias = v;
        })
    }

	/// Check to see if all parameters are within the proper range and clip any that aren't
    pub fn check_bounds(&mut self) {
        cpp!(unsafe [self as "R3DSDK::HdrProcessingSettings *"] {
            self->CheckBounds();
        })
    }
}
