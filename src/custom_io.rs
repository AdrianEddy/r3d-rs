// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use cpp::*;
use core::ffi::{ CStr, c_char, c_int, c_uchar, c_ulonglong, c_void };
use std::ptr;
use std::sync::Arc;

cpp!{{
    #include "R3DSDKCustomIO.h"

    extern "C" void* r3d_rs_io_open(void *inst, const char* path, int access);
    extern "C" unsigned long long r3d_rs_io_filesize(void *inst, void* handle);
    extern "C" void r3d_rs_io_close(void *inst, void* handle);
    extern "C" bool r3d_rs_io_read(void *inst, unsigned char* out_buf, size_t bytes, unsigned long long offset, void* handle);
    extern "C" bool r3d_rs_io_write(void *inst, const unsigned char* in_buf, size_t bytes, void* handle);
    extern "C" bool r3d_rs_io_create_path(void *inst, const char* path);

    struct RustIO : public R3DSDK::IOInterface {
        void *instance{nullptr};
        RustIO(void *inst) : R3DSDK::IOInterface(), instance(inst) { }

        R3DSDK::IOInterface::Handle Open(const char *utf8Path, R3DSDK::IOInterface::FileAccess access) override {
            return r3d_rs_io_open(instance, utf8Path, (int)access);
        }
        unsigned long long Filesize(R3DSDK::IOInterface::Handle h) override {
            return r3d_rs_io_filesize(instance, h);
        }
        void Close(R3DSDK::IOInterface::Handle h) override {
            r3d_rs_io_close(instance, h);
        }
        bool Read(void *outBuffer, size_t bytes, unsigned long long offset, R3DSDK::IOInterface::Handle h) override {
            return r3d_rs_io_read(instance, reinterpret_cast<unsigned char *>(outBuffer), bytes, offset, h);
        }
        bool Write(const void *inBuffer, size_t bytes, R3DSDK::IOInterface::Handle h) override {
            return r3d_rs_io_write(instance, reinterpret_cast<const unsigned char *>(inBuffer), bytes, h);
        }
        bool CreatePath(const char *utf8Path) override {
            return r3d_rs_io_create_path(instance, utf8Path);
        }
    };
}}

/// How the SDK wants to open a file.
#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FileAccess {
    Read = 1,
    Write = 2,
}

/// Opaque file handle handed to the SDK.
pub type Handle = *mut c_void;
/// Returned from [`IoInterface::open`] when the file cannot be opened.
pub const HANDLE_ERROR: Handle = ptr::null_mut();
/// Returned from [`IoInterface::open`] to let the SDK open the file itself.
pub const HANDLE_FALLBACK: Handle = usize::MAX as *mut c_void;

/// File I/O the SDK performs instead of its own, installed with
/// [`Sdk::initialize_with_io`](crate::Sdk::initialize_with_io).
///
/// The SDK calls it from its worker threads, hence `Send + Sync`.
pub trait IoInterface: Send + Sync {
    /// Open file (binary, positioned at start). Return HANDLE_ERROR / HANDLE_FALLBACK or an opaque handle.
    fn open(&self, path: &str, access: FileAccess) -> Handle;

    /// File size for handle.
    fn filesize(&self, handle: Handle) -> u64;

    /// Close handle.
    fn close(&self, handle: Handle);

    /// Read exactly `bytes` from `offset` into `out`. Return true on full read.
    fn read(&self, out: &mut [u8], offset: u64, handle: Handle) -> bool;

    /// Append/sequential write. Return true on full write.
    fn write(&self, data: &[u8], handle: Handle) -> bool;

    /// Create output path if needed (camera streaming). Return true if ok or not needed.
    fn create_path(&self, path: &str) -> bool;
}

/// An [`IoInterface`] installed as the SDK's I/O interface, see
/// [`Sdk::initialize_with_io`](crate::Sdk::initialize_with_io).
pub(crate) struct InstalledIo {
    /// The `RustIO` C++ object the SDK calls.
    raw: *mut c_void,
    /// The interface `raw` calls into, boxed so its address is stable.
    io: Box<Arc<dyn IoInterface>>,
}
// SAFETY: `raw` is only used to uninstall the interface, and the interface
// itself is `Send + Sync`.
unsafe impl Send for InstalledIo {}
unsafe impl Sync for InstalledIo {}

impl InstalledIo {
    /// Install `io` as the SDK's I/O interface. The SDK requires this after
    /// `InitializeSdk` and before any clip is opened.
    pub(crate) fn install(io: Arc<dyn IoInterface>) -> Self {
        let io = Box::new(io);
        let instance = &*io as *const Arc<dyn IoInterface> as *mut c_void;
        let raw = cpp!(unsafe [instance as "void *"] -> *mut c_void as "void *" {
            RustIO *ptr = new RustIO(instance);
            R3DSDK::SetIoInterface(ptr);
            return ptr;
        });
        Self { raw, io }
    }

    /// Restore the SDK's own I/O. The SDK requires every clip to be closed.
    /// Returns the interface, no longer used by the SDK, for the caller to drop.
    pub(crate) fn uninstall(self) -> Box<Arc<dyn IoInterface>> {
        let ptr = self.raw;
        cpp!(unsafe [ptr as "RustIO*"] {
            R3DSDK::ResetIoInterface();
            delete ptr;
        });
        self.io
    }
}

/// The interface behind the `instance` pointer the SDK passes back.
///
/// # Safety
/// `instance` must be the pointer `InstalledIo::install` handed to `RustIO`,
/// and that `InstalledIo` must not be uninstalled yet.
unsafe fn io_instance<'a>(instance: *mut c_void) -> &'a dyn IoInterface {
    unsafe { &**(instance as *const Arc<dyn IoInterface>) }
}

#[unsafe(no_mangle)]
extern "C" fn r3d_rs_io_open(instance: *mut c_void, path: *const c_char, access: c_int) -> Handle {
    if path.is_null() || instance.is_null() {
        return HANDLE_FALLBACK;
    }
    let access = match access {
        1 => FileAccess::Read,
        2 => FileAccess::Write,
        _ => return HANDLE_FALLBACK,
    };
    // SAFETY: the SDK passes back the instance it was given and a valid C string.
    unsafe {
        let path = CStr::from_ptr(path);
        io_instance(instance).open(path.to_string_lossy().as_ref(), access)
    }
}

#[unsafe(no_mangle)]
extern "C" fn r3d_rs_io_filesize(instance: *mut c_void, handle: Handle) -> c_ulonglong {
    if handle.is_null() || instance.is_null() {
        return 0;
    }
    // SAFETY: the SDK passes back the instance it was given.
    unsafe { io_instance(instance).filesize(handle) }
}

#[unsafe(no_mangle)]
extern "C" fn r3d_rs_io_close(instance: *mut c_void, handle: Handle) {
    if handle.is_null() || instance.is_null() {
        return;
    }
    // SAFETY: the SDK passes back the instance it was given.
    unsafe { io_instance(instance).close(handle) }
}

#[unsafe(no_mangle)]
extern "C" fn r3d_rs_io_read(instance: *mut c_void, out_buf: *mut c_uchar, bytes: usize, offset: c_ulonglong, handle: Handle) -> bool {
    if handle.is_null() || instance.is_null() || out_buf.is_null() {
        return false;
    }
    // SAFETY: the SDK passes back the instance it was given and a buffer of `bytes` bytes.
    unsafe {
        let out = std::slice::from_raw_parts_mut(out_buf, bytes);
        io_instance(instance).read(out, offset, handle)
    }
}

#[unsafe(no_mangle)]
extern "C" fn r3d_rs_io_write(instance: *mut c_void, in_buf: *const c_uchar, bytes: usize, handle: Handle) -> bool {
    if handle.is_null() || instance.is_null() || in_buf.is_null() {
        return false;
    }
    // SAFETY: the SDK passes back the instance it was given and a buffer of `bytes` bytes.
    unsafe {
        let data = std::slice::from_raw_parts(in_buf, bytes);
        io_instance(instance).write(data, handle)
    }
}

#[unsafe(no_mangle)]
extern "C" fn r3d_rs_io_create_path(instance: *mut c_void, path: *const c_char) -> bool {
    if path.is_null() || instance.is_null() {
        return false;
    }
    // SAFETY: the SDK passes back the instance it was given and a valid C string.
    unsafe {
        let path = CStr::from_ptr(path);
        io_instance(instance).create_path(path.to_string_lossy().as_ref())
    }
}

///////////////////////////////////////////////////////////////////////////////

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{ AtomicUsize, Ordering };

/// [`IoInterface`] backed by the filesystem.
#[derive(Default)]
pub struct FilesystemIo {
    files: Mutex<HashMap<usize, File>>,
    next_handle: AtomicUsize,
}

impl FilesystemIo {
    pub fn new() -> Self {
        Self::default()
    }
}

impl IoInterface for FilesystemIo {
    fn open(&self, path: &str, access: FileAccess) -> Handle {
        let res = match access {
            FileAccess::Read => OpenOptions::new().read(true).open(path),
            FileAccess::Write => OpenOptions::new().write(true).create(true).truncate(true).open(path),
        };

        match res {
            Ok(f) => {
                // Handles count up from 1, never reaching HANDLE_ERROR (0) or HANDLE_FALLBACK (usize::MAX).
                let id = self.next_handle.fetch_add(1, Ordering::Relaxed) + 1;
                self.files.lock().unwrap().insert(id, f);
                id as Handle
            }
            Err(e) => {
                // Non-existent file in read mode -> HANDLE_ERROR
                if e.kind() == std::io::ErrorKind::NotFound {
                    HANDLE_ERROR
                } else {
                    HANDLE_FALLBACK
                }
            }
        }
    }

    fn filesize(&self, handle: Handle) -> u64 {
        let map = self.files.lock().unwrap();
        map.get(&(handle as usize)).and_then(|f| f.metadata().ok()).map_or(0, |md| md.len())
    }

    fn close(&self, handle: Handle) {
        // Dropping the `File` closes it.
        self.files.lock().unwrap().remove(&(handle as usize));
    }

    fn read(&self, out: &mut [u8], offset: u64, handle: Handle) -> bool {
        let map = self.files.lock().unwrap();
        let Some(mut f) = map.get(&(handle as usize)) else { return false };
        f.seek(SeekFrom::Start(offset)).is_ok() && f.read_exact(out).is_ok()
    }

    fn write(&self, data: &[u8], handle: Handle) -> bool {
        let map = self.files.lock().unwrap();
        let Some(mut f) = map.get(&(handle as usize)) else { return false };
        f.write_all(data).is_ok()
    }

    fn create_path(&self, path: &str) -> bool {
        match std::fs::create_dir_all(path) {
            Ok(_) => true,
            Err(e) => {
                // If it already exists, treat as success.
                e.kind() == std::io::ErrorKind::AlreadyExists
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

/// A readable, seekable stream that can be read from the SDK's threads.
pub trait ReadSeek: Read + Seek + Send {}
impl<T: Read + Seek + Send + ?Sized> ReadSeek for T {}

use std::sync::RwLock;

/// A stream shared between [`StreamIo`] and the SDK's open handles.
pub type SharedStream = Arc<Mutex<dyn ReadSeek>>;
type StreamCallback = Box<dyn Fn(&str) -> Option<(SharedStream, Option<u64>)> + Send + Sync>;

struct HandleState {
    entry: SharedStream,
    size: u64,
}

/// Read-only [`IoInterface`] backed by user-provided `Read + Seek` streams.
///
/// Keep an `Arc` of it to add streams after installing it with
/// [`Sdk::initialize_with_io`](crate::Sdk::initialize_with_io).
pub struct StreamIo {
    /// filename -> (stream, filesize)
    table: RwLock<HashMap<String, (SharedStream, u64)>>,
    callbacks: RwLock<Vec<StreamCallback>>,
    filesystem_fallback: bool,
}

impl Default for StreamIo {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamIo {
    /// Serve only the inserted streams (and the callbacks').
    pub fn new() -> Self {
        Self { table: RwLock::new(HashMap::new()), callbacks: RwLock::new(Vec::new()), filesystem_fallback: false }
    }

    /// Like [`new`](StreamIo::new), but let the SDK open any other file itself.
    pub fn with_filesystem_fallback() -> Self {
        Self { filesystem_fallback: true, ..Self::new() }
    }

    /// Provide streams for files that were not inserted, on demand. The callback
    /// returns the stream and optionally its size (otherwise found by seeking),
    /// or `None` for files it does not serve. Callbacks are asked in the order
    /// they were added.
    pub fn add_callback<F>(&self, cb: F) where F: Fn(&str) -> Option<(SharedStream, Option<u64>)> + Send + Sync + 'static {
        self.callbacks.write().unwrap().push(Box::new(cb));
    }

    /// Insert/replace one entry. If `size` is `None` it is found by seeking to the end.
    pub fn insert<S: Into<String>, T: Read + Seek + Send + 'static>(&self, name: S, mut stream: T, size: Option<u64>) {
        let size = size.unwrap_or_else(|| stream_len(&mut stream));
        self.table.write().unwrap().insert(name.into(), (Arc::new(Mutex::new(stream)), size));
    }

    /// Remove an inserted entry. Files the SDK still has open keep their stream
    /// until it closes them. Returns whether there was such an entry.
    pub fn remove(&self, name: &str) -> bool {
        self.table.write().unwrap().remove(name).is_some()
    }

    /// The handle's state, with its own reference count (the handle keeps its own).
    fn handle_to_arc(handle: Handle) -> Option<Arc<HandleState>> {
        if handle.is_null() || handle == HANDLE_FALLBACK {
            return None;
        }
        let ptr = handle as *const HandleState;
        // SAFETY: we created this pointer via Arc::into_raw in `open`, and the
        // SDK does not use it after `close`.
        unsafe {
            Arc::increment_strong_count(ptr);
            Some(Arc::from_raw(ptr))
        }
    }
}

/// The length of `stream`, leaving its position unchanged; 0 if it cannot seek.
fn stream_len<S: Seek + ?Sized>(stream: &mut S) -> u64 {
    stream.stream_position().and_then(|cur| {
        let end = stream.seek(SeekFrom::End(0))?;
        if cur != end {
            stream.seek(SeekFrom::Start(cur))?;
        }
        Ok(end)
    }).unwrap_or_default()
}

impl IoInterface for StreamIo {
    fn open(&self, path: &str, access: FileAccess) -> Handle {
        let not_found = if self.filesystem_fallback { HANDLE_FALLBACK } else { HANDLE_ERROR };
        if access == FileAccess::Write {
            // We only support read; ask SDK to try its own writer.
            return HANDLE_FALLBACK;
        }
        let state = match self.table.read().unwrap().get(path) {
            Some((arc, sz)) => HandleState { entry: Arc::clone(arc), size: *sz },
            None => match self.callbacks.read().unwrap().iter().find_map(|cb| cb(path)) {
                Some((stream, size)) => {
                    let size = size.unwrap_or_else(|| stream_len(&mut *stream.lock().unwrap()));
                    HandleState { entry: stream, size }
                }
                None => return not_found,
            },
        };
        Arc::into_raw(Arc::new(state)) as Handle
    }

    fn filesize(&self, handle: Handle) -> u64 {
        Self::handle_to_arc(handle).map(|a| a.size).unwrap_or(0)
    }

    fn close(&self, handle: Handle) {
        if handle.is_null() || handle == HANDLE_FALLBACK {
            return;
        }
        // SAFETY: reconstruct the Arc from the raw pointer, dropping the handle's reference.
        unsafe {
            drop(Arc::from_raw(handle as *const HandleState));
        }
    }

    fn read(&self, out: &mut [u8], offset: u64, handle: Handle) -> bool {
        let Some(arc) = Self::handle_to_arc(handle) else { return false };
        let Ok(mut guard) = arc.entry.lock() else { return false };
        guard.seek(SeekFrom::Start(offset)).is_ok() && guard.read_exact(out).is_ok()
    }

    fn write(&self, _data: &[u8], _handle: Handle) -> bool {
        false
    }

    fn create_path(&self, _path: &str) -> bool {
        false
    }
}
