// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Compile-only check that the per-type Send / Sync classification (the
//! `unsafe impl` blocks sit next to each type, each with a `// SAFETY:` note)
//! stays as intended.
//! Any future refactor that accidentally adds a blanket `unsafe impl<T> Sync`
//! would silently re-enable Sync on the Send-only types — the
//! `assert_not_impl_all!` lines below catch that immediately.

use r3d_rs::*;
use static_assertions::{assert_impl_all, assert_not_impl_all};

// ─────────────────────────────────────────────────────────────────────
// Send + Sync — SDK objects with documented thread-safe `&self` surface.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn sdk_is_send_sync() {
    // Shared by every object using the SDK, which may live on any thread.
    assert_impl_all!(Sdk: Send, Sync);
}

#[test]
fn decoders_are_send_sync() {
    assert_impl_all!(AsyncDecoder: Send, Sync);
    assert_impl_all!(GpuDecoder:   Send, Sync);
    assert_impl_all!(R3dDecoder:   Send, Sync);
    assert_impl_all!(Clip:         Send, Sync);
}

#[test]
fn lane_debayers_are_send_sync() {
    #[cfg(not(target_os = "macos"))]
    assert_impl_all!(RedCuda:   Send, Sync);
    #[cfg(feature = "metal-debayer")]
    assert_impl_all!(RedMetal:  Send, Sync);
    assert_impl_all!(RedOpenCl: Send, Sync);
}

#[test]
fn device_info_is_send_sync() {
    assert_impl_all!(OpenCLDeviceInfo: Send, Sync);
    assert_impl_all!(CudaDeviceInfo:   Send, Sync);
}

#[test]
fn callback_future_is_send_sync() {
    // T: Send is sufficient; `AsyncDecompressJob` is Send-only, but the
    // CallbackFuture audit (see the `future.rs` doc comment) lifts to Send + Sync.
    assert_impl_all!(CallbackFuture<AsyncDecompressJob>: Send, Sync);
    assert_impl_all!(CallbackFuture<R3dDecodeJob>:       Send, Sync);
}

#[test]
fn gpu_pipeline_is_send_sync() {
    assert_impl_all!(gpu::GpuDecodePipeline: Send, Sync);
    assert_impl_all!(gpu::GpuLane:           Send, Sync);
    // Same audit as `CallbackFuture<AsyncDecompressJob>`, which it wraps.
    assert_impl_all!(gpu::PendingDecompress: Send, Sync);
}

#[test]
fn gpu_stage_handoffs_are_send_only() {
    // A pipelined caller moves these between threads, but each wraps a
    // Send-only job, so they must not become Sync.
    assert_impl_all!(gpu::DecompressedFrame: Send);
    assert_not_impl_all!(gpu::DecompressedFrame: Sync);
    #[cfg(not(target_os = "macos"))]
    assert_impl_all!(gpu::PendingDebayer: Send);
    #[cfg(not(target_os = "macos"))]
    assert_not_impl_all!(gpu::PendingDebayer: Sync);
}

// ─────────────────────────────────────────────────────────────────────
// Send-only — single-owner cursor-state mutators.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn jobs_are_send_only() {
    assert_impl_all!(AsyncDecompressJob: Send);
    assert_impl_all!(R3dDecodeJob:       Send);
    #[cfg(not(target_os = "macos"))]
    assert_impl_all!(DebayerCudaJob:     Send);
    #[cfg(feature = "metal-debayer")]
    assert_impl_all!(DebayerMetalJob:    Send);
    assert_impl_all!(DebayerOpenCLJob:   Send);
}

#[test]
fn raw_ptr_settings_and_buffer_are_send_only() {
    assert_impl_all!(R3dDecoderOptions: Send);
    assert_impl_all!(AlignedBuffer:     Send);
}

#[test]
fn cpp_class_value_types_are_send_sync() {
    // The cpp_class!-generated wrappers hold opaque inline byte
    // storage. All mutators take `&mut self` (exclusive); all
    // `&self` getters are read-only. Rust's borrow rules forbid
    // simultaneous shared + exclusive access, so Sync is sound — see
    // the `// SAFETY:` note next to each type's `unsafe impl Send`.
    assert_impl_all!(ImageProcessingSettings: Send, Sync);
    assert_impl_all!(HdrProcessingSettings:   Send, Sync);
    assert_impl_all!(Metadata:                Send, Sync);
    assert_impl_all!(FrameMetadata:           Send, Sync);
}

#[test]
fn raw_ptr_types_are_not_sync() {
    // The strong assertion: a blanket `unsafe impl<T> Sync` slipping in
    // would silently re-enable Sync on these. The negative checks catch
    // that immediately. Note: cpp_class!-generated types are
    // auto-Sync via their `[u8; N]` storage (see test above); the
    // assertions here cover only the raw-pointer / single-owner SDK
    // wrappers where the Send-only audit conclusion actually applies.
    assert_not_impl_all!(AsyncDecompressJob:       Sync);
    assert_not_impl_all!(R3dDecodeJob:             Sync);
    #[cfg(not(target_os = "macos"))]
    assert_not_impl_all!(DebayerCudaJob:           Sync);
    #[cfg(feature = "metal-debayer")]
    assert_not_impl_all!(DebayerMetalJob:          Sync);
    assert_not_impl_all!(DebayerOpenCLJob:         Sync);
    assert_not_impl_all!(R3dDecoderOptions:        Sync);
    assert_not_impl_all!(AlignedBuffer:            Sync);
}
