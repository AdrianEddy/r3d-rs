// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Helpers for values crossing the C++ boundary.

use crate::{ RedError, RedResult };
use core::ffi::c_char;
use std::ffi::CStr;

/// Defines a fieldless `#[repr(i32)]` enum mirroring a C++ SDK enum, together
/// with a checked `TryFrom<i32>` (the error is the unrecognised raw value), so
/// values coming back from the SDK are validated instead of transmuted.
macro_rules! sdk_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $value:expr
            ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[repr(i32)]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant = $value,
            )+
        }

        impl TryFrom<i32> for $name {
            type Error = i32;
            fn try_from(value: i32) -> Result<Self, i32> {
                $(
                    if value == $name::$variant as i32 { return Ok($name::$variant); }
                )+
                Err(value)
            }
        }
    };
}

/// Turn an SDK status code into a `Result`. Every SDK status enum uses 0 for
/// success; any other value maps to the matching [`RedError`], or to
/// [`RedError::UnknownStatus`] if this crate does not know the code.
pub(crate) fn check_status<S>(raw: i32) -> RedResult<()>
where
    S: TryFrom<i32, Error = i32>,
    RedError: From<S>,
{
    if raw == 0 {
        return Ok(());
    }
    Err(S::try_from(raw).map_or(RedError::UnknownStatus(raw), RedError::from))
}

/// Convert the value of a C++ enum field the SDK reports. The SDK refuses to
/// load dynamic libraries of a different version than its headers, so an
/// unknown value means these bindings are out of sync with the SDK headers.
#[track_caller]
pub(crate) fn enum_from_sdk<T: TryFrom<i32, Error = i32>>(raw: i32) -> T {
    T::try_from(raw).unwrap_or_else(|v| panic!("the R3D SDK returned {v}, which is not a valid {}", std::any::type_name::<T>()))
}

/// Copy a NUL-terminated C string the SDK owns into a `String`, replacing
/// invalid UTF-8. `None` if `ptr` is null.
pub(crate) fn string_from_c(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: the SDK hands out valid NUL-terminated strings or null.
    Some(unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned())
}

/// Copy a fixed-size `char[N]` field into a `String`, stopping at the first
/// NUL or at the end of the array if it is not NUL-terminated.
pub(crate) fn string_from_c_array(bytes: &[u8]) -> String {
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..len]).into_owned()
}
