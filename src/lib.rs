// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

#![doc = include_str!("../README.md")]
#![recursion_limit = "1024"]
#![cfg_attr(docsrs, feature(doc_cfg))]
// With `docs-only`, every `cpp!` block expands to a `panic!("docs-only");` stub.
#![cfg_attr(feature = "docs-only", allow(semicolon_in_expressions_from_macros, unreachable_code, unused, clippy::diverging_sub_expression, clippy::never_loop))]

#[macro_use]
mod ffi;

mod asyncdecoder; pub use asyncdecoder::*;
mod clip;         pub use clip::*;
mod custom_io;    pub use custom_io::*;
mod debayer;      pub use debayer::*;
mod enums;        pub use enums::*;
mod error;        pub use error::*;
mod future;       pub use future::*;
mod metadata;     pub use metadata::*;
mod r3ddecoder;   pub use r3ddecoder::*;
mod sdk;          pub use sdk::*;
mod image_processing_settings; pub use image_processing_settings::*;
mod image_processing_limits;   pub use image_processing_limits::*;
pub mod gpu;
