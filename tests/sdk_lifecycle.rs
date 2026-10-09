// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! The process-wide SDK lifecycle: one loaded SDK at a time, kept loaded by
//! every object using it, components checked before they are used, and the
//! custom I/O interface installed for the SDK's whole lifetime.
//!
//! One test, as the SDK is process-wide. With `R3D_TEST_CLIP` set, the clip is
//! also opened through a `StreamIo`.

mod common;

use r3d_rs::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A [`StreamIo`] that counts the files the SDK opens through it.
struct CountingIo {
    inner: StreamIo,
    opens: AtomicUsize,
}
impl IoInterface for CountingIo {
    fn open(&self, path: &str, access: FileAccess) -> Handle {
        self.opens.fetch_add(1, Ordering::Relaxed);
        self.inner.open(path, access)
    }
    fn filesize(&self, handle: Handle) -> u64 { self.inner.filesize(handle) }
    fn close(&self, handle: Handle) { self.inner.close(handle) }
    fn read(&self, out: &mut [u8], offset: u64, handle: Handle) -> bool { self.inner.read(out, offset, handle) }
    fn write(&self, data: &[u8], handle: Handle) -> bool { self.inner.write(data, handle) }
    fn create_path(&self, path: &str) -> bool { self.inner.create_path(path) }
}

#[test]
fn sdk_lifecycle() {
    assert!(Sdk::current().is_none());

    let sdk = Sdk::initialize(&common::redist_dir(), InitializeFlags::None).expect("R3D SDK init");
    assert!(matches!(Sdk::initialize(&common::redist_dir(), InitializeFlags::None), Err(RedError::AlreadyInitialized)));
    assert!(Sdk::current().is_some());

    // Components the SDK was not initialized with are refused, not called.
    assert!(matches!(R3dDecoderOptions::new(&sdk), Err(RedError::NotInitializedWith(InitializeFlags::R3DDecoder))));
    assert!(matches!(R3dDecodeJob::new(&sdk), Err(RedError::NotInitializedWith(InitializeFlags::R3DDecoder))));
    assert!(matches!(R3dDecoderOptions::cuda_device_list(&sdk), Err(RedError::NotInitializedWith(InitializeFlags::R3DDecoder))));
    #[cfg(not(target_os = "macos"))]
    assert!(matches!(RedCuda::new(&sdk), Err(RedError::NotInitializedWith(InitializeFlags::Cuda))));
    assert!(matches!(RedOpenCl::new(&sdk, ""), Err(RedError::NotInitializedWith(InitializeFlags::OpenCL))));

    // A clip keeps the SDK loaded after the last `Sdk` is gone.
    let clip = Clip::new(&sdk);
    drop(sdk);
    assert!(Sdk::current().is_some());
    assert_eq!(clip.status(), LoadStatus::NoClipOpen);
    drop(clip);
    assert!(Sdk::current().is_none());

    // Once unloaded, the SDK can be initialized again, here with a component.
    let sdk = Sdk::initialize(&common::redist_dir(), InitializeFlags::R3DDecoder).expect("R3D SDK re-init");
    assert!(R3dDecoderOptions::new(&sdk).is_ok());
    drop(sdk);

    // With a custom I/O interface, the SDK opens files through it.
    let io = Arc::new(CountingIo { inner: StreamIo::new(), opens: AtomicUsize::new(0) });
    let sdk = Sdk::initialize_with_io(&common::redist_dir(), InitializeFlags::None, io.clone()).expect("R3D SDK init with I/O");
    assert!(matches!(Clip::from_path(&sdk, "streamed/missing.R3D"), Err(RedError::ClipFailedToOpenFile)));
    assert!(io.opens.load(Ordering::Relaxed) > 0, "the SDK opened the clip through the custom I/O");

    if let Ok(path) = std::env::var("R3D_TEST_CLIP") {
        let file = std::fs::File::open(&path).expect("open R3D_TEST_CLIP");
        io.inner.insert("streamed/clip.R3D", std::io::BufReader::new(file), None);
        let clip = Clip::from_path(&sdk, "streamed/clip.R3D").expect("open the clip through StreamIo");
        assert!(clip.video_frame_count() > 0);
        assert!(clip.metadata_for_frame(0).expect("frame metadata").count() > 0);
    }
    drop(sdk);
    assert!(Sdk::current().is_none());
}
