// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! The bundled `IoInterface` implementations, exercised directly the way the
//! SDK calls them. No SDK, clip or GPU needed.

use r3d_rs::{FileAccess, FilesystemIo, HANDLE_ERROR, HANDLE_FALLBACK, IoInterface, StreamIo};
use std::io::Cursor;

#[test]
fn filesystem_io_reads_at_offsets() {
    let path = std::env::temp_dir().join(format!("r3d-rs-custom-io-{}.bin", std::process::id()));
    std::fs::write(&path, b"0123456789").unwrap();
    let io = FilesystemIo::new();

    let handle = io.open(path.to_str().unwrap(), FileAccess::Read);
    assert!(handle != HANDLE_ERROR && handle != HANDLE_FALLBACK);
    assert_eq!(io.filesize(handle), 10);

    let mut out = [0u8; 4];
    assert!(io.read(&mut out, 3, handle));
    assert_eq!(&out, b"3456");
    assert!(!io.read(&mut out, 8, handle), "a read past the end is incomplete");

    io.close(handle);
    assert_eq!(io.filesize(handle), 0, "closed handles are gone");
    // Closing an unknown or already closed handle is a no-op.
    io.close(handle);

    let missing = path.with_extension("missing");
    assert_eq!(io.open(missing.to_str().unwrap(), FileAccess::Read), HANDLE_ERROR);
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn stream_io_serves_inserted_streams() {
    let io = StreamIo::new();
    io.insert("clip.R3D", Cursor::new(b"abcdef".to_vec()), None);

    assert_eq!(io.open("other.R3D", FileAccess::Read), HANDLE_ERROR);
    assert_eq!(io.open("clip.R3D", FileAccess::Write), HANDLE_FALLBACK);

    let handle = io.open("clip.R3D", FileAccess::Read);
    assert!(handle != HANDLE_ERROR && handle != HANDLE_FALLBACK);
    assert_eq!(io.filesize(handle), 6);
    let mut out = [0u8; 3];
    assert!(io.read(&mut out, 2, handle));
    assert_eq!(&out, b"cde");
    io.close(handle);

    let fallback = StreamIo::with_filesystem_fallback();
    assert_eq!(fallback.open("other.R3D", FileAccess::Read), HANDLE_FALLBACK);
}
