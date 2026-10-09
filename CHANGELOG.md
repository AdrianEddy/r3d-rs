# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Initial release, based on R3D RAW SDK 9.2.1.

### Added

- SDK initialization (`Sdk`) for Windows (x64 and ARM64), Linux and macOS.
- Clips: loading, clip information, metadata and per-frame metadata, timecodes,
  IMU samples, audio decoding and synchronous CPU decoding.
- Image processing settings, their limits, and HDRx settings.
- Asynchronous decoding with futures: `R3dDecoder` (CUDA, OpenCL), and
  `AsyncDecoder` / `GpuDecoder` decompression for GPU debayering.
- GPU debayering with CUDA (`RedCuda`), OpenCL (`RedOpenCl`) and Metal
  (`RedMetal`, behind the `metal-debayer` feature), and the bundled
  `gpu::GpuDecodePipeline`.
- Custom file I/O (`CustomIO`, `FilesystemIo`, `StreamIo`).
- `docs-only` feature to build the documentation without the SDK.

### Changed

Compared to the unpublished development versions:

- Values the SDK returns are validated instead of transmuted: unknown status
  codes become `RedError::UnknownStatus`, and `RedError` is `#[non_exhaustive]`.
- Paths and keys containing a NUL byte return an error instead of panicking.
- `Clip` has no `Default` implementation, which created a null clip handle.
- `AlignedBuffer` memory is zero-initialized; its fields are private (use
  `as_ptr`, `as_mut_ptr`, `len` and `align`), and `as_slice` accepts only
  plain number types and checks alignment.
- `Sdk` is a cloneable handle that every object using the SDK holds, so the
  SDK is unloaded only after the last of them is dropped. Initializing while
  it is loaded fails with `RedError::AlreadyInitialized`; `Sdk::current`
  returns the loaded one. Constructors take an `&Sdk`: `Clip::new`,
  `Clip::from_path`, `AsyncDecoder::new`, `AsyncDecoder::threads_available`,
  `GpuDecoder::new`, `AsyncDecompressJob::new`, `R3dDecoderOptions::new`,
  `R3dDecoderOptions::{cuda,opencl}_device_list`, `R3dDecodeJob::new`,
  `RedCuda::new`, `RedOpenCl::new` and `RedMetal::new`.
- Calls into an SDK component the SDK was not initialized with fail with
  `RedError::NotInitializedWith` instead of crashing.
- `CustomIO` is replaced by `Sdk::initialize_with_io`, which keeps the I/O
  interface installed for the SDK's whole lifetime, as the SDK requires.
- `IoInterface` requires `Send + Sync`, as the SDK calls it from its worker
  threads. `StreamIo` has no lifetime parameter; its streams must be
  `Send + 'static`, callbacks are added with `add_callback(&self)`, and
  entries can be removed with `remove`.
- Asynchronous jobs keep what the SDK reads alive: `set_clip` takes an
  `&Arc<Clip>`, `set_image_processing` copies the settings, in-flight jobs
  keep their decoder open, and debayer jobs keep their `RedCuda` /
  `RedOpenCl` / `RedMetal`. `R3dDecodeJob::allocate_internal_buffer` uses the
  job's clip, and `R3dDecodeJob` gained `set_internal_buffer`,
  `internal_buffer` and `take_internal_buffer`.
- Functions taking raw pointers the SDK writes through or raw GPU handles are
  `unsafe`: the jobs' `set_output_buffer`, the debayer jobs'
  `set_raw_host_mem` / `set_raw_device_mem` / `set_output_device_mem` /
  `set_output_device_image`, and the debayers' `check_compatibility` /
  `process_blocking` / `process_async` / `flush`. `gpu::GpuLane` and
  `gpu::GpuOutput` are opaque, created with `unsafe` constructors
  (`GpuLane::cuda`, `GpuOutput::device_mem`, ...), and `GpuOutput` has no
  lifetime parameter. `GpuDecodePipeline` takes the clip as `&Arc<Clip>`.
- `Metadata` is only available by reference; `Clip::metadata_for_frame`
  returns an owned `FrameMetadata`. (`Metadata` could be cloned into a
  double free, and default-constructed without the SDK.)
- `VideoDecodeJob` is private, as no public API consumed it.

[Unreleased]: https://github.com/AdrianEddy/r3d-rs/commits/main
