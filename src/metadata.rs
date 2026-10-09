// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use cpp::*;
use core::ffi::{ c_char, c_void };
use std::ptr::NonNull;
use crate::Sdk;
use crate::enums::MetadataType;
use crate::ffi::string_from_c;

cpp!{{
    #include "R3DSDKMetadata.h"

    // `R3DSDK::Metadata` without a default constructor (it needs the loaded
    // SDK) and without its implicit copy (which would share, then double-free,
    // its storage), so the Rust `Metadata` gets neither and is only ever
    // handed out by reference.
    struct R3dRsMetadata : R3DSDK::Metadata {
        R3dRsMetadata() = delete;
        R3dRsMetadata(const R3dRsMetadata &) = delete;
    };
}}

/// A metadata value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Int(u32),
    Float(f32),
}
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(v) => write!(f, "{v}"),
            Value::Int(v)    => write!(f, "{v}"),
            Value::Float(v)  => write!(f, "{v}"),
        }
    }
}

cpp_class! {
    /// A metadata table, e.g. the per-frame metadata of a decoded frame. Only
    /// available by reference: from a decode job, or a [`FrameMetadata`].
    pub unsafe struct Metadata as "R3dRsMetadata"
}
// SAFETY: opaque SDK state; the public surface is read-only queries (`count`,
// `exists`, `get`) — all `&self`. `Send` impl'd explicitly (macro elides it);
// `Sync` auto-derives from the inline `[u8; N]` storage.
unsafe impl Send for Metadata {}

/// An owned [`Metadata`] table, see [`Clip::metadata_for_frame`](crate::Clip::metadata_for_frame).
pub struct FrameMetadata {
    ptr: NonNull<Metadata>,
    _sdk: Sdk,
}
// SAFETY: as for `Metadata`, which this owns on the heap.
unsafe impl Send for FrameMetadata {}
unsafe impl Sync for FrameMetadata {}

impl FrameMetadata {
    pub(crate) fn new(sdk: &Sdk) -> Self {
        let ptr: *mut Metadata = cpp!(unsafe [] -> *mut Metadata as "R3DSDK::Metadata *" {
            return new R3DSDK::Metadata();
        });
        Self { ptr: NonNull::new(ptr).expect("operator new returned null"), _sdk: sdk.clone() }
    }
}
impl std::ops::Deref for FrameMetadata {
    type Target = Metadata;
    fn deref(&self) -> &Metadata {
        // SAFETY: `ptr` is the table allocated in `new`, owned by `self`.
        unsafe { self.ptr.as_ref() }
    }
}
impl std::ops::DerefMut for FrameMetadata {
    fn deref_mut(&mut self) -> &mut Metadata {
        // SAFETY: as in `deref`, and `&mut self` makes the access exclusive.
        unsafe { self.ptr.as_mut() }
    }
}
impl Drop for FrameMetadata {
    fn drop(&mut self) {
        let ptr = self.ptr.as_ptr();
        cpp!(unsafe [ptr as "R3DSDK::Metadata *"] { delete ptr; });
    }
}

impl Metadata {
    /// Number of items in the table.
    pub fn count(&self) -> usize {
        cpp!(unsafe [self as "const R3DSDK::Metadata*"] -> usize as "size_t" {
            return self->MetadataCount();
        })
    }
    /// Whether the table holds an item named `key`.
    pub fn exists(&self, key: &str) -> bool {
        let Ok(c_key) = std::ffi::CString::new(key) else { return false };
        let c_key = c_key.as_ptr();
        cpp!(unsafe [self as "const R3DSDK::Metadata*", c_key as "const char*"] -> bool as "bool" {
            return self->MetadataExists(c_key);
        })
    }
    /// The item named `key`, or `None` if there is none.
    pub fn get(&self, key: &str) -> Option<Value> {
        let c_key = std::ffi::CString::new(key).ok()?;
        let c_key = c_key.as_ptr();
        let meta_type: i32 = cpp!(unsafe [self as "const R3DSDK::Metadata*", c_key as "const char*"] -> i32 as "int" {
            return (int)self->MetadataItemType(c_key);
        });
        match MetadataType::try_from(meta_type) {
            Ok(MetadataType::Int) => {
                let value: u32 = cpp!(unsafe [self as "const R3DSDK::Metadata*", c_key as "const char*"] -> u32 as "unsigned int" {
                    return self->MetadataItemAsInt(c_key);
                });
                Some(Value::Int(value))
            }
            Ok(MetadataType::String) => {
                let cppstr: *mut c_void = cpp!(unsafe [self as "const R3DSDK::Metadata*", c_key as "const char*"] -> *mut c_void as "void *" {
                    return new std::string(self->MetadataItemAsString(c_key));
                });
                Some(Value::String(take_cpp_string(cppstr)))
            }
            Ok(MetadataType::Float) => {
                let value: f32 = cpp!(unsafe [self as "const R3DSDK::Metadata*", c_key as "const char*"] -> f32 as "float" {
                    return self->MetadataItemAsFloat(c_key);
                });
                Some(Value::Float(value))
            }
            _ => None,
        }
    }

    /// Iterate over all items of the table.
    pub fn iter(&self) -> MetadataIterator<'_> {
        MetadataIterator {
            inner: self,
            index: 0,
            count: self.count(),
        }
    }
}

/// Copy a heap-allocated `std::string` into a `String` and free it.
pub(crate) fn take_cpp_string(cppstr: *mut c_void) -> String {
    let c_ptr = cpp!(unsafe [cppstr as "std::string*"] -> *const c_char as "const char *" {
        return cppstr->c_str();
    });
    let value = string_from_c(c_ptr).unwrap_or_default();
    cpp!(unsafe [cppstr as "std::string*"] { delete cppstr; });
    value
}

/// Iterator over the items of a [`Metadata`] table.
pub struct MetadataIterator<'a> {
    pub(crate) inner: &'a Metadata,
    pub(crate) index: usize,
    pub(crate) count: usize,
}
impl Iterator for MetadataIterator<'_> {
    type Item = (String, Value);
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.count {
            let index = self.index;
            let self_ptr = self.inner;
            let key = take_cpp_string(cpp!(unsafe [self_ptr as "const R3DSDK::Metadata*", index as "size_t"] -> *mut c_void as "void *" {
                return new std::string(self_ptr->MetadataItemKey(index));
            }));
            let value = self.inner.get(&key)?;
            self.index += 1;
            Some((key, value))
        } else {
            None
        }
    }
}

/// Iterator over the items of a clip's metadata table, see
/// [`Clip::metadata_iter`](crate::Clip::metadata_iter).
pub struct ClipMetadataIterator<'a> {
    pub(crate) inner: &'a crate::Clip,
    pub(crate) index: usize,
    pub(crate) count: usize,
}
impl Iterator for ClipMetadataIterator<'_> {
    type Item = (String, Value);
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.count {
            let index = self.index;
            let self_ptr = self.inner;
            let key = take_cpp_string(cpp!(unsafe [self_ptr as "const R3dRsClip *", index as "size_t"] -> *mut c_void as "void *" {
                return new std::string((*self_ptr)->MetadataItemKey(index));
            }));
            let value = self.inner.metadata(&key)?;
            self.index += 1;
            Some((key, value))
        } else {
            None
        }
    }
}
