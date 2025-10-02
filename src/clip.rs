// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use crate::{RedResult, RedError, metadata::*};
use cpp::*;
use core::ffi::c_void;
use crate::enums::*;
use crate::image_processing_settings::*;

cpp!{{
	#include "R3DSDK.h"
	#include <memory>
}}

cpp_class! {
	/// Class for loading a clip, retrieving information and extracting images.
	/// This class is thread-safe as long as no call to LoadFrom() or Close()
	/// is in progress.
	pub unsafe struct Clip as "std::unique_ptr<R3DSDK::Clip>"
}
impl Clip {
    /// Create a new Clip instance. Use this if you do not want to load a clip
	/// when constructing this class. Use LoadFrom() to load a clip before
	/// calling any other functions.
    pub fn new() -> Self {
        cpp!(unsafe [] -> Clip as "std::unique_ptr<R3DSDK::Clip>" { return std::make_unique<R3DSDK::Clip>(); })
    }


	pub fn as_mut_ptr(&self) -> *mut core::ffi::c_void {
		cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *"] -> *mut core::ffi::c_void as "void *" { return (*self).get(); })
	}

	/// Create a new Clip instance from a file path. Use this if you want to load a clip from a UTF-8
	/// path when you construct the class. Load status can be checked through Status().
    pub fn from_path(path: &str) -> RedResult<Self> {
        let c_path = std::ffi::CString::new(path).unwrap();
		let c_ptr = c_path.as_ptr();
        let clip = cpp!(unsafe [c_ptr as "const char *"] -> Clip as "std::unique_ptr<R3DSDK::Clip>" { return std::make_unique<R3DSDK::Clip>(c_ptr); });
        let status = clip.status();
        if status == LoadStatus::LSClipLoaded {
            Ok(clip)
        } else {
            Err(status.into())
        }
    }

	pub fn status(&self) -> LoadStatus {
		unsafe {
			std::mem::transmute(cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *"] -> i32 as "int" {
				return (int)(*self)->Status();
			}))
		}
	}

    /// Load the R3D file from the given UTF-8 path. If there are any other R3D
	/// files that belong to the same clip these will get loaded as well.
	/// If a clip was already open it will be closed before opening the
	/// one specified in the pathToFile parameter.
    pub fn load_from(&mut self, path: &str) -> RedResult<()> {
        let c_path = std::ffi::CString::new(path).unwrap();
		let c_ptr = c_path.as_ptr();
        let status: LoadStatus = unsafe { std::mem::transmute(cpp!([self as "const std::unique_ptr<R3DSDK::Clip> *", c_ptr as "const char *"] -> i32 as "int" {
            return (int)(*self)->LoadFrom(c_ptr);
        })) };
        if status == LoadStatus::LSClipLoaded {
            Ok(())
        } else {
            Err(status.into())
        }
    }

	/// Closes any open files (class destructor will also call this function).
	pub fn close(&mut self) {
        cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *"]  { (*self)->Close(); })
    }

	/// Closes all handles for all R3D parts belonging to the clip. Handles
	/// will be re-opened on a per R3D part basis as needed automatically.
	pub fn close_file_handles(&mut self) {
        cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *"]  { (*self)->CloseFileHandles(); })
    }

	/// Indicates if RMD sidecar file was found when clip was loaded
	pub fn rmd_sidecar_present(&self) -> bool {
        cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *"] -> bool as "bool" { return (*self)->RmdSidecarPresent(); })
    }

	/// Retrieve the UTF-8 path for the RMD sidecar file the R3D SDK will look
	/// for when trying to load or update an RMD file or create a new one.
	/// You will receive a valid path even if no RMD file is present
	/// Returns null if no clip has been (succesfully) loaded.
	pub fn get_rmd_path(&self) -> Option<String> {
        let ptr = cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *"] -> *const core::ffi::c_char as "const char *" { return (*self)->GetRmdPath(); });
        if ptr.is_null() {
            None
        } else {
            let c_str = unsafe { std::ffi::CStr::from_ptr(ptr) };
            Some(c_str.to_str().unwrap().to_string())
        }
    }
	/// Checks frame CRC if present. For HDRx both tracks are checked. CRC specific return values:
	///
	/// DSDecodeOK             : CRC present and matched
	/// DSDecodeFailed         : CRC present and check failed
	/// DSUnsupportedClipFormat: clip/frame does not have CRC
    pub fn check_frame(&self, video_frame_no: usize) -> RedResult<()> {
		let status: DecodeStatus = unsafe { std::mem::transmute(cpp!([self as "const std::unique_ptr<R3DSDK::Clip> *", video_frame_no as "size_t"] -> i32 as "int" {
			return (int)(*self)->CheckFrame(video_frame_no);
		})) };
        match status {
            DecodeStatus::DSDecodeOK => Ok(()),
            status => Err(status.into()),
        }
    }
    pub fn extended_highlights_enabled(&self) -> bool {
        cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *"] -> bool as "bool" { return (*self)->ExtendedHighlightsEnabled(); })
    }
    pub fn set_extended_highlights(&mut self, enabled: bool) -> RedResult<bool> {
        match cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *", enabled as "bool"] -> bool as "bool" {
            return (*self)->SetExtendedHighlights(enabled);
        }) {
            true => Ok(true),
            false => Err(RedError::Other("Failed to set extended highlights".into())),
        }
    }

	/// Returns number of video tracks in clip. This will always return 1 for non-HDRx clips
    pub fn video_track_count(&self) -> usize {
        cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *"] -> usize as "size_t" { return (*self)->VideoTrackCount(); })
    }


	/*// ******************************
	// SAVING CLIPS
	// ******************************
	// The callback will be called whenever a frame has been added, the
	// operation has been completed or an error has occurred. The last callback
	// you will get is either the done status or an error. If 'status' does
	// not equal CSFrameAdded the process is done or an error was encountered.
	//
	// Return 'true' from your callback if you want the trim to continue
	// Return 'false' if you want to abort the trim
	typedef bool (*TrimCallback)(CreateStatus status, void * privateData, size_t frameNoDone, size_t lastFrameToDo);

	// Create a new clip from another one with a specific in and out point.
	// 'destination' is an existing folder where the output files will be created
	// 'destination' is a UTF-8 path
	//
	// This is an asynchronous call, it will return immediately. The 'destination'
	// parameter can be free'd or deleted as soon as this call returns. The
	// 'source' clip must remain around for the duration of this function.
	// The function returns CSStarted if the trim is under way and you will
	// receive at least one more callback when this value is returned.
	static CreateStatus CreateTrimFrom(const Clip & source, const char * destination, size_t startFrameNo, size_t endFrameNo, bool includeAudio, void * privateData, TrimCallback callback);

	// Create single frame clip from another one. 'destination' is a full UTF-8 path
	// including filename (unlike CreateTrimFrom above). The extension '.R3D'
	// must be at the end and it must be in capitals! The last character in the
	// filename part of the path CANNOT be a digit! This call returns immediately.
	static CreateStatus CreateSnapshotFrom(const Clip & source, const char * destination, size_t frameNo);

	// If you try to include a frame (either through CreateTrimFrom or CreateSnapshotFrom)
	// that is a dropped frame you will receive CSFailedToGetSourceFrame return code.
	// In this case CreateSnapshotFrom will not have created an output file at all
	// In this case CreateTrimFrom will have created a clip up to the dropped frame.*/


	// ******************************
	// SYNCHRONOUS SOFTWARE DECODING (MAIN TRACK)
	// ******************************

	/// Compute total buffer size in bytes for a decode with given clip dimensions, mode and pixel type.
	/// This mirrors the C++ sample: contiguous data, no extra per-row padding, only 16B buffer-start alignment.
	pub fn calculate_buffer_size(&self, mode: &VideoDecodeMode, pixel_type: &VideoPixelType) -> RedResult<usize> {
		let src_width  = self.width() as u32;
		let src_height = self.height() as u32;
		let (w, h) = scaled_dims(src_width, src_height, mode);
		let info = pixel_info(pixel_type);

		let bytes = if info.interleaved {
			w.checked_mul(h)
				.and_then(|px| px.checked_mul(info.bpp_or_sample))
				.unwrap()
		} else {
			// planar RGB: channels * (w * h * bytes_per_sample)
			let per_plane = w.checked_mul(h)
				.and_then(|px| px.checked_mul(info.bpp_or_sample))
				.unwrap();
			per_plane.checked_mul(info.channels)
				.unwrap()
		};

		Ok(bytes)
	}

	pub fn allocate_aligned_buffer(&self, mode: &VideoDecodeMode, pixel_type: &VideoPixelType, align: usize) -> RedResult<AlignedBuffer> {
		AlignedBuffer::new(self.calculate_buffer_size(mode, pixel_type)?, align)
	}

	/// Decode the given video frame with the supplied decode settings,
	/// output buffer and image processing settings (through decodeJob)
	///
	/// - If `buffer` is `Some`, it must be large enough and 16B-aligned; returns `Ok(None)` on success.
	/// - If `buffer` is `None`, it allocates a 16B-aligned buffer internally and returns `Ok(Some(vec))`.
	pub fn decode_video_frame(
		&self,
		video_frame_no: usize,
		mode: VideoDecodeMode,
		pixel_type: VideoPixelType,
		image_settings: Option<&ImageProcessingSettings>,
		hdr_settings: Option<&HdrProcessingSettings>,
		buffer: Option<&mut AlignedBuffer>,
	) -> RedResult<Option<AlignedBuffer>> {
		// Validate combination and compute needed size based on the clip dimensions.
		let size_needed = self.calculate_buffer_size(&mode, &pixel_type)?;

		// Build the decode job
		let mut job = VideoDecodeJob::default();
		job.set_mode(mode);
		job.set_pixel_type(pixel_type);
		if let Some(image_settings) = image_settings {
			job.set_image_processing(image_settings);
		}
		if let Some(hdr_settings) = hdr_settings {
			job.set_hdr_processing(hdr_settings);
		}

		let job_ref = &job as *const _;
		match buffer {
			Some(out) => {
				if out.len() < size_needed {
					return Err(RedError::BufferTooSmall { needed: size_needed, provided: out.len() });
				}
				job.set_output_buffer(out.ptr as *mut _, size_needed);


				let result: DecodeStatus = unsafe { std::mem::transmute(cpp!([self as "const std::unique_ptr<R3DSDK::Clip> *", video_frame_no as "size_t", job_ref as "const R3DSDK::VideoDecodeJob *"] -> i32 as "int" {
					return (int)(*self)->DecodeVideoFrame(video_frame_no, *job_ref);
				})) };
				match result {
					DecodeStatus::DSDecodeOK => Ok(None),
					s => Err(s.into()),
				}
			}
			None => {
				// Allocate an aligned buffer internally (over-allocate and align the pointer)
    			let buf = AlignedBuffer::new(size_needed, 16)?;
				job.set_output_buffer(buf.ptr as *mut _, buf.len());

				let status: DecodeStatus = unsafe { std::mem::transmute(cpp!([self as "const std::unique_ptr<R3DSDK::Clip> *", video_frame_no as "size_t", job_ref as "const R3DSDK::VideoDecodeJob *"] -> i32 as "int" {
					return (int)(*self)->DecodeVideoFrame(video_frame_no, *job_ref);
				})) };
				if status != DecodeStatus::DSDecodeOK {
					return Err(status.into());
				}

				Ok(Some(buf))
			}
		}
	}

	// ******************************
	// CLIP INFORMATION
	// ******************************

	/// Get a unique 16-byte clip identifier. There is no guarantee that this
	/// uuid follows the ITU-T Rec. X.667 / ISO/IEC 9834-8:2005 standards!
	/// Returns false if no clip is loaded or uuid is NULL, otherwise returns
	/// true. Output uuid buffer must be able to hold 16 bytes.
    pub fn uuid(&self) -> Option<[u8; 16]> {
        let mut uuid = [0u8; 16];
		let ptr = uuid.as_mut_ptr();
        let result = cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *", ptr as "uint8_t *"] -> bool as "bool" { return (*self)->Uuid(ptr); });
        if result { Some(uuid) } else { None }
    }

	/// Width of the clip, will return 0 if no clip is loaded.
    pub fn width(&self) -> usize {
        cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *"] -> usize as "size_t" { return (*self)->Width(); })
    }

	/// Height of the clip, will return 0 if no clip is loaded.
    pub fn height(&self) -> usize {
        cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *"] -> usize as "size_t" { return (*self)->Height(); })
    }

	/// Number of video frames in the clip, will return 0 if no clip is loaded.
    pub fn video_frame_count(&self) -> usize {
        cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *"] -> usize as "size_t" { return (*self)->VideoFrameCount(); })
    }

	/// Number of audio channels in the clip, will return 0 if no
	/// clip is loaded or if the clip does not have any audio. To
	/// get the channel map (which of the 4 channels was used),
	/// query the RMD_CHANNEL_MASK metadata item.
    pub fn audio_channel_count(&self) -> usize {
        cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *"] -> usize as "size_t" { return (*self)->AudioChannelCount(); })
    }

	/// Number of audio blocks and their maximum size in the clip
	/// Will return 0 if no clip is loaded or if maximumSize is
	/// NULL, otherwise returns number of audio blocks and the maximum
    /// buffer size that will be needed to process all audio blocks.
    /// This size will be a multiple of 512 (guaranteed)
    pub fn audio_block_count_and_size(&self) -> (usize, usize) {
        let mut maximum_size: usize = 0;
		let ptr = &mut maximum_size as *mut usize;
        let count = cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *", ptr as "size_t *"] -> usize as "size_t" {
            return (*self)->AudioBlockCountAndSize(ptr);
        });
        (count, maximum_size)
    }

	/// Number of audio samples *per channel* in the clip. All
	/// channels have the exact same number of samples. Returns
	/// 0 if an error is encountered or if the clip has no audio
    pub fn audio_sample_count(&self) -> u64 {
        cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *"] -> u64 as "uint64_t" { return (*self)->AudioSampleCount(); })
    }

	/// Framerate for the video & audio 'track(s)'. Numerator & denominator
	/// can be found seperately in the metadata table. Will return 0.0 if no
	/// clip is loaded.
	pub fn video_audio_framerate(&self) -> f32 {
		cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *"] -> f32 as "float" { return (*self)->VideoAudioFramerate(); })
	}

	/// Framerate for the timecode 'track'. This will be running at half the
	/// video & audio 'track' rate if these are running at over 30 fps. The
	/// timecode functions below will alternate '.' and ':' in that case!
	/// Will return 0.0 if no clip is loaded.
	pub fn timecode_framerate(&self) -> f32 {
		cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *"] -> f32 as "float" { return (*self)->TimecodeFramerate(); })
    }

	/// Retrieve the clip's default timecode 'track' for a given video frame.
	/// This default was set on the camera when the clip was recorded. The
	/// returned value is valid until the next time this function is called
	/// or either AbsoluteTimecode() or EdgeTimecode() depending on which
	/// 'track' is the default. Will return None if no clip is loaded or if videoFrameNo is out of bounds.
    ///
    ///
	/// You can encounter two kinds of timecodes in an R3D file:
	///
	/// 01:00:00:00 -> 01:00:00:01 etc. for clips running at 30 fps and below
	/// 01.00.00.00 -> 01:00:00:00 etc. for clips running over 30 fps
	// const char * Timecode(size_t videoFrameNo);
    pub fn timecode(&mut self, video_frame_no: usize) -> Option<String> {
        let ptr = cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *", video_frame_no as "size_t"] -> *const i8 as "const char *" {
            return (*self)->Timecode(video_frame_no);
        });
        if ptr.is_null() {
            None
        } else {
            let c_str = unsafe { std::ffi::CStr::from_ptr(ptr) };
            Some(c_str.to_str().unwrap().to_string())
        }
    }

	/// Retrieve the clip's absolute (time of day or external) timecode for
	/// a given video frame. The returned value is valid until the next time
	/// this function or Timecode() is called. Will return None if
	/// no clip is loaded or if VideoFrameNo is out of bounds.
    pub fn absolute_timecode(&mut self, video_frame_no: usize) -> Option<String> {
        let ptr = cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *", video_frame_no as "size_t"] -> *const i8 as "const char *" {
            return (*self)->AbsoluteTimecode(video_frame_no);
        });
        if ptr.is_null() {
            None
        } else {
            let c_str = unsafe { std::ffi::CStr::from_ptr(ptr) };
            Some(c_str.to_str().unwrap().to_string())
        }
    }


	/// Retrieve the clip's edge (aka run record) timecode for a given video
	/// frame. The returned value is valid until the next time this function
	/// or Timecode() is called. Will return None if no clip is
	/// loaded or videoFrameNo is out of bounds.
    pub fn edge_timecode(&mut self, video_frame_no: usize) -> Option<String> {
        let ptr = cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *", video_frame_no as "size_t"] -> *const i8 as "const char *" {
            return (*self)->EdgeTimecode(video_frame_no);
        });
        if ptr.is_null() {
            None
        } else {
            let c_str = unsafe { std::ffi::CStr::from_ptr(ptr) };
            Some(c_str.to_str().unwrap().to_string())
        }
    }
	/// Get either the clip image processing settings or overriding RMD sidecar metadata
	/// (display this in your UI as the clip defaults!). Use GetClipImageProcessingSettings()
	/// function below to always get the clip settings. PERFORMANCE WARNING: This function
	/// will check for, and parse the RMD sidecar, EVERY time this function gets called!
    pub fn default_image_processing_settings(&self) -> ImageProcessingSettings {
        let mut settings = ImageProcessingSettings::default();
		let ptr_to_settings = &mut settings;
		cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *", ptr_to_settings as "R3DSDK::ImageProcessingSettings *"] {
			(*self)->GetDefaultImageProcessingSettings(*ptr_to_settings);
		});
		settings
    }

	/// Get the image processing settings as they were set on camera when the clip was
	/// recorded. Use this to offer "reset to clip/recorded metadata" in your application.
    pub fn clip_image_processing_settings(&self) -> ImageProcessingSettings {
        let mut settings = ImageProcessingSettings::default();
		let ptr_to_settings = &mut settings;
		cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *", ptr_to_settings as "R3DSDK::ImageProcessingSettings *"] {
			(*self)->GetClipImageProcessingSettings(*ptr_to_settings);
		});
		settings
    }

	/// Function to retrieve per-frame metadata. This is an expensive call as
	/// the frame is loaded from the disk everytime this function is called!
	/// All decoder interfaces (Software & GPU) can return per-frame
	/// metadata as frames are decoded which incurs no extra disk I/O hit.
	/// Recommended use for GetFrameMetadata() is when dumping per-frame metadata
	/// for all (or a certain range of) frames without needing the decoded image.
	pub fn metadata_for_frame(&self, video_frame_no: usize) -> RedResult<Metadata> {
		let mut metadata = Metadata::default();
		let metadata_ptr = &mut metadata;
		let status = unsafe { std::mem::transmute(cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *", video_frame_no as "size_t", metadata_ptr as "R3DSDK::Metadata *"] -> i32 as "int" {
			return (*self)->GetFrameMetadata(*metadata_ptr, video_frame_no);
		})) };
        match status {
            DecodeStatus::DSDecodeOK => Ok(metadata),
            status => Err(status.into()),
        }
	}
	/// Retrieve a metadata item from the clip's metadata table.
	/// Returns None if the key does not exist or if no clip is loaded.
    pub fn metadata(&self, key: &str) -> Option<Value> {
        unsafe {
            let c_key = std::ffi::CString::new(key).unwrap();
            let c_key = c_key.as_ptr();
            let meta_type: MetadataType = std::mem::transmute(cpp!([self as "const std::unique_ptr<R3DSDK::Clip> *", c_key as "const char*"] -> i32 as "int" {
                return (int)(*self)->MetadataItemType(c_key);
            }));
            match meta_type {
                MetadataType::Int => {
                    let value: u32 = cpp!([self as "const std::unique_ptr<R3DSDK::Clip> *", c_key as "const char*"] -> u32 as "unsigned int" {
                        return (*self)->MetadataItemAsInt(c_key);
                    });
                    Some(Value::Int(value))
                }
                MetadataType::String => {
                    let cppstr: *mut c_void = cpp!([self as "const std::unique_ptr<R3DSDK::Clip> *", c_key as "const char*"] -> *mut c_void as "void *" {
                        return new std::string((*self)->MetadataItemAsString(c_key));
                    });
                    let c_ptr: *mut c_void = cpp!([cppstr as "std::string*"] -> *mut c_void as "const char *" {
                        return cppstr->c_str();
                    });
                    let value = std::ffi::CStr::from_ptr(c_ptr as *const i8).to_str().map(|x| x.to_string()).unwrap_or_default();
                    cpp!([cppstr as "std::string*"] { delete cppstr; });
                    Some(Value::String(value))
                }
                MetadataType::Float => {
                    let value: f32 = cpp!([self as "const std::unique_ptr<R3DSDK::Clip> *", c_key as "const char*"] -> f32 as "float" {
                        return (*self)->MetadataItemAsFloat(c_key);
                    });
                    Some(Value::Float(value))
                }
                _ => None,
            }
        }
    }

    pub fn metadata_iter<'a>(&'a self) -> ClipMetadataIterator<'a> {
		let count = cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *"] -> usize as "size_t" { return (*self)->MetadataCount(); });
        ClipMetadataIterator {
            inner: self,
            index: 0,
            count,
        }
    }

	/// ASYNC IMU API FOR V-RAPTOR RECORDINGS
	/// see RMD_FRAME_TIMESTAMP metadata and Clip::GetFrameMetadata() API as well
	pub fn imu_samples<T: std::ops::RangeBounds<u64>>(&self, range: T) -> Vec<IMUSample> {
		let sample_count = cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *"] -> usize as "size_t" { return (*self)->IMUSampleCount(); });

		let start = match range.start_bound() {
            std::ops::Bound::Included(&start) => start,
            std::ops::Bound::Excluded(&start) => start + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&end) => end + 1,
            std::ops::Bound::Excluded(&end) => end,
            std::ops::Bound::Unbounded => sample_count as u64,
        };
        let count = end - start;
		if start >= sample_count as u64 || end > sample_count as u64 || count == 0 {
			return Vec::new();
		}
		let mut samples = Vec::with_capacity(count as usize);
		let ptr = samples.as_mut_ptr();
		let success = cpp!(unsafe [self as "const std::unique_ptr<R3DSDK::Clip> *", ptr as "R3DSDK::IMUSample *", start as "size_t", count as "size_t"] -> bool as "bool" {
			return (*self)->DecodeIMUSamples(ptr, start, count);
		});
		if success {
			unsafe { samples.set_len(count as usize); }
			samples
		} else {
			Vec::new()
		}
	}
}

/*
	// Decode the given audio block into the supplied buffer. This audio block has 24- or 32-bit, see
	// the RMD_SAMPLE_SIZE metadata, Big Endian audio. For 24-bit audio it will be MSB aligned in a
	// 32-bit word. In that case the lower 8-bits are set to zero. If there is more than one audio
	// channel (see AudioChannelCount() API below) the samples will be interleaved.
	//
	// Clips with floating point audio will automatically be converted to integer with the gains set
	// to the values returned by the GetFloatAudioDefaultConversionGain() API.
	//
	// outputBuffer must point to a 512-byte aligned buffer that is big enough to hold the requested
	// audio block (see AudioBlockCountAndSize() API below). The size of the memory block must be set
	// in bufferSize which will be updated by the actual number of bytes written into the audio buffer
	// which will always be a multiple of 4 * AudioChannelCount().
	//
	// The function will fail if no clip is open, no audio is present, audioBlockNo is out of range or
	// outputBuffer and bufferSize are NULL or not aligned properly. This function is *NOT* thread-safe!
	DecodeStatus DecodeAudioBlock(size_t audioBlockNo, void * outputBuffer, size_t * bufferSize) const;

	// Decode raw audio samples from the clip. startSample and numberOfSamples are expressed per channel,
	// do not multiply them by the channel count! The audio returned has 24- or 32-bit, see the
	// RMD_SAMPLE_SIZE metadata, Big Endian audio. For 24-bit audio it will be MSB aligned in a 32-bit
	// word. In that case the lower 8-bits are set to zero. If there is more than one audio channel
	// (see AudioChannelCount() API below) the samples will be interleaved.
	//
	// numberOfSamples will be updated to reflect how many samples were actually written into the
	// output buffer. Normally this is the same as the number of samples requested, unless more
	// samples were requested than exist in the clip
	//
	// Clips with floating point audio will automatically be converted to integer with the gains set
	// the values returned by the GetFloatAudioDefaultConversionGain() API.
	//
	// outputBuffer must point to a 512-byte aligned buffer that is big enough to hold the requested
	// audio samples. The size of the memory block must be set in bufferSize, which will be updated by
	// the actual number of bytes written into the audio buffer, which will always be a multiple of
	// 4 * AudioChannelCount().
	//
	// The function will fail if no clip is open, no audio is present, startSample is out of range or
	// outputBuffer and bufferSize are NULL or not aligned properly. On failure numberOfSamples will
	// be set to zero. This function is *NOT* thread-safe!
	DecodeStatus DecodeAudio(unsigned long long startSample, size_t * numberOfSamples, void * outputBuffer, size_t bufferSize) const;

	// Overloaded versions of DecodeAudioBlock() & DecodeAudio() APIs. These versions allow setting
	// the float-to-integer conversion gain to be used per-channel.
	//
	// channelGainDecibels may be NULL, in which case the values returned by the
	// GetFloatAudioDefaultConversionGain() API are used. When channelGainDecibels is not NULL the
	// pointer to array MUST have at least AudioChannelCount() entries!
	//
	// Channel gain values below -50 dB apply 0 gain (mutes the channel) and values above +50dB
	// are clamped to 50 to have a useful User Interface range.
	//
	// If these functions are called for a clip without floating point audio DSUnsupportedClipFormat
	// is returned as the SDK does not support decoding integer audio with gains applied.
	DecodeStatus DecodeAudioBlock(size_t audioBlockNo, void * outputBuffer, size_t * bufferSize, const int * channelGainDecibels) const;
	DecodeStatus DecodeAudio(unsigned long long startSample, size_t * numberOfSamples, void * outputBuffer, size_t bufferSize, const int * channelGainDecibels) const;

	// Auadio decode APIs for use with native float audio clips to decode the floating point audio as-is.
	// These work exactly the same as the integer versions with the exception that they will fail if
	// the clip does not have floating point audio as the SDK does not support integer-to-float conversion.
	DecodeStatus DecodeFloatAudioBlock(size_t audioBlockNo, void * outputBuffer, size_t * bufferSize) const;
	DecodeStatus DecodeFloatAudio(unsigned long long startSample, size_t * numberOfSamples, void * outputBuffer, size_t bufferSize) const;

	// Return the per-channel audio peak amplitude for clips with floating point audio. The function
	// will fail and return 0.0 if no clip is open, the clip has no floating point audio or
	// channelNo is out of bounds.
	float GetFloatAudioPeakAmplitude(size_t channelNo) const;

	// Return the pre-calculated per-chnnal audio gain in dB that can be used to convert floating
	// point audio to integer or could be used as a default in a User Interface. This function will
	// fail and return 0dB if no clip is open, the clip has no floating point audio or channelNo
	// is out of bounds.
	int GetFloatAudioDefaultConversionGain(size_t channelNo) const;

	// ******************************
	// MULTI-TRACK VIDEO DECODING
	// Can be used to do manual HDR blending or decode a single track
	// ******************************

	// Decode the given video frame on the given track with the supplied decode
	// settings, output buffer and image processing settings (through decodeJob)
	DecodeStatus VideoTrackDecodeFrame(size_t videoTrackNo, size_t videoFrameNo, const VideoDecodeJob & decodeJob) const;

	// ******************************
	// XMP RMD FUNCTIONS
	// ******************************

	// Adds or updates XMP data inside the RMD sidecar file. If this file does
	// not exist it will be created. In the latter case it probably makes sense
	// to call CreateOrUpdateRmd() below as well to add image processing settings.
	// The 'xmpString' parameter must be a std::string UTF-8 encoded.
	// This function returns true if XMP was added. It returns false if an empty
	// input string was supplied or the add/update failed for other reasons.
	bool CreateOrUpdateRmd(const std::string xmpInput) const;

	// Fills 'xmpOutput' with the XMP data (in UTF-8 as a std::string) from the
	// RMD sidecar file if present. If the file is not present or does not contain
	// any XMP data then 'xmpOutput' will be untouched and false will be returned
	bool GetRmdXmp(std::string & xmpOutput) const;

	// ******************************
	// IMAGE PROCESSING SETTINGS
	// ******************************

	// Create or update existing RMD sidecar file. Returns true if success, false otherwise
	// The ImageProcessingSettings struct *must* have Version set to ColorVersion2 or above
	// for this call to succeed. ColorVersion1 is not supported within RMD sidecar files.
	// The RMD sidecar file is an XML based format that travels with the clips. In the future
	// RED cameras will create this file and third party's can add their own data.
	bool CreateOrUpdateRmd(const ImageProcessingSettings & settingsToWrite) const;

	// Get the HDRx processing settings from an existing RMD sidecar file
	// Returns blend settings OR track number to decode. If an error occurs, or no HDR
	// processing settings exist this function will return track 0 (A / main track)
	HdrMode GetRmdHdrProcessingSettings(HdrProcessingSettings & settingsToFill, size_t & trackNoToFill) const;

	// Create or update existing RMD sidecar file either a track number OR HDRx
	// processing settings. Returns true if success, false otherwise
	bool CreateOrUpdateRmd(HdrMode mode, const HdrProcessingSettings & settingsToWrite, size_t trackNoToWrite) const;

	// Whitebalance: convert a pixel area to Kelvin & Tint
	// End-user can pick a point on the image for a given frame. X and Y are in 0.0 - 1.0
	// range and go from upper left (0,0) to lower right (1,1). Function fails if x, y or
	// videoFrameNo are out of bounds, clip has not been loaded or frame is corrupt.
	// It will only update Kelvin & Tint for the supplied ImageProcessingSettings, nothing else
	DecodeStatus GetWhitebalance(ImageProcessingSettings & settingsToFill, size_t videoFrameNo, float x, float y) const;

	// Minimum color version of the ImageProcessingSettings structure required for the clip.
	// If the version is set lower in ImageProcessingSettings than the value returned by this
	// function it will be changed to the highest color version available. This function can
	// help to identify older footage that you may want to enable older image processing for
	// to match how the image used to look in an older SDK (ie, backwards compatibility).
	ColorVersion MinimumColorVersion() const;

	// Returns Default Color Version for clip. ColorVersion3 for clips recorded in-camera
	// with IPP2 color science, ColorVersion2 (Legacy) for all other clips.
	ColorVersion DefaultColorVersion() const;

	// True if 3D LUT was set by user in-camera when clip was recorded.
	bool Camera3DLutPresent() const;

	// Load 3D LUT (.cube only!) for use in IPP2 decoding. Load search sequence:
	// 	1) if filename only, try to load as sidecar file next to the clip.
	// 	2) try to load as a full path
	// 	3) try to load from current directory
	// If the 3D LUT is a sidecar file then it's important to only supply the filename and not
	// the full path so it can later be saved correctly to the sidecar RMD file when CreateOrUpdateRmd()
	// is called above. There is also a static version without sidecar behavior.
	Handle3DLut Load3DLut(const char * path) const;

	// External Metadata is metadata that was fed in to the camera during record
	// from some external source. This is only available through this API,
	// and the amount of metadata present can change from frame to frame.
	// Supply a pointer to externalHasChanged to see if any of the external
	// metadata has changed from the previous frame recorded.
	DecodeStatus GetFrameMetadata(Metadata & metadataToFill, size_t videoFrameNo, Metadata * externalMetadataToFill, bool * externalHasChanged) const;

	bool ExternalETMetadataPresent() const;
	size_t ExternalETMetadataCount() const;
	const char * ExternalETMetadata(size_t index) const;

	// Functions to get still frame markers creating during record
	size_t MarkerCount() const;
	bool GetMarker(size_t markerNo, Marker & markerToFill) const;

	enum FileType
	{
		FileType_Invalid  = 0,	// fileIdx invalid in FileList() call

		// Files that are part of the clip itself, there's always at least 1 of these
		FileType_Clip     = 1,	// .R3D or .NEV

		// Optional sidecar / external files
		FileType_Metadata = 2,	// .RMD or .cdl
		FileType_LUT      = 3,	// .cube
		FileType_Audio    = 5	// .wav
	};

	// These APIs only return information about about the clip sidecar files with
	// the above extensions. Any other files in the same folder are ignored
	size_t FileListCount() const;
	FileType FileList(size_t fileIdx, std::string & pathOutput) const;

*/



cpp_class! {
	/// The decode job structure. This structure needs to be constructed
	/// to submit a decode job to the decoder. Every field must be set,
	/// with the exception of 'ImageProcessing' & "HdrProcessing'
	pub unsafe struct VideoDecodeJob as "R3DSDK::VideoDecodeJob"
}
impl VideoDecodeJob {
	/// Resolution/speed to decode the image at. This will also
	/// influence how much memory is needed for the output buffer
	pub fn mode(&self) -> VideoDecodeMode {
		unsafe { std::mem::transmute(cpp!([self as "const R3DSDK::VideoDecodeJob *"] -> i32 as "int" { return (int)(*self).Mode; })) }
	}
	/// Resolution/speed to decode the image at. This will also
	/// influence how much memory is needed for the output buffer
	pub fn set_mode(&mut self, v: VideoDecodeMode) {
		let v: i32 = v as i32;
		cpp!(unsafe [self as "R3DSDK::VideoDecodeJob *", v as "int"] { (*self).Mode = (R3DSDK::VideoDecodeMode)v; })
	}
	/// Which pixel type to decode the image in. See the VideoPixelType enum for the available options. To get the image in the
	/// original pixel type supported by earlier SDK's set PixelType_16Bit_RGB_Planar
	pub fn pixel_type(&self) -> VideoPixelType {
		unsafe { std::mem::transmute(cpp!([self as "const R3DSDK::VideoDecodeJob *"] -> i32 as "int" { return (int)(*self).PixelType; })) }
	}
	/// Which pixel type to decode the image in. See the VideoPixelType enum for the available options. To get the image in the
	/// original pixel type supported by earlier SDK's set PixelType_16Bit_RGB_Planar
	pub fn set_pixel_type(&mut self, v: VideoPixelType) {
		let v: i32 = v as i32;
		cpp!(unsafe [self as "R3DSDK::VideoDecodeJob *", v as "int"] { (*self).PixelType = (R3DSDK::VideoPixelType)v; })
	}
	/// Pointer to the buffer to store the image in. This cannot be
	/// NULL otherwise the decode will fail. The buffer must be aligned
	/// on a 16-byte boundary (see sample code).
	pub fn set_output_buffer(&mut self, buf: *mut core::ffi::c_void, size: usize) {
		cpp!(unsafe [self as "R3DSDK::VideoDecodeJob *", buf as "void *", size as "size_t"] {
			(*self).OutputBuffer = buf;
			(*self).OutputBufferSize = size;
		})
	}

	/// Image processing settings to apply to the decode. This parameter is optional (set it to NULL if not used!).
	/// The decoder will simply use the clip's default settings if no image processing structure is used to override.
	pub fn set_image_processing(&mut self, v: &ImageProcessingSettings) {
		let ptr = v as *const _;
		cpp!(unsafe [self as "R3DSDK::VideoDecodeJob *", ptr as "R3DSDK::ImageProcessingSettings *"] {
			(*self).ImageProcessing = ptr;
		})
	}
	/// HDRx processing settings to apply to the decode.
	/// This parameter is optional (set it to NULL if not used!).
	/// If the clip is not an HDRx clip, the decode will fail.
	/// If the clip is an HDRx clip and no settings are supplied the main track (A track) will be decoded, or the specified track when using VideoTrackDecodeFrame().
	/// Video track number is ignored when this field is not NULL and you're using VideoTrackDecodeFrame() API!
	pub fn set_hdr_processing(&mut self, v: &HdrProcessingSettings) {
		let ptr = v as *const _;
		cpp!(unsafe [self as "R3DSDK::VideoDecodeJob *", ptr as "R3DSDK::HdrProcessingSettings *"] {
			(*self).HdrProcessing = ptr;
		})
	}

	/// Allocate internal metadata object to receive per-frame metadata.
    /// Call this before submitting the job if you want to receive metadata.
	///
	/// You must call release_frame_metadata() to free the metadata object!
    pub fn allocate_frame_metadata(&mut self) {
        cpp!(unsafe [self as "R3DSDK::VideoDecodeJob *"] {
            self->OutputFrameMetadata = new R3DSDK::Metadata();
        })
    }
    /// Optionally return the decoded frame's per-frame metadata
    pub fn metadata(&self) -> RedResult<&Metadata> {
        let ptr = cpp!(unsafe [self as "R3DSDK::VideoDecodeJob *"] -> *const core::ffi::c_void as "void *" {
            return self->OutputFrameMetadata;
        });
        Ok(unsafe { &*(ptr as *const Metadata) })
    }
	pub fn release_frame_metadata(&mut self) {
		cpp!(unsafe [self as "R3DSDK::VideoDecodeJob *"] {
			delete self->OutputFrameMetadata;
			self->OutputFrameMetadata = nullptr;
		})
	}
}


#[inline]
fn mode_divisor(mode: &VideoDecodeMode) -> usize {
    match mode {
        VideoDecodeMode::FullResPremium   => 1,
        VideoDecodeMode::HalfResPremium   => 2,
        VideoDecodeMode::HalfResGood      => 2,
        VideoDecodeMode::QuarterResGood   => 4,
        VideoDecodeMode::EightResGood     => 8,
        VideoDecodeMode::SixteenthResGood => 16,
    }
}
/// Return the decoded image dimensions *after* the mode scaling.
fn scaled_dims(src_w: u32, src_h: u32, mode: &VideoDecodeMode) -> (usize, usize) {
	let div = mode_divisor(mode);
	(src_w as usize / div, src_h as usize / div)
}

#[derive(Clone, Copy)]
struct PixelInfo {
    /// bytes per pixel (interleaved) OR bytes per *sample* for planar channels
    bpp_or_sample: usize,
    /// number of channels/planes
    channels: usize,
    /// true if interleaved RGB(A); false if planar RGB
    interleaved: bool,
}

/// Known pixel formats per RED SDK docs in the sample.
fn pixel_info(pt: &VideoPixelType) -> PixelInfo {
    match pt {
        VideoPixelType::Rgb16bitInterleaved     => PixelInfo { bpp_or_sample: 6, channels: 1, interleaved: true },  // 3 * 16-bit
        VideoPixelType::RgbHalfFloatInterleaved => PixelInfo { bpp_or_sample: 6, channels: 1, interleaved: true },  // 3 * 16-bit half
        VideoPixelType::RgbHalfFloatAcesInt     => PixelInfo { bpp_or_sample: 6, channels: 1, interleaved: true },  // 3 * 16-bit half
        VideoPixelType::Rgb16bitPlanar          => PixelInfo { bpp_or_sample: 2, channels: 3, interleaved: false }, // per-plane sample = 2B
        VideoPixelType::Bgr8bitInterleaved      => PixelInfo { bpp_or_sample: 3, channels: 1, interleaved: true },  // 3 * 8-bit
        VideoPixelType::Bgra8bitInterleaved     => PixelInfo { bpp_or_sample: 4, channels: 1, interleaved: true },  // 4 * 8-bit
        VideoPixelType::Dpx10bitMethodB         => PixelInfo { bpp_or_sample: 4, channels: 1, interleaved: true },  // packed; SDK uses 4B/px
    }
}

pub struct AlignedBuffer {
	pub ptr: *mut core::ffi::c_void,
	pub layout: std::alloc::Layout
}

impl AlignedBuffer {
	pub fn new(size: usize, alignment: usize) -> RedResult<Self> {
		println!("Allocating aligned buffer: size={size}, alignment={alignment}");
		let layout = std::alloc::Layout::from_size_align(size, alignment)?;
		let ptr = unsafe { std::alloc::alloc(layout) } as *mut core::ffi::c_void;
		if ptr.is_null() {
			Err(RedError::Other(format!("Failed to allocate {size} aligned to {alignment} bytes")))
		} else {
			Ok(Self { ptr, layout })
		}
	}
	pub fn len(&self) -> usize {
		self.layout.size()
	}
	pub fn as_slice<T>(&self) -> &[T] {
		let len = self.len() / std::mem::size_of::<T>();
		unsafe { std::slice::from_raw_parts(self.ptr as *const T, len) }
	}
}
impl Drop for AlignedBuffer {
	fn drop(&mut self) {
		println!("Deallocating aligned buffer: size={}, alignment={}", self.layout.size(), self.layout.align());
		unsafe { std::alloc::dealloc(self.ptr as *mut u8, self.layout); }
	}
}