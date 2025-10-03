// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use crate::enums::*;
use cpp::*;

/// This structure gives access to the default, minimum and maximum setting
/// for each of the image processing parameters. Use this data in your User
/// Interface so new options in future versions are automatically added.
pub struct ImageProcessingLimits;

impl ImageProcessingLimits {
	#[inline] pub fn kelvin_min() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::KelvinMin; }) }
	#[inline] pub fn kelvin_max() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::KelvinMax; }) }
	#[inline] pub fn kelvin_default() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::KelvinDefault; }) }

	#[inline] pub fn tint_min() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::TintMin; }) }
	#[inline] pub fn tint_max() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::TintMax; }) }
	#[inline] pub fn tint_default() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::TintDefault; }) }

	#[inline] pub fn exposure_min() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::ExposureMin; }) }
	#[inline] pub fn exposure_max() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::ExposureMax; }) }
	#[inline] pub fn exposure_default() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::ExposureDefault; }) }

	#[inline] pub fn gains_min() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::GainsMin; }) }
	#[inline] pub fn gains_max() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::GainsMax; }) }
	#[inline] pub fn gains_default() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::GainsDefault; }) }

	#[inline] pub fn saturation_min() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::SaturationMin; }) }
	#[inline] pub fn saturation_max() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::SaturationMax; }) }
	#[inline] pub fn saturation_default() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::SaturationDefault; }) }

	#[inline] pub fn contrast_min() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::ContrastMin; }) }
	#[inline] pub fn contrast_max() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::ContrastMax; }) }
	#[inline] pub fn contrast_default() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::ContrastDefault; }) }

	#[inline] pub fn brightness_min() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::BrightnessMin; }) }
	#[inline] pub fn brightness_max() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::BrightnessMax; }) }
	#[inline] pub fn brightness_default() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::BrightnessDefault; }) }

	#[inline] pub fn drx_min() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::DRXMin; }) }
	#[inline] pub fn drx_max() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::DRXMax; }) }
	#[inline] pub fn drx_default() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::DRXDefault; }) }

	#[inline] pub fn shadow_min() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::ShadowMin; }) }
	#[inline] pub fn shadow_max() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::ShadowMax; }) }
	#[inline] pub fn shadow_default() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::ShadowDefault; }) }

	#[inline] pub fn flut_min() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::FLUTMin; }) }
	#[inline] pub fn flut_max() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::FLUTMax; }) }
	#[inline] pub fn flut_default() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::FLUTDefault; }) }

	#[inline] pub fn lgg_lift_min() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::LggLiftMin; }) }
	#[inline] pub fn lgg_lift_max() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::LggLiftMax; }) }
	#[inline] pub fn lgg_lift_default() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::LggLiftDefault; }) }

	#[inline] pub fn lgg_gamma_min() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::LggGammaMin; }) }
	#[inline] pub fn lgg_gamma_max() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::LggGammaMax; }) }
	#[inline] pub fn lgg_gamma_default() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::LggGammaDefault; }) }

	#[inline] pub fn lgg_gain_min() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::LggGainMin; }) }
	#[inline] pub fn lgg_gain_max() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::LggGainMax; }) }
	#[inline] pub fn lgg_gain_default() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::LggGainDefault; }) }

	#[inline] pub fn iso_default() -> usize { cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::ImageProcessingLimits::ISODefault; }) }

	#[inline] pub fn gamma_curve_default() -> ImageGammaCurve { unsafe { std::mem::transmute(cpp!([] -> i32 as "int" { return (int)R3DSDK::ImageProcessingLimits::GammaCurveDefault; })) } }
	#[inline] pub fn color_space_default() -> ImageColorSpace { unsafe { std::mem::transmute(cpp!([] -> i32 as "int" { return (int)R3DSDK::ImageProcessingLimits::ColorSpaceDefault; })) } }

	#[inline] pub fn custom_pdlog_black_point_min() -> usize { cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::ImageProcessingLimits::CustomPDLogBlackPointMin; }) }
	#[inline] pub fn custom_pdlog_black_point_max() -> usize { cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::ImageProcessingLimits::CustomPDLogBlackPointMax; }) }
	#[inline] pub fn custom_pdlog_black_point_default() -> usize { cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::ImageProcessingLimits::CustomPDLogBlackPointDefault; }) }

	#[inline] pub fn custom_pdlog_white_point_min() -> usize { cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::ImageProcessingLimits::CustomPDLogWhitePointMin; }) }
	#[inline] pub fn custom_pdlog_white_point_max() -> usize { cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::ImageProcessingLimits::CustomPDLogWhitePointMax; }) }
	#[inline] pub fn custom_pdlog_white_point_default() -> usize { cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::ImageProcessingLimits::CustomPDLogWhitePointDefault; }) }

	#[inline] pub fn custom_pdlog_gamma_min() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::CustomPDLogGammaMin; }) }
	#[inline] pub fn custom_pdlog_gamma_max() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::CustomPDLogGammaMax; }) }
	#[inline] pub fn custom_pdlog_gamma_default() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::CustomPDLogGammaDefault; }) }

	#[inline] pub fn image_pipeline_mode_default() -> ImagePipeline { unsafe { std::mem::transmute(cpp!([] -> i32 as "int" { return (int)R3DSDK::ImageProcessingLimits::ImagePipelineModeDefault; })) } }
	#[inline] pub fn highlight_roll_off_default() -> RollOff { unsafe { std::mem::transmute(cpp!([] -> i32 as "int" { return (int)R3DSDK::ImageProcessingLimits::HighlightRollOffDefault; })) } }
	#[inline] pub fn output_tone_map_default() -> ToneMap { unsafe { std::mem::transmute(cpp!([] -> i32 as "int" { return (int)R3DSDK::ImageProcessingLimits::OutputToneMapDefault; })) } }

	#[inline] pub fn exposure_adjust_min() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::ExposureAdjustMin; }) }
	#[inline] pub fn exposure_adjust_max() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::ExposureAdjustMax; }) }
	#[inline] pub fn exposure_adjust_default() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::ExposureAdjustDefault; }) }

	#[inline] pub fn hdr_peak_nits_min() -> u32 { cpp!(unsafe [] -> u32 as "unsigned int" { return R3DSDK::ImageProcessingLimits::HdrPeakNitsMin; }) }
	#[inline] pub fn hdr_peak_nits_max() -> u32 { cpp!(unsafe [] -> u32 as "unsigned int" { return R3DSDK::ImageProcessingLimits::HdrPeakNitsMax; }) }
	#[inline] pub fn hdr_peak_nits_default() -> u32 { cpp!(unsafe [] -> u32 as "unsigned int" { return R3DSDK::ImageProcessingLimits::HdrPeakNitsDefault; }) }

	#[inline] pub fn cdl_slope_min() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::CdlSlopeMin; }) }
	#[inline] pub fn cdl_slope_max() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::CdlSlopeMax; }) }
	#[inline] pub fn cdl_slope_default() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::CdlSlopeDefault; }) }

	#[inline] pub fn cdl_offset_min() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::CdlOffsetMin; }) }
	#[inline] pub fn cdl_offset_max() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::CdlOffsetMax; }) }
	#[inline] pub fn cdl_offset_default() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::CdlOffsetDefault; }) }

	#[inline] pub fn cdl_power_min() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::CdlPowerMin; }) }
	#[inline] pub fn cdl_power_max() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::CdlPowerMax; }) }
	#[inline] pub fn cdl_power_default() -> f32 { cpp!(unsafe [] -> f32 as "float" { return R3DSDK::ImageProcessingLimits::CdlPowerDefault; }) }

	/// Get the list of valid ISO values
	pub fn iso_list() -> Vec<usize> {
		let count = cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::ImageProcessingLimits::ISOCount; });
		let mut result = Vec::with_capacity(count);
		for i in 0..count {
			let val = cpp!(unsafe [i as "size_t"] -> usize as "size_t" {
				return R3DSDK::ImageProcessingLimits::ISOList[i];
			});
			result.push(val);
		}
		result
	}

	/// Get the list of gamma curve options
	pub fn gamma_curve_map() -> Vec<ImageGammaCurve> {
		let count = cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::ImageProcessingLimits::GammaCurveCount; });
		let mut result = Vec::with_capacity(count);
		for i in 0..count {
			let val = cpp!(unsafe [i as "size_t"] -> i32 as "int" {
				return (int)R3DSDK::ImageProcessingLimits::GammaCurveMap[i];
			});
			result.push(unsafe { std::mem::transmute(val) });
		}
		result
	}

	/// Get the labels for gamma curve options
	pub fn gamma_curve_labels() -> Vec<String> {
		let count = cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::ImageProcessingLimits::GammaCurveCount; });
		let mut result = Vec::with_capacity(count);
		for i in 0..count {
			let ptr = cpp!(unsafe [i as "size_t"] -> *const std::os::raw::c_char as "const char*" {
				return R3DSDK::ImageProcessingLimits::GammaCurveLabels[i];
			});
			let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
			result.push(cstr.to_string_lossy().into_owned());
		}
		result
	}

	/// Get the list of color space options
	pub fn color_space_map() -> Vec<ImageColorSpace> {
		let count = cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::ImageProcessingLimits::ColorSpaceCount; });
		let mut result = Vec::with_capacity(count);
		for i in 0..count {
			let val = cpp!(unsafe [i as "size_t"] -> i32 as "int" {
				return (int)R3DSDK::ImageProcessingLimits::ColorSpaceMap[i];
			});
			result.push(unsafe { std::mem::transmute(val) });
		}
		result
	}

	/// Get the labels for color space options
	pub fn color_space_labels() -> Vec<String> {
		let count = cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::ImageProcessingLimits::ColorSpaceCount; });
		let mut result = Vec::with_capacity(count);
		for i in 0..count {
			let ptr = cpp!(unsafe [i as "size_t"] -> *const std::os::raw::c_char as "const char*" {
				return R3DSDK::ImageProcessingLimits::ColorSpaceLabels[i];
			});
			let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
			result.push(cstr.to_string_lossy().into_owned());
		}
		result
	}

	/// Get the list of image pipeline mode options
	pub fn image_pipeline_mode_map() -> Vec<ImagePipeline> {
		let count = cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::ImageProcessingLimits::ImagePipelineModeCount; });
		let mut result = Vec::with_capacity(count);
		for i in 0..count {
			let val = cpp!(unsafe [i as "size_t"] -> i32 as "int" {
				return (int)R3DSDK::ImageProcessingLimits::ImagePipelineModeMap[i];
			});
			result.push(unsafe { std::mem::transmute(val) });
		}
		result
	}

	/// Get the labels for image pipeline mode options
	pub fn image_pipeline_mode_labels() -> Vec<String> {
		let count = cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::ImageProcessingLimits::ImagePipelineModeCount; });
		let mut result = Vec::with_capacity(count);
		for i in 0..count {
			let ptr = cpp!(unsafe [i as "size_t"] -> *const std::os::raw::c_char as "const char*" {
				return R3DSDK::ImageProcessingLimits::ImagePipelineModeLabels[i];
			});
			let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
			result.push(cstr.to_string_lossy().into_owned());
		}
		result
	}

	/// Get the list of highlight roll-off options
	pub fn highlight_roll_off_map() -> Vec<RollOff> {
		let count = cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::ImageProcessingLimits::HighlightRollOffCount; });
		let mut result = Vec::with_capacity(count);
		for i in 0..count {
			let val = cpp!(unsafe [i as "size_t"] -> i32 as "int" {
				return (int)R3DSDK::ImageProcessingLimits::HighlightRollOffMap[i];
			});
			result.push(unsafe { std::mem::transmute(val) });
		}
		result
	}

	/// Get the labels for highlight roll-off options
	pub fn highlight_roll_off_labels() -> Vec<String> {
		let count = cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::ImageProcessingLimits::HighlightRollOffCount; });
		let mut result = Vec::with_capacity(count);
		for i in 0..count {
			let ptr = cpp!(unsafe [i as "size_t"] -> *const std::os::raw::c_char as "const char*" {
				return R3DSDK::ImageProcessingLimits::HighlightRollOffLabels[i];
			});
			let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
			result.push(cstr.to_string_lossy().into_owned());
		}
		result
	}

	/// Get the list of output tone map options
	pub fn output_tone_map_map() -> Vec<ToneMap> {
		let count = cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::ImageProcessingLimits::OutputToneMapCount; });
		let mut result = Vec::with_capacity(count);
		for i in 0..count {
			let val = cpp!(unsafe [i as "size_t"] -> i32 as "int" {
				return (int)R3DSDK::ImageProcessingLimits::OutputToneMapMap[i];
			});
			result.push(unsafe { std::mem::transmute(val) });
		}
		result
	}

	/// Get the labels for output tone map options
	pub fn output_tone_map_labels() -> Vec<String> {
		let count = cpp!(unsafe [] -> usize as "size_t" { return R3DSDK::ImageProcessingLimits::OutputToneMapCount; });
		let mut result = Vec::with_capacity(count);
		for i in 0..count {
			let ptr = cpp!(unsafe [i as "size_t"] -> *const std::os::raw::c_char as "const char*" {
				return R3DSDK::ImageProcessingLimits::OutputToneMapLabels[i];
			});
			let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
			result.push(cstr.to_string_lossy().into_owned());
		}
		result
	}
}
