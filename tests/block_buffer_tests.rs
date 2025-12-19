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

#[test]
fn test_block_buffer_debug_impl() {
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_debug::<CMBlockBuffer>();
}

#[test]
fn test_block_buffer_display_impl() {
    fn assert_display<T: std::fmt::Display>() {}
    assert_display::<CMBlockBuffer>();
}

#[test]
fn test_block_buffer_clone_impl() {
    fn assert_clone<T: Clone>() {}
    assert_clone::<CMBlockBuffer>();
}

#[test]
fn test_block_buffer_create_with_data() {
    let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
    let buffer = CMBlockBuffer::create(&data).expect("Failed to create buffer");
    assert_eq!(buffer.data_length(), 8);
    assert!(!buffer.is_empty());
}

#[test]
fn test_block_buffer_create_empty() {
    let buffer = CMBlockBuffer::create_empty().expect("Failed to create empty buffer");
    assert!(buffer.is_empty());
    assert_eq!(buffer.data_length(), 0);
}

#[test]
fn test_block_buffer_create_with_empty_slice() {
    let data: Vec<u8> = vec![];
    let buffer = CMBlockBuffer::create(&data).expect("Failed to create buffer from empty slice");
    assert!(buffer.is_empty());
}

#[test]
fn test_block_buffer_copy_data_bytes() {
    let data = vec![10u8, 20, 30, 40, 50];
    let buffer = CMBlockBuffer::create(&data).expect("Failed to create buffer");

    let copied = buffer.copy_data_bytes(0, 5);
    assert!(copied.is_some());
    assert_eq!(copied.unwrap(), data);
}

#[test]
fn test_block_buffer_copy_data_bytes_partial() {
    let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
    let buffer = CMBlockBuffer::create(&data).expect("Failed to create buffer");

    let copied = buffer.copy_data_bytes(2, 4);
    assert!(copied.is_some());
    assert_eq!(copied.unwrap(), vec![3, 4, 5, 6]);
}

#[test]
fn test_block_buffer_copy_data_bytes_zero_length() {
    let data = vec![1u8, 2, 3, 4];
    let buffer = CMBlockBuffer::create(&data).expect("Failed to create buffer");

    let copied = buffer.copy_data_bytes(0, 0);
    assert!(copied.is_some());
    assert!(copied.unwrap().is_empty());
}

#[test]
fn test_block_buffer_copy_data_bytes_into() {
    let data = vec![100u8, 101, 102, 103];
    let buffer = CMBlockBuffer::create(&data).expect("Failed to create buffer");

    let mut dest = [0u8; 4];
    let result = buffer.copy_data_bytes_into(0, &mut dest);
    assert!(result.is_ok());
    assert_eq!(dest, [100, 101, 102, 103]);
}

#[test]
fn test_block_buffer_copy_data_bytes_into_empty() {
    let data = vec![1u8, 2, 3, 4];
    let buffer = CMBlockBuffer::create(&data).expect("Failed to create buffer");

    let mut dest: [u8; 0] = [];
    let result = buffer.copy_data_bytes_into(0, &mut dest);
    assert!(result.is_ok());
}

#[test]
fn test_block_buffer_is_range_contiguous() {
    let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
    let buffer = CMBlockBuffer::create(&data).expect("Failed to create buffer");

    // Buffer created with create_with_data should be contiguous
    assert!(buffer.is_range_contiguous(0, 8));
    assert!(buffer.is_range_contiguous(0, 4));
    assert!(buffer.is_range_contiguous(4, 4));
}

#[test]
fn test_block_buffer_data_pointer() {
    let data = vec![42u8, 43, 44, 45];
    let buffer = CMBlockBuffer::create(&data).expect("Failed to create buffer");

    let result = buffer.data_pointer(0);
    assert!(result.is_some());
    let (ptr, len) = result.unwrap();
    assert!(!ptr.is_null());
    assert!(len >= 4);

    // Verify the data
    let slice = unsafe { std::slice::from_raw_parts(ptr, 4) };
    assert_eq!(slice, &[42, 43, 44, 45]);
}

#[test]
fn test_block_buffer_as_slice() {
    let data = vec![10u8, 20, 30, 40, 50];
    let buffer = CMBlockBuffer::create(&data).expect("Failed to create buffer");

    let slice = buffer.as_slice();
    assert!(slice.is_some());
    assert_eq!(slice.unwrap(), &data[..]);
}

#[test]
fn test_block_buffer_cursor() {
    use std::io::{Read, Seek, SeekFrom};

    let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
    let buffer = CMBlockBuffer::create(&data).expect("Failed to create buffer");

    let mut cursor = buffer.cursor().expect("Failed to create cursor");

    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf).unwrap();
    assert_eq!(buf, [1, 2, 3, 4]);

    cursor.seek(SeekFrom::Start(0)).unwrap();
    cursor.read_exact(&mut buf).unwrap();
    assert_eq!(buf, [1, 2, 3, 4]);
}

#[test]
fn test_block_buffer_cursor_ref() {
    use std::io::Read;

    let data = vec![100u8, 101, 102, 103];
    let buffer = CMBlockBuffer::create(&data).expect("Failed to create buffer");

    let mut cursor = buffer.cursor_ref().expect("Failed to create cursor_ref");

    let mut buf = [0u8; 2];
    cursor.read_exact(&mut buf).unwrap();
    assert_eq!(buf, [100, 101]);
}

#[test]
fn test_block_buffer_clone() {
    let data = vec![1u8, 2, 3, 4];
    let buffer = CMBlockBuffer::create(&data).expect("Failed to create buffer");
    let cloned = buffer.clone();

    assert_eq!(buffer.data_length(), cloned.data_length());
}

#[test]
fn test_block_buffer_debug_format() {
    let data = vec![1u8, 2, 3, 4];
    let buffer = CMBlockBuffer::create(&data).expect("Failed to create buffer");

    let debug_str = format!("{buffer:?}");
    assert!(debug_str.contains("CMBlockBuffer"));
    assert!(debug_str.contains("data_length"));
}

#[test]
fn test_block_buffer_display_format() {
    let data = vec![1u8, 2, 3, 4, 5];
    let buffer = CMBlockBuffer::create(&data).expect("Failed to create buffer");

    let display_str = format!("{buffer}");
    assert!(display_str.contains("5 bytes"));
}
