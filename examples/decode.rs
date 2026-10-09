//! Open a clip, print its properties and metadata, decode a few frames with
//! the R3DDecoder on the first GPU, then decode the first frame on the CPU
//! and save it as `frame.jpg`.
//!
//! Usage: cargo run --example decode -- <redistributable dir> <clip.R3D>
//!
//! `<redistributable dir>` is the folder holding the SDK's dynamic libraries,
//! e.g. `$R3DSDK_DIR/Redistributable/win`.

use r3d_rs::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let (Some(redist_dir), Some(clip_path)) = (args.next(), args.next()) else {
        return Err("usage: decode <redistributable dir> <clip.R3D>".into());
    };

    let _sdk = Sdk::initialize(&redist_dir, InitializeFlags::R3DDecoder)?;
    println!("SDK Version: {}", Sdk::version());

    let mut clip = Clip::from_path(&clip_path)?;

    println!("Clip               {:?}", clip.status());
    println!("Resolution         {}x{}", clip.width(), clip.height());
    println!("Video track count: {}", clip.video_track_count());
    println!("Timecode(0):       {:?}", clip.timecode(0));
    println!("Frames:            {}", clip.video_frame_count());
    println!("Metadata:");
    for (key, value) in clip.metadata_iter() {
        println!("{key: <30} {value:?}");
    }
    println!("First 20 IMU samples:");
    for x in clip.imu_samples(..20) {
        println!("{x:?}");
    }
    println!("Processing settings:");
    let mut settings = clip.default_image_processing_settings();
    println!("Brightness {}", settings.brightness());
    println!("ISO        {}", settings.iso());
    println!("Kelvin     {}", settings.kelvin());

    // Set custom settings
    settings.set_hdr_peak_nits(1000);
    settings.set_cdl_enabled(true);
    settings.set_output_tone_map(ToneMap::None);
    settings.set_image_pipeline_mode(ImagePipeline::FullGraded);

    let mode = VideoDecodeMode::HalfResGood;
    gpu_decode(&clip, &settings, mode)?;

    // Synchronously decode the first frame on the CPU.
    let buffer = clip
        .decode_video_frame(0, mode, VideoPixelType::Bgr8bitInterleaved, Some(&settings), None, None)?
        .expect("decode_video_frame allocates when no buffer is passed");
    let (width, height) = gpu::scaled_dims(clip.width(), clip.height(), mode);
    let rgb: Vec<u8> = buffer.as_slice::<u8>()
        .chunks_exact(3)
        .flat_map(|bgr| [bgr[2], bgr[1], bgr[0]])
        .collect();
    image::RgbImage::from_raw(width as u32, height as u32, rgb)
        .ok_or("decoded frame does not match its dimensions")?
        .save("frame.jpg")?;
    println!("Saved frame 0 to frame.jpg");

    Ok(())
}

/// Decode up to 10 frames concurrently with the R3DDecoder on the first CUDA
/// or OpenCL device. The R3DDecoder needs a GPU and never falls back to the
/// CPU, so this is skipped when there is none.
fn gpu_decode(clip: &Clip, settings: &ImageProcessingSettings, mode: VideoDecodeMode) -> Result<(), RedError> {
    let mut decoder_opts = R3dDecoderOptions::new()?;
    decoder_opts.set_memory_pool_size(4096)?;
    decoder_opts.set_concurrent_image_count(8)?;

    if let Some(dev) = R3dDecoderOptions::cuda_device_list().unwrap_or_default().first() {
        println!("Device {dev:?}");
        decoder_opts.use_cuda_device(dev)?;
    } else if let Some(dev) = R3dDecoderOptions::opencl_device_list().unwrap_or_default().first() {
        println!("Device {dev:?}");
        decoder_opts.use_opencl_device(dev)?;
    } else {
        println!("No CUDA or OpenCL device found, skipping the GPU decode");
        return Ok(());
    }

    let decoder = R3dDecoder::new(&decoder_opts)?;

    // Submit the jobs; each one decodes in the background.
    let mut tasks = Vec::new();
    for i in 0..clip.video_frame_count().min(10) {
        let mut job = R3dDecodeJob::new()?;
        job.set_clip(clip);
        job.set_mode(mode);
        job.set_pixel_type(VideoPixelType::Rgb16bitInterleaved);
        job.set_video_track_no(0);
        job.set_video_frame_no(i);
        job.set_image_processing(settings);
        job.allocate_internal_buffer(clip)?;
        job.allocate_frame_metadata();
        tasks.push(decoder.decode(job)?);
    }
    println!("All jobs submitted");

    let results = pollster::block_on(futures_util::future::join_all(tasks));
    println!("All jobs finished");

    for job in results {
        let job = job?;
        println!("Frame {} metadata:", job.video_frame_no());
        for (key, value) in job.metadata()?.iter() {
            println!("{key: <30} {value:?}");
        }
    }
    Ok(())
}
