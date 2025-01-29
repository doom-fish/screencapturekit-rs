use core_foundation::error::CFError;
use objc::{sel, sel_impl};

use crate::utils::objc::{get_property, set_property};

use super::internal::SCStreamConfiguration;

impl SCStreamConfiguration {
    /// Sets the showsCursor of this [`SCStreamConfiguration`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn set_shows_cursor(mut self, shows_cursor: bool) -> Result<Self, CFError> {
        set_property(&mut self, sel!(setShowsCursor:), shows_cursor)?;
        Ok(self)
    }

    pub fn get_shows_cursor(&self) -> bool {
        get_property(self, sel!(showsCursor))
    }
}

#[cfg(test)]
mod sc_stream_configuration_test {
    use crate::stream::configuration::SCStreamConfiguration;

    #[test]
    fn test_setters_and_getters() {
        let config = SCStreamConfiguration::default();
        let config = config
            .set_shows_cursor(true)
            .expect("Failed to set showsCursor");
        assert!(config.get_shows_cursor());
    }
}
