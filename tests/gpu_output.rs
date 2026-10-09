// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2026 Adrian <adrian.eddy at gmail>

//! Output-sizing helpers of the GPU pipeline: the `cl_image` pixel format
//! pairing and the decode-mode scaling. These are pure functions, so the tests
//! neither initialize the SDK nor touch a GPU or a clip; `gpu_pipeline_hw.rs`
//! covers the pipeline itself.

use r3d_rs::{
    VideoDecodeMode, VideoPixelType,
    gpu::{ClImageDataType, scaled_dims},
};

// ─────────────────────────────────────────────────────────────────────
// ClImageDataType — SDK pairing contract
// ─────────────────────────────────────────────────────────────────────

/// `ClImageDataType::expected_pixel_type` must mirror the pairing documented
/// in the SDK's `REDOpenCLGpuDecodeSample.cpp`. A silent edit that breaks
/// this would surface deep inside `GpuDecodePipeline::run_debayer` as
/// `Status_InvalidJobParameter_pixelType` — by then the caller has
/// already allocated the cl_image; pin it here.
#[test]
fn cl_image_data_type_pairs_with_sdk_pixel_type() {
    assert_eq!(
        ClImageDataType::UnormInt16.expected_pixel_type(),
        VideoPixelType::Rgb16bitInterleaved,
    );
    assert_eq!(
        ClImageDataType::HalfFloat.expected_pixel_type(),
        VideoPixelType::RgbHalfFloatInterleaved,
    );
}

/// Per the SDK's `REDOpenCLGpuDecodeSample.cpp`, the SDK
/// `output_device_mem_size` for both supported channel types is
/// `width × height × 4 × 2`. Pin the per-pixel byte count.
#[test]
fn cl_image_data_type_bytes_per_pixel_is_eight() {
    assert_eq!(ClImageDataType::UnormInt16.bytes_per_pixel(), 8);
    assert_eq!(ClImageDataType::HalfFloat.bytes_per_pixel(), 8);
}

// ─────────────────────────────────────────────────────────────────────
// scaled_dims — VideoDecodeMode divisor table
// ─────────────────────────────────────────────────────────────────────

/// `scaled_dims` divides the clip dims by 1 / 2 / 4 / 8 / 16 per
/// `VideoDecodeMode`. cl_image / MTLTexture outputs are sized with this
/// helper; a mismatch with the SDK's internal divisor would cause the SDK
/// to write past the allocation's end.
#[test]
fn scaled_dims_matches_video_decode_mode_divisors() {
    // FullResPremium: identity.
    assert_eq!(scaled_dims(4096, 2160, VideoDecodeMode::FullResPremium), (4096, 2160));
    // HalfResPremium / HalfResGood: ÷ 2.
    assert_eq!(scaled_dims(4096, 2160, VideoDecodeMode::HalfResPremium), (2048, 1080));
    assert_eq!(scaled_dims(4096, 2160, VideoDecodeMode::HalfResGood),    (2048, 1080));
    // QuarterResGood: ÷ 4.
    assert_eq!(scaled_dims(4096, 2160, VideoDecodeMode::QuarterResGood), (1024, 540));
    // EightResGood: ÷ 8.
    assert_eq!(scaled_dims(4096, 2160, VideoDecodeMode::EightResGood),   (512,  270));
    // SixteenthResGood: ÷ 16.
    assert_eq!(scaled_dims(4096, 2160, VideoDecodeMode::SixteenthResGood), (256, 135));
}

/// Edge: scaled_dims does NOT round up to a 16-pixel multiple. That
/// is the cl_image / MTLTexture contract — the destination object
/// carries its own row pitch and is sized to the raw scaled dims.
/// The flat-buffer output size (`GpuDecodePipeline::result_frame_size`)
/// is where the 16-pixel rounding lives.
#[test]
fn scaled_dims_preserves_non_16_multiple_dims() {
    // 4080 % 16 == 0 already; pick 4081 to force a non-multiple.
    assert_eq!(scaled_dims(4081, 2161, VideoDecodeMode::FullResPremium), (4081, 2161));
    // ÷ 2 of 4081 = 2040; ÷ 2 of 2161 = 1080. Not 16-multiples.
    assert_eq!(scaled_dims(4081, 2161, VideoDecodeMode::HalfResPremium), (2040, 1080));
}

/// scaled_dims integer-divides — when the scaling divisor exceeds a
/// dimension, the result is zero, NOT one. Callers must treat zero
/// as an error.
#[test]
fn scaled_dims_integer_division_zeroes_below_divisor() {
    assert_eq!(scaled_dims(10, 10, VideoDecodeMode::SixteenthResGood), (0, 0));
    assert_eq!(scaled_dims(15, 32, VideoDecodeMode::SixteenthResGood), (0, 2));
}
