// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Hardware-free guard test for the PCM audio decode bindings
//! (`Clip::decode_audio_block` / `Clip::decode_audio`).
//!
//! Both APIs require a 512-byte aligned output buffer (SDK contract). The
//! Rust guard rejects an under-aligned buffer *before* any FFI decode call,
//! so this exercises the reject path without a `.R3D` clip and without any
//! GPU component — it only loads the base RED library from the SDK's
//! redistributable folder (see `common::redist_dir`), since the SDK requires
//! `InitializeSdk` before any class is constructed.

mod common;

use r3d_rs::{AlignedBuffer, Clip, InitializeFlags, RedError, Sdk};

#[test]
fn audio_decode_rejects_underaligned_buffer() {
    // Base library only — no GPU components needed to construct a Clip.
    let sdk = Sdk::initialize(&common::redist_dir(), InitializeFlags::None)
        .expect("R3D SDK init from the redistributable dir");

    let mut clip = Clip::new(&sdk);
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
