use std::fmt::{self, Display};

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
#[repr(C)]
#[allow(clippy::module_name_repetitions)]
pub enum SCStreamOutputType {
    Screen,
    Audio,
}
unsafe impl objc::Encode for SCStreamOutputType {
    fn encode() -> objc::Encoding {
        i8::encode()
    }
}
impl Display for SCStreamOutputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Screen => write!(f, "Screen"),
            Self::Audio => write!(f, "Audio"),
        }
    }
}
