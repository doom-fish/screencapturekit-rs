use super::internal::SCStreamConfiguration;
use crate::cm::CMTime;

#[cfg(feature = "macos_14_0")]
use super::SCCaptureResolutionType;

/// Smallest `queueDepth` `ScreenCaptureKit` accepts.
pub const MIN_QUEUE_DEPTH: u32 = 3;
/// Largest `queueDepth` `ScreenCaptureKit` accepts.
pub const MAX_QUEUE_DEPTH: u32 = 8;

impl SCStreamConfiguration {
    /// Set the queue depth for frame buffering.
    ///
    /// `ScreenCaptureKit` documents a valid range of
    /// [`MIN_QUEUE_DEPTH`] to [`MAX_QUEUE_DEPTH`] (3–8). Values outside that
    /// range are **clamped** rather than forwarded: `SCStream` rejects an
    /// out-of-range depth when capture starts, and it does so with a generic
    /// `SCStreamErrorFailedToStart`, which is far harder to diagnose than a
    /// silently clamped buffer count. Read the value back with
    /// [`queue_depth`](Self::queue_depth) to see what was applied.
    ///
    /// Larger depths absorb more downstream jitter at the cost of latency and
    /// memory (each slot holds a full frame); smaller depths minimise latency
    /// but drop frames sooner when the consumer stalls.
    pub fn set_queue_depth(&mut self, queue_depth: u32) -> &mut Self {
        let clamped = queue_depth.clamp(MIN_QUEUE_DEPTH, MAX_QUEUE_DEPTH);
        // `clamped` is 3..=8, so the isize cast can never wrap.
        #[allow(clippy::cast_possible_wrap)]
        unsafe {
            crate::ffi::sc_stream_configuration_set_queue_depth(self.as_ptr(), clamped as isize);
        }
        self
    }

    /// Set the queue depth (builder pattern)
    ///
    /// See [`set_queue_depth`](Self::set_queue_depth) for the clamping rules.
    #[must_use]
    pub fn with_queue_depth(mut self, queue_depth: u32) -> Self {
        self.set_queue_depth(queue_depth);
        self
    }

    /// Get the configured queue depth.
    pub fn queue_depth(&self) -> u32 {
        let raw = unsafe { crate::ffi::sc_stream_configuration_get_queue_depth(self.as_ptr()) };
        u32::try_from(raw).unwrap_or(0)
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
    /// Returns 0 if the frame interval is zero or invalid (i.e. the stream is
    /// uncapped — see [`set_fps`](Self::set_fps)).
    #[allow(clippy::cast_possible_truncation)]
    pub fn fps(&self) -> u32 {
        let cm_time = self.minimum_frame_interval();
        if cm_time.value == 0 || cm_time.timescale == 0 {
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
    /// Passing `0` sets the interval to `kCMTimeZero` (`0/1`), which is
    /// `ScreenCaptureKit`'s "no minimum interval — deliver frames as fast as
    /// the source produces them" value. The naive `1/0` this used to build was
    /// a zero-timescale `CMTime`: still flagged valid, but degenerate, so
    /// `CMTimeGetSeconds` divides by zero and the stream's frame pacing is
    /// undefined.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::stream::configuration::SCStreamConfiguration;
    ///
    /// let config = SCStreamConfiguration::new()
    ///     .with_fps(60);
    /// ```
    pub fn set_fps(&mut self, fps: u32) -> &mut Self {
        let cm_time = if fps == 0 {
            CMTime::new(0, 1)
        } else {
            #[allow(clippy::cast_possible_wrap)]
            CMTime::new(1, fps as i32)
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
    /// ```no_run
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
    pub fn with_capture_resolution_type(
        mut self,
        resolution_type: SCCaptureResolutionType,
    ) -> Self {
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
