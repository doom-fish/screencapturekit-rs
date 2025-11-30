//! Audio input device enumeration using `AVFoundation`.
//!
//! This module provides access to available microphone devices on macOS.

use crate::utils::ffi_string::{ffi_string_from_buffer, SMALL_BUFFER_SIZE};

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
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    pub fn list() -> Vec<Self> {
        let count = unsafe { crate::ffi::sc_audio_get_input_device_count() };
        let mut devices = Vec::with_capacity(count as usize);

        for i in 0..count {
            let id = unsafe {
                ffi_string_from_buffer(SMALL_BUFFER_SIZE, |buf, len| {
                    crate::ffi::sc_audio_get_input_device_id(i, buf, len)
                })
            };
            let name = unsafe {
                ffi_string_from_buffer(SMALL_BUFFER_SIZE, |buf, len| {
                    crate::ffi::sc_audio_get_input_device_name(i, buf, len)
                })
            };
            let is_default = unsafe { crate::ffi::sc_audio_is_default_input_device(i) };

            if let (Some(id), Some(name)) = (id, name) {
                devices.push(Self {
                    id,
                    name,
                    is_default,
                });
            }
        }

        devices
    }

    /// Get the default audio input device, if any.
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
    pub fn default_device() -> Option<Self> {
        let id = unsafe {
            ffi_string_from_buffer(SMALL_BUFFER_SIZE, |buf, len| {
                crate::ffi::sc_audio_get_default_input_device_id(buf, len)
            })
        };
        let name = unsafe {
            ffi_string_from_buffer(SMALL_BUFFER_SIZE, |buf, len| {
                crate::ffi::sc_audio_get_default_input_device_name(buf, len)
            })
        };

        match (id, name) {
            (Some(id), Some(name)) => Some(Self {
                id,
                name,
                is_default: true,
            }),
            _ => None,
        }
    }
}
