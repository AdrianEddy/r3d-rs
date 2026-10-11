// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Reusing an `AsyncDecompressJob`: across the two decompressors, whose
//! buffer sizes differ, and after an abort request. Both decompressors run
//! without a GPU. The decodes need `R3D_TEST_CLIP`.
//!
//! One test, as the SDK is process-wide.

mod common;

use pollster::block_on;
use r3d_rs::*;
use std::sync::Arc;

/// Submits a job to one of the decoders.
type Submit<'a> = &'a dyn Fn(AsyncDecompressJob) -> RedResult<CallbackFuture<AsyncDecompressJob>>;
/// That decoder's `size_buffer_needed`.
type SizeNeeded = fn(&AsyncDecompressJob) -> usize;

/// Decode `job` on `decoder` and check that it got a buffer of the size that
/// decoder needs.
fn decode_on(decoder: Submit, needed: SizeNeeded, job: AsyncDecompressJob, what: &str) -> AsyncDecompressJob {
    let need = needed(&job);
    assert!(need > 0, "{what}: the decoder sizes the buffer");
    let job = block_on(decoder(job).unwrap_or_else(|e| panic!("{what}: submit failed: {e}")))
        .unwrap_or_else(|e| panic!("{what}: decode failed: {e}"));
    let len = job.internal_buffer().expect("an internal buffer").len();
    assert!(len >= need, "{what}: buffer of {len} bytes, the decoder needs {need}");
    job
}

#[test]
fn async_decompress_job_reuse() {
    let sdk = Sdk::initialize(&common::redist_dir(), InitializeFlags::None).expect("R3D SDK init");

    // A new job has a valid mode; the SDK itself leaves it at 0.
    let job = AsyncDecompressJob::new(&sdk);
    assert_eq!(job.mode(), VideoDecodeMode::FullResPremium);
    assert_eq!(job.video_frame_no(), 0);

    let Ok(path) = std::env::var("R3D_TEST_CLIP") else {
        eprintln!("R3D_TEST_CLIP is not set; skipping the decodes");
        return;
    };
    let clip = Arc::new(Clip::from_path(&sdk, &path).expect("open R3D_TEST_CLIP"));
    let async_decoder = AsyncDecoder::new(&sdk, 0);
    let gpu_decoder = GpuDecoder::new(&sdk);
    let on_async = |job| async_decoder.decode_for_gpu_sdk(job);
    let on_gpu = |job| gpu_decoder.decode_for_gpu_sdk(job);
    let gpu_supported = GpuDecoder::decode_supported_for_clip(&clip);

    // One job moved between the decoders. Their sizes differ in both
    // directions depending on the mode, so the buffer must follow the
    // decoder the job goes to.
    let mut job = AsyncDecompressJob::new(&sdk);
    job.set_clip(&clip);
    job.set_mode(VideoDecodeMode::SixteenthResGood);
    let mut job = decode_on(&on_async, AsyncDecoder::size_buffer_needed, job, "AsyncDecoder, 1/16");
    if gpu_supported {
        job = decode_on(&on_gpu, GpuDecoder::size_buffer_needed, job, "then GpuDecoder, 1/16");
        job.set_mode(VideoDecodeMode::FullResPremium);
        job = decode_on(&on_async, AsyncDecoder::size_buffer_needed, job, "then AsyncDecoder, full");
    }

    // An internal buffer too small for the decoder is replaced.
    job.set_mode(VideoDecodeMode::QuarterResGood);
    job.set_internal_buffer(AlignedBuffer::new(16, 16).unwrap());
    let job = decode_on(&on_async, AsyncDecoder::size_buffer_needed, job, "AsyncDecoder, 16-byte buffer");

    // A caller-owned buffer is used as is, and the SDK refuses one too small,
    // even when an unused internal buffer would be replaced.
    let mut job = job;
    job.set_internal_buffer(AlignedBuffer::new(16, 16).unwrap());
    let mut small = AlignedBuffer::new(4096, 16).unwrap();
    // SAFETY: `small` outlives the job, which the failed submit drops.
    unsafe { job.set_output_buffer(small.as_mut_ptr(), small.len()) };
    assert!(matches!(async_decoder.decode_for_gpu_sdk(job), Err(RedError::OutputBufferInvalid)));

    // A job handed back after an abort request must be reusable. The SDK
    // never clears the request and fails a job submitted with it set.
    let decoders: &[(&str, Submit, SizeNeeded)] = if gpu_supported {
        &[("AsyncDecoder", &on_async, AsyncDecoder::size_buffer_needed), ("GpuDecoder", &on_gpu, GpuDecoder::size_buffer_needed)]
    } else {
        &[("AsyncDecoder", &on_async, AsyncDecoder::size_buffer_needed)]
    };
    for &(name, decode, needed) in decoders {
        let mut job = AsyncDecompressJob::new(&sdk);
        job.set_clip(&clip);
        job.set_mode(VideoDecodeMode::QuarterResGood);
        let fut = decode(job).expect("submit");
        fut.abort();
        match block_on(fut) {
            Ok(job) => {
                decode_on(decode, needed, job, &format!("{name}, reused after an abort request"));
            }
            Err(e) => eprintln!("{name}: the SDK stopped the aborted decode ({e}), so there is no job to reuse"),
        }
    }
}
