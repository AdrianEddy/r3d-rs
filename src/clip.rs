// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use crate::{ RedResult, RedError, Sdk, metadata::* };
use crate::ffi::{ check_status, enum_from_sdk, string_from_c };
use cpp::*;
use core::ffi::{ c_char, c_void };
use crate::enums::*;
use crate::image_processing_settings::*;

cpp!{{
    #include "R3DSDK.h"
    #include <memory>

    // Owning handle for an `R3DSDK::Clip`. It has no default constructor, so
    // the Rust `Clip` never holds a null clip.
    struct R3dRsClip : std::unique_ptr<R3DSDK::Clip> {
        R3dRsClip() = delete;
        explicit R3dRsClip(R3DSDK::Clip *clip) : std::unique_ptr<R3DSDK::Clip>(clip) { }
    };
}}

cpp_class! {
    /// The C++ side of a [`Clip`].
    unsafe struct ClipHandle as "R3dRsClip"
}

/// Class for loading a clip, retrieving information and extracting images.
/// This class is thread-safe as long as no call to [`load_from`](Clip::load_from)
/// or [`close`](Clip::close) is in progress.
///
/// Asynchronous decode jobs take an `Arc<Clip>`, as they read the clip until
/// they complete.
// `repr(C)` with `handle` first: the `cpp!` blocks pass `&Clip` as the
// `const R3dRsClip *` it starts with.
#[repr(C)]
pub struct Clip {
    handle: ClipHandle,
    sdk: Sdk,
}
const _: () = assert!(std::mem::offset_of!(Clip, handle) == 0);

// SAFETY: `Send` only is impl'd here; `Sync` auto-derives from the `cpp_class!`
// inline `[u8; N]` storage. Soundness rests on Rust's borrow discipline: every
// call the SDK does not document as thread-safe takes `&mut self` — `LoadFrom`
// / `Close` (the class doc above), `DecodeAudioBlock` / `DecodeAudio` (marked
// "*NOT* thread-safe" in `R3DSDK.h`) and the timecode getters (their returned
// string is only valid until the next call). Every remaining `&self` call is
// covered by the class's thread-safety guarantee. `assert_impl_all!(Clip: Send,
// Sync)` in `tests/send_sync_surface.rs` pins this.
unsafe impl Send for Clip {}

impl Clip {
    /// Create a new Clip instance. Use this if you do not want to load a clip
    /// when constructing this class. Use [`load_from`](Clip::load_from) to load
    /// a clip before calling any other functions.
    pub fn new(sdk: &Sdk) -> Self {
        let handle: ClipHandle = cpp!(unsafe [] -> ClipHandle as "R3dRsClip" { return R3dRsClip(new R3DSDK::Clip()); });
        Self { handle, sdk: sdk.clone() }
    }

    /// Pointer to the underlying `R3DSDK::Clip`.
    pub fn as_mut_ptr(&self) -> *mut c_void {
        cpp!(unsafe [self as "const R3dRsClip *"] -> *mut c_void as "void *" { return self->get(); })
    }

    /// Create a new Clip instance from a file path. Use this if you want to load a clip from a UTF-8
    /// path when you construct the class.
    pub fn from_path(sdk: &Sdk, path: &str) -> RedResult<Self> {
        let c_path = std::ffi::CString::new(path).map_err(|_| RedError::InvalidPath)?;
        let c_ptr = c_path.as_ptr();
        let handle: ClipHandle = cpp!(unsafe [c_ptr as "const char *"] -> ClipHandle as "R3dRsClip" { return R3dRsClip(new R3DSDK::Clip(c_ptr)); });
        let clip = Self { handle, sdk: sdk.clone() };
        let status = clip.status();
        if status == LoadStatus::ClipLoaded {
            Ok(clip)
        } else {
            Err(status.into())
        }
    }

    /// The SDK this clip was opened with.
    pub fn sdk(&self) -> &Sdk {
        &self.sdk
    }

    /// Load status of the clip.
    pub fn status(&self) -> LoadStatus {
        enum_from_sdk(cpp!(unsafe [self as "const R3dRsClip *"] -> i32 as "int" {
            return (int)(*self)->Status();
        }))
    }

    /// Load the R3D file from the given UTF-8 path. If there are any other R3D
    /// files that belong to the same clip these will get loaded as well.
    /// If a clip was already open it will be closed before opening the
    /// one specified in the `path` parameter.
    pub fn load_from(&mut self, path: &str) -> RedResult<()> {
        let c_path = std::ffi::CString::new(path).map_err(|_| RedError::InvalidPath)?;
        let c_ptr = c_path.as_ptr();
        let status: i32 = cpp!(unsafe [self as "const R3dRsClip *", c_ptr as "const char *"] -> i32 as "int" {
            return (int)(*self)->LoadFrom(c_ptr);
        });
        check_status::<LoadStatus>(status)
    }

    /// Closes any open files (class destructor will also call this function).
    pub fn close(&mut self) {
        cpp!(unsafe [self as "const R3dRsClip *"]  { (*self)->Close(); })
    }

    /// Closes all handles for all R3D parts belonging to the clip. Handles
    /// will be re-opened on a per R3D part basis as needed automatically.
    pub fn close_file_handles(&mut self) {
        cpp!(unsafe [self as "const R3dRsClip *"]  { (*self)->CloseFileHandles(); })
    }

    /// Indicates if RMD sidecar file was found when clip was loaded
    pub fn rmd_sidecar_present(&self) -> bool {
        cpp!(unsafe [self as "const R3dRsClip *"] -> bool as "bool" { return (*self)->RmdSidecarPresent(); })
    }

    /// Retrieve the UTF-8 path for the RMD sidecar file the R3D SDK will look
    /// for when trying to load or update an RMD file or create a new one.
    /// You will receive a valid path even if no RMD file is present
    /// Returns `None` if no clip has been (successfully) loaded.
    pub fn get_rmd_path(&self) -> Option<String> {
        string_from_c(cpp!(unsafe [self as "const R3dRsClip *"] -> *const c_char as "const char *" { return (*self)->GetRmdPath(); }))
    }

    /// Checks frame CRC if present. For HDRx both tracks are checked. CRC specific return values:
    ///
    /// - `Ok(())`: CRC present and matched
    /// - [`RedError::DecodeFailed`]: CRC present and check failed
    /// - [`RedError::UnsupportedClipFormat`]: clip/frame does not have CRC
    pub fn check_frame(&self, video_frame_no: usize) -> RedResult<()> {
        check_status::<DecodeStatus>(cpp!(unsafe [self as "const R3dRsClip *", video_frame_no as "size_t"] -> i32 as "int" {
            return (int)(*self)->CheckFrame(video_frame_no);
        }))
    }
    pub fn extended_highlights_enabled(&self) -> bool {
        cpp!(unsafe [self as "const R3dRsClip *"] -> bool as "bool" { return (*self)->ExtendedHighlightsEnabled(); })
    }
    pub fn set_extended_highlights(&mut self, enabled: bool) -> RedResult<bool> {
        match cpp!(unsafe [self as "const R3dRsClip *", enabled as "bool"] -> bool as "bool" {
            return (*self)->SetExtendedHighlights(enabled);
        }) {
            true => Ok(true),
            false => Err(RedError::Other("Failed to set extended highlights".into())),
        }
    }

    /// Returns number of video tracks in clip. This will always return 1 for non-HDRx clips
    pub fn video_track_count(&self) -> usize {
        cpp!(unsafe [self as "const R3dRsClip *"] -> usize as "size_t" { return (*self)->VideoTrackCount(); })
    }

    // ******************************
    // SYNCHRONOUS SOFTWARE DECODING (MAIN TRACK)
    // ******************************

    /// Compute total buffer size in bytes for a decode with given clip dimensions, mode and pixel type.
    /// This mirrors the C++ sample: contiguous data, no extra per-row padding, only 16B buffer-start alignment.
    pub fn calculate_buffer_size(&self, mode: &VideoDecodeMode, pixel_type: &VideoPixelType) -> RedResult<usize> {
        let (w, h) = scaled_dims(self.width(), self.height(), *mode);
        w.checked_mul(h)
            .and_then(|px| px.checked_mul(bytes_per_pixel(*pixel_type)))
            .ok_or_else(|| RedError::Other(format!("{w}x{h} {pixel_type:?} frame size overflows usize")))
    }

    /// Allocate a buffer of [`calculate_buffer_size`](Clip::calculate_buffer_size)
    /// bytes, aligned to `align` bytes.
    pub fn allocate_aligned_buffer(&self, mode: &VideoDecodeMode, pixel_type: &VideoPixelType, align: usize) -> RedResult<AlignedBuffer> {
        AlignedBuffer::new(self.calculate_buffer_size(mode, pixel_type)?, align)
    }

    /// Decode the given video frame with the supplied decode settings,
    /// output buffer and image processing settings.
    ///
    /// - If `buffer` is `Some`, it must be large enough and 16B-aligned; returns `Ok(None)` on success.
    /// - If `buffer` is `None`, it allocates a 16B-aligned buffer internally and returns `Ok(Some(buffer))`.
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

        // The job points at `image_settings`, `hdr_settings` and the output
        // buffer, all of which outlive this synchronous call.
        let mut job = VideoDecodeJob::default();
        job.set_mode(mode);
        job.set_pixel_type(pixel_type);
        if let Some(image_settings) = image_settings {
            job.set_image_processing(image_settings);
        }
        if let Some(hdr_settings) = hdr_settings {
            job.set_hdr_processing(hdr_settings);
        }

        let mut allocated = None;
        let out_ptr = match buffer {
            Some(out) => {
                if out.len() < size_needed {
                    return Err(RedError::BufferTooSmall { needed: size_needed, provided: out.len() });
                }
                out.ptr
            }
            None => allocated.insert(AlignedBuffer::new(size_needed, 16)?).ptr,
        };
        job.set_output_buffer(out_ptr, size_needed);

        let job_ref = &job as *const VideoDecodeJob;
        check_status::<DecodeStatus>(cpp!(unsafe [self as "const R3dRsClip *", video_frame_no as "size_t", job_ref as "const R3DSDK::VideoDecodeJob *"] -> i32 as "int" {
            return (int)(*self)->DecodeVideoFrame(video_frame_no, *job_ref);
        }))?;
        Ok(allocated)
    }

    /// Decode the audio block `audio_block_no` into `buffer`.
    ///
    /// `buffer` MUST be 512-byte aligned — allocate it via
    /// `AlignedBuffer::new(size, 512)` where `size` is at least the maximum
    /// block size reported by [`Clip::audio_block_count_and_size`] (that value
    /// is already a multiple of 512). Samples are 24- or 32-bit Big-Endian; for
    /// 24-bit audio each sample is MSB-aligned in a 32-bit word (low 8 bits
    /// zero), and multichannel audio is interleaved.
    ///
    /// A clip without audio reports a maximum block size of 0, and
    /// `AlignedBuffer::new(0, _)` fails, so check
    /// [`Clip::audio_channel_count`] first.
    ///
    /// Returns the number of bytes actually written, always a multiple of
    /// `4 * audio_channel_count()`. The SDK call is not thread-safe, so this
    /// takes `&mut self`.
    pub fn decode_audio_block(&mut self, audio_block_no: usize, buffer: &mut AlignedBuffer) -> RedResult<usize> {
        if buffer.layout.align() < 512 {
            return Err(RedError::BufferNotAligned);
        }
        let mut buffer_size: usize = buffer.len();
        let out_ptr = buffer.ptr;
        let size_ptr = &mut buffer_size as *mut usize;
        check_status::<DecodeStatus>(cpp!(unsafe [self as "const R3dRsClip *", audio_block_no as "size_t", out_ptr as "void *", size_ptr as "size_t *"] -> i32 as "int" {
            return (int)(*self)->DecodeAudioBlock(audio_block_no, out_ptr, size_ptr);
        }))?;
        Ok(buffer_size)
    }

    /// Decode `num_samples` raw audio samples (expressed *per channel* — do NOT
    /// multiply by the channel count, the SDK does that) starting at
    /// `start_sample` into `buffer`.
    ///
    /// `buffer` MUST be 512-byte aligned — allocate it via
    /// `AlignedBuffer::new(size, 512)` with room for
    /// `align_to_512(num_samples * audio_channel_count() * 4)` bytes. Samples
    /// are 24- or 32-bit Big-Endian, interleaved across channels (see
    /// [`Clip::decode_audio_block`]). Fewer samples than requested are written
    /// when `start_sample + num_samples` runs past the end of the clip.
    ///
    /// Returns the number of bytes actually written, always a multiple of
    /// `4 * audio_channel_count()`; the per-channel sample count is
    /// `bytes / (4 * audio_channel_count())`. The SDK call is not thread-safe,
    /// so this takes `&mut self`.
    pub fn decode_audio(&mut self, start_sample: u64, num_samples: usize, buffer: &mut AlignedBuffer) -> RedResult<usize> {
        if buffer.layout.align() < 512 {
            return Err(RedError::BufferNotAligned);
        }
        let mut number_of_samples: usize = num_samples;
        let buffer_size: usize = buffer.len();
        let out_ptr = buffer.ptr;
        let samples_ptr = &mut number_of_samples as *mut usize;
        check_status::<DecodeStatus>(cpp!(unsafe [self as "const R3dRsClip *", start_sample as "unsigned long long", samples_ptr as "size_t *", out_ptr as "void *", buffer_size as "size_t"] -> i32 as "int" {
            return (int)(*self)->DecodeAudio(start_sample, samples_ptr, out_ptr, buffer_size);
        }))?;
        Ok(number_of_samples * self.audio_channel_count() * 4)
    }

    // ******************************
    // CLIP INFORMATION
    // ******************************

    /// Get a unique 16-byte clip identifier. There is no guarantee that this
    /// uuid follows the ITU-T Rec. X.667 / ISO/IEC 9834-8:2005 standards!
    /// Returns `None` if no clip is loaded.
    pub fn uuid(&self) -> Option<[u8; 16]> {
        let mut uuid = [0u8; 16];
        let ptr = uuid.as_mut_ptr();
        let result = cpp!(unsafe [self as "const R3dRsClip *", ptr as "uint8_t *"] -> bool as "bool" { return (*self)->Uuid(ptr); });
        if result { Some(uuid) } else { None }
    }

    /// Width of the clip, will return 0 if no clip is loaded.
    pub fn width(&self) -> usize {
        cpp!(unsafe [self as "const R3dRsClip *"] -> usize as "size_t" { return (*self)->Width(); })
    }

    /// Height of the clip, will return 0 if no clip is loaded.
    pub fn height(&self) -> usize {
        cpp!(unsafe [self as "const R3dRsClip *"] -> usize as "size_t" { return (*self)->Height(); })
    }

    /// Number of video frames in the clip, will return 0 if no clip is loaded.
    pub fn video_frame_count(&self) -> usize {
        cpp!(unsafe [self as "const R3dRsClip *"] -> usize as "size_t" { return (*self)->VideoFrameCount(); })
    }

    /// Number of audio channels in the clip, will return 0 if no
    /// clip is loaded or if the clip does not have any audio. To
    /// get the channel map (which of the 4 channels was used),
    /// query the RMD_CHANNEL_MASK metadata item.
    pub fn audio_channel_count(&self) -> usize {
        cpp!(unsafe [self as "const R3dRsClip *"] -> usize as "size_t" { return (*self)->AudioChannelCount(); })
    }

    /// Number of audio blocks and their maximum size in the clip.
    /// Returns `(0, 0)` if no clip is loaded, otherwise returns number of audio blocks and the maximum
    /// buffer size that will be needed to process all audio blocks.
    /// This size will be a multiple of 512 (guaranteed)
    pub fn audio_block_count_and_size(&self) -> (usize, usize) {
        let mut maximum_size: usize = 0;
        let ptr = &mut maximum_size as *mut usize;
        let count = cpp!(unsafe [self as "const R3dRsClip *", ptr as "size_t *"] -> usize as "size_t" {
            return (*self)->AudioBlockCountAndSize(ptr);
        });
        (count, maximum_size)
    }

    /// Number of audio samples *per channel* in the clip. All
    /// channels have the exact same number of samples. Returns
    /// 0 if an error is encountered or if the clip has no audio
    pub fn audio_sample_count(&self) -> u64 {
        cpp!(unsafe [self as "const R3dRsClip *"] -> u64 as "uint64_t" { return (*self)->AudioSampleCount(); })
    }

    /// Framerate for the video & audio 'track(s)'. Numerator & denominator
    /// can be found separately in the metadata table. Will return 0.0 if no
    /// clip is loaded.
    pub fn video_audio_framerate(&self) -> f32 {
        cpp!(unsafe [self as "const R3dRsClip *"] -> f32 as "float" { return (*self)->VideoAudioFramerate(); })
    }

    /// Framerate for the timecode 'track'. This will be running at half the
    /// video & audio 'track' rate if these are running at over 30 fps. The
    /// timecode functions below will alternate '.' and ':' in that case!
    /// Will return 0.0 if no clip is loaded.
    pub fn timecode_framerate(&self) -> f32 {
        cpp!(unsafe [self as "const R3dRsClip *"] -> f32 as "float" { return (*self)->TimecodeFramerate(); })
    }

    /// Retrieve the clip's default timecode 'track' for a given video frame.
    /// This default was set on the camera when the clip was recorded.
    /// Will return `None` if no clip is loaded or if `video_frame_no` is out of bounds.
    ///
    /// You can encounter two kinds of timecodes in an R3D file:
    ///
    /// 01:00:00:00 -> 01:00:00:01 etc. for clips running at 30 fps and below
    /// 01.00.00.00 -> 01:00:00:00 etc. for clips running over 30 fps
    pub fn timecode(&mut self, video_frame_no: usize) -> Option<String> {
        string_from_c(cpp!(unsafe [self as "const R3dRsClip *", video_frame_no as "size_t"] -> *const c_char as "const char *" {
            return (*self)->Timecode(video_frame_no);
        }))
    }

    /// Retrieve the clip's absolute (time of day or external) timecode for
    /// a given video frame. Will return `None` if no clip is loaded or if
    /// `video_frame_no` is out of bounds.
    pub fn absolute_timecode(&mut self, video_frame_no: usize) -> Option<String> {
        string_from_c(cpp!(unsafe [self as "const R3dRsClip *", video_frame_no as "size_t"] -> *const c_char as "const char *" {
            return (*self)->AbsoluteTimecode(video_frame_no);
        }))
    }

    /// Retrieve the clip's edge (aka run record) timecode for a given video
    /// frame. Will return `None` if no clip is loaded or `video_frame_no` is
    /// out of bounds.
    pub fn edge_timecode(&mut self, video_frame_no: usize) -> Option<String> {
        string_from_c(cpp!(unsafe [self as "const R3dRsClip *", video_frame_no as "size_t"] -> *const c_char as "const char *" {
            return (*self)->EdgeTimecode(video_frame_no);
        }))
    }

    /// Get either the clip image processing settings or overriding RMD sidecar metadata
    /// (display this in your UI as the clip defaults!). Use
    /// [`clip_image_processing_settings`](Clip::clip_image_processing_settings)
    /// to always get the clip settings. PERFORMANCE WARNING: This function
    /// will check for, and parse the RMD sidecar, EVERY time this function gets called!
    pub fn default_image_processing_settings(&self) -> ImageProcessingSettings {
        let mut settings = ImageProcessingSettings::default();
        let ptr_to_settings = &mut settings;
        cpp!(unsafe [self as "const R3dRsClip *", ptr_to_settings as "R3DSDK::ImageProcessingSettings *"] {
            (*self)->GetDefaultImageProcessingSettings(*ptr_to_settings);
        });
        settings
    }

    /// Get the image processing settings as they were set on camera when the clip was
    /// recorded. Use this to offer "reset to clip/recorded metadata" in your application.
    pub fn clip_image_processing_settings(&self) -> ImageProcessingSettings {
        let mut settings = ImageProcessingSettings::default();
        let ptr_to_settings = &mut settings;
        cpp!(unsafe [self as "const R3dRsClip *", ptr_to_settings as "R3DSDK::ImageProcessingSettings *"] {
            (*self)->GetClipImageProcessingSettings(*ptr_to_settings);
        });
        settings
    }

    /// Function to retrieve per-frame metadata. This is an expensive call as
    /// the frame is loaded from the disk every time this function is called!
    /// All decoder interfaces (Software & GPU) can return per-frame
    /// metadata as frames are decoded which incurs no extra disk I/O hit.
    /// Recommended use for this function is when dumping per-frame metadata
    /// for all (or a certain range of) frames without needing the decoded image.
    pub fn metadata_for_frame(&self, video_frame_no: usize) -> RedResult<FrameMetadata> {
        let mut metadata = FrameMetadata::new(&self.sdk);
        let metadata_ptr: &mut Metadata = &mut metadata;
        check_status::<DecodeStatus>(cpp!(unsafe [self as "const R3dRsClip *", video_frame_no as "size_t", metadata_ptr as "R3DSDK::Metadata *"] -> i32 as "int" {
            return (int)(*self)->GetFrameMetadata(*metadata_ptr, video_frame_no);
        }))?;
        Ok(metadata)
    }

    /// Retrieve a metadata item from the clip's metadata table.
    /// Returns `None` if the key does not exist or if no clip is loaded.
    pub fn metadata(&self, key: &str) -> Option<Value> {
        let c_key = std::ffi::CString::new(key).ok()?;
        let c_key = c_key.as_ptr();
        let meta_type: i32 = cpp!(unsafe [self as "const R3dRsClip *", c_key as "const char*"] -> i32 as "int" {
            return (int)(*self)->MetadataItemType(c_key);
        });
        match MetadataType::try_from(meta_type) {
            Ok(MetadataType::Int) => {
                let value: u32 = cpp!(unsafe [self as "const R3dRsClip *", c_key as "const char*"] -> u32 as "unsigned int" {
                    return (*self)->MetadataItemAsInt(c_key);
                });
                Some(Value::Int(value))
            }
            Ok(MetadataType::String) => {
                let cppstr: *mut c_void = cpp!(unsafe [self as "const R3dRsClip *", c_key as "const char*"] -> *mut c_void as "void *" {
                    return new std::string((*self)->MetadataItemAsString(c_key));
                });
                Some(Value::String(take_cpp_string(cppstr)))
            }
            Ok(MetadataType::Float) => {
                let value: f32 = cpp!(unsafe [self as "const R3dRsClip *", c_key as "const char*"] -> f32 as "float" {
                    return (*self)->MetadataItemAsFloat(c_key);
                });
                Some(Value::Float(value))
            }
            _ => None,
        }
    }

    /// Iterate over all items of the clip's metadata table.
    pub fn metadata_iter(&self) -> ClipMetadataIterator<'_> {
        let count = cpp!(unsafe [self as "const R3dRsClip *"] -> usize as "size_t" { return (*self)->MetadataCount(); });
        ClipMetadataIterator {
            inner: self,
            index: 0,
            count,
        }
    }

    /// Asynchronous IMU samples of V-RAPTOR recordings, see the RMD_FRAME_TIMESTAMP
    /// metadata and [`metadata_for_frame`](Clip::metadata_for_frame) as well.
    /// Returns an empty `Vec` if the range is empty or out of bounds.
    pub fn imu_samples<T: std::ops::RangeBounds<u64>>(&self, range: T) -> Vec<IMUSample> {
        let sample_count = cpp!(unsafe [self as "const R3dRsClip *"] -> usize as "size_t" { return (*self)->IMUSampleCount(); }) as u64;

        let start = match range.start_bound() {
            std::ops::Bound::Included(&start) => start,
            std::ops::Bound::Excluded(&start) => start.saturating_add(1),
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&end) => end.saturating_add(1),
            std::ops::Bound::Excluded(&end) => end,
            std::ops::Bound::Unbounded => sample_count,
        };
        if start >= end || end > sample_count {
            return Vec::new();
        }
        let (start, count) = (start as usize, (end - start) as usize);
        let mut samples = Vec::with_capacity(count);
        let ptr = samples.as_mut_ptr();
        let success: bool = cpp!(unsafe [self as "const R3dRsClip *", ptr as "R3DSDK::IMUSample *", start as "size_t", count as "size_t"] -> bool as "bool" {
            return (*self)->DecodeIMUSamples(ptr, start, count);
        });
        if success {
            // SAFETY: the SDK filled all `count` samples.
            unsafe { samples.set_len(count); }
        }
        samples
    }
}

cpp_class! {
    /// The decode job of [`Clip::decode_video_frame`]. It only points at the
    /// output buffer and settings, so it is private to the synchronous decode.
    pub(crate) unsafe struct VideoDecodeJob as "R3DSDK::VideoDecodeJob"
}
impl VideoDecodeJob {
    fn set_mode(&mut self, v: VideoDecodeMode) {
        let v: i32 = v as i32;
        cpp!(unsafe [self as "R3DSDK::VideoDecodeJob *", v as "int"] { (*self).Mode = (R3DSDK::VideoDecodeMode)v; })
    }
    fn set_pixel_type(&mut self, v: VideoPixelType) {
        let v: i32 = v as i32;
        cpp!(unsafe [self as "R3DSDK::VideoDecodeJob *", v as "int"] { (*self).PixelType = (R3DSDK::VideoPixelType)v; })
    }
    fn set_output_buffer(&mut self, buf: *mut c_void, size: usize) {
        cpp!(unsafe [self as "R3DSDK::VideoDecodeJob *", buf as "void *", size as "size_t"] {
            (*self).OutputBuffer = buf;
            (*self).OutputBufferSize = size;
        })
    }
    fn set_image_processing(&mut self, v: &ImageProcessingSettings) {
        let ptr = v as *const _;
        cpp!(unsafe [self as "R3DSDK::VideoDecodeJob *", ptr as "R3DSDK::ImageProcessingSettings *"] {
            (*self).ImageProcessing = ptr;
        })
    }
    fn set_hdr_processing(&mut self, v: &HdrProcessingSettings) {
        let ptr = v as *const _;
        cpp!(unsafe [self as "R3DSDK::VideoDecodeJob *", ptr as "R3DSDK::HdrProcessingSettings *"] {
            (*self).HdrProcessing = ptr;
        })
    }
}

/// The integer downscale divisor for a [`VideoDecodeMode`]: `1` for full-res,
/// `2`/`4`/`8`/`16` for the reduced-res modes. The single source for CPU
/// buffer sizing here and GPU output sizing in `gpu.rs`.
#[inline]
pub(crate) const fn mode_divisor(mode: VideoDecodeMode) -> usize {
    match mode {
        VideoDecodeMode::FullResPremium   => 1,
        VideoDecodeMode::HalfResPremium   => 2,
        VideoDecodeMode::HalfResGood      => 2,
        VideoDecodeMode::QuarterResGood   => 4,
        VideoDecodeMode::EightResGood     => 8,
        VideoDecodeMode::SixteenthResGood => 16,
    }
}
/// Return the decoded image dimensions *after* the mode scaling (integer
/// division, no rounding).
pub(crate) const fn scaled_dims(src_w: usize, src_h: usize, mode: VideoDecodeMode) -> (usize, usize) {
    let div = mode_divisor(mode);
    (src_w / div, src_h / div)
}

/// Bytes per pixel across all channels, per the RED SDK docs in the sample.
/// Planar RGB stores one plane of samples per channel, so a w * h frame is
/// w * h * bytes_per_pixel for every format. The single source for CPU
/// buffer sizing here and GPU output sizing in `gpu.rs`.
pub(crate) const fn bytes_per_pixel(pt: VideoPixelType) -> usize {
    match pt {
        VideoPixelType::Rgb16bitInterleaved     => 6, // 3 * 16-bit
        VideoPixelType::RgbHalfFloatInterleaved => 6, // 3 * 16-bit half
        VideoPixelType::RgbHalfFloatAcesInt     => 6, // 3 * 16-bit half
        VideoPixelType::Rgb16bitPlanar          => 6, // 3 planes * 16-bit
        VideoPixelType::Bgr8bitInterleaved      => 3, // 3 * 8-bit
        VideoPixelType::Bgra8bitInterleaved     => 4, // 4 * 8-bit
        VideoPixelType::Dpx10bitMethodB         => 4, // packed; SDK uses 4B/px
    }
}

mod sealed {
    pub trait Sealed {}
}

/// Element types an [`AlignedBuffer`] can be viewed as: plain numbers, valid
/// for any bit pattern.
pub trait BufferElement: sealed::Sealed + Copy {}
macro_rules! buffer_element {
    ($($t:ty),*) => { $( impl sealed::Sealed for $t {} impl BufferElement for $t {} )* };
}
buffer_element!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);

/// A zero-initialized heap buffer with a caller-chosen alignment, as the SDK
/// requires for its output buffers.
pub struct AlignedBuffer {
    pub(crate) ptr: *mut c_void,
    pub(crate) layout: std::alloc::Layout
}
// SAFETY: `Send` only — a single-owner heap allocation handed to the SDK once
// per submission then dropped/recycled. Concurrent `&self` would alias the
// buffer the SDK writes into, so not `Sync` (pinned negative in
// `tests/send_sync_surface.rs`).
unsafe impl Send for AlignedBuffer {}

impl AlignedBuffer {
    /// Allocate `size` zeroed bytes aligned to `alignment` (a power of two).
    ///
    /// Fails for `size == 0`: a zero-sized allocation is undefined behaviour in
    /// the global allocator, and an empty buffer would hand the SDK a pointer
    /// it cannot write through. A size of 0 from the SDK (e.g. the audio block
    /// size of a clip without audio) means there is nothing to decode.
    pub fn new(size: usize, alignment: usize) -> RedResult<Self> {
        if size == 0 {
            return Err(RedError::Other("cannot allocate a zero-sized AlignedBuffer".into()));
        }
        let layout = std::alloc::Layout::from_size_align(size, alignment)?;
        // SAFETY: `layout` has a non-zero size.
        let ptr = unsafe { std::alloc::alloc_zeroed(layout) } as *mut c_void;
        if ptr.is_null() {
            Err(RedError::Other(format!("Failed to allocate {size} aligned to {alignment} bytes")))
        } else {
            Ok(Self { ptr, layout })
        }
    }
    /// Size of the buffer in bytes.
    pub fn len(&self) -> usize {
        self.layout.size()
    }
    /// Always `false`: zero-sized buffers cannot be created.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Alignment of the buffer in bytes.
    pub fn align(&self) -> usize {
        self.layout.align()
    }
    /// Pointer to the start of the buffer.
    pub fn as_ptr(&self) -> *const c_void {
        self.ptr
    }
    /// Mutable pointer to the start of the buffer, e.g. to hand to the SDK.
    pub fn as_mut_ptr(&mut self) -> *mut c_void {
        self.ptr
    }
    /// View the buffer as a slice of `T`. Trailing bytes that do not fill a
    /// whole `T` are left out.
    ///
    /// # Panics
    /// If the buffer is not aligned for `T`.
    pub fn as_slice<T: BufferElement>(&self) -> &[T] {
        assert!(self.ptr.cast::<T>().is_aligned(), "AlignedBuffer is not aligned for {}", std::any::type_name::<T>());
        let len = self.len() / std::mem::size_of::<T>();
        // SAFETY: the allocation is `len()` initialized bytes, aligned for `T`,
        // and every bit pattern is a valid `T`.
        unsafe { std::slice::from_raw_parts(self.ptr as *const T, len) }
    }
    /// View the buffer as a mutable slice of `T`. Trailing bytes that do not
    /// fill a whole `T` are left out.
    ///
    /// # Panics
    /// If the buffer is not aligned for `T`.
    pub fn as_mut_slice<T: BufferElement>(&mut self) -> &mut [T] {
        assert!(self.ptr.cast::<T>().is_aligned(), "AlignedBuffer is not aligned for {}", std::any::type_name::<T>());
        let len = self.len() / std::mem::size_of::<T>();
        // SAFETY: as in `as_slice`, and `&mut self` makes the access exclusive.
        unsafe { std::slice::from_raw_parts_mut(self.ptr as *mut T, len) }
    }
}
impl std::fmt::Debug for AlignedBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlignedBuffer")
            .field("ptr", &self.ptr)
            .field("len", &self.len())
            .field("align", &self.align())
            .finish()
    }
}
impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        // SAFETY: `ptr` was allocated in `new` with `layout`.
        unsafe { std::alloc::dealloc(self.ptr as *mut u8, self.layout); }
    }
}
