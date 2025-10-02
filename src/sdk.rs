// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use crate::{ RedResult, enums::* };
use cpp::*;

cpp! {{
    #include "R3DSDK.h"
}}

pub struct Sdk;

impl Sdk {
    /// This must be called one time before calling any other functions or
    /// constructing any classes. Do not call this for each single thread.
    /// Static initializers must have run before calling this.
    /// path must point to the folder where the dynamic libraries can be found, it cannot be an empty string.
    pub fn initialize(path: &str, flags: InitializeFlags) -> RedResult<Self> {
        let c_path = std::ffi::CString::new(path).unwrap();
        unsafe {
            let c_ptr = c_path.as_ptr();
            let flags = flags.bits();
            let status: InitializeStatus = std::mem::transmute(cpp!(unsafe [c_ptr as "const char *", flags as "unsigned int"] -> i32 as "int" {
                return (int)R3DSDK::InitializeSdk(c_ptr, flags);
            }));
            if status == InitializeStatus::ISInitializeOK {
                Ok(Sdk {})
            } else {
                Err(status.into())
            }
        }
    }

    /// Returns version information for the static/API version as well as any dynamic
    /// libraries succesfully loaded. Should be called after InitializedSdk(), but will
    /// return useful information even if initialization failed. If possible log the
    /// output of this function to a log file to help with customer issue debugging.
    pub fn version() -> String {
        unsafe {
            let ver = cpp!(unsafe [] -> *const core::ffi::c_char as "const char *" {
                return R3DSDK::GetSdkVersion();
            });
            let c_str = std::ffi::CStr::from_ptr(ver);
            c_str.to_str().unwrap().to_string()
        }
    }

    /// Identify the type of file / clip in a quick way without full parsing. This
    /// is particularly useful to determine if a clip can have multiple file parts
    /// (FileId::R3D) that should be presented as a single clip or if each file is
    /// its own individual clip. A positive identification of a format does not
    /// guarantee the SDK can successfully open & decode the given file / clip.
    pub fn identify_file(path: &str) -> FileId {
        let c_path = std::ffi::CString::new(path).unwrap();
        let c_ptr = c_path.as_ptr();
        match cpp!(unsafe [c_ptr as "const char *"] -> i32 as "int" { return (int)R3DSDK::IdentifyFile(c_ptr); }) {
            1 => FileId::R3D,
            3 => FileId::NevNraw,
            4 => FileId::R3dNe,
            _ => FileId::Unknown,
        }
    }
}

impl Drop for Sdk {
    fn drop(&mut self) {
        cpp!(unsafe []  { R3DSDK::FinalizeSdk(); })
    }
}

/* todo!()

// Load 3D LUT (.cube only!) for use in IPP2 decoding. Load search sequence:
// 	1) if filename only, try to load from clipPath if present
// 	2) try to load as a full path
// 	3) try to load from current directory
// If the 3D LUT is a sidecar file then it's important to call Clip::Load3DLut()
// version of this API instead! See description of that API to get more information.
Handle3DLut Load3DLut(const char * lutPath, const char * clipPath = NULL);

// Unload 3D LUTs loaded through one of the Load3DLut() APIs. The supplied
// handle will be set to NULL for you. Can be safely called on R3D/RMD auto loaded 3D LUTs.
void Unload3DLut(Handle3DLut * handle);

// Get the path for the given 3D LUT. Returns NULL if handle is invalid.
// If this is a sidecar 3D LUT then only the filename will be returned.
const char * Get3DLutPath(Handle3DLut handle);

// Save IPP2 entire Output Transform to disk as a single .cube 3D LUT.
// The Output Transform takes the RWG/Log3G10 to the final output image via the user selected settings.
// Returns CSDone if succesful, otherwise returns error code. The following parameters are used:
// - utf8OutputPath  : UTF-8 path to folder where LUT will be created in. Existing LUT with same name will be overwritten!
// - edgeLength      : must be in range 17 -- 65 inclusive
// - colorSpace      : output color space to convert to from REDWideGamutRGB
// - gammaCurve      : output gamma to convert to from Log3G10
// - outputToneMap   : tone map to use
// - highlightRollOff: highlight roll off to use
// - hdrPeakNits     : only used when HDR2084 is selected
CreateStatus SaveRWGLog3G10ToOutputTransform3DLut(const char * utf8OutputPath, int edgeLength,
                                                  ImageColorSpace colorSpace, ImageGammaCurve gammaCurve, ToneMap outputToneMap, RollOff highlightRollOff, unsigned int hdrPeakNits);

// Save IPP2 entire Output Transform to memory as a 3D LUT.
// The Output Transform takes the RWG/Log3G10 to the final output image via the user selected settings.
// Returns false if an invalid parameter was supplied. The following parameters are used:
// - output3DLut     : memory must be allocated before calling function. Size must be at least:
//                     edgeLength * edgeLength * edgeLength * sizeof(float)
// - edgeLength      : must be in range 17 -- 65 inclusive
// - colorSpace      : output color space to convert to from REDWideGamutRGB
// - gammaCurve      : output gamma to convert to from Log3G10
// - outputToneMap   : tone map to use
// - highlightRollOff: highlight roll off to use
// - hdrPeakNits     : only used when HDR2084 is selected
bool CreateRWGLog3G10ToOutputTransform3DLut(float * output3DLut, int edgeLength,
                                            ImageColorSpace colorSpace, ImageGammaCurve gammaCurve, ToneMap outputToneMap, RollOff highlightRollOff, unsigned int hdrPeakNits);

// Save IPP2 Output Transform to memory as a 1D LUT for the Tone Map Curve followed by a 3D LUT.
// The Output Transform takes the RWG/Log3G10 to the final output image via the user selected settings.
// Returns false if an invalid parameter was supplied. The following parameters are used:
// - output3DLut     : memory must be allocated before calling function. Size must be at least:
//                       3 * edgeLength * edgeLength * edgeLength * sizeof(float)
// - edgeLength      : must be in range 17 -- 65 inclusive
// - output1DLut     : memory must be allocated before calling function. Size must be at least:
//                       lutSize * sizeof(float)
// - lutSize         : entries to create in 1D Tone Curve LUT. Must be in range 1024 -- 65536 inclusive.
// - lutScaleFactor  : factor to scale entries in 1D Tone Curve LUT by. Must be 1 or higher.
// - colorSpace      : output color space to convert to from REDWideGamutRGB
// - gammaCurve      : output gamma to convert to from Log3G10
// - outputToneMap   : tone map to use
// - highlightRollOff: highlight roll off to use
// - hdrPeakNits     : only used when HDR2084 is selected
bool CreateRWGLog3G10ToOutputTransformLuts(float * output3DLut, int edgeLength,
                                           float * output1DLut, int lutSize, int lutScaleFactor,
                                           ImageColorSpace colorSpace, ImageGammaCurve gammaCurve, ToneMap outputToneMap, RollOff highlightRollOff, unsigned int hdrPeakNits);

*/
