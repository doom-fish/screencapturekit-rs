//! Metal rendering helpers

use std::mem::size_of;

use crate::vertex::Vertex;

// Re-export the library's native Metal types
pub use screencapturekit::metal::{
    MTLBlendFactor, MTLBlendOperation, MTLLoadAction, MTLPixelFormat, MTLPrimitiveType,
    MTLStoreAction, MTLVertexFormat, MTLVertexStepFunction, MetalBuffer, MetalCapturedTextures,
    MetalCommandQueue, MetalDevice, MetalLayer, MetalLibrary, MetalRenderPassDescriptor,
    MetalRenderPipelineDescriptor, MetalRenderPipelineState, MetalVertexDescriptor, SHADER_SOURCE,
};

/// Error type for Metal rendering operations
#[derive(Debug)]
pub enum RenderError {
    /// Shader function not found in library
    FunctionNotFound(String),
    /// Failed to create pipeline state
    PipelineCreationFailed,
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FunctionNotFound(name) => write!(f, "Shader function not found: {name}"),
            Self::PipelineCreationFailed => write!(f, "Failed to create pipeline state"),
        }
    }
}

impl std::error::Error for RenderError {}

/// Type alias for captured textures
pub type CaptureTextures = MetalCapturedTextures;

/// Create a vertex descriptor for the `ColoredVertex` layout
/// - attribute 0: float2 position at offset 0
/// - attribute 1: float4 color at offset 8
pub fn create_vertex_descriptor() -> MetalVertexDescriptor {
    let desc = MetalVertexDescriptor::new();
    // Position: float2 at offset 0
    desc.set_attribute(0, MTLVertexFormat::Float2, 0, 0);
    // Color: float4 at offset 8 (after float2)
    desc.set_attribute(1, MTLVertexFormat::Float4, size_of::<[f32; 2]>(), 0);
    // Layout: stride = sizeof(Vertex), per-vertex step
    desc.set_layout(0, size_of::<Vertex>(), MTLVertexStepFunction::PerVertex);
    desc
}

/// Create a render pipeline with alpha blending enabled
///
/// # Errors
/// Returns an error if shader functions are not found or pipeline creation fails.
pub fn create_pipeline(
    device: &MetalDevice,
    library: &MetalLibrary,
    vertex_fn: &str,
    fragment_fn: &str,
) -> Result<MetalRenderPipelineState, RenderError> {
    let vert = library
        .get_function(vertex_fn)
        .ok_or_else(|| RenderError::FunctionNotFound(vertex_fn.to_string()))?;
    let frag = library
        .get_function(fragment_fn)
        .ok_or_else(|| RenderError::FunctionNotFound(fragment_fn.to_string()))?;
    let desc = MetalRenderPipelineDescriptor::new();
    desc.set_vertex_function(&vert);
    desc.set_fragment_function(&frag);
    // Set vertex descriptor for vertex_colored shader
    let vertex_desc = create_vertex_descriptor();
    desc.set_vertex_descriptor(&vertex_desc);
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
    device
        .create_render_pipeline_state(&desc)
        .ok_or(RenderError::PipelineCreationFailed)
}
