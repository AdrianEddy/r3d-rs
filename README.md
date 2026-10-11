# r3d-rs

[![Crates.io](https://img.shields.io/crates/v/r3d-rs.svg)](https://crates.io/crates/r3d-rs)
[![Documentation](https://docs.rs/r3d-rs/badge.svg)](https://docs.rs/r3d-rs)
[![CI](https://github.com/AdrianEddy/r3d-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/AdrianEddy/r3d-rs/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/r3d-rs.svg)](#license)
[![MSRV: 1.88](https://img.shields.io/badge/MSRV-1.88-blue.svg)](https://blog.rust-lang.org/2025/06/26/Rust-1.88.0/)

Safe, ergonomic, async-first Rust bindings for RED's **R3D RAW SDK**, for reading
and decoding RED `.R3D`, Nikon N-RAW (`.NEV`) and R3D NE clips.

## Highlights

* **Safe, high-level interfaces** for clips, metadata, timecode, IMU samples, audio,
  image processing settings, and CPU and GPU decoding.
* **Async jobs, no callbacks**: decodes return futures that resolve to the finished job.
* **Async runtime agnostic**: works with any executor; use `pollster` to block when you want simplicity.
* **GPU decoding** through the SDK's `R3DDecoder` (CUDA, OpenCL) or a bundled
  decompress + debayer pipeline ([`gpu::GpuDecodePipeline`]) for CUDA, OpenCL and Metal.
* **Custom I/O**: read clips from any `Read + Seek` source (`Sdk::initialize_with_io`).
* **Sound lifetimes**: every object keeps the SDK loaded, and asynchronous jobs keep their
  clip, settings and decoder alive until the SDK is done with them.
* **Proper error handling**, idiomatic iterators and enums.
* **Documented properly**: items describe what the SDK does and requires, with links between related items.
* **Cross-platform**: Windows (x64 and ARM64), Linux (x64), macOS.

Based on **R3D RAW SDK 9.2.1**.

## Requirements

* The **R3D RAW SDK**, available from [RED's developer page](https://www.red.com/developers)
  under RED's own license.
* `R3DSDK_DIR` set to the SDK folder (the one containing `Include` and `Lib`) when
  building. It defaults to `sdk` in the crate root, which is convenient in a checkout of this repository.
* A C++ compiler, as for the [`cc`](https://crates.io/crates/cc) crate.
* Rust 1.88 or newer.
* At runtime, the SDK's redistributable dynamic libraries. Pass their folder to `Sdk::initialize`:
  * **Windows**: `RED*-x64.dll` (x64) or `RED*-a64.dll` (ARM64).
  * **Linux**: `RED*-x64.so`.
  * **macOS**: `RED*.dylib`.

On Windows ARM64 the SDK supports CPU decoding and OpenCL processing only; requesting
`InitializeFlags::Cuda` there fails with `RedError::CudaNotAvailable`.

```toml
[dependencies]
r3d-rs = "0.1"
pollster = "1" # optional, for simple blocking
```

## Cargo features

| Feature         | Default | Description |
|-----------------|---------|-------------|
| `link`          | yes     | Link the SDK's static library from `R3DSDK_DIR`. Disable it to link the library yourself. |
| `metal-debayer` | no      | Metal GPU debayering (macOS). |
| `docs-only`     | no      | Build without the SDK, for documentation only (e.g. on docs.rs): every SDK call panics. |

## Example

Load the SDK, open a clip, inspect its metadata, and decode frames on the GPU and on the CPU.
See [`examples/decode.rs`](https://github.com/AdrianEddy/r3d-rs/blob/main/examples/decode.rs) for a complete program.

```rust,no_run
use r3d_rs::*;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the SDK's dynamic libraries from its redistributable folder.
    let sdk = Sdk::initialize("path/to/Redistributable/win", InitializeFlags::R3DDecoder)?;
    println!("SDK version: {}", Sdk::version());

    let mut clip = Clip::from_path(&sdk, "A001_C001_0101AB_001.R3D")?;
    println!("{}x{}, {} frames", clip.width(), clip.height(), clip.video_frame_count());
    println!("Timecode: {:?}", clip.timecode(0));
    for (key, value) in clip.metadata_iter() {
        println!("{key: <30} {value}");
    }

    let mut settings = clip.default_image_processing_settings();
    settings.set_output_tone_map(ToneMap::Medium);

    // Decode 4 frames at once on the first CUDA device. The jobs share the
    // clip, as they read it until they complete.
    let clip = Arc::new(clip);
    let mut options = R3dDecoderOptions::new(&sdk)?;
    let device = R3dDecoderOptions::cuda_device_list(&sdk)?.into_iter().next().ok_or("no CUDA device")?;
    options.use_cuda_device(&device)?;
    let decoder = R3dDecoder::new(&options)?;

    let mut jobs = Vec::new();
    for frame in 0..4 {
        let mut job = R3dDecodeJob::new(&sdk)?;
        job.set_clip(&clip);
        job.set_mode(VideoDecodeMode::HalfResGood);
        job.set_pixel_type(VideoPixelType::Rgb16bitInterleaved);
        job.set_video_frame_no(frame);
        job.set_image_processing(&settings);
        job.allocate_internal_buffer()?;
        jobs.push(decoder.decode(job)?);
    }
    // Any executor works; `pollster` simply blocks on the futures.
    for job in pollster::block_on(futures_util::future::join_all(jobs)) {
        println!("Decoded frame {}", job?.video_frame_no());
    }

    // Decode a frame on the CPU, synchronously.
    let mode = VideoDecodeMode::QuarterResGood;
    let frame = clip
        .decode_video_frame(0, mode, VideoPixelType::Bgra8bitInterleaved, Some(&settings), None, None)?
        .expect("decode_video_frame allocates the buffer when none is passed");
    let (width, height) = gpu::scaled_dims(clip.width(), clip.height(), mode);
    println!("{width}x{height} BGRA frame: {} bytes", frame.as_slice::<u8>().len());

    Ok(())
}
```

## Not yet covered

These parts of the SDK have no bindings yet:

* Clip trimming and snapshots.
* 3D LUT loading and export.
* Reading and writing RMD sidecar files (XMP, image and HDR settings).
* Markers, external metadata, white balance picking and floating point audio.
* Camera streaming and RED ROCKET.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-Apache-2.0) at your option.

The R3D RAW SDK is distributed by RED under its own license. You must comply with RED's
terms when downloading the SDK and redistributing its libraries.

[`gpu::GpuDecodePipeline`]: https://docs.rs/r3d-rs/latest/r3d_rs/gpu/struct.GpuDecodePipeline.html
