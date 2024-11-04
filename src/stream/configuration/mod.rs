mod internal;

pub mod audio;
pub mod dimensions;

#[allow(clippy::module_name_repetitions)] 
pub use internal::SCStreamConfiguration;
impl SCStreamConfiguration {
    #[must_use]
    pub fn new() -> Self {
        Self::internal_init()
    }
}
