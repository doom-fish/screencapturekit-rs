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
}

/// Create a Metal texture from an IOSurface (zero-copy)
///
/// # Safety
/// The `iosurface_ptr` must be a valid IOSurface pointer.
pub unsafe fn create_texture_from_iosurface(
    device: &Device,
    iosurface_ptr: *const c_void,
) -> Option<Texture> {
    if iosurface_ptr.is_null() {
        return None;
    }
    let width = IOSurfaceGetWidth(iosurface_ptr);
    let height = IOSurfaceGetHeight(iosurface_ptr);
    if width == 0 || height == 0 {
        return None;
    }
    let desc = TextureDescriptor::new();
    desc.set_texture_type(MTLTextureType::D2);
    desc.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
    desc.set_width(width as u64);
    desc.set_height(height as u64);
    desc.set_storage_mode(MTLStorageMode::Shared);
    desc.set_usage(MTLTextureUsage::ShaderRead);
    let texture: *mut MTLTexture = msg_send![
        device.as_ptr() as *mut objc::runtime::Object,
        newTextureWithDescriptor: desc.as_ptr() as *mut objc::runtime::Object
        iosurface: iosurface_ptr
        plane: 0usize
    ];
    if texture.is_null() {
        None
    } else {
        Some(Texture::from_ptr(texture))
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
struct Uniforms { float2 viewport_size; float2 texture_size; float time; float padding[3]; };
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
fragment float4 fragment_textured(TexturedVertexOut in [[stage_in]], texture2d<float> tex [[texture(0)]]) {
    constexpr sampler s(mag_filter::linear, min_filter::linear); return tex.sample(s, in.texcoord);
}
"#;
