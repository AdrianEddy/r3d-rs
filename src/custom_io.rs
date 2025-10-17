// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use cpp::*;
use core::ffi::{ CStr, c_char, c_int, c_uchar, c_ulonglong, c_void };
use std::ptr;

cpp!{{
    #include "R3DSDKCustomIO.h"

    extern "C" void* rs_io_open(void *inst, const char* path, int access);
    extern "C" unsigned long long rs_io_filesize(void *inst, void* handle);
    extern "C" void rs_io_close(void *inst, void* handle);
    extern "C" bool rs_io_read(void *inst, unsigned char* out_buf, size_t bytes, unsigned long long offset, void* handle);
    extern "C" bool rs_io_write(void *inst, const unsigned char* in_buf, size_t bytes, void* handle);
    extern "C" bool rs_io_create_path(void *inst, const char* path);

    struct RustIO : public R3DSDK::IOInterface {
        void *instance{nullptr};
        RustIO(void *inst) : R3DSDK::IOInterface(), instance(inst) { }

        R3DSDK::IOInterface::Handle Open(const char *utf8Path, R3DSDK::IOInterface::FileAccess access) override {
            return rs_io_open(instance, utf8Path, (int)access);
        }
        unsigned long long Filesize(R3DSDK::IOInterface::Handle h) override {
            return rs_io_filesize(instance, h);
        }
        void Close(R3DSDK::IOInterface::Handle h) override {
            rs_io_close(instance, h);
        }
        bool Read(void *outBuffer, size_t bytes, unsigned long long offset, R3DSDK::IOInterface::Handle h) override {
            return rs_io_read(instance, reinterpret_cast<unsigned char *>(outBuffer), bytes, offset, h);
        }
        bool Write(const void *inBuffer, size_t bytes, R3DSDK::IOInterface::Handle h) override {
            return rs_io_write(instance, reinterpret_cast<const unsigned char *>(inBuffer), bytes, h);
        }
        bool CreatePath(const char *utf8Path) override {
            return rs_io_create_path(instance, utf8Path);
        }
    };
}}

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FileAccess {
    Read = 1,
    Write = 2,
}

pub type Handle = *mut c_void;
pub const HANDLE_ERROR: Handle = ptr::null_mut();
pub const HANDLE_FALLBACK: Handle = usize::MAX as *mut c_void;

/// Trait you implement in Rust.
pub trait IoInterface {
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

pub struct CustomIO {
    #[allow(dead_code)]
    instance: Box<Box<dyn IoInterface>>,
    raw: *mut c_void,
}
impl CustomIO {
    pub fn install(instance: Box<dyn IoInterface>) -> Self {
        let mut instance = Box::new(instance);
        let instance_ptr = &mut *instance as *mut _ as *mut c_void;

        let raw = cpp!(unsafe [instance_ptr as "void *"] -> *mut c_void as "void *" {
            RustIO *ptr = new RustIO(instance_ptr);
            R3DSDK::SetIoInterface(ptr);
            return ptr;
        });
        Self { instance, raw }
    }
}
impl Drop for CustomIO {
    fn drop(&mut self) {
        let ptr = self.raw;
        cpp!(unsafe [ptr as "RustIO*"] {
            R3DSDK::ResetIoInterface();
            delete ptr;
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rs_io_open(instance: *mut c_void, path: *const c_char, access: c_int) -> Handle {
    if path.is_null() || instance.is_null() {
        return HANDLE_FALLBACK;
    }
    let access = match access {
        1 => FileAccess::Read,
        2 => FileAccess::Write,
        _ => return HANDLE_FALLBACK,
    };
    unsafe {
        let instance = &*(instance as *mut Box<dyn IoInterface>);
        let c = CStr::from_ptr(path);
        instance.open(c.to_string_lossy().as_ref(), access)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rs_io_filesize(instance: *mut c_void, handle: Handle) -> c_ulonglong {
    if handle.is_null() || instance.is_null() {
        return 0;
    }
    unsafe {
        let instance = &*(instance as *mut Box<dyn IoInterface>);
        instance.filesize(handle) as c_ulonglong
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rs_io_close(instance: *mut c_void, handle: Handle) {
    if handle.is_null() || instance.is_null() {
        return;
    }
    unsafe {
        let instance = &*(instance as *mut Box<dyn IoInterface>);
        instance.close(handle);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rs_io_read(instance: *mut c_void, out_buf: *mut c_uchar, bytes: usize, offset: c_ulonglong, handle: Handle) -> bool {
    if handle.is_null() || instance.is_null() || out_buf.is_null() {
        return false;
    }
    unsafe {
        let instance = &*(instance as *mut Box<dyn IoInterface>);
        let out = std::slice::from_raw_parts_mut(out_buf, bytes as usize);
        instance.read(out, offset as u64, handle)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rs_io_write(instance: *mut c_void, in_buf: *const c_uchar, bytes: usize, handle: Handle) -> bool {
    if handle.is_null() || instance.is_null() || in_buf.is_null() {
        return false;
    }
    unsafe {
        let instance = &*(instance as *mut Box<dyn IoInterface>);
        let data = std::slice::from_raw_parts(in_buf, bytes as usize);
        instance.write(data, handle)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rs_io_create_path(instance: *mut c_void, path: *const c_char) -> bool {
    if path.is_null() || instance.is_null() {
        return false;
    }
    unsafe {
        let instance = &*(instance as *mut Box<dyn IoInterface>);
        let c = CStr::from_ptr(path);
        instance.create_path(c.to_string_lossy().as_ref())
    }
}

///////////////////////////////////////////////////////////////////////////////

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct FilesystemIo {
    files: Mutex<HashMap<Handle, Box<File>>>,
}

impl FilesystemIo {
    pub fn new() -> Self {
        Self { files: Mutex::new(HashMap::new()) }
    }

    fn make_handle(file: Box<File>) -> Handle {
        // Leak the Box<File> pointer as an opaque handle; we still keep it in the map
        // for convenient lookup/cleanup. The pointer itself serves as the unique key.
        Box::into_raw(file) as *mut c_void
    }

    fn take_file_ptr(handle: Handle) -> *mut File {
        handle as *mut File
    }
}

impl IoInterface for FilesystemIo {
    fn open(&self, path: &str, access: FileAccess) -> Handle {
        let res = match access {
            FileAccess::Read => OpenOptions::new().read(true).open(path),
            FileAccess::Write => OpenOptions::new().write(true).create(true).truncate(true).open(path),
        };

        match res {
            Ok(mut f) => {
                if f.seek(SeekFrom::Start(0)).is_err() {
                    return HANDLE_ERROR;
                }
                let h = Self::make_handle(Box::new(f));
                self.files.lock().unwrap().insert(h, unsafe { Box::from_raw(Self::take_file_ptr(h)) });
                h
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
        if let Some(f) = map.get(&handle) {
            match f.metadata() {
                Ok(md) => md.len(),
                Err(_) => 0,
            }
        } else { 0 }
    }

    fn close(&self, handle: Handle) {
        let mut map = self.files.lock().unwrap();
        if let Some(bx) = map.remove(&handle) {
            // Drop closes the file.
            drop(bx);
        } else {
            // If it's not in the map but still a pointer, ensure we don't leak:
            unsafe {
                let _ = Box::from_raw(Self::take_file_ptr(handle));
            }
        }
    }

    fn read(&self, out: &mut [u8], offset: u64, handle: Handle) -> bool {
        let map = self.files.lock().unwrap();
        let f = match map.get(&handle) {
            Some(f) => f,
            None => return false,
        };
        // Temporarily clone file handle by dup'ing via try_clone to avoid locking file positions:
        let mut temp = match f.try_clone() {
            Ok(t) => t,
            Err(_) => return false,
        };
        if temp.seek(SeekFrom::Start(offset)).is_err() { return false; }
        let mut total = 0usize;
        while total < out.len() {
            match temp.read(&mut out[total..]) {
                Ok(0) => break,
                Ok(n) => total += n,
                Err(_) => return false,
            }
        }
        total == out.len()
    }

    fn write(&self, data: &[u8], handle: Handle) -> bool {
        let map = self.files.lock().unwrap();
        let f = match map.get(&handle) {
            Some(f) => f,
            None => return false,
        };
        let mut temp = match f.try_clone() {
            Ok(t) => t,
            Err(_) => return false,
        };
        temp.write_all(data).is_ok()
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

use std::sync::Arc;
struct HandleState<T> {
    entry: Arc<Mutex<T>>,
    size: u64,
}

/// Read-only IO backed by user-provided homogeneous Read+Seek streams (all T).
pub struct StreamIo<T: Read + Seek + Send + Sync + 'static> {
    /// filename -> (stream, filesize)
    table: HashMap<String, (Arc<Mutex<T>>, u64)>,
}

impl<T: Read + Seek + Send + Sync + 'static> StreamIo<T> {
    pub fn new() -> Self {
        Self { table: HashMap::new() }
    }

    /// Insert/replace one entry.
    pub fn insert<S: Into<String>>(&mut self, name: S, stream: T, size: u64) {
        self.table.insert(name.into(), (Arc::new(Mutex::new(stream)), size));
    }

    /// Batch constructor from an iterator.
    pub fn from_iter<I, S>(items: I) -> Self
    where
        I: IntoIterator<Item = (S, (T, u64))>,
        S: Into<String>,
    {
        let mut map = HashMap::new();
        for (name, (stream, size)) in items {
            map.insert(name.into(), (Arc::new(Mutex::new(stream)), size));
        }
        Self { table: map }
    }

    /// Reinterpret an SDK handle back into our typed Arc<HandleState<T>> (without changing refcount).
    fn handle_to_arc(handle: Handle) -> Option<Arc<HandleState<T>>> {
        if handle.is_null() || handle == HANDLE_FALLBACK {
            return None;
        }
        // SAFETY: we created this pointer via Arc::into_raw in `open`, so it’s valid for this T.
        let arc = unsafe { Arc::from_raw(handle as *const HandleState<T>) };
        // Increase strong count by cloning, then forget the temp to keep original count unchanged.
        let cloned = Arc::clone(&arc);
        std::mem::forget(arc); // keep original alive
        Some(cloned)
    }
}

impl<T: Read + Seek + Send + Sync + 'static> IoInterface for StreamIo<T> {
    fn open(&self, path: &str, access: FileAccess) -> Handle {
        if access == FileAccess::Write {
            // We only support read; ask SDK to try its own writer.
            return HANDLE_FALLBACK;
        }
        let (entry, size) = match self.table.get(path) {
            Some((arc, sz)) => (Arc::clone(arc), *sz),
            None => return HANDLE_ERROR,
        };
        let state = HandleState { entry, size };
        // Hand back a stable pointer without Box: store as Arc and leak the raw pointer.
        let arc = Arc::new(state);
        Arc::into_raw(arc) as *mut c_void
    }

    fn filesize(&self, handle: Handle) -> u64 {
        Self::handle_to_arc(handle).map(|a| a.size).unwrap_or(0)
    }

    fn close(&self, handle: Handle) {
        if handle.is_null() || handle == HANDLE_FALLBACK {
            return;
        }
        // SAFETY: reconstruct the Arc from the raw pointer, dropping one strong ref.
        unsafe {
            let _ = Arc::from_raw(handle as *const HandleState<T>);
        }
    }

    fn read(&self, out: &mut [u8], offset: u64, handle: Handle) -> bool {
        let arc = match Self::handle_to_arc(handle) {
            Some(a) => a,
            None => return false,
        };
        let mut guard = match arc.entry.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };
        if guard.seek(SeekFrom::Start(offset)).is_err() {
            return false;
        }
        let mut total = 0usize;
        while total < out.len() {
            match guard.read(&mut out[total..]) {
                Ok(0) => break,
                Ok(n) => total += n,
                Err(_) => return false,
            }
        }
        total == out.len()
    }

    fn write(&self, _data: &[u8], _handle: Handle) -> bool {
        false
    }

    fn create_path(&self, _path: &str) -> bool {
        false
    }
}
