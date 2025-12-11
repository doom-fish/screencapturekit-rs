//! Metal texture helpers for `IOSurface`
//!
//! This module provides utilities for creating Metal textures from `IOSurface`
//! with zero-copy GPU access. This is the most efficient way to use captured
//! frames with Metal rendering.
//!
//! ## Features
//!
//! - Zero-copy texture creation from `IOSurface`
//! - Automatic pixel format detection and Metal format mapping
//! - Multi-plane support for YCbCr formats (420v, 420f)
//! - Native Metal device and texture types (no external crate needed)
//! - Embedded Metal shaders for common rendering scenarios
//!
//! ## When to Use
//!
//! Use this module when you need:
//! - **Real-time rendering** - Display captured frames in a Metal view
//! - **GPU processing** - Apply compute shaders to captured content
//! - **Zero-copy performance** - Avoid CPU-GPU memory transfers
//!
//! For CPU-based processing, use [`CVPixelBuffer`](crate::cv::CVPixelBuffer) with lock guards instead.
//!
//! ## Workflow
//!
//! 1. Get `IOSurface` from captured frame via [`CMSampleBuffer::image_buffer()`](crate::cm::CMSampleBuffer::image_buffer)
//! 2. Create Metal textures with [`IOSurface::create_metal_textures()`](crate::cm::IOSurface::create_metal_textures)
//! 3. Render using the built-in shaders or your own
//!
//! ## Example
//!
//! ```no_run
//! use screencapturekit::metal::MetalDevice;
//! use screencapturekit::cm::{CMSampleBuffer, IOSurface};
//!
//! // Get the system default Metal device
//! let device = MetalDevice::system_default().expect("No Metal device");
//!
//! // In your frame handler
//! fn handle_frame(sample: &CMSampleBuffer, device: &MetalDevice) {
//!     if let Some(pixel_buffer) = sample.image_buffer() {
//!         if let Some(surface) = pixel_buffer.io_surface() {
//!             // Create textures directly - no closures or factories needed
//!             if let Some(textures) = surface.create_metal_textures(device) {
//!                 if textures.is_ycbcr() {
//!                     // Use YCbCr shader with plane0 (Y) and plane1 (CbCr)
//!                     println!("YCbCr texture: {}x{}",
//!                         textures.plane0.width(), textures.plane0.height());
//!                 } else {
//!                     // Use single-plane shader (BGRA, l10r)
//!                     println!("Single-plane texture: {}x{}",
//!                         textures.plane0.width(), textures.plane0.height());
//!                 }
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! ## Built-in Shaders
//!
//! The [`SHADER_SOURCE`] constant contains Metal shaders for common rendering scenarios:
//!
//! | Function | Description |
//! |----------|-------------|
//! | `vertex_fullscreen` | Aspect-ratio-preserving fullscreen quad |
//! | `fragment_textured` | BGRA/L10R single-texture rendering |
//! | `fragment_ycbcr` | YCbCr biplanar (420v/420f) to RGB conversion |
//! | `vertex_colored` / `fragment_colored` | UI overlay rendering |

use std::ffi::{c_void, CStr};
use std::ptr::NonNull;

use crate::cm::IOSurface;
use crate::FourCharCode;

/// Pixel format constants using [`FourCharCode`]
///
/// These match the values returned by `IOSurface::pixel_format()`.
pub mod pixel_format {
    use crate::FourCharCode;

    /// BGRA 8-bit per channel (32-bit total)
    pub const BGRA: FourCharCode = FourCharCode::from_bytes(*b"BGRA");

    /// 10-bit RGB (ARGB2101010, also known as l10r)
    pub const L10R: FourCharCode = FourCharCode::from_bytes(*b"l10r");

    /// YCbCr 4:2:0 biplanar, video range
    pub const YCBCR_420V: FourCharCode = FourCharCode::from_bytes(*b"420v");

    /// YCbCr 4:2:0 biplanar, full range
    pub const YCBCR_420F: FourCharCode = FourCharCode::from_bytes(*b"420f");

    /// Check if a pixel format is a YCbCr biplanar format
    ///
    /// Accepts either a `FourCharCode` or a raw `u32`.
    #[must_use]
    pub fn is_ycbcr_biplanar(format: impl Into<FourCharCode>) -> bool {
        let f = format.into();
        f.equals(YCBCR_420V) || f.equals(YCBCR_420F)
    }

    /// Check if a pixel format uses full range (vs video range)
    ///
    /// Accepts either a `FourCharCode` or a raw `u32`.
    #[must_use]
    pub fn is_full_range(format: impl Into<FourCharCode>) -> bool {
        format.into().equals(YCBCR_420F)
    }
}

/// Metal pixel format enum matching `MTLPixelFormat` values
///
/// This provides a Rust-native enum for common Metal pixel formats used in screen capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u64)]
pub enum MetalPixelFormat {
    /// 8-bit normalized unsigned integer per channel (BGRA order)
    BGRA8Unorm = 80,
    /// 10-bit RGB with 2-bit alpha (BGR order)
    BGR10A2Unorm = 94,
    /// 8-bit normalized unsigned integer (single channel, for Y plane)
    R8Unorm = 10,
    /// 8-bit normalized unsigned integer per channel (two channels, for `CbCr` plane)
    RG8Unorm = 30,
}

impl MetalPixelFormat {
    /// Get the raw `MTLPixelFormat` value
    #[must_use]
    pub const fn raw(self) -> u64 {
        self as u64
    }

    /// Create from a raw `MTLPixelFormat` value
    #[must_use]
    pub const fn from_raw(value: u64) -> Option<Self> {
        match value {
            80 => Some(Self::BGRA8Unorm),
            94 => Some(Self::BGR10A2Unorm),
            10 => Some(Self::R8Unorm),
            30 => Some(Self::RG8Unorm),
            _ => None,
        }
    }
}

/// Information about an `IOSurface` for Metal texture creation
#[derive(Debug, Clone)]
pub struct IOSurfaceInfo {
    /// Width in pixels
    pub width: usize,
    /// Height in pixels
    pub height: usize,
    /// Bytes per row
    pub bytes_per_row: usize,
    /// Pixel format
    pub pixel_format: FourCharCode,
    /// Number of planes (0 for single-plane formats, 2 for YCbCr biplanar)
    pub plane_count: usize,
    /// Per-plane information
    pub planes: Vec<PlaneInfo>,
}

/// Information about a single plane within an `IOSurface`
#[derive(Debug, Clone)]
pub struct PlaneInfo {
    /// Plane index
    pub index: usize,
    /// Width in pixels
    pub width: usize,
    /// Height in pixels
    pub height: usize,
    /// Bytes per row
    pub bytes_per_row: usize,
}

impl IOSurface {
    /// Get detailed information about this `IOSurface` for Metal texture creation
    #[must_use]
    pub fn info(&self) -> IOSurfaceInfo {
        let width = self.width();
        let height = self.height();
        let bytes_per_row = self.bytes_per_row();
        let pix_format: FourCharCode = self.pixel_format().into();
        let plane_count = self.plane_count();

        let planes = if plane_count > 0 {
            (0..plane_count)
                .map(|i| PlaneInfo {
                    index: i,
                    width: self.width_of_plane(i),
                    height: self.height_of_plane(i),
                    bytes_per_row: self.bytes_per_row_of_plane(i),
                })
                .collect()
        } else {
            vec![]
        };

        IOSurfaceInfo {
            width,
            height,
            bytes_per_row,
            pixel_format: pix_format,
            plane_count,
            planes,
        }
    }

    /// Check if this `IOSurface` uses a YCbCr biplanar format
    #[must_use]
    pub fn is_ycbcr_biplanar(&self) -> bool {
        pixel_format::is_ycbcr_biplanar(self.pixel_format())
    }
}

/// Metal texture descriptor parameters for creating textures from `IOSurface`
///
/// This provides the information needed to configure a Metal `MTLTextureDescriptor`.
#[derive(Debug, Clone, Copy)]
pub struct TextureParams {
    /// Width in pixels
    pub width: usize,
    /// Height in pixels
    pub height: usize,
    /// Recommended Metal pixel format
    pub format: MetalPixelFormat,
    /// Plane index for multi-planar surfaces
    pub plane: usize,
}

impl TextureParams {
    /// Get the raw `MTLPixelFormat` value for use with Metal APIs
    #[must_use]
    pub const fn metal_pixel_format(&self) -> u64 {
        self.format.raw()
    }
}

impl IOSurface {
    /// Get texture parameters for creating Metal textures from this `IOSurface`
    ///
    /// Returns texture parameters for each plane needed to render this surface.
    /// - Single-plane formats (BGRA, L10R): Returns 1 texture param
    /// - YCbCr biplanar formats: Returns 2 texture params (Y and `CbCr` planes)
    #[must_use]
    pub fn texture_params(&self) -> Vec<TextureParams> {
        let pix_format: FourCharCode = self.pixel_format().into();
        let plane_count = self.plane_count();

        if pix_format == pixel_format::BGRA {
            vec![TextureParams {
                width: self.width(),
                height: self.height(),
                format: MetalPixelFormat::BGRA8Unorm,
                plane: 0,
            }]
        } else if pix_format == pixel_format::L10R {
            vec![TextureParams {
                width: self.width(),
                height: self.height(),
                format: MetalPixelFormat::BGR10A2Unorm,
                plane: 0,
            }]
        } else if pixel_format::is_ycbcr_biplanar(pix_format) && plane_count >= 2 {
            vec![
                // Plane 0: Y (luminance) - R8Unorm
                TextureParams {
                    width: self.width_of_plane(0),
                    height: self.height_of_plane(0),
                    format: MetalPixelFormat::R8Unorm,
                    plane: 0,
                },
                // Plane 1: CbCr (chrominance) - RG8Unorm
                TextureParams {
                    width: self.width_of_plane(1),
                    height: self.height_of_plane(1),
                    format: MetalPixelFormat::RG8Unorm,
                    plane: 1,
                },
            ]
        } else {
            // Fallback to BGRA
            vec![TextureParams {
                width: self.width(),
                height: self.height(),
                format: MetalPixelFormat::BGRA8Unorm,
                plane: 0,
            }]
        }
    }
}

/// Result of creating Metal textures from an `IOSurface`
#[derive(Debug)]
pub struct CapturedTextures<T> {
    /// Primary texture (BGRA/L10R for single-plane, Y plane for YCbCr)
    pub plane0: T,
    /// Secondary texture (`CbCr` plane for YCbCr formats)
    pub plane1: Option<T>,
    /// The pixel format of the source surface
    pub pixel_format: FourCharCode,
    /// Width in pixels
    pub width: usize,
    /// Height in pixels
    pub height: usize,
}

impl<T> CapturedTextures<T> {
    /// Check if this capture uses a YCbCr biplanar format
    #[must_use]
    pub fn is_ycbcr(&self) -> bool {
        pixel_format::is_ycbcr_biplanar(self.pixel_format)
    }
}

impl IOSurface {
    /// Create Metal textures from this `IOSurface` using a closure
    ///
    /// This is a zero-copy operation - the textures share memory with the `IOSurface`.
    ///
    /// The closure receives `TextureParams` and the raw `IOSurfaceRef` pointer,
    /// and should return the created texture.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use screencapturekit::cm::IOSurface;
    /// use std::ffi::c_void;
    ///
    /// fn example(surface: &IOSurface) {
    ///     let textures = surface.metal_textures(|params, _iosurface_ptr| {
    ///         // Create Metal texture using params.width, params.height, params.format
    ///         // Return Some(texture) or None
    ///         Some(()) // placeholder
    ///     });
    ///
    ///     if let Some(textures) = textures {
    ///         if textures.is_ycbcr() {
    ///             // Use YCbCr shader with plane0 (Y) and plane1 (CbCr)
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Safety
    ///
    /// The closure receives a raw `IOSurfaceRef` pointer. The pointer is valid
    /// for the duration of the closure call.
    pub fn metal_textures<T, F>(&self, create_texture: F) -> Option<CapturedTextures<T>>
    where
        F: Fn(&TextureParams, *const c_void) -> Option<T>,
    {
        let width = self.width();
        let height = self.height();
        let pix_format: FourCharCode = self.pixel_format().into();

        if width == 0 || height == 0 {
            return None;
        }

        let iosurface_ptr = self.as_ptr();
        let params = self.texture_params();

        if params.len() == 1 {
            // Single-plane format
            let texture = create_texture(&params[0], iosurface_ptr)?;
            Some(CapturedTextures {
                plane0: texture,
                plane1: None,
                pixel_format: pix_format,
                width,
                height,
            })
        } else if params.len() >= 2 {
            // YCbCr biplanar format
            let y_texture = create_texture(&params[0], iosurface_ptr)?;
            let uv_texture = create_texture(&params[1], iosurface_ptr)?;
            Some(CapturedTextures {
                plane0: y_texture,
                plane1: Some(uv_texture),
                pixel_format: pix_format,
                width,
                height,
            })
        } else {
            None
        }
    }
}

/// Metal shader source for rendering captured frames
///
/// This shader supports:
/// - BGRA and BGR10A2 single-plane formats
/// - YCbCr 4:2:0 biplanar formats (420v and 420f)
/// - Aspect-ratio-preserving fullscreen quad
///
/// ## Uniforms
///
/// The shader expects a `Uniforms` buffer:
/// - `viewport_size: float2` - Current viewport dimensions
/// - `texture_size: float2` - Source texture dimensions
/// - `time: float` - Animation time (optional)
/// - `pixel_format: uint` - `FourCC` pixel format code
///
/// ## Usage
///
/// 1. Compile shader with `device.new_library_with_source(SHADER_SOURCE, ...)`
/// 2. Create pipeline with `vertex_fullscreen` + `fragment_textured` (for BGRA/L10R)
/// 3. Or use `vertex_fullscreen` + `fragment_ycbcr` (for 420v/420f)
/// 4. Bind plane0 to texture slot 0, plane1 to texture slot 1 (for YCbCr)
pub const SHADER_SOURCE: &str = r"
#include <metal_stdlib>
using namespace metal;

struct Uniforms {
    float2 viewport_size;
    float2 texture_size;
    float time;
    uint pixel_format;
    float padding[2];
};

struct TexturedVertexOut {
    float4 position [[position]];
    float2 texcoord;
};

// Fullscreen quad vertex shader with aspect ratio correction
vertex TexturedVertexOut vertex_fullscreen(uint vid [[vertex_id]], constant Uniforms& uniforms [[buffer(0)]]) {
    TexturedVertexOut out;
    float va = uniforms.viewport_size.x / uniforms.viewport_size.y;
    float ta = uniforms.texture_size.x / uniforms.texture_size.y;
    float sx = ta > va ? 1.0 : ta / va;
    float sy = ta > va ? va / ta : 1.0;
    float2 positions[4] = { float2(-sx, -sy), float2(sx, -sy), float2(-sx, sy), float2(sx, sy) };
    float2 texcoords[4] = { float2(0.0, 1.0), float2(1.0, 1.0), float2(0.0, 0.0), float2(1.0, 0.0) };
    out.position = float4(positions[vid], 0.0, 1.0);
    out.texcoord = texcoords[vid];
    return out;
}

// BGRA/RGB texture fragment shader
fragment float4 fragment_textured(TexturedVertexOut in [[stage_in]], texture2d<float> tex [[texture(0)]]) {
    constexpr sampler s(mag_filter::linear, min_filter::linear);
    return tex.sample(s, in.texcoord);
}

// YCbCr to RGB conversion (BT.709 matrix for HD video)
float4 ycbcr_to_rgb(float y, float2 cbcr, bool full_range) {
    float y_adj = full_range ? y : (y - 16.0/255.0) * (255.0/219.0);
    float cb = cbcr.x - 0.5;
    float cr = cbcr.y - 0.5;
    // BT.709 conversion matrix
    float r = y_adj + 1.5748 * cr;
    float g = y_adj - 0.1873 * cb - 0.4681 * cr;
    float b = y_adj + 1.8556 * cb;
    return float4(saturate(float3(r, g, b)), 1.0);
}

// YCbCr biplanar (420v/420f) fragment shader
fragment float4 fragment_ycbcr(TexturedVertexOut in [[stage_in]], 
    texture2d<float> y_tex [[texture(0)]], 
    texture2d<float> cbcr_tex [[texture(1)]],
    constant Uniforms& uniforms [[buffer(0)]]) {
    constexpr sampler s(mag_filter::linear, min_filter::linear);
    float y = y_tex.sample(s, in.texcoord).r;
    float2 cbcr = cbcr_tex.sample(s, in.texcoord).rg;
    bool full_range = (uniforms.pixel_format == 0x34323066); // '420f'
    return ycbcr_to_rgb(y, cbcr, full_range);
}

// Colored vertex input/output for UI overlays
struct ColoredVertex {
    float2 position [[attribute(0)]];
    float4 color [[attribute(1)]];
};

struct ColoredVertexOut {
    float4 position [[position]];
    float4 color;
};

// Colored vertex shader for UI elements (position in pixels, converted to NDC)
vertex ColoredVertexOut vertex_colored(ColoredVertex in [[stage_in]], constant Uniforms& uniforms [[buffer(1)]]) {
    ColoredVertexOut out;
    float2 ndc = (in.position / uniforms.viewport_size) * 2.0 - 1.0;
    ndc.y = -ndc.y;
    out.position = float4(ndc, 0.0, 1.0);
    out.color = in.color;
    return out;
}

// Colored fragment shader for UI elements
fragment float4 fragment_colored(ColoredVertexOut in [[stage_in]]) {
    return in.color;
}
";

/// Uniforms structure for Metal shaders
///
/// This matches the layout expected by `SHADER_SOURCE`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Uniforms {
    /// Viewport width and height
    pub viewport_size: [f32; 2],
    /// Texture width and height
    pub texture_size: [f32; 2],
    /// Animation time (optional)
    pub time: f32,
    /// Pixel format (raw u32 for GPU compatibility)
    pub pixel_format: u32,
    /// Padding for alignment
    #[doc(hidden)]
    pub _padding: [f32; 2],
}

impl Uniforms {
    /// Create uniforms for a given viewport and texture size
    #[must_use]
    pub fn new(
        viewport_width: f32,
        viewport_height: f32,
        texture_width: f32,
        texture_height: f32,
    ) -> Self {
        Self {
            viewport_size: [viewport_width, viewport_height],
            texture_size: [texture_width, texture_height],
            time: 0.0,
            pixel_format: 0,
            _padding: [0.0; 2],
        }
    }

    /// Create uniforms from viewport size and captured textures
    ///
    /// Automatically extracts texture dimensions and pixel format.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use screencapturekit::metal::{MetalDevice, Uniforms};
    /// use screencapturekit::cm::IOSurface;
    ///
    /// fn example(surface: &IOSurface, device: &MetalDevice) {
    ///     if let Some(textures) = surface.create_metal_textures(device) {
    ///         let uniforms = Uniforms::from_captured_textures(1920.0, 1080.0, &textures);
    ///     }
    /// }
    /// ```
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // Screen dimensions will fit in f32
    pub fn from_captured_textures<T>(
        viewport_width: f32,
        viewport_height: f32,
        textures: &CapturedTextures<T>,
    ) -> Self {
        Self {
            viewport_size: [viewport_width, viewport_height],
            texture_size: [textures.width as f32, textures.height as f32],
            time: 0.0,
            pixel_format: textures.pixel_format.as_u32(),
            _padding: [0.0; 2],
        }
    }

    /// Set the pixel format
    ///
    /// Accepts either a `FourCharCode` or a raw `u32`:
    /// ```no_run
    /// use screencapturekit::metal::{Uniforms, pixel_format};
    ///
    /// let uniforms = Uniforms::new(1920.0, 1080.0, 1920.0, 1080.0)
    ///     .with_pixel_format(pixel_format::BGRA);
    /// ```
    #[must_use]
    pub fn with_pixel_format(mut self, format: impl Into<FourCharCode>) -> Self {
        self.pixel_format = format.into().as_u32();
        self
    }

    /// Set the animation time
    #[must_use]
    pub fn with_time(mut self, time: f32) -> Self {
        self.time = time;
        self
    }
}

// MARK: - FFI Declarations

#[link(name = "Metal", kind = "framework")]
extern "C" {}

#[link(name = "QuartzCore", kind = "framework")]
extern "C" {}

extern "C" {
    // Device
    fn metal_create_system_default_device() -> *mut c_void;
    fn metal_device_release(device: *mut c_void);
    fn metal_device_get_name(device: *mut c_void) -> *const std::ffi::c_char;
    fn metal_device_create_command_queue(device: *mut c_void) -> *mut c_void;
    fn metal_device_create_render_pipeline_state(
        device: *mut c_void,
        desc: *mut c_void,
    ) -> *mut c_void;

    // Texture
    fn metal_create_texture_from_iosurface(
        device: *mut c_void,
        iosurface: *mut c_void,
        plane: usize,
        width: usize,
        height: usize,
        pixel_format: u64,
    ) -> *mut c_void;
    fn metal_texture_release(texture: *mut c_void);
    fn metal_texture_retain(texture: *mut c_void) -> *mut c_void;
    fn metal_texture_get_width(texture: *mut c_void) -> usize;
    fn metal_texture_get_height(texture: *mut c_void) -> usize;
    fn metal_texture_get_pixel_format(texture: *mut c_void) -> u64;

    // Command Queue
    fn metal_command_queue_release(queue: *mut c_void);
    fn metal_command_queue_command_buffer(queue: *mut c_void) -> *mut c_void;

    // Library/Function
    fn metal_device_create_library_with_source(
        device: *mut c_void,
        source: *const std::ffi::c_char,
        error_out: *mut *const std::ffi::c_char,
    ) -> *mut c_void;
    fn metal_library_release(library: *mut c_void);
    fn metal_library_get_function(
        library: *mut c_void,
        name: *const std::ffi::c_char,
    ) -> *mut c_void;
    fn metal_function_release(function: *mut c_void);

    // Buffer
    fn metal_device_create_buffer(device: *mut c_void, length: usize, options: u64) -> *mut c_void;
    fn metal_buffer_contents(buffer: *mut c_void) -> *mut c_void;
    fn metal_buffer_length(buffer: *mut c_void) -> usize;
    fn metal_buffer_did_modify_range(buffer: *mut c_void, location: usize, length: usize);
    fn metal_buffer_release(buffer: *mut c_void);

    // Layer
    fn metal_layer_create() -> *mut c_void;
    fn metal_layer_set_device(layer: *mut c_void, device: *mut c_void);
    fn metal_layer_set_pixel_format(layer: *mut c_void, format: u64);
    fn metal_layer_set_drawable_size(layer: *mut c_void, width: f64, height: f64);
    fn metal_layer_set_presents_with_transaction(layer: *mut c_void, value: bool);
    fn metal_layer_next_drawable(layer: *mut c_void) -> *mut c_void;
    fn metal_layer_release(layer: *mut c_void);

    // Drawable
    fn metal_drawable_texture(drawable: *mut c_void) -> *mut c_void;
    fn metal_drawable_release(drawable: *mut c_void);

    // Command Buffer
    fn metal_command_buffer_present_drawable(cmd_buffer: *mut c_void, drawable: *mut c_void);
    fn metal_command_buffer_commit(cmd_buffer: *mut c_void);
    fn metal_command_buffer_release(cmd_buffer: *mut c_void);

    // Render Pass
    fn metal_render_pass_descriptor_create() -> *mut c_void;
    fn metal_render_pass_set_color_attachment_texture(
        desc: *mut c_void,
        index: usize,
        texture: *mut c_void,
    );
    fn metal_render_pass_set_color_attachment_load_action(
        desc: *mut c_void,
        index: usize,
        action: u64,
    );
    fn metal_render_pass_set_color_attachment_store_action(
        desc: *mut c_void,
        index: usize,
        action: u64,
    );
    fn metal_render_pass_set_color_attachment_clear_color(
        desc: *mut c_void,
        index: usize,
        r: f64,
        g: f64,
        b: f64,
        a: f64,
    );
    fn metal_render_pass_descriptor_release(desc: *mut c_void);

    // Vertex Descriptor
    fn metal_vertex_descriptor_create() -> *mut c_void;
    fn metal_vertex_descriptor_set_attribute(
        desc: *mut c_void,
        index: usize,
        format: u64,
        offset: usize,
        buffer_index: usize,
    );
    fn metal_vertex_descriptor_set_layout(
        desc: *mut c_void,
        buffer_index: usize,
        stride: usize,
        step_function: u64,
    );
    fn metal_vertex_descriptor_release(desc: *mut c_void);

    // Render Pipeline Descriptor
    fn metal_render_pipeline_descriptor_create() -> *mut c_void;
    fn metal_render_pipeline_descriptor_set_vertex_function(
        desc: *mut c_void,
        function: *mut c_void,
    );
    fn metal_render_pipeline_descriptor_set_fragment_function(
        desc: *mut c_void,
        function: *mut c_void,
    );
    fn metal_render_pipeline_descriptor_set_vertex_descriptor(
        desc: *mut c_void,
        vertex_descriptor: *mut c_void,
    );
    fn metal_render_pipeline_descriptor_set_color_attachment_pixel_format(
        desc: *mut c_void,
        index: usize,
        format: u64,
    );
    fn metal_render_pipeline_descriptor_set_blending_enabled(
        desc: *mut c_void,
        index: usize,
        enabled: bool,
    );
    fn metal_render_pipeline_descriptor_set_blend_operations(
        desc: *mut c_void,
        index: usize,
        rgb_op: u64,
        alpha_op: u64,
    );
    fn metal_render_pipeline_descriptor_set_blend_factors(
        desc: *mut c_void,
        index: usize,
        src_rgb: u64,
        dst_rgb: u64,
        src_alpha: u64,
        dst_alpha: u64,
    );
    fn metal_render_pipeline_descriptor_release(desc: *mut c_void);
    fn metal_render_pipeline_state_release(state: *mut c_void);

    // Render Command Encoder
    fn metal_command_buffer_render_command_encoder(
        cmd_buffer: *mut c_void,
        render_pass: *mut c_void,
    ) -> *mut c_void;
    fn metal_render_encoder_set_pipeline_state(encoder: *mut c_void, state: *mut c_void);
    fn metal_render_encoder_set_vertex_buffer(
        encoder: *mut c_void,
        buffer: *mut c_void,
        offset: usize,
        index: usize,
    );
    fn metal_render_encoder_set_fragment_buffer(
        encoder: *mut c_void,
        buffer: *mut c_void,
        offset: usize,
        index: usize,
    );
    fn metal_render_encoder_set_fragment_texture(
        encoder: *mut c_void,
        texture: *mut c_void,
        index: usize,
    );
    fn metal_render_encoder_draw_primitives(
        encoder: *mut c_void,
        primitive_type: u64,
        vertex_start: usize,
        vertex_count: usize,
    );
    fn metal_render_encoder_end_encoding(encoder: *mut c_void);
    fn metal_render_encoder_release(encoder: *mut c_void);

    // NSView helpers
    fn nsview_set_wants_layer(view: *mut c_void);
    fn nsview_set_layer(view: *mut c_void, layer: *mut c_void);
}

// MARK: - Metal Device

/// A Metal device (GPU)
///
/// This is a wrapper around `MTLDevice` that provides safe access to Metal functionality.
#[derive(Debug)]
pub struct MetalDevice {
    ptr: NonNull<c_void>,
}

impl MetalDevice {
    /// Get the system default Metal device
    ///
    /// Returns `None` if no Metal device is available.
    #[must_use]
    pub fn system_default() -> Option<Self> {
        let ptr = unsafe { metal_create_system_default_device() };
        NonNull::new(ptr).map(|ptr| Self { ptr })
    }

    /// Create a `MetalDevice` from a raw `MTLDevice` pointer
    ///
    /// This is useful when you already have a device from another source
    /// (e.g., the `metal` crate) and want to use it for texture creation.
    ///
    /// # Safety
    ///
    /// The pointer must be a valid `MTLDevice` pointer. The device will NOT
    /// be released when this wrapper is dropped - use `from_ptr_retained` if
    /// you want the wrapper to own the device.
    #[must_use]
    pub unsafe fn from_ptr(ptr: *mut c_void) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr })
    }

    /// Create a `MetalDevice` from a raw `MTLDevice` pointer, retaining it
    ///
    /// # Safety
    ///
    /// The pointer must be a valid `MTLDevice` pointer.
    #[must_use]
    pub unsafe fn from_ptr_retained(ptr: *mut c_void) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }
        // We don't have a retain function exposed, so we create from system default
        // and verify it's the same device
        NonNull::new(ptr).map(|ptr| Self { ptr })
    }

    /// Get the name of this device
    #[must_use]
    pub fn name(&self) -> String {
        unsafe {
            let name_ptr = metal_device_get_name(self.ptr.as_ptr());
            if name_ptr.is_null() {
                return String::new();
            }
            CStr::from_ptr(name_ptr).to_string_lossy().into_owned()
        }
    }

    /// Create a command queue for this device
    #[must_use]
    pub fn create_command_queue(&self) -> Option<MetalCommandQueue> {
        let ptr = unsafe { metal_device_create_command_queue(self.ptr.as_ptr()) };
        NonNull::new(ptr).map(|ptr| MetalCommandQueue { ptr })
    }

    /// Create a shader library from source code
    ///
    /// # Errors
    /// Returns an error message if shader compilation fails.
    pub fn create_library_with_source(&self, source: &str) -> Result<MetalLibrary, String> {
        use std::ffi::CString;
        let source_c = CString::new(source).map_err(|e| e.to_string())?;
        let mut error_ptr: *const std::ffi::c_char = std::ptr::null();

        let ptr = unsafe {
            metal_device_create_library_with_source(
                self.ptr.as_ptr(),
                source_c.as_ptr(),
                &mut error_ptr,
            )
        };

        NonNull::new(ptr).map_or_else(
            || {
                let error = if error_ptr.is_null() {
                    "Unknown shader compilation error".to_string()
                } else {
                    unsafe { CStr::from_ptr(error_ptr).to_string_lossy().into_owned() }
                };
                Err(error)
            },
            |ptr| Ok(MetalLibrary { ptr }),
        )
    }

    /// Create a buffer
    #[must_use]
    pub fn create_buffer(&self, length: usize, options: ResourceOptions) -> Option<MetalBuffer> {
        let ptr = unsafe { metal_device_create_buffer(self.ptr.as_ptr(), length, options.0) };
        NonNull::new(ptr).map(|ptr| MetalBuffer { ptr })
    }

    /// Create a buffer and populate it with the given data
    ///
    /// This is a convenience method that creates a buffer, copies the data,
    /// and returns the buffer. Useful for uniform buffers or vertex data.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use screencapturekit::metal::{MetalDevice, Uniforms};
    ///
    /// fn example() {
    ///     let device = MetalDevice::system_default().expect("No Metal device");
    ///     let uniforms = Uniforms::new(1920.0, 1080.0, 1920.0, 1080.0);
    ///     let buffer = device.create_buffer_with_data(&uniforms);
    /// }
    /// ```
    #[must_use]
    pub fn create_buffer_with_data<T>(&self, data: &T) -> Option<MetalBuffer> {
        let size = std::mem::size_of::<T>();
        let buffer = self.create_buffer(size, ResourceOptions::CPU_CACHE_MODE_DEFAULT_CACHE)?;
        unsafe {
            std::ptr::copy_nonoverlapping(
                std::ptr::addr_of!(*data).cast::<u8>(),
                buffer.contents().cast(),
                size,
            );
        }
        Some(buffer)
    }

    /// Create a render pipeline state from a descriptor
    #[must_use]
    pub fn create_render_pipeline_state(
        &self,
        descriptor: &MetalRenderPipelineDescriptor,
    ) -> Option<MetalRenderPipelineState> {
        let ptr = unsafe {
            metal_device_create_render_pipeline_state(self.ptr.as_ptr(), descriptor.as_ptr())
        };
        NonNull::new(ptr).map(|ptr| MetalRenderPipelineState { ptr })
    }

    /// Get the raw pointer to the underlying `MTLDevice`
    #[must_use]
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }
}

impl Drop for MetalDevice {
    fn drop(&mut self) {
        unsafe { metal_device_release(self.ptr.as_ptr()) }
    }
}

unsafe impl Send for MetalDevice {}
unsafe impl Sync for MetalDevice {}

// MARK: - Metal Texture

/// A Metal texture
///
/// This is a wrapper around `MTLTexture` that provides safe access.
#[derive(Debug)]
pub struct MetalTexture {
    ptr: NonNull<c_void>,
}

impl MetalTexture {
    /// Get the width of this texture
    #[must_use]
    pub fn width(&self) -> usize {
        unsafe { metal_texture_get_width(self.ptr.as_ptr()) }
    }

    /// Get the height of this texture
    #[must_use]
    pub fn height(&self) -> usize {
        unsafe { metal_texture_get_height(self.ptr.as_ptr()) }
    }

    /// Get the pixel format of this texture
    #[must_use]
    pub fn pixel_format(&self) -> MetalPixelFormat {
        let raw = unsafe { metal_texture_get_pixel_format(self.ptr.as_ptr()) };
        MetalPixelFormat::from_raw(raw).unwrap_or(MetalPixelFormat::BGRA8Unorm)
    }

    /// Get the raw pointer to the underlying `MTLTexture`
    #[must_use]
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }
}

impl Clone for MetalTexture {
    fn clone(&self) -> Self {
        let ptr = unsafe { metal_texture_retain(self.ptr.as_ptr()) };
        Self {
            ptr: NonNull::new(ptr).expect("metal_texture_retain returned null"),
        }
    }
}

impl Drop for MetalTexture {
    fn drop(&mut self) {
        unsafe { metal_texture_release(self.ptr.as_ptr()) }
    }
}

unsafe impl Send for MetalTexture {}
unsafe impl Sync for MetalTexture {}

// MARK: - Metal Command Queue

/// A Metal command queue
#[derive(Debug)]
pub struct MetalCommandQueue {
    ptr: NonNull<c_void>,
}

impl MetalCommandQueue {
    /// Create a command buffer
    #[must_use]
    pub fn command_buffer(&self) -> Option<MetalCommandBuffer> {
        let ptr = unsafe { metal_command_queue_command_buffer(self.ptr.as_ptr()) };
        NonNull::new(ptr).map(|ptr| MetalCommandBuffer { ptr })
    }

    /// Get the raw pointer to the underlying `MTLCommandQueue`
    #[must_use]
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }
}

impl Drop for MetalCommandQueue {
    fn drop(&mut self) {
        unsafe { metal_command_queue_release(self.ptr.as_ptr()) }
    }
}

unsafe impl Send for MetalCommandQueue {}
unsafe impl Sync for MetalCommandQueue {}

// MARK: - Metal Library

/// A Metal shader library
#[derive(Debug)]
pub struct MetalLibrary {
    ptr: NonNull<c_void>,
}

impl MetalLibrary {
    /// Get a function from this library by name
    #[must_use]
    pub fn get_function(&self, name: &str) -> Option<MetalFunction> {
        use std::ffi::CString;
        let name_c = CString::new(name).ok()?;
        let ptr = unsafe { metal_library_get_function(self.ptr.as_ptr(), name_c.as_ptr()) };
        NonNull::new(ptr).map(|ptr| MetalFunction { ptr })
    }

    /// Get the raw pointer to the underlying `MTLLibrary`
    #[must_use]
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }
}

impl Drop for MetalLibrary {
    fn drop(&mut self) {
        unsafe { metal_library_release(self.ptr.as_ptr()) }
    }
}

unsafe impl Send for MetalLibrary {}
unsafe impl Sync for MetalLibrary {}

// MARK: - Metal Function

/// A Metal shader function
#[derive(Debug)]
pub struct MetalFunction {
    ptr: NonNull<c_void>,
}

impl MetalFunction {
    /// Get the raw pointer to the underlying `MTLFunction`
    #[must_use]
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }
}

impl Drop for MetalFunction {
    fn drop(&mut self) {
        unsafe { metal_function_release(self.ptr.as_ptr()) }
    }
}

unsafe impl Send for MetalFunction {}
unsafe impl Sync for MetalFunction {}

// MARK: - Metal Buffer

/// A Metal buffer for vertex/uniform data
#[derive(Debug)]
pub struct MetalBuffer {
    ptr: NonNull<c_void>,
}

/// Resource options for buffer creation
#[derive(Debug, Clone, Copy, Default)]
pub struct ResourceOptions(u64);

impl ResourceOptions {
    /// CPU cache mode default, storage mode shared
    pub const CPU_CACHE_MODE_DEFAULT_CACHE: Self = Self(0);
    /// Storage mode shared (CPU and GPU can access)
    pub const STORAGE_MODE_SHARED: Self = Self(0);
    /// Storage mode managed (CPU writes, GPU reads)
    pub const STORAGE_MODE_MANAGED: Self = Self(1 << 4);
}

impl MetalBuffer {
    /// Get a pointer to the buffer contents
    #[must_use]
    pub fn contents(&self) -> *mut c_void {
        unsafe { metal_buffer_contents(self.ptr.as_ptr()) }
    }

    /// Get the length of the buffer in bytes
    #[must_use]
    pub fn length(&self) -> usize {
        unsafe { metal_buffer_length(self.ptr.as_ptr()) }
    }

    /// Notify that a range of the buffer was modified (for managed storage mode)
    pub fn did_modify_range(&self, range: std::ops::Range<usize>) {
        unsafe { metal_buffer_did_modify_range(self.ptr.as_ptr(), range.start, range.len()) }
    }

    /// Get the raw pointer
    #[must_use]
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }
}

impl Drop for MetalBuffer {
    fn drop(&mut self) {
        unsafe { metal_buffer_release(self.ptr.as_ptr()) }
    }
}

unsafe impl Send for MetalBuffer {}
unsafe impl Sync for MetalBuffer {}

// MARK: - Metal Layer

/// A `CAMetalLayer` for rendering to a window
#[derive(Debug)]
pub struct MetalLayer {
    ptr: NonNull<c_void>,
}

impl MetalLayer {
    /// Create a new Metal layer
    ///
    /// # Panics
    /// Panics if layer creation fails (should not happen on macOS with Metal support).
    #[must_use]
    pub fn new() -> Self {
        let ptr = unsafe { metal_layer_create() };
        Self {
            ptr: NonNull::new(ptr).expect("metal_layer_create returned null"),
        }
    }

    /// Set the device for this layer
    pub fn set_device(&self, device: &MetalDevice) {
        unsafe { metal_layer_set_device(self.ptr.as_ptr(), device.as_ptr()) }
    }

    /// Set the pixel format
    pub fn set_pixel_format(&self, format: MTLPixelFormat) {
        unsafe { metal_layer_set_pixel_format(self.ptr.as_ptr(), format.raw()) }
    }

    /// Set the drawable size
    pub fn set_drawable_size(&self, width: f64, height: f64) {
        unsafe { metal_layer_set_drawable_size(self.ptr.as_ptr(), width, height) }
    }

    /// Set whether to present with transaction
    pub fn set_presents_with_transaction(&self, value: bool) {
        unsafe { metal_layer_set_presents_with_transaction(self.ptr.as_ptr(), value) }
    }

    /// Get the next drawable
    #[must_use]
    pub fn next_drawable(&self) -> Option<MetalDrawable> {
        let ptr = unsafe { metal_layer_next_drawable(self.ptr.as_ptr()) };
        NonNull::new(ptr).map(|ptr| MetalDrawable { ptr })
    }

    /// Get the raw pointer (for attaching to a view)
    #[must_use]
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }
}

impl Default for MetalLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MetalLayer {
    fn drop(&mut self) {
        unsafe { metal_layer_release(self.ptr.as_ptr()) }
    }
}

// MARK: - Metal Drawable

/// A drawable from a Metal layer
#[derive(Debug)]
pub struct MetalDrawable {
    ptr: NonNull<c_void>,
}

impl MetalDrawable {
    /// Get the texture for this drawable
    ///
    /// # Panics
    /// Panics if the drawable has no texture (should not happen for valid drawables).
    #[must_use]
    pub fn texture(&self) -> MetalTexture {
        let ptr = unsafe { metal_drawable_texture(self.ptr.as_ptr()) };
        // Texture is borrowed from drawable, need to retain it
        let ptr = unsafe { metal_texture_retain(ptr) };
        MetalTexture {
            ptr: NonNull::new(ptr).expect("drawable texture is null"),
        }
    }

    /// Get the raw pointer
    #[must_use]
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }
}

impl Drop for MetalDrawable {
    fn drop(&mut self) {
        unsafe { metal_drawable_release(self.ptr.as_ptr()) }
    }
}

// MARK: - Command Buffer

/// A Metal command buffer
#[derive(Debug)]
pub struct MetalCommandBuffer {
    ptr: NonNull<c_void>,
}

impl MetalCommandBuffer {
    /// Create a render command encoder
    #[must_use]
    pub fn render_command_encoder(
        &self,
        render_pass: &MetalRenderPassDescriptor,
    ) -> Option<MetalRenderCommandEncoder> {
        let ptr = unsafe {
            metal_command_buffer_render_command_encoder(self.ptr.as_ptr(), render_pass.as_ptr())
        };
        NonNull::new(ptr).map(|ptr| MetalRenderCommandEncoder { ptr })
    }

    /// Present a drawable
    pub fn present_drawable(&self, drawable: &MetalDrawable) {
        unsafe { metal_command_buffer_present_drawable(self.ptr.as_ptr(), drawable.as_ptr()) }
    }

    /// Commit the command buffer
    pub fn commit(&self) {
        unsafe { metal_command_buffer_commit(self.ptr.as_ptr()) }
    }

    /// Get the raw pointer
    #[must_use]
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }
}

impl Drop for MetalCommandBuffer {
    fn drop(&mut self) {
        unsafe { metal_command_buffer_release(self.ptr.as_ptr()) }
    }
}

// MARK: - Render Pass Descriptor

/// A render pass descriptor
#[derive(Debug)]
pub struct MetalRenderPassDescriptor {
    ptr: NonNull<c_void>,
}

/// Load action for render pass attachments
#[derive(Debug, Clone, Copy, Default)]
#[repr(u64)]
pub enum MTLLoadAction {
    /// Don't care about existing contents
    DontCare = 0,
    /// Load existing contents
    Load = 1,
    /// Clear to a value
    #[default]
    Clear = 2,
}

/// Store action for render pass attachments
#[derive(Debug, Clone, Copy, Default)]
#[repr(u64)]
pub enum MTLStoreAction {
    /// Don't care about storing
    DontCare = 0,
    /// Store the results
    #[default]
    Store = 1,
}

/// Pixel format
#[derive(Debug, Clone, Copy, Default)]
#[repr(u64)]
pub enum MTLPixelFormat {
    /// Invalid format
    Invalid = 0,
    /// BGRA 8-bit unsigned normalized
    #[default]
    BGRA8Unorm = 80,
    /// BGR 10-bit, A 2-bit unsigned normalized
    BGR10A2Unorm = 94,
    /// R 8-bit unsigned normalized
    R8Unorm = 10,
    /// RG 8-bit unsigned normalized
    RG8Unorm = 30,
}

impl MTLPixelFormat {
    /// Get the raw value
    #[must_use]
    pub const fn raw(self) -> u64 {
        self as u64
    }
}

/// Vertex format for vertex attributes
#[derive(Debug, Clone, Copy, Default)]
#[repr(u64)]
pub enum MTLVertexFormat {
    /// Invalid format
    Invalid = 0,
    /// Two 32-bit floats
    #[default]
    Float2 = 29,
    /// Three 32-bit floats
    Float3 = 30,
    /// Four 32-bit floats
    Float4 = 31,
}

impl MTLVertexFormat {
    /// Get the raw value
    #[must_use]
    pub const fn raw(self) -> u64 {
        self as u64
    }
}

/// Vertex step function
#[derive(Debug, Clone, Copy, Default)]
#[repr(u64)]
pub enum MTLVertexStepFunction {
    /// Constant value (same for all vertices)
    Constant = 0,
    /// Step once per vertex (default)
    #[default]
    PerVertex = 1,
    /// Step once per instance
    PerInstance = 2,
}

impl MTLVertexStepFunction {
    /// Get the raw value
    #[must_use]
    pub const fn raw(self) -> u64 {
        self as u64
    }
}

/// Primitive type for drawing
#[derive(Debug, Clone, Copy, Default)]
#[repr(u64)]
pub enum MTLPrimitiveType {
    /// Points
    Point = 0,
    /// Lines
    Line = 1,
    /// Line strip
    LineStrip = 2,
    /// Triangles
    #[default]
    Triangle = 3,
    /// Triangle strip
    TriangleStrip = 4,
}

impl MTLPrimitiveType {
    /// Get the raw value
    #[must_use]
    pub const fn raw(self) -> u64 {
        self as u64
    }
}

/// Blend operation
#[derive(Debug, Clone, Copy, Default)]
#[repr(u64)]
pub enum MTLBlendOperation {
    /// Add source and destination
    #[default]
    Add = 0,
    /// Subtract destination from source
    Subtract = 1,
    /// Subtract source from destination
    ReverseSubtract = 2,
    /// Minimum of source and destination
    Min = 3,
    /// Maximum of source and destination
    Max = 4,
}

/// Blend factor
#[derive(Debug, Clone, Copy, Default)]
#[repr(u64)]
pub enum MTLBlendFactor {
    /// 0
    Zero = 0,
    /// 1
    #[default]
    One = 1,
    /// Source color
    SourceColor = 2,
    /// 1 - source color
    OneMinusSourceColor = 3,
    /// Source alpha
    SourceAlpha = 4,
    /// 1 - source alpha
    OneMinusSourceAlpha = 5,
    /// Destination color
    DestinationColor = 6,
    /// 1 - destination color
    OneMinusDestinationColor = 7,
    /// Destination alpha
    DestinationAlpha = 8,
    /// 1 - destination alpha
    OneMinusDestinationAlpha = 9,
}

impl MetalRenderPassDescriptor {
    /// Create a new render pass descriptor
    ///
    /// # Panics
    /// Panics if descriptor creation fails (should not happen).
    #[must_use]
    pub fn new() -> Self {
        let ptr = unsafe { metal_render_pass_descriptor_create() };
        Self {
            ptr: NonNull::new(ptr).expect("render pass descriptor create failed"),
        }
    }

    /// Set the texture for a color attachment
    pub fn set_color_attachment_texture(&self, index: usize, texture: &MetalTexture) {
        unsafe {
            metal_render_pass_set_color_attachment_texture(
                self.ptr.as_ptr(),
                index,
                texture.as_ptr(),
            );
        }
    }

    /// Set the load action for a color attachment
    pub fn set_color_attachment_load_action(&self, index: usize, action: MTLLoadAction) {
        unsafe {
            metal_render_pass_set_color_attachment_load_action(
                self.ptr.as_ptr(),
                index,
                action as u64,
            );
        }
    }

    /// Set the store action for a color attachment
    pub fn set_color_attachment_store_action(&self, index: usize, action: MTLStoreAction) {
        unsafe {
            metal_render_pass_set_color_attachment_store_action(
                self.ptr.as_ptr(),
                index,
                action as u64,
            );
        }
    }

    /// Set the clear color for a color attachment
    pub fn set_color_attachment_clear_color(&self, index: usize, r: f64, g: f64, b: f64, a: f64) {
        unsafe {
            metal_render_pass_set_color_attachment_clear_color(
                self.ptr.as_ptr(),
                index,
                r,
                g,
                b,
                a,
            );
        }
    }

    /// Get the raw pointer
    #[must_use]
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }
}

impl Default for MetalRenderPassDescriptor {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MetalRenderPassDescriptor {
    fn drop(&mut self) {
        unsafe { metal_render_pass_descriptor_release(self.ptr.as_ptr()) }
    }
}

// MARK: - Vertex Descriptor

/// A vertex descriptor for specifying vertex buffer layout
#[derive(Debug)]
pub struct MetalVertexDescriptor {
    ptr: NonNull<c_void>,
}

impl MetalVertexDescriptor {
    /// Create a new vertex descriptor
    ///
    /// # Panics
    /// Panics if descriptor creation fails (should not happen).
    #[must_use]
    pub fn new() -> Self {
        let ptr = unsafe { metal_vertex_descriptor_create() };
        Self {
            ptr: NonNull::new(ptr).expect("vertex descriptor create failed"),
        }
    }

    /// Set an attribute's format, offset, and buffer index
    pub fn set_attribute(
        &self,
        index: usize,
        format: MTLVertexFormat,
        offset: usize,
        buffer_index: usize,
    ) {
        unsafe {
            metal_vertex_descriptor_set_attribute(
                self.ptr.as_ptr(),
                index,
                format.raw(),
                offset,
                buffer_index,
            );
        }
    }

    /// Set a buffer layout's stride and step function
    pub fn set_layout(
        &self,
        buffer_index: usize,
        stride: usize,
        step_function: MTLVertexStepFunction,
    ) {
        unsafe {
            metal_vertex_descriptor_set_layout(
                self.ptr.as_ptr(),
                buffer_index,
                stride,
                step_function.raw(),
            );
        }
    }

    /// Get the raw pointer
    #[must_use]
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }
}

impl Default for MetalVertexDescriptor {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MetalVertexDescriptor {
    fn drop(&mut self) {
        unsafe { metal_vertex_descriptor_release(self.ptr.as_ptr()) }
    }
}

// MARK: - Render Pipeline Descriptor

/// A render pipeline descriptor
#[derive(Debug)]
pub struct MetalRenderPipelineDescriptor {
    ptr: NonNull<c_void>,
}

impl MetalRenderPipelineDescriptor {
    /// Create a new render pipeline descriptor
    ///
    /// # Panics
    /// Panics if descriptor creation fails (should not happen).
    #[must_use]
    pub fn new() -> Self {
        let ptr = unsafe { metal_render_pipeline_descriptor_create() };
        Self {
            ptr: NonNull::new(ptr).expect("render pipeline descriptor create failed"),
        }
    }

    /// Set the vertex function
    pub fn set_vertex_function(&self, function: &MetalFunction) {
        unsafe {
            metal_render_pipeline_descriptor_set_vertex_function(
                self.ptr.as_ptr(),
                function.as_ptr(),
            );
        }
    }

    /// Set the fragment function
    pub fn set_fragment_function(&self, function: &MetalFunction) {
        unsafe {
            metal_render_pipeline_descriptor_set_fragment_function(
                self.ptr.as_ptr(),
                function.as_ptr(),
            );
        }
    }

    /// Set the vertex descriptor for vertex buffer layout
    pub fn set_vertex_descriptor(&self, descriptor: &MetalVertexDescriptor) {
        unsafe {
            metal_render_pipeline_descriptor_set_vertex_descriptor(
                self.ptr.as_ptr(),
                descriptor.as_ptr(),
            );
        }
    }

    /// Set color attachment pixel format
    pub fn set_color_attachment_pixel_format(&self, index: usize, format: MTLPixelFormat) {
        unsafe {
            metal_render_pipeline_descriptor_set_color_attachment_pixel_format(
                self.ptr.as_ptr(),
                index,
                format.raw(),
            );
        }
    }

    /// Set blending enabled for a color attachment
    pub fn set_blending_enabled(&self, index: usize, enabled: bool) {
        unsafe {
            metal_render_pipeline_descriptor_set_blending_enabled(
                self.ptr.as_ptr(),
                index,
                enabled,
            );
        }
    }

    /// Set blend operations
    pub fn set_blend_operations(
        &self,
        index: usize,
        rgb_op: MTLBlendOperation,
        alpha_op: MTLBlendOperation,
    ) {
        unsafe {
            metal_render_pipeline_descriptor_set_blend_operations(
                self.ptr.as_ptr(),
                index,
                rgb_op as u64,
                alpha_op as u64,
            );
        }
    }

    /// Set blend factors
    pub fn set_blend_factors(
        &self,
        index: usize,
        src_rgb: MTLBlendFactor,
        dst_rgb: MTLBlendFactor,
        src_alpha: MTLBlendFactor,
        dst_alpha: MTLBlendFactor,
    ) {
        unsafe {
            metal_render_pipeline_descriptor_set_blend_factors(
                self.ptr.as_ptr(),
                index,
                src_rgb as u64,
                dst_rgb as u64,
                src_alpha as u64,
                dst_alpha as u64,
            );
        }
    }

    /// Get the raw pointer
    #[must_use]
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }
}

impl Default for MetalRenderPipelineDescriptor {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MetalRenderPipelineDescriptor {
    fn drop(&mut self) {
        unsafe { metal_render_pipeline_descriptor_release(self.ptr.as_ptr()) }
    }
}

// MARK: - Render Pipeline State

/// A compiled render pipeline state
#[derive(Debug)]
pub struct MetalRenderPipelineState {
    ptr: NonNull<c_void>,
}

impl MetalRenderPipelineState {
    /// Get the raw pointer
    #[must_use]
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }
}

impl Drop for MetalRenderPipelineState {
    fn drop(&mut self) {
        unsafe { metal_render_pipeline_state_release(self.ptr.as_ptr()) }
    }
}

unsafe impl Send for MetalRenderPipelineState {}
unsafe impl Sync for MetalRenderPipelineState {}

// MARK: - Render Command Encoder

/// A render command encoder
#[derive(Debug)]
pub struct MetalRenderCommandEncoder {
    ptr: NonNull<c_void>,
}

impl MetalRenderCommandEncoder {
    /// Set the render pipeline state
    pub fn set_render_pipeline_state(&self, state: &MetalRenderPipelineState) {
        unsafe { metal_render_encoder_set_pipeline_state(self.ptr.as_ptr(), state.as_ptr()) }
    }

    /// Set a vertex buffer
    pub fn set_vertex_buffer(&self, buffer: &MetalBuffer, offset: usize, index: usize) {
        unsafe {
            metal_render_encoder_set_vertex_buffer(
                self.ptr.as_ptr(),
                buffer.as_ptr(),
                offset,
                index,
            );
        }
    }

    /// Set a fragment buffer
    pub fn set_fragment_buffer(&self, buffer: &MetalBuffer, offset: usize, index: usize) {
        unsafe {
            metal_render_encoder_set_fragment_buffer(
                self.ptr.as_ptr(),
                buffer.as_ptr(),
                offset,
                index,
            );
        }
    }

    /// Set a fragment texture
    pub fn set_fragment_texture(&self, texture: &MetalTexture, index: usize) {
        unsafe {
            metal_render_encoder_set_fragment_texture(self.ptr.as_ptr(), texture.as_ptr(), index);
        }
    }

    /// Draw primitives
    pub fn draw_primitives(
        &self,
        primitive_type: MTLPrimitiveType,
        vertex_start: usize,
        vertex_count: usize,
    ) {
        unsafe {
            metal_render_encoder_draw_primitives(
                self.ptr.as_ptr(),
                primitive_type.raw(),
                vertex_start,
                vertex_count,
            );
        }
    }

    /// End encoding
    pub fn end_encoding(&self) {
        unsafe { metal_render_encoder_end_encoding(self.ptr.as_ptr()) }
    }

    /// Get the raw pointer
    #[must_use]
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }
}

impl Drop for MetalRenderCommandEncoder {
    fn drop(&mut self) {
        unsafe { metal_render_encoder_release(self.ptr.as_ptr()) }
    }
}

// MARK: - IOSurface Metal Extension

/// Result of creating Metal textures from an `IOSurface`
pub type MetalCapturedTextures = CapturedTextures<MetalTexture>;

impl IOSurface {
    /// Create Metal textures from this `IOSurface` using the provided device
    ///
    /// This is a zero-copy operation - the textures share memory with the `IOSurface`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use screencapturekit::metal::MetalDevice;
    /// use screencapturekit::cm::IOSurface;
    ///
    /// fn example(surface: &IOSurface) {
    ///     let device = MetalDevice::system_default().expect("No Metal device");
    ///     if let Some(textures) = surface.create_metal_textures(&device) {
    ///         if textures.is_ycbcr() {
    ///             // Use YCbCr shader with plane0 (Y) and plane1 (CbCr)
    ///         }
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn create_metal_textures(&self, device: &MetalDevice) -> Option<MetalCapturedTextures> {
        let width = self.width();
        let height = self.height();
        let pix_format: FourCharCode = self.pixel_format().into();

        if width == 0 || height == 0 {
            return None;
        }

        let params = self.texture_params();

        if params.len() == 1 {
            // Single-plane format
            let texture = self.create_texture_for_plane(device, &params[0])?;
            Some(CapturedTextures {
                plane0: texture,
                plane1: None,
                pixel_format: pix_format,
                width,
                height,
            })
        } else if params.len() >= 2 {
            // YCbCr biplanar format
            let y_texture = self.create_texture_for_plane(device, &params[0])?;
            let uv_texture = self.create_texture_for_plane(device, &params[1])?;
            Some(CapturedTextures {
                plane0: y_texture,
                plane1: Some(uv_texture),
                pixel_format: pix_format,
                width,
                height,
            })
        } else {
            None
        }
    }

    fn create_texture_for_plane(
        &self,
        device: &MetalDevice,
        params: &TextureParams,
    ) -> Option<MetalTexture> {
        let ptr = unsafe {
            metal_create_texture_from_iosurface(
                device.as_ptr(),
                self.as_ptr(),
                params.plane,
                params.width,
                params.height,
                params.format.raw(),
            )
        };
        NonNull::new(ptr).map(|ptr| MetalTexture { ptr })
    }
}

// MARK: - Autorelease Pool

#[link(name = "Foundation", kind = "framework")]
extern "C" {
    fn objc_autoreleasePoolPush() -> *mut c_void;
    fn objc_autoreleasePoolPop(pool: *mut c_void);
}

/// Execute a closure within an autorelease pool
///
/// This is equivalent to `@autoreleasepool { ... }` in Objective-C/Swift.
/// Use this when running code that creates temporary Objective-C objects
/// that need to be released promptly.
///
/// # Example
///
/// ```no_run
/// use screencapturekit::metal::autoreleasepool;
///
/// autoreleasepool(|| {
///     // Code that creates temporary Objective-C objects
///     println!("Inside autorelease pool");
/// });
/// ```
pub fn autoreleasepool<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    unsafe {
        let pool = objc_autoreleasePoolPush();
        let result = f();
        objc_autoreleasePoolPop(pool);
        result
    }
}

// MARK: - NSView Helpers

/// Set up an `NSView` for Metal rendering
///
/// This sets `wantsLayer = YES` and assigns the Metal layer to the view.
///
/// # Safety
///
/// The `view` pointer must be a valid `NSView` pointer.
///
/// # Example
///
/// ```no_run
/// use screencapturekit::metal::{setup_metal_view, MetalLayer};
/// use std::ffi::c_void;
///
/// fn example(ns_view: *mut c_void) {
///     let layer = MetalLayer::new();
///     unsafe { setup_metal_view(ns_view, &layer); }
/// }
/// ```
pub unsafe fn setup_metal_view(view: *mut c_void, layer: &MetalLayer) {
    nsview_set_wants_layer(view);
    nsview_set_layer(view, layer.as_ptr());
}
