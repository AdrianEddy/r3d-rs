use r3d_rs::*;

fn main() -> Result<(), RedError> {
    pollster::block_on(async {
        let _sdk = Sdk::initialize(".", InitializeFlags::R3DDecoder)?;
        println!("SDK Version: {}", Sdk::version());

        let mut clip = Clip::from_path("sample.R3D")?;

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

        let mut decoder_opts = R3dDecoderOptions::new()?;
        decoder_opts.set_memory_pool_size(4096)?;
        decoder_opts.set_concurrent_image_count(8)?;

        // Use first CUDA device
        for dev in R3dDecoderOptions::cuda_device_list()? {
            println!("Device {dev:?}");
            decoder_opts.use_cuda_device(&dev)?;
            break;
        }

        let decoder = R3dDecoder::new(&decoder_opts)?;

        // Spawn 10 simultaneous tasks
        let mut tasks = Vec::new();
        for i in 0..10 {
            let mut job = R3dDecodeJob::new()?;
            job.set_clip(&clip);
            job.set_mode(VideoDecodeMode::HalfResGood);
            job.set_pixel_type(VideoPixelType::Rgb16bitInterleaved);
            job.set_video_track_no(0);
            job.set_video_frame_no(i);
            job.set_image_processing(&settings);
            job.allocate_internal_buffer(&clip)?;
            job.allocate_frame_metadata();
            // Start decoding in the backgroun
            tasks.push(decoder.decode(job)?);
        }

        println!("All jobs submitted");

        let results = futures_util::future::join_all(tasks).await;

        println!("All jobs finished");

        for job in results {
            let job = job?;
            println!("Frame {} metadata:", job.video_frame_no());
            for (key, value) in job.metadata()?.iter() {
                println!("{key: <30} {value:?}");
            }
        }

        // Synchronously decode first frame on the CPU:
        let buffer = clip.decode_video_frame(0, VideoDecodeMode::HalfResGood, VideoPixelType::Bgra8bitInterleaved, Some(&settings), None, None)?;
        image::DynamicImage::ImageRgba8(image::ImageBuffer::from_raw(clip.width() as u32 / 2, clip.height() as u32 / 2, buffer.unwrap().as_slice::<u8>().to_vec()).unwrap())
            .save("frame.jpg").unwrap();

        Ok(())
    })
}