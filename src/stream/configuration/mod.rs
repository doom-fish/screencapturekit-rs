mod internal;

pub mod audio;
pub mod dimensions;
pub mod colors;
pub mod pixel_format;

#[allow(clippy::module_name_repetitions)] 
pub use internal::SCStreamConfiguration;
impl SCStreamConfiguration {
    #[must_use]
    pub fn new() -> Self {
        Self::internal_init()
    }
}

impl Default for SCStreamConfiguration {
    fn default() -> Self {
        Self::new()
    }
}
