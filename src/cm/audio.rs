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
/// Access the raw bytes via [`data()`](Self::data).
///
/// The fields are private: they are populated by the Swift bridge and are
/// load-bearing for the `data_ptr`/`data_bytes_size` pair used to build a
/// slice. Letting callers assign them would make [`data()`](Self::data)
/// construct an out-of-bounds slice from safe code.
#[repr(C)]
pub struct AudioBuffer {
    number_channels: u32,
    data_bytes_size: u32,
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
    /// Number of audio channels interleaved in this buffer.
    #[must_use]
    pub const fn number_channels(&self) -> u32 {
        self.number_channels
    }

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

    /// Get the size of the data in bytes
    pub fn data_byte_size(&self) -> usize {
        self.data_bytes_size as usize
    }
}

/// Reference to an audio buffer with convenience methods
pub struct AudioBufferRef<'a> {
    buffer: &'a AudioBuffer,
}

impl<'a> AudioBufferRef<'a> {
    /// Get the size of the data in bytes
    pub fn data_byte_size(&self) -> usize {
        self.buffer.data_byte_size()
    }

    /// Number of audio channels interleaved in the wrapped buffer.
    #[must_use]
    pub const fn number_channels(&self) -> u32 {
        self.buffer.number_channels
    }

    /// Get the raw audio data as a byte slice
    ///
    /// The returned slice is tied to the lifetime `'a` of the wrapped buffer
    /// reference rather than to this `AudioBufferRef`, because the underlying
    /// block-buffer memory lives at least as long as the borrowed
    /// [`AudioBuffer`] (and the [`AudioBufferList`] that owns it).
    pub fn data(&self) -> &'a [u8] {
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

/// Raw view of the buffer array handed over by the Swift bridge.
///
/// Every field is crate-private and only ever written by
/// `AudioBufferList::from_bridge`, which rejects inconsistent values. The
/// type stays public because it is named in the module's re-export list.
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
///
/// Whole descriptors are intentionally immutable. Allowing two independent
/// lists to yield `&mut AudioBuffer` would let safe code swap descriptors
/// between different backing block buffers:
///
/// ```compile_fail
/// use screencapturekit::cm::AudioBufferList;
///
/// fn swap_descriptors(mut first: AudioBufferList, mut second: AudioBufferList) {
///     std::mem::swap(first.get_mut(0).unwrap(), second.get_mut(0).unwrap());
/// }
/// ```
pub struct AudioBufferList {
    pub(crate) inner: AudioBufferListRaw,
    /// Block buffer that owns the audio data - must be kept alive
    pub(crate) block_buffer_ptr: *mut std::ffi::c_void,
}

impl AudioBufferList {
    /// Adopt the buffer array produced by `cm_sample_buffer_get_audio_buffer_list`.
    ///
    /// Returns `None` — after releasing anything the bridge already handed
    /// over — when the reported shape is not self-consistent. Every later
    /// accessor builds slices from these numbers, so validating once here is
    /// what makes [`AudioBuffer::data`] safe.
    ///
    /// # Safety
    ///
    /// `buffers_ptr` must be either null or a Swift-allocated array of
    /// exactly `buffers_len` [`AudioBuffer`]s, and `block_buffer_ptr` must be
    /// either null or a `+1`-retained `CMBlockBuffer` that owns the sample
    /// memory the buffers point into.
    pub(crate) unsafe fn from_bridge(
        num_buffers: u32,
        buffers_ptr: *mut AudioBuffer,
        buffers_len: usize,
        block_buffer_ptr: *mut std::ffi::c_void,
    ) -> Option<Self> {
        let consistent = num_buffers != 0
            && !buffers_ptr.is_null()
            && usize::try_from(num_buffers).is_ok_and(|n| n == buffers_len);

        if !consistent {
            unsafe { Self::release_bridge_allocations(buffers_ptr, buffers_len, block_buffer_ptr) };
            return None;
        }

        Some(Self {
            inner: AudioBufferListRaw {
                num_buffers,
                buffers_ptr,
                buffers_len,
            },
            block_buffer_ptr,
        })
    }

    /// Free the buffers array allocated in Swift via
    /// `UnsafeMutablePointer.allocate()` and drop the retained block buffer.
    ///
    /// The array is returned to the Swift allocator that created it.
    unsafe fn release_bridge_allocations(
        buffers_ptr: *mut AudioBuffer,
        buffers_len: usize,
        block_buffer_ptr: *mut std::ffi::c_void,
    ) {
        if !buffers_ptr.is_null() && buffers_len != 0 {
            unsafe { ffi::cm_audio_buffer_bridge_array_free(buffers_ptr.cast()) };
        }
        if !block_buffer_ptr.is_null() {
            unsafe {
                ffi::cm_block_buffer_release(block_buffer_ptr);
            }
        }
    }

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

    /// Get the raw audio data for one buffer as a mutable byte slice.
    ///
    /// The returned slice is tied to this list, so safe code cannot move or
    /// swap the descriptor away from the block buffer that owns its bytes.
    ///
    /// # Safety
    ///
    /// The bytes are also visible through the source `CMSampleBuffer`, other
    /// lists created from that sample, and framework-internal references. For
    /// the returned slice's entire lifetime, the caller must ensure no other
    /// reader or writer can access the same block-buffer range.
    pub unsafe fn data_mut(&mut self, index: usize) -> Option<&mut [u8]> {
        if index >= self.num_buffers() {
            None
        } else {
            let buffer = unsafe { &*self.inner.buffers_ptr.add(index) };
            if buffer.data_ptr.is_null() || buffer.data_bytes_size == 0 {
                Some(&mut [])
            } else {
                Some(unsafe {
                    std::slice::from_raw_parts_mut(
                        buffer.data_ptr.cast::<u8>(),
                        buffer.data_bytes_size as usize,
                    )
                })
            }
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
        unsafe {
            Self::release_bridge_allocations(
                self.inner.buffers_ptr,
                self.inner.buffers_len,
                self.block_buffer_ptr,
            );
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
            .field(
                "remaining",
                &(self.list.num_buffers().saturating_sub(self.index)),
            )
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
