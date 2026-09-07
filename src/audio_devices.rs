//! Audio input device enumeration using `AVFoundation`.
//!
//! This module provides access to available microphone devices on macOS.

use std::ffi::c_void;

/// Represents an audio input device (microphone).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioInputDevice {
    /// The unique device ID used with `SCStreamConfiguration::with_microphone_capture_device_id`
    pub id: String,
    /// Human-readable device name
    pub name: String,
    /// Whether this is the system default audio input device
    pub is_default: bool,
}

impl AudioInputDevice {
    /// List all available audio input devices.
    ///
    /// The whole list is read from a single device-discovery pass, so a
    /// microphone plugged in or unplugged during the call cannot produce a
    /// half-updated result (a stale count paired with fresh names, or an entry
    /// whose `is_default` disagrees with [`default_device`](Self::default_device)).
    ///
    /// # Caching
    ///
    /// **Not cached.** Each call walks Apple's audio device list and copies the
    /// per-device strings across the FFI boundary. The cost is small in
    /// absolute terms (microseconds) but is **non-zero on every call**. Code
    /// that repeatedly needs the device list (e.g. inside a UI render loop or
    /// per-frame decision) should cache the result and re-list only when the
    /// user signals a possible device change (e.g. on a settings-pane open or
    /// an `AVAudioRouteChangeNotification`).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use screencapturekit::audio_devices::AudioInputDevice;
    ///
    /// let devices = AudioInputDevice::list();
    /// for device in &devices {
    ///     println!("{}: {} {}", device.id, device.name,
    ///         if device.is_default { "(default)" } else { "" });
    /// }
    /// ```
    #[must_use]
    pub fn list() -> Vec<Self> {
        Snapshot::take().devices()
    }

    /// Get the default audio input device, if any.
    ///
    /// Reads from the same single-pass snapshot as [`list`](Self::list), so the
    /// returned device is guaranteed to be one of the listed devices rather
    /// than an entry that appeared or vanished between two queries.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use screencapturekit::audio_devices::AudioInputDevice;
    ///
    /// if let Some(device) = AudioInputDevice::default_device() {
    ///     println!("Default microphone: {}", device.name);
    /// }
    /// ```
    #[must_use]
    pub fn default_device() -> Option<Self> {
        let snapshot = Snapshot::take();
        let index = isize::try_from(snapshot.default_index()?).ok()?;
        snapshot.device_at(index)
    }

    /// List all devices and identify the default one in a single pass.
    ///
    /// Returns the device list plus the index of the default device within it.
    /// Prefer this over calling [`list`](Self::list) and
    /// [`default_device`](Self::default_device) separately: it halves the work
    /// and rules out the two results disagreeing.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use screencapturekit::audio_devices::AudioInputDevice;
    ///
    /// let (devices, default_index) = AudioInputDevice::list_with_default();
    /// if let Some(default) = default_index.and_then(|i| devices.get(i)) {
    ///     println!("Default: {}", default.name);
    /// }
    /// ```
    #[must_use]
    pub fn list_with_default() -> (Vec<Self>, Option<usize>) {
        let snapshot = Snapshot::take();
        let devices = snapshot.devices();
        // Derive the index from the returned list rather than reusing the
        // snapshot's index: `devices()` drops entries whose id or name is
        // unreadable, so the snapshot index addresses the unfiltered array and
        // would designate the wrong device, or fall outside the list entirely.
        let default_index = devices.iter().position(|device| device.is_default);
        (devices, default_index)
    }
}

/// RAII wrapper around a Swift-side frozen device list.
struct Snapshot {
    ptr: *const c_void,
}

impl Snapshot {
    fn take() -> Self {
        Self {
            ptr: unsafe { crate::ffi::sc_audio_input_devices_snapshot_create() },
        }
    }

    fn count(&self) -> usize {
        if self.ptr.is_null() {
            return 0;
        }
        let count = unsafe { crate::ffi::sc_audio_input_devices_snapshot_count(self.ptr) };
        usize::try_from(count).unwrap_or(0)
    }

    fn default_index(&self) -> Option<usize> {
        if self.ptr.is_null() {
            return None;
        }
        let index = unsafe { crate::ffi::sc_audio_input_devices_snapshot_default_index(self.ptr) };
        usize::try_from(index).ok().filter(|i| *i < self.count())
    }

    fn device_at(&self, index: isize) -> Option<AudioInputDevice> {
        let id = unsafe {
            crate::utils::ffi_string::ffi_string_owned(|| {
                crate::ffi::sc_audio_input_devices_snapshot_id_owned(self.ptr, index)
            })
        }?;
        let name = unsafe {
            crate::utils::ffi_string::ffi_string_owned(|| {
                crate::ffi::sc_audio_input_devices_snapshot_name_owned(self.ptr, index)
            })
        }?;
        let is_default =
            unsafe { crate::ffi::sc_audio_input_devices_snapshot_is_default(self.ptr, index) };
        Some(AudioInputDevice {
            id,
            name,
            is_default,
        })
    }

    fn devices(&self) -> Vec<AudioInputDevice> {
        let count = self.count();
        (0..count)
            .filter_map(|i| isize::try_from(i).ok().and_then(|i| self.device_at(i)))
            .collect()
    }
}

impl Drop for Snapshot {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { crate::ffi::sc_audio_input_devices_snapshot_release(self.ptr) };
        }
    }
}
