//! `CMFormatDescription` — re-exported from `apple-cf`. The `media_types`
//! and `codec_types` constant modules are also re-exported so existing
//! `screencapturekit::cm::media_types::VIDEO` paths keep resolving.

pub use apple_cf::cm::format_description::CMFormatDescription;
pub use apple_cf::cm::format_description::{codec_types, media_types};
