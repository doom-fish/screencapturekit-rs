//! Metal rendering helpers

// Re-export the library's native Metal types
pub use screencapturekit::output::metal::{
    MTLBlendFactor, MTLBlendOperation, MTLLoadAction, MTLPixelFormat, MTLPrimitiveType,
    MTLStoreAction, MetalCapturedTextures, MetalDevice, MetalLayer, MetalLibrary,
    MetalRenderPassDescriptor, MetalRenderPipelineDescriptor, MetalRenderPipelineState,
    ResourceOptions, SHADER_SOURCE,
};

/// Type alias for captured textures
pub type CaptureTextures = MetalCapturedTextures;

/// Create a render pipeline with alpha blending enabled
pub fn create_pipeline(
    device: &MetalDevice,
    library: &MetalLibrary,
    vertex_fn: &str,
    fragment_fn: &str,
) -> MetalRenderPipelineState {
    let vert = library.get_function(vertex_fn).unwrap();
    let frag = library.get_function(fragment_fn).unwrap();
    let desc = MetalRenderPipelineDescriptor::new();
    desc.set_vertex_function(&vert);
    desc.set_fragment_function(&frag);
    desc.set_color_attachment_pixel_format(0, MTLPixelFormat::BGRA8Unorm);
    desc.set_blending_enabled(0, true);
    desc.set_blend_operations(0, MTLBlendOperation::Add, MTLBlendOperation::Add);
    desc.set_blend_factors(
        0,
        MTLBlendFactor::SourceAlpha,
        MTLBlendFactor::OneMinusSourceAlpha,
        MTLBlendFactor::SourceAlpha,
        MTLBlendFactor::OneMinusSourceAlpha,
    );
    device.create_render_pipeline_state(&desc).unwrap()
}
