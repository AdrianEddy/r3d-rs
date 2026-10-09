// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Helpers shared by the integration tests.

/// The folder holding the SDK's redistributable dynamic libraries:
/// `$R3D_TEST_REDIST_DIR`, or else `$R3DSDK_DIR/Redistributable/<os>`, with
/// `R3DSDK_DIR` defaulting to `sdk` in the crate root, as in `build.rs`.
pub fn redist_dir() -> String {
    if let Ok(dir) = std::env::var("R3D_TEST_REDIST_DIR") {
        return dir;
    }
    let sdk = std::env::var("R3DSDK_DIR")
        .unwrap_or_else(|_| format!("{}/sdk", env!("CARGO_MANIFEST_DIR")));
    let os = if cfg!(windows) {
        "win"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else {
        "linux"
    };
    format!("{sdk}/Redistributable/{os}")
}
