// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! `ImageProcessingSettings` is the SDK's own C++ struct, so a field the bindings
//! read or write at the wrong place corrupts its neighbours. Round-trip the SDK 9.2
//! `LensDistortionCorrection` field — the last one in the struct — and check its
//! neighbours keep their values.

use r3d_rs::{ImageProcessingSettings, LensCorrection};

/// The settings are plain data, usable without the SDK loaded.
#[test]
fn lens_distortion_correction_round_trips() {
    let mut settings = ImageProcessingSettings::default();
    assert_eq!(settings.lens_distortion_correction(), LensCorrection::Auto, "defaults to following the clip's metadata");
    let (maskline, peak_nits) = (settings.maskline_adjust(), settings.hdr_peak_nits());

    for v in [LensCorrection::On, LensCorrection::Off, LensCorrection::Auto] {
        settings.set_lens_distortion_correction(v);
        assert_eq!(settings.lens_distortion_correction(), v);
        assert_eq!((settings.maskline_adjust(), settings.hdr_peak_nits()), (maskline, peak_nits), "neighbouring fields untouched");
    }
}
