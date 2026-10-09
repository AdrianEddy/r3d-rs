// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! `ImageProcessingSettings` is the SDK's own C++ struct, so a field the bindings
//! read or write at the wrong place corrupts its neighbours. Round-trip the SDK 9.2
//! `LensDistortionCorrection` field — the last one in the struct — and check its
//! neighbours keep their values.

use r3d_rs::{ImageProcessingSettings, InitializeFlags, LensCorrection, Sdk};

#[cfg(target_os = "windows")]
const REDIST_SUBDIR: &str = "sdk/Redistributable/win";
#[cfg(target_os = "macos")]
const REDIST_SUBDIR: &str = "sdk/Redistributable/mac";
#[cfg(target_os = "linux")]
const REDIST_SUBDIR: &str = "sdk/Redistributable/linux";

#[test]
fn lens_distortion_correction_round_trips() {
    // The SDK must be initialised before any of its classes is constructed.
    let _sdk = Sdk::initialize(&format!("{}/{}", env!("CARGO_MANIFEST_DIR"), REDIST_SUBDIR), InitializeFlags::None)
        .expect("R3D SDK init from bundled redistributable dir");

    let mut settings = ImageProcessingSettings::default();
    assert_eq!(settings.lens_distortion_correction(), LensCorrection::Auto, "defaults to following the clip's metadata");
    let (maskline, peak_nits) = (settings.maskline_adjust(), settings.hdr_peak_nits());

    for v in [LensCorrection::On, LensCorrection::Off, LensCorrection::Auto] {
        settings.set_lens_distortion_correction(v);
        assert_eq!(settings.lens_distortion_correction(), v);
        assert_eq!((settings.maskline_adjust(), settings.hdr_peak_nits()), (maskline, peak_nits), "neighbouring fields untouched");
    }
}
