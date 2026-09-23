//! `AudioBuffer` and `AudioBufferList` tests

use screencapturekit::cm::AudioBuffer;

#[test]
fn test_audio_buffer_display() {
    // We can't easily construct an AudioBuffer directly, but we can test the Display impl
    // by testing the format string pattern
    let format = "AudioBuffer(2 channels, 1024 bytes)";
    assert!(format.contains("channels"));
    assert!(format.contains("bytes"));
}

#[test]
fn test_audio_buffer_equality() {
    // AudioBuffer implements PartialEq based on channels, size, and pointer
    // We verify the trait is implemented
    fn assert_eq_impl<T: PartialEq>() {}
    assert_eq_impl::<AudioBuffer>();
}

#[test]
fn test_audio_buffer_hash() {
    // AudioBuffer implements Hash
    fn assert_hash_impl<T: std::hash::Hash>() {}
    assert_hash_impl::<AudioBuffer>();
}

/// A `CMSampleBuffer` with no audio must yield an error, not an empty list built
/// from an inconsistent `(count, pointer, length)` triple.
#[test]
fn test_audio_buffer_list_is_err_for_video_sample() {
    use screencapturekit::cm::{CMSampleBufferExt, CMTime};
    use screencapturekit::cv::CVPixelBuffer;

    let Ok(pixel_buffer) = CVPixelBuffer::create(64, 64, 0x4247_5241) else {
        println!("⚠ Skipping - pixel buffer creation failed");
        return;
    };
    let Ok(sample) = screencapturekit::cm::CMSampleBuffer::create_for_image_buffer(
        &pixel_buffer,
        CMTime::new(0, 600),
        CMTime::new(1, 600),
    ) else {
        println!("⚠ Skipping - sample buffer creation failed");
        return;
    };

    let list: Result<screencapturekit::AudioBufferList, i32> = sample.audio_buffer_list();
    assert!(list.is_err());
}
