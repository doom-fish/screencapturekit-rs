//! Metal rendering helpers

use std::ffi::c_void;

use metal::foreign_types::ForeignType;
use metal::*;
use objc::{msg_send, sel, sel_impl};

#[link(name = "Metal", kind = "framework")]
#[link(name = "IOSurface", kind = "framework")]
extern "C" {
    fn IOSurfaceGetWidth(surface: *const c_void) -> usize;
    fn IOSurfaceGetHeight(surface: *const c_void) -> usize;
    fn IOSurfaceGetPixelFormat(surface: *const c_void) -> u32;
    fn IOSurfaceGetPlaneCount(surface: *const c_void) -> usize;
    fn IOSurfaceGetWidthOfPlane(surface: *const c_void, plane: usize) -> usize;
    fn IOSurfaceGetHeightOfPlane(surface: *const c_void, plane: usize) -> usize;
}

// Pixel format constants (FourCC codes)
pub const PIXEL_FORMAT_BGRA: u32 = 0x42475241; // 'BGRA'
pub const PIXEL_FORMAT_L10R: u32 = 0x6C313072; // 'l10r' - ARGB2101010
pub const PIXEL_FORMAT_420V: u32 = 0x34323076; // '420v' - YCbCr 420 video range
pub const PIXEL_FORMAT_420F: u32 = 0x34323066; // '420f' - YCbCr 420 full range

pub struct CaptureTextures {
    pub plane0: Texture,         // Y plane for YCbCr, or BGRA texture
    pub plane1: Option<Texture>, // CbCr plane for YCbCr formats
    pub pixel_format: u32,
    #[allow(dead_code)]
    pub width: usize,
    #[allow(dead_code)]
    pub height: usize,
}

/// Create Metal textures from an IOSurface (zero-copy)
///
/// # Safety
/// The `iosurface_ptr` must be a valid IOSurface pointer.
pub unsafe fn create_textures_from_iosurface(
    device: &Device,
    iosurface_ptr: *const c_void,
) -> Option<CaptureTextures> {
    if iosurface_ptr.is_null() {
        return None;
    }
    let width = IOSurfaceGetWidth(iosurface_ptr);
    let height = IOSurfaceGetHeight(iosurface_ptr);
    let pixel_format = IOSurfaceGetPixelFormat(iosurface_ptr);
    let plane_count = IOSurfaceGetPlaneCount(iosurface_ptr);

    if width == 0 || height == 0 {
        return None;
    }

    // Determine Metal pixel format and create appropriate textures
    match pixel_format {
        PIXEL_FORMAT_BGRA => {
            // Single plane BGRA
            let desc = TextureDescriptor::new();
            desc.set_texture_type(MTLTextureType::D2);
            desc.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
            desc.set_width(width as u64);
            desc.set_height(height as u64);
            desc.set_storage_mode(MTLStorageMode::Shared);
            desc.set_usage(MTLTextureUsage::ShaderRead);
            let texture: *mut MTLTexture = msg_send![device.as_ptr() as *mut objc::runtime::Object,
                newTextureWithDescriptor: desc.as_ptr() as *mut objc::runtime::Object
                iosurface: iosurface_ptr plane: 0usize];
            if texture.is_null() {
                return None;
            }
            Some(CaptureTextures {
                plane0: Texture::from_ptr(texture),
                plane1: None,
                pixel_format,
                width,
                height,
            })
        }
        PIXEL_FORMAT_L10R => {
            // 10-bit ARGB2101010
            let desc = TextureDescriptor::new();
            desc.set_texture_type(MTLTextureType::D2);
            desc.set_pixel_format(MTLPixelFormat::BGR10A2Unorm);
            desc.set_width(width as u64);
            desc.set_height(height as u64);
            desc.set_storage_mode(MTLStorageMode::Shared);
            desc.set_usage(MTLTextureUsage::ShaderRead);
            let texture: *mut MTLTexture = msg_send![device.as_ptr() as *mut objc::runtime::Object,
                newTextureWithDescriptor: desc.as_ptr() as *mut objc::runtime::Object
                iosurface: iosurface_ptr plane: 0usize];
            if texture.is_null() {
                return None;
            }
            Some(CaptureTextures {
                plane0: Texture::from_ptr(texture),
                plane1: None,
                pixel_format,
                width,
                height,
            })
        }
        PIXEL_FORMAT_420V | PIXEL_FORMAT_420F => {
            // YCbCr 4:2:0 - two planes: Y (R8) and CbCr (RG8)
            if plane_count < 2 {
                return None;
            }

            // Plane 0: Y (luminance)
            let y_width = IOSurfaceGetWidthOfPlane(iosurface_ptr, 0);
            let y_height = IOSurfaceGetHeightOfPlane(iosurface_ptr, 0);
            let y_desc = TextureDescriptor::new();
            y_desc.set_texture_type(MTLTextureType::D2);
            y_desc.set_pixel_format(MTLPixelFormat::R8Unorm);
            y_desc.set_width(y_width as u64);
            y_desc.set_height(y_height as u64);
            y_desc.set_storage_mode(MTLStorageMode::Shared);
            y_desc.set_usage(MTLTextureUsage::ShaderRead);
            let y_texture: *mut MTLTexture = msg_send![device.as_ptr() as *mut objc::runtime::Object,
                newTextureWithDescriptor: y_desc.as_ptr() as *mut objc::runtime::Object
                iosurface: iosurface_ptr plane: 0usize];
            if y_texture.is_null() {
                return None;
            }

            // Plane 1: CbCr (chroma)
            let uv_width = IOSurfaceGetWidthOfPlane(iosurface_ptr, 1);
            let uv_height = IOSurfaceGetHeightOfPlane(iosurface_ptr, 1);
            let uv_desc = TextureDescriptor::new();
            uv_desc.set_texture_type(MTLTextureType::D2);
            uv_desc.set_pixel_format(MTLPixelFormat::RG8Unorm);
            uv_desc.set_width(uv_width as u64);
            uv_desc.set_height(uv_height as u64);
            uv_desc.set_storage_mode(MTLStorageMode::Shared);
            uv_desc.set_usage(MTLTextureUsage::ShaderRead);
            let uv_texture: *mut MTLTexture = msg_send![device.as_ptr() as *mut objc::runtime::Object,
                newTextureWithDescriptor: uv_desc.as_ptr() as *mut objc::runtime::Object
                iosurface: iosurface_ptr plane: 1usize];
            if uv_texture.is_null() {
                return None;
            }

            Some(CaptureTextures {
                plane0: Texture::from_ptr(y_texture),
                plane1: Some(Texture::from_ptr(uv_texture)),
                pixel_format,
                width,
                height,
            })
        }
        _ => {
            // Unknown format - try as BGRA
            eprintln!(
                "Unknown pixel format: 0x{:08x}, trying as BGRA",
                pixel_format
            );
            let desc = TextureDescriptor::new();
            desc.set_texture_type(MTLTextureType::D2);
            desc.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
            desc.set_width(width as u64);
            desc.set_height(height as u64);
            desc.set_storage_mode(MTLStorageMode::Shared);
            desc.set_usage(MTLTextureUsage::ShaderRead);
            let texture: *mut MTLTexture = msg_send![device.as_ptr() as *mut objc::runtime::Object,
                newTextureWithDescriptor: desc.as_ptr() as *mut objc::runtime::Object
                iosurface: iosurface_ptr plane: 0usize];
            if texture.is_null() {
                return None;
            }
            Some(CaptureTextures {
                plane0: Texture::from_ptr(texture),
                plane1: None,
                pixel_format: PIXEL_FORMAT_BGRA,
                width,
                height,
            })
        }
    }
}

pub fn create_pipeline(
    device: &Device,
    library: &Library,
    vertex_fn: &str,
    fragment_fn: &str,
) -> RenderPipelineState {
    let vert = library.get_function(vertex_fn, None).unwrap();
    let frag = library.get_function(fragment_fn, None).unwrap();
    let desc = RenderPipelineDescriptor::new();
    desc.set_vertex_function(Some(&vert));
    desc.set_fragment_function(Some(&frag));
    let attachment = desc.color_attachments().object_at(0).unwrap();
    attachment.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
    attachment.set_blending_enabled(true);
    attachment.set_rgb_blend_operation(MTLBlendOperation::Add);
    attachment.set_alpha_blend_operation(MTLBlendOperation::Add);
    attachment.set_source_rgb_blend_factor(MTLBlendFactor::SourceAlpha);
    attachment.set_source_alpha_blend_factor(MTLBlendFactor::SourceAlpha);
    attachment.set_destination_rgb_blend_factor(MTLBlendFactor::OneMinusSourceAlpha);
    attachment.set_destination_alpha_blend_factor(MTLBlendFactor::OneMinusSourceAlpha);
    device.new_render_pipeline_state(&desc).unwrap()
}

pub const SHADER_SOURCE: &str = r#"
#include <metal_stdlib>
using namespace metal;
struct Vertex { packed_float2 position; packed_float4 color; };
struct Uniforms { float2 viewport_size; float2 texture_size; float time; uint pixel_format; float padding[2]; };
struct VertexOut { float4 position [[position]]; float4 color; };
struct TexturedVertexOut { float4 position [[position]]; float2 texcoord; };
vertex VertexOut vertex_colored(const device Vertex* vertices [[buffer(0)]], constant Uniforms& uniforms [[buffer(1)]], uint vid [[vertex_id]]) {
    VertexOut out; float2 pos = vertices[vid].position; float2 ndc = (pos / uniforms.viewport_size) * 2.0 - 1.0; ndc.y = -ndc.y;
    out.position = float4(ndc, 0.0, 1.0); out.color = float4(vertices[vid].color); return out;
}
fragment float4 fragment_colored(VertexOut in [[stage_in]]) { return in.color; }
vertex TexturedVertexOut vertex_fullscreen(uint vid [[vertex_id]], constant Uniforms& uniforms [[buffer(0)]]) {
    TexturedVertexOut out; float va = uniforms.viewport_size.x / uniforms.viewport_size.y; float ta = uniforms.texture_size.x / uniforms.texture_size.y;
    float sx = ta > va ? 1.0 : ta / va; float sy = ta > va ? va / ta : 1.0;
    float2 positions[4] = { float2(-sx, -sy), float2(sx, -sy), float2(-sx, sy), float2(sx, sy) };
    float2 texcoords[4] = { float2(0.0, 1.0), float2(1.0, 1.0), float2(0.0, 0.0), float2(1.0, 0.0) };
    out.position = float4(positions[vid], 0.0, 1.0); out.texcoord = texcoords[vid]; return out;
}
// BGRA/RGB texture fragment shader
fragment float4 fragment_textured(TexturedVertexOut in [[stage_in]], texture2d<float> tex [[texture(0)]]) {
    constexpr sampler s(mag_filter::linear, min_filter::linear); return tex.sample(s, in.texcoord);
}
// YCbCr to RGB conversion (BT.709 matrix for HD video)
float4 ycbcr_to_rgb(float y, float2 cbcr, bool full_range) {
    // Adjust for video vs full range
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
"#;
