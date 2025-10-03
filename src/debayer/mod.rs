// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

mod opencl;
pub use opencl::*;

#[cfg(not(target_os = "macos"))]
mod cuda;
#[cfg(not(target_os = "macos"))]
pub use cuda::*;

#[cfg(feature = "metal-debayer")]
mod metal;
#[cfg(feature = "metal-debayer")]
pub use metal::*;