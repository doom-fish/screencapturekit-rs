//! Audio buffer types

use std::fmt;

pub struct AudioBuffer {
    pub number_channels: u32,
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
            self.number_channels,
            self.data_bytes_size
        )
    }
}

impl AudioBuffer {
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

    pub fn get_data_byte_size(&self) -> usize {
        self.data_bytes_size as usize
    }
}

/// Reference to an audio buffer with convenience methods
pub struct AudioBufferRef<'a> {
    buffer: &'a AudioBuffer,
}

impl AudioBufferRef<'_> {
    pub fn get_data_byte_size(&self) -> usize {
        self.buffer.get_data_byte_size()
    }

    pub fn data(&self) -> &[u8] {
        self.buffer.data()
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

pub struct AudioBufferList {
    pub(crate) inner: AudioBufferListRaw,
}

impl AudioBufferList {
    pub fn num_buffers(&self) -> usize {
        self.inner.num_buffers as usize
    }

    pub fn get_number_buffers(&self) -> usize {
        self.num_buffers()
    }

    pub fn get(&self, index: usize) -> Option<&AudioBuffer> {
        if index >= self.num_buffers() {
            None
        } else {
            unsafe {
                Some(&*self.inner.buffers_ptr.add(index))
            }
        }
    }

    pub fn get_buffer(&self, index: usize) -> Option<AudioBufferRef<'_>> {
        self.get(index).map(|buffer| AudioBufferRef { buffer })
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut AudioBuffer> {
        if index >= self.num_buffers() {
            None
        } else {
            unsafe {
                Some(&mut *self.inner.buffers_ptr.add(index))
            }
        }
    }

    pub fn iter(&self) -> AudioBufferListIter<'_> {
        AudioBufferListIter {
            list: self,
            index: 0,
        }
    }
}

impl Drop for AudioBufferList {
    fn drop(&mut self) {
        if !self.inner.buffers_ptr.is_null() {
            unsafe {
                Vec::from_raw_parts(
                    self.inner.buffers_ptr,
                    self.inner.buffers_len,
                    self.inner.buffers_len,
                );
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

pub struct AudioBufferListIter<'a> {
    list: &'a AudioBufferList,
    index: usize,
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

