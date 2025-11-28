//! `CMBlockBuffer` tests

use screencapturekit::cm::CMBlockBuffer;

#[test]
fn test_block_buffer_from_raw_null() {
    let buffer = CMBlockBuffer::from_raw(std::ptr::null_mut());
    assert!(buffer.is_none());
}

#[test]
fn test_block_buffer_equality() {
    fn assert_eq_impl<T: PartialEq + Eq>() {}
    assert_eq_impl::<CMBlockBuffer>();
}

#[test]
fn test_block_buffer_hash() {
    fn assert_hash_impl<T: std::hash::Hash>() {}
    assert_hash_impl::<CMBlockBuffer>();
}

#[test]
fn test_block_buffer_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<CMBlockBuffer>();
    assert_sync::<CMBlockBuffer>();
}
