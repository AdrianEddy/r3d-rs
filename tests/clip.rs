// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! `Clip`, `AlignedBuffer` and enum conversion behaviour that needs no `.R3D`
//! clip and no GPU.

mod common;

use r3d_rs::{AlignedBuffer, Clip, InitializeFlags, LoadStatus, RedError, Sdk, VideoDecodeMode};

#[test]
fn clip_without_a_loaded_file() {
    // One test, so the SDK is not initialised from several threads at once.
    let sdk = Sdk::initialize(&common::redist_dir(), InitializeFlags::None)
        .expect("R3D SDK init from the redistributable dir");

    let clip = Clip::new(&sdk);
    assert_eq!(clip.status(), LoadStatus::NoClipOpen);
    assert_eq!((clip.width(), clip.height(), clip.video_frame_count()), (0, 0, 0));
    assert!(clip.metadata_iter().next().is_none());
    assert!(clip.imu_samples(..).is_empty());

    // Reversed and overflowing ranges are empty, not a panic.
    #[allow(clippy::reversed_empty_ranges)]
    let reversed = clip.imu_samples(10..5);
    assert!(reversed.is_empty());
    assert!(clip.imu_samples(..=u64::MAX).is_empty());

    // Strings with an interior NUL are rejected instead of panicking.
    assert!(matches!(Clip::from_path(&sdk, "bad\0path.R3D"), Err(RedError::InvalidPath)));
    assert!(clip.metadata("bad\0key").is_none());

    // A decompress job without a clip is refused before it reaches the SDK.
    let job = r3d_rs::AsyncDecompressJob::new(&sdk);
    assert_eq!(r3d_rs::AsyncDecoder::size_buffer_needed(&job), 0);
    assert!(matches!(r3d_rs::AsyncDecoder::new(&sdk, 1).decode_for_gpu_sdk(job), Err(RedError::NoClipOpen)));

    // So is one whose clip has nothing loaded, instead of getting a zero-sized buffer.
    let mut job = r3d_rs::AsyncDecompressJob::new(&sdk);
    job.set_clip(&std::sync::Arc::new(Clip::new(&sdk)));
    assert!(matches!(r3d_rs::AsyncDecoder::new(&sdk, 1).decode_for_gpu_sdk(job), Err(RedError::NoClipOpen)));

    // The SDK reports a missing file as one it failed to open.
    assert!(matches!(
        Clip::from_path(&sdk, &format!("{}/does-not-exist.R3D", env!("CARGO_MANIFEST_DIR"))),
        Err(RedError::ClipFailedToOpenFile)
    ));
}

#[test]
fn aligned_buffer_is_zeroed_and_aligned() {
    let buf = AlignedBuffer::new(4096, 512).unwrap();
    assert_eq!((buf.len(), buf.align()), (4096, 512));
    assert_eq!(buf.as_ptr() as usize % 512, 0);
    assert!(buf.as_slice::<u8>().iter().all(|&b| b == 0));
    assert_eq!(buf.as_slice::<u16>().len(), 2048);
    assert!(AlignedBuffer::new(0, 16).is_err());
    assert!(AlignedBuffer::new(16, 3).is_err(), "alignment must be a power of two");
}

#[test]
fn sdk_enums_convert_from_raw_values() {
    assert_eq!(VideoDecodeMode::try_from(VideoDecodeMode::HalfResGood as i32), Ok(VideoDecodeMode::HalfResGood));
    assert_eq!(VideoDecodeMode::try_from(0), Err(0));
    assert_eq!(LoadStatus::try_from(7), Ok(LoadStatus::NoClipOpen));
    assert_eq!(LoadStatus::try_from(-1), Err(-1));
}
