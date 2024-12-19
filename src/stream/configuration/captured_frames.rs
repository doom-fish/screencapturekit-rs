use core_foundation::error::CFError;
use core_media_rs::cm_time::CMTime;
use objc::{sel, sel_impl};

use super::internal::SCStreamConfiguration;
use crate::utils::objc::{get_property, set_property};

impl SCStreamConfiguration {
    /// Sets the queueDepth of this [`SCStreamConfiguration`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn set_queue_depth(mut self, queue_depth: u32) -> Result<Self, CFError> {
        set_property(&mut self, sel!(setQueueDepth:), queue_depth)?;
        Ok(self)
    }
    pub fn get_queue_depth(&self) -> u32 {
        get_property(self, sel!(queueDepth))
    }

    /// Sets the minimumFrameInterval of this [`SCStreamConfiguration`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn set_minimum_frame_interval(mut self, cm_time: &CMTime) -> Result<Self, CFError> {
        set_property(&mut self, sel!(setMinimumFrameInterval:), cm_time)?;
        Ok(self)
    }
    pub fn get_minimum_frame_interval(&self) -> CMTime {
        get_property(self, sel!(minimumFrameInterval))
    }
}

#[cfg(test)]
mod sc_stream_configuration_test {
    use core_foundation::error::CFError;
    use core_media_rs::cm_time::CMTime;

    use super::SCStreamConfiguration;

    #[test]
    fn test_setters_and_getters() -> Result<(), CFError> {
        let cm_time = CMTime {
            value: 4,
            timescale: 1,
            flags: 1,
            epoch: 1,
        };
        let queue_depth = 10;
        let config = SCStreamConfiguration::new()
            .set_queue_depth(queue_depth)?
            .set_minimum_frame_interval(&cm_time)?;

        assert!(config.get_queue_depth() == queue_depth);

        let acquired_cm_time = config.get_minimum_frame_interval();
        assert!(acquired_cm_time.value == cm_time.value);
        assert!(acquired_cm_time.timescale == cm_time.timescale);
        assert!(acquired_cm_time.flags == cm_time.flags);
        assert!(acquired_cm_time.epoch == cm_time.epoch);
        Ok(())
    }
}
