//! Audio buffer types for captured audio samples
//!
//! This module provides types for accessing audio data from captured samples.
//!
//! ## Main Types
//!
//! - [`AudioBuffer`] - Single audio buffer containing sample data
//! - [`AudioBufferList`] - Collection of audio buffers (typically one per channel)
//! - [`AudioBufferRef`] - Reference to an audio buffer with convenience methods

use super::ffi;
use std::fmt;

/// Raw audio buffer containing sample data
///
/// An `AudioBuffer` represents a single channel or interleaved audio data.
/// Access the raw bytes via [`data()`](Self::data) or [`data_mut()`](Self::data_mut).
#[repr(C)]
pub struct AudioBuffer {
    /// Number of audio channels in this buffer
    pub number_channels: u32,
    /// Size of the audio data in bytes
    pub data_bytes_size: u32,
    data_ptr: *mut std::ffi::c_void,
}

impl PartialEq for AudioBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.number_channels == other.number_channels
            && self.data_bytes_size == other.data_bytes_size
            && self.data_ptr == other.data_ptr
    }
}

impl Eq for AudioBuffer {}

impl std::hash::Hash for AudioBuffer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.number_channels.hash(state);
        self.data_bytes_size.hash(state);
        self.data_ptr.hash(state);
    }
}

impl fmt::Display for AudioBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AudioBuffer({} channels, {} bytes)",
            self.number_channels, self.data_bytes_size
        )
    }
}

impl AudioBuffer {
    /// Get the raw audio data as a byte slice
    pub fn data(&self) -> &[u8] {
        if self.data_ptr.is_null() || self.data_bytes_size == 0 {
            &[]
        } else {
            unsafe {
                std::slice::from_raw_parts(
                    self.data_ptr as *const u8,
                    self.data_bytes_size as usize,
                )
            }
        }
    }

    /// Get the raw audio data as a mutable byte slice
    pub fn data_mut(&mut self) -> &mut [u8] {
        if self.data_ptr.is_null() || self.data_bytes_size == 0 {
            &mut []
        } else {
            unsafe {
                std::slice::from_raw_parts_mut(
                    self.data_ptr.cast::<u8>(),
                    self.data_bytes_size as usize,
                )
            }
        }
    }

    /// Get the size of the data in bytes
    pub fn data_byte_size(&self) -> usize {
        self.data_bytes_size as usize
    }
}

/// Reference to an audio buffer with convenience methods
pub struct AudioBufferRef<'a> {
    buffer: &'a AudioBuffer,
}

impl AudioBufferRef<'_> {
    /// Get the size of the data in bytes
    pub fn data_byte_size(&self) -> usize {
        self.buffer.data_byte_size()
    }

    /// Get the raw audio data as a byte slice
    pub fn data(&self) -> &[u8] {
        self.buffer.data()
    }
}

impl std::fmt::Debug for AudioBufferRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioBufferRef")
            .field("channels", &self.buffer.number_channels)
            .field("data_bytes", &self.buffer.data_bytes_size)
            .finish()
    }
}

impl std::fmt::Debug for AudioBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioBuffer")
            .field("number_channels", &self.number_channels)
            .field("data_bytes_size", &self.data_bytes_size)
            .finish_non_exhaustive()
    }
}

/// List of audio buffers from an audio sample
#[repr(C)]
#[derive(Debug)]
pub struct AudioBufferListRaw {
    pub(crate) num_buffers: u32,
    pub(crate) buffers_ptr: *mut AudioBuffer,
    pub(crate) buffers_len: usize,
}

/// List of audio buffers from an audio sample
///
/// Contains one or more [`AudioBuffer`]s, typically one per audio channel.
/// Use [`iter()`](Self::iter) to iterate over the buffers.
pub struct AudioBufferList {
    pub(crate) inner: AudioBufferListRaw,
    /// Block buffer that owns the audio data - must be kept alive
    pub(crate) block_buffer_ptr: *mut std::ffi::c_void,
}

impl AudioBufferList {
    /// Get the number of buffers in the list
    pub fn num_buffers(&self) -> usize {
        self.inner.num_buffers as usize
    }

    /// Get a buffer by index
    pub fn get(&self, index: usize) -> Option<&AudioBuffer> {
        if index >= self.num_buffers() {
            None
        } else {
            unsafe { Some(&*self.inner.buffers_ptr.add(index)) }
        }
    }

    /// Get a buffer reference by index
    pub fn buffer(&self, index: usize) -> Option<AudioBufferRef<'_>> {
        self.get(index).map(|buffer| AudioBufferRef { buffer })
    }

    /// Get a mutable buffer by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut AudioBuffer> {
        if index >= self.num_buffers() {
            None
        } else {
            unsafe { Some(&mut *self.inner.buffers_ptr.add(index)) }
        }
    }

    /// Iterate over the audio buffers
    pub fn iter(&self) -> AudioBufferListIter<'_> {
        AudioBufferListIter {
            list: self,
            index: 0,
        }
    }
}

impl Drop for AudioBufferList {
    fn drop(&mut self) {
        // Free the buffers array allocated in Swift
        if !self.inner.buffers_ptr.is_null() {
            unsafe {
                Vec::from_raw_parts(
                    self.inner.buffers_ptr,
                    self.inner.buffers_len,
                    self.inner.buffers_len,
                );
            }
        }
        // Release the block buffer that owns the audio data
        if !self.block_buffer_ptr.is_null() {
            unsafe {
                ffi::cm_block_buffer_release(self.block_buffer_ptr);
            }
        }
    }
}

impl<'a> IntoIterator for &'a AudioBufferList {
    type Item = &'a AudioBuffer;
    type IntoIter = AudioBufferListIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl fmt::Display for AudioBufferList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AudioBufferList({} buffers)", self.num_buffers())
    }
}

impl fmt::Debug for AudioBufferList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioBufferList")
            .field("num_buffers", &self.num_buffers())
            .finish()
    }
}

/// Iterator over audio buffers in an [`AudioBufferList`]
pub struct AudioBufferListIter<'a> {
    list: &'a AudioBufferList,
    index: usize,
}

impl std::fmt::Debug for AudioBufferListIter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioBufferListIter")
            .field("total", &self.list.num_buffers())
            .field("remaining", &(self.list.num_buffers().saturating_sub(self.index)))
            .finish()
    }
}

impl<'a> Iterator for AudioBufferListIter<'a> {
    type Item = &'a AudioBuffer;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.list.num_buffers() {
            let buffer = self.list.get(self.index);
            self.index += 1;
            buffer
        } else {
            None
        }
    }
}
