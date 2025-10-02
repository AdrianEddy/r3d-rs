// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use cpp::*;
use core::ffi::c_void;
use crate::enums::MetadataType;

cpp!{{
	#include "R3DSDKMetadata.h"
}}

#[derive(Debug, Clone)]
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
	pub unsafe struct Metadata as "R3DSDK::Metadata"
}

impl Metadata {
    pub fn count(&self) -> usize {
        cpp!(unsafe [self as "const R3DSDK::Metadata*"] -> usize as "size_t" {
            return self->MetadataCount();
        })
    }
    pub fn exists(&self, key: &str) -> bool {
        let c_key = std::ffi::CString::new(key).unwrap();
        let c_key = c_key.as_ptr();
        cpp!(unsafe [self as "const R3DSDK::Metadata*", c_key as "const char*"] -> bool as "bool" {
            return self->MetadataExists(c_key);
        })
    }
    pub fn get(&self, key: &str) -> Option<Value> {
        unsafe {
            let c_key = std::ffi::CString::new(key).unwrap();
            let c_key = c_key.as_ptr();
            let meta_type: MetadataType = std::mem::transmute(cpp!([self as "const R3DSDK::Metadata*", c_key as "const char*"] -> i32 as "int" {
                return (int)self->MetadataItemType(c_key);
            }));
            match meta_type {
                MetadataType::Int => {
                    let value: u32 = cpp!([self as "const R3DSDK::Metadata*", c_key as "const char*"] -> u32 as "unsigned int" {
                        return self->MetadataItemAsInt(c_key);
                    });
                    Some(Value::Int(value))
                }
                MetadataType::String => {
                    let cppstr: *mut c_void = cpp!([self as "const R3DSDK::Metadata*", c_key as "const char*"] -> *mut c_void as "void *" {
                        return new std::string(self->MetadataItemAsString(c_key));
                    });
                    let c_ptr: *mut c_void = cpp!([cppstr as "std::string*"] -> *mut c_void as "const char *" {
                        return cppstr->c_str();
                    });
                    let value = std::ffi::CStr::from_ptr(c_ptr as *const i8).to_str().map(|x| x.to_string()).unwrap_or_default();
                    cpp!([cppstr as "std::string*"] { delete cppstr; });
                    Some(Value::String(value))
                }
                MetadataType::Float => {
                    let value: f32 = cpp!([self as "const R3DSDK::Metadata*", c_key as "const char*"] -> f32 as "float" {
                        return self->MetadataItemAsFloat(c_key);
                    });
                    Some(Value::Float(value))
                }
                _ => None,
            }
        }
    }

    pub fn iter<'a>(&'a self) -> MetadataIterator<'a> {
        MetadataIterator {
            inner: self,
            index: 0,
            count: self.count(),
        }
    }
}

pub struct MetadataIterator<'a> {
    pub(crate) inner: &'a Metadata,
    pub(crate) index: usize,
    pub(crate) count: usize,
}
impl<'a> Iterator for MetadataIterator<'a> {
    type Item = (String, Value);
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.count {
            let key = unsafe {
                let index = self.index;
                let self_ptr = self.inner;
                let cppstr: *mut c_void = cpp!([self_ptr as "const R3DSDK::Metadata*", index as "size_t"] -> *mut c_void as "void *" {
                    return new std::string(self_ptr->MetadataItemKey(index));
                });
                let c_ptr: *mut c_void = cpp!([cppstr as "std::string*"] -> *mut c_void as "const char *" {
                    return cppstr->c_str();
                });
                let key = std::ffi::CStr::from_ptr(c_ptr as *const i8).to_str().map(|x| x.to_string()).unwrap_or_default();
                cpp!([cppstr as "std::string*"] { delete cppstr; });
                key
            };
            let value = self.inner.get(&key)?;
            self.index += 1;
            Some((key, value))
        } else {
            None
        }
    }
}

pub struct ClipMetadataIterator<'a> {
    pub(crate) inner: &'a crate::Clip,
    pub(crate) index: usize,
    pub(crate) count: usize,
}
impl<'a> Iterator for ClipMetadataIterator<'a> {
    type Item = (String, Value);
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.count {
            let key = unsafe {
                let index = self.index;
                let self_ptr = self.inner;
                let cppstr: *mut c_void = cpp!([self_ptr as "const std::unique_ptr<R3DSDK::Clip> *", index as "size_t"] -> *mut c_void as "void *" {
                    return new std::string((*self_ptr)->MetadataItemKey(index));
                });
                let c_ptr: *mut c_void = cpp!([cppstr as "std::string*"] -> *mut c_void as "const char *" {
                    return cppstr->c_str();
                });
                let key = std::ffi::CStr::from_ptr(c_ptr as *const i8).to_str().map(|x| x.to_string()).unwrap_or_default();
                cpp!([cppstr as "std::string*"] { delete cppstr; });
                key
            };
            let value = self.inner.metadata(&key)?;
            self.index += 1;
            Some((key, value))
        } else {
            None
        }
    }
}
