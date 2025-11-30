use super::internal::SCStreamConfiguration;
use crate::cm::CMTime;

#[cfg(feature = "macos_14_0")]
use super::SCCaptureResolutionType;

impl SCStreamConfiguration {
    /// Set the queue depth for frame buffering
    pub fn set_queue_depth(&mut self, queue_depth: u32) -> &mut Self {
        // FFI expects isize; u32 may wrap on 32-bit platforms (acceptable)
        #[allow(clippy::cast_possible_wrap)]
        unsafe {
            crate::ffi::sc_stream_configuration_set_queue_depth(
                self.as_ptr(),
                queue_depth as isize,
            );
        }
        self
    }

    /// Set the queue depth (builder pattern)
    #[must_use]
    pub fn with_queue_depth(mut self, queue_depth: u32) -> Self {
        self.set_queue_depth(queue_depth);
        self
    }

    pub fn queue_depth(&self) -> u32 {
        // FFI returns isize but queue depth is always positive and fits in u32
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        unsafe {
            crate::ffi::sc_stream_configuration_get_queue_depth(self.as_ptr()) as u32
        }
    }

    /// Set the minimum frame interval
    pub fn set_minimum_frame_interval(&mut self, cm_time: &CMTime) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_minimum_frame_interval(
                self.as_ptr(),
                cm_time.value,
                cm_time.timescale,
                cm_time.flags,
                cm_time.epoch,
            );
        }
        self
    }

    /// Set the minimum frame interval (builder pattern)
    #[must_use]
    pub fn with_minimum_frame_interval(mut self, cm_time: &CMTime) -> Self {
        self.set_minimum_frame_interval(cm_time);
        self
    }

    pub fn minimum_frame_interval(&self) -> CMTime {
        unsafe {
            let mut value: i64 = 0;
            let mut timescale: i32 = 0;
            let mut flags: u32 = 0;
            let mut epoch: i64 = 0;

            crate::ffi::sc_stream_configuration_get_minimum_frame_interval(
                self.as_ptr(),
                &mut value,
                &mut timescale,
                &mut flags,
                &mut epoch,
            );

            CMTime {
                value,
                timescale,
                flags,
                epoch,
            }
        }
    }

    /// Get the target frame rate in frames per second
    ///
    /// Converts the minimum frame interval (`CMTime`) to FPS.
    /// Returns 0 if the frame interval is invalid.
    #[allow(clippy::cast_possible_truncation)]
    pub fn fps(&self) -> u32 {
        let cm_time = self.minimum_frame_interval();
        if cm_time.value == 0 {
            return 0;
        }
        #[allow(clippy::cast_sign_loss)]
        let fps = (i64::from(cm_time.timescale) / cm_time.value) as u32;
        fps
    }

    /// Set the target frame rate in frames per second
    ///
    /// This is a convenience method that creates the appropriate `CMTime` for the given FPS.
    /// For example, 60 FPS creates a frame interval of 1/60 second.
    ///
    /// # Arguments
    /// * `fps` - Target frames per second (e.g., 30, 60, 120)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use screencapturekit::stream::configuration::SCStreamConfiguration;
    ///
    /// let config = SCStreamConfiguration::new()
    ///     .with_fps(60);
    /// ```
    pub fn set_fps(&mut self, fps: u32) -> &mut Self {
        let cm_time = CMTime {
            value: 1,
            #[allow(clippy::cast_possible_wrap)]
            timescale: fps as i32,
            flags: 1, // kCMTimeFlags_Valid
            epoch: 0,
        };
        self.set_minimum_frame_interval(&cm_time)
    }

    /// Set the target frame rate (builder pattern)
    ///
    /// See [`set_fps`](Self::set_fps) for details.
    #[must_use]
    pub fn with_fps(mut self, fps: u32) -> Self {
        self.set_fps(fps);
        self
    }

    /// Set the capture resolution type (macOS 14.0+)
    ///
    /// Controls how the capture resolution is determined.
    ///
    /// # Arguments
    /// * `resolution_type` - The resolution strategy to use
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use screencapturekit::stream::configuration::{SCStreamConfiguration, SCCaptureResolutionType};
    ///
    /// let config = SCStreamConfiguration::new()
    ///     .with_capture_resolution_type(SCCaptureResolutionType::Best);
    /// ```
    #[cfg(feature = "macos_14_0")]
    pub fn set_capture_resolution_type(
        &mut self,
        resolution_type: SCCaptureResolutionType,
    ) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_capture_resolution_type(
                self.as_ptr(),
                resolution_type as i32,
            );
        }
        self
    }

    /// Set the capture resolution type (builder pattern, macOS 14.0+)
    #[cfg(feature = "macos_14_0")]
    #[must_use]
    pub fn with_capture_resolution_type(mut self, resolution_type: SCCaptureResolutionType) -> Self {
        self.set_capture_resolution_type(resolution_type);
        self
    }

    /// Get the capture resolution type (macOS 14.0+)
    #[cfg(feature = "macos_14_0")]
    pub fn capture_resolution_type(&self) -> SCCaptureResolutionType {
        let value = unsafe {
            crate::ffi::sc_stream_configuration_get_capture_resolution_type(self.as_ptr())
        };
        match value {
            1 => SCCaptureResolutionType::Best,
            2 => SCCaptureResolutionType::Nominal,
            _ => SCCaptureResolutionType::Automatic,
        }
    }
}
