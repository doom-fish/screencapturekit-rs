//! `IOSurface` — re-exported from `apple-cf` so all doom-fish crates share
//! one canonical `IOSurface` type.
//!
//! Metal-specific extension methods live on the `IOSurfaceMetalExt` trait in
//! `screencapturekit::metal` (bring it into scope to call `surface.info()` /
//! `surface.create_metal_textures(...)`, etc.).

pub use apple_cf::iosurface::{
    IOSurface, IOSurfaceLockGuard, IOSurfaceLockOptions, PlaneProperties,
};
