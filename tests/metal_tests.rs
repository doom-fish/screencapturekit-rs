//! Tests for Metal integration and pixel format utilities

use screencapturekit::output::metal::pixel_format;
use screencapturekit::output::metal::Uniforms;
use screencapturekit::output::metal::{
    MTLBlendFactor, MTLBlendOperation, MTLLoadAction, MTLPixelFormat, MTLPrimitiveType,
    MTLStoreAction, MTLVertexFormat, MTLVertexStepFunction, MetalPixelFormat, ResourceOptions,
};

#[test]
fn test_pixel_format_detection() {
    assert!(pixel_format::is_ycbcr_biplanar(pixel_format::YCBCR_420V));
    assert!(pixel_format::is_ycbcr_biplanar(pixel_format::YCBCR_420F));
    assert!(!pixel_format::is_ycbcr_biplanar(pixel_format::BGRA));
    assert!(!pixel_format::is_ycbcr_biplanar(pixel_format::L10R));
}

#[test]
fn test_pixel_format_display() {
    assert_eq!(pixel_format::BGRA.display(), "BGRA");
    assert_eq!(pixel_format::YCBCR_420V.display(), "420v");
    assert_eq!(pixel_format::YCBCR_420F.display(), "420f");
}

#[test]
fn test_uniforms_creation() {
    let uniforms = Uniforms::new(1920.0, 1080.0, 1920.0, 1080.0)
        .with_pixel_format(pixel_format::BGRA)
        .with_time(1.5);

    // Use epsilon comparison for floats
    assert!((uniforms.viewport_size[0] - 1920.0).abs() < f32::EPSILON);
    assert!((uniforms.viewport_size[1] - 1080.0).abs() < f32::EPSILON);
    assert!((uniforms.texture_size[0] - 1920.0).abs() < f32::EPSILON);
    assert!((uniforms.texture_size[1] - 1080.0).abs() < f32::EPSILON);
    assert_eq!(uniforms.pixel_format, pixel_format::BGRA.as_u32());
    assert!((uniforms.time - 1.5).abs() < f32::EPSILON);
}

#[test]
fn test_uniforms_default() {
    let uniforms = Uniforms::default();
    assert!((uniforms.viewport_size[0] - 0.0).abs() < f32::EPSILON);
    assert!((uniforms.viewport_size[1] - 0.0).abs() < f32::EPSILON);
    assert!((uniforms.time - 0.0).abs() < f32::EPSILON);
    assert_eq!(uniforms.pixel_format, 0);
}

#[test]
fn test_uniforms_with_pixel_format_raw_u32() {
    let uniforms = Uniforms::new(100.0, 100.0, 100.0, 100.0).with_pixel_format(0x42475241_u32);
    assert_eq!(uniforms.pixel_format, 0x42475241);
}

#[test]
fn test_pixel_format_full_range() {
    assert!(pixel_format::is_full_range(pixel_format::YCBCR_420F));
    assert!(!pixel_format::is_full_range(pixel_format::YCBCR_420V));
    assert!(!pixel_format::is_full_range(pixel_format::BGRA));
}

#[test]
fn test_metal_pixel_format_enum() {
    assert_eq!(MetalPixelFormat::BGRA8Unorm.raw(), 80);
    assert_eq!(MetalPixelFormat::BGR10A2Unorm.raw(), 94);
    assert_eq!(MetalPixelFormat::R8Unorm.raw(), 10);
    assert_eq!(MetalPixelFormat::RG8Unorm.raw(), 30);
}

#[test]
fn test_metal_pixel_format_from_raw() {
    assert_eq!(
        MetalPixelFormat::from_raw(80),
        Some(MetalPixelFormat::BGRA8Unorm)
    );
    assert_eq!(
        MetalPixelFormat::from_raw(94),
        Some(MetalPixelFormat::BGR10A2Unorm)
    );
    assert_eq!(
        MetalPixelFormat::from_raw(10),
        Some(MetalPixelFormat::R8Unorm)
    );
    assert_eq!(
        MetalPixelFormat::from_raw(30),
        Some(MetalPixelFormat::RG8Unorm)
    );
    assert_eq!(MetalPixelFormat::from_raw(999), None);
}

#[test]
fn test_mtl_load_action_values() {
    assert_eq!(MTLLoadAction::DontCare as u64, 0);
    assert_eq!(MTLLoadAction::Load as u64, 1);
    assert_eq!(MTLLoadAction::Clear as u64, 2);
}

#[test]
fn test_mtl_store_action_values() {
    assert_eq!(MTLStoreAction::DontCare as u64, 0);
    assert_eq!(MTLStoreAction::Store as u64, 1);
}

#[test]
fn test_mtl_pixel_format_values() {
    assert_eq!(MTLPixelFormat::Invalid.raw(), 0);
    assert_eq!(MTLPixelFormat::BGRA8Unorm.raw(), 80);
    assert_eq!(MTLPixelFormat::BGR10A2Unorm.raw(), 94);
    assert_eq!(MTLPixelFormat::R8Unorm.raw(), 10);
    assert_eq!(MTLPixelFormat::RG8Unorm.raw(), 30);
}

#[test]
fn test_mtl_vertex_format_values() {
    assert_eq!(MTLVertexFormat::Invalid.raw(), 0);
    assert_eq!(MTLVertexFormat::Float2.raw(), 29);
    assert_eq!(MTLVertexFormat::Float3.raw(), 30);
    assert_eq!(MTLVertexFormat::Float4.raw(), 31);
}

#[test]
fn test_mtl_vertex_step_function_values() {
    assert_eq!(MTLVertexStepFunction::Constant.raw(), 0);
    assert_eq!(MTLVertexStepFunction::PerVertex.raw(), 1);
    assert_eq!(MTLVertexStepFunction::PerInstance.raw(), 2);
}

#[test]
fn test_mtl_primitive_type_values() {
    assert_eq!(MTLPrimitiveType::Point.raw(), 0);
    assert_eq!(MTLPrimitiveType::Line.raw(), 1);
    assert_eq!(MTLPrimitiveType::LineStrip.raw(), 2);
    assert_eq!(MTLPrimitiveType::Triangle.raw(), 3);
    assert_eq!(MTLPrimitiveType::TriangleStrip.raw(), 4);
}

#[test]
fn test_mtl_blend_operation_values() {
    assert_eq!(MTLBlendOperation::Add as u64, 0);
    assert_eq!(MTLBlendOperation::Subtract as u64, 1);
    assert_eq!(MTLBlendOperation::ReverseSubtract as u64, 2);
    assert_eq!(MTLBlendOperation::Min as u64, 3);
    assert_eq!(MTLBlendOperation::Max as u64, 4);
}

#[test]
fn test_mtl_blend_factor_values() {
    assert_eq!(MTLBlendFactor::Zero as u64, 0);
    assert_eq!(MTLBlendFactor::One as u64, 1);
    assert_eq!(MTLBlendFactor::SourceColor as u64, 2);
    assert_eq!(MTLBlendFactor::OneMinusSourceColor as u64, 3);
    assert_eq!(MTLBlendFactor::SourceAlpha as u64, 4);
    assert_eq!(MTLBlendFactor::OneMinusSourceAlpha as u64, 5);
    assert_eq!(MTLBlendFactor::DestinationColor as u64, 6);
    assert_eq!(MTLBlendFactor::OneMinusDestinationColor as u64, 7);
    assert_eq!(MTLBlendFactor::DestinationAlpha as u64, 8);
    assert_eq!(MTLBlendFactor::OneMinusDestinationAlpha as u64, 9);
}

#[test]
fn test_resource_options() {
    // Just verify the constants exist and can be used
    let _ = ResourceOptions::CPU_CACHE_MODE_DEFAULT_CACHE;
    let _ = ResourceOptions::STORAGE_MODE_SHARED;
    let _ = ResourceOptions::STORAGE_MODE_MANAGED;
}

#[test]
fn test_uniforms_debug() {
    let uniforms = Uniforms::new(100.0, 100.0, 100.0, 100.0);
    let debug_str = format!("{uniforms:?}");
    assert!(debug_str.contains("Uniforms"));
    assert!(debug_str.contains("viewport_size"));
}

#[test]
fn test_uniforms_clone() {
    let uniforms = Uniforms::new(1920.0, 1080.0, 1920.0, 1080.0).with_time(2.0);
    let cloned = uniforms;
    assert!((cloned.time - 2.0).abs() < f32::EPSILON);
}

#[test]
fn test_default_impls() {
    let _ = MTLLoadAction::default();
    let _ = MTLStoreAction::default();
    let _ = MTLPixelFormat::default();
    let _ = MTLVertexFormat::default();
    let _ = MTLVertexStepFunction::default();
    let _ = MTLPrimitiveType::default();
    let _ = MTLBlendOperation::default();
    let _ = MTLBlendFactor::default();
    let _ = ResourceOptions::default();
}

#[test]
fn test_shader_source_exists() {
    use screencapturekit::output::metal::SHADER_SOURCE;
    assert!(!SHADER_SOURCE.is_empty());
    assert!(SHADER_SOURCE.contains("vertex_fullscreen"));
    assert!(SHADER_SOURCE.contains("fragment_textured"));
    assert!(SHADER_SOURCE.contains("fragment_ycbcr"));
    assert!(SHADER_SOURCE.contains("vertex_colored"));
    assert!(SHADER_SOURCE.contains("fragment_colored"));
}
