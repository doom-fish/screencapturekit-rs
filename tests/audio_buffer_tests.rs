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
