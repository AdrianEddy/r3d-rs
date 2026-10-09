// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Hardware-free guard test for the PCM audio decode bindings
//! (`Clip::decode_audio_block` / `Clip::decode_audio`).
//!
//! Both APIs require a 512-byte aligned output buffer (SDK contract). The
//! Rust guard rejects an under-aligned buffer *before* any FFI decode call,
//! so this exercises the reject path without a `.R3D` clip and without any
//! GPU component — it only loads the base RED library from the redistributable
//! dir bundled in `sdk/` so that constructing a `Clip` does not fault (the SDK
//! requires `InitializeSdk` before any class is constructed).

use r3d_rs::{AlignedBuffer, Clip, InitializeFlags, RedError, Sdk};

// Bundled RED redistributable dynamic-library folder for the host OS.
#[cfg(target_os = "windows")]
const REDIST_SUBDIR: &str = "sdk/Redistributable/win";
#[cfg(target_os = "macos")]
const REDIST_SUBDIR: &str = "sdk/Redistributable/mac";
#[cfg(target_os = "linux")]
const REDIST_SUBDIR: &str = "sdk/Redistributable/linux";

#[test]
fn audio_decode_rejects_underaligned_buffer() {
    let dylib_dir = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), REDIST_SUBDIR);
    // Base library only — no GPU components needed to construct a Clip.
    let _sdk = Sdk::initialize(&dylib_dir, InitializeFlags::None)
        .expect("R3D SDK init from bundled redistributable dir");

    let mut clip = Clip::new();
    // 256-byte alignment is below the required 512 → rejected before any FFI.
    let mut buf = AlignedBuffer::new(1024, 256).unwrap();
    assert!(matches!(
        clip.decode_audio_block(0, &mut buf),
        Err(RedError::BufferNotAligned)
    ));
    assert!(matches!(
        clip.decode_audio(0, 128, &mut buf),
        Err(RedError::BufferNotAligned)
    ));
}
