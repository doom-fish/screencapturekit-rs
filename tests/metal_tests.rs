//! Tests for Metal integration and pixel format utilities

use screencapturekit::metal::pixel_format;
use screencapturekit::metal::Uniforms;
use screencapturekit::metal::{
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
    use screencapturekit::metal::SHADER_SOURCE;
    assert!(!SHADER_SOURCE.is_empty());
    assert!(SHADER_SOURCE.contains("vertex_fullscreen"));
    assert!(SHADER_SOURCE.contains("fragment_textured"));
    assert!(SHADER_SOURCE.contains("fragment_ycbcr"));
    assert!(SHADER_SOURCE.contains("vertex_colored"));
    assert!(SHADER_SOURCE.contains("fragment_colored"));
}

// ============================================================================
// Metal Device Tests (requires actual GPU)
// ============================================================================

mod metal_device_tests {
    use screencapturekit::metal::MetalDevice;

    #[test]
    fn test_metal_device_system_default() {
        let device = MetalDevice::system_default();
        assert!(device.is_some(), "Should have a Metal device on macOS");
    }

    #[test]
    fn test_metal_device_name() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let name = device.name();
        assert!(!name.is_empty(), "Device should have a name");
        println!("Metal device: {}", name);
    }

    #[test]
    fn test_metal_device_as_ptr() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let ptr = device.as_ptr();
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_metal_device_debug() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let debug_str = format!("{:?}", device);
        assert!(debug_str.contains("MetalDevice"));
    }

    #[test]
    fn test_metal_command_queue_creation() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let queue = device.create_command_queue();
        assert!(queue.is_some(), "Should be able to create command queue");
    }

    #[test]
    fn test_metal_command_queue_as_ptr() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let queue = device.create_command_queue().expect("No command queue");
        let ptr = queue.as_ptr();
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_metal_command_queue_debug() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let queue = device.create_command_queue().expect("No command queue");
        let debug_str = format!("{:?}", queue);
        assert!(debug_str.contains("MetalCommandQueue"));
    }

    #[test]
    fn test_metal_library_creation() {
        use screencapturekit::metal::SHADER_SOURCE;

        let device = MetalDevice::system_default().expect("No Metal device");
        let library = device.create_library_with_source(SHADER_SOURCE);
        assert!(library.is_ok(), "Should compile shaders: {:?}", library);
    }

    #[test]
    fn test_metal_library_get_function() {
        use screencapturekit::metal::SHADER_SOURCE;

        let device = MetalDevice::system_default().expect("No Metal device");
        let library = device
            .create_library_with_source(SHADER_SOURCE)
            .expect("Failed to compile shaders");

        // Test getting vertex and fragment functions
        let vertex_fn = library.get_function("vertex_fullscreen");
        assert!(vertex_fn.is_some(), "Should find vertex_fullscreen");

        let fragment_fn = library.get_function("fragment_textured");
        assert!(fragment_fn.is_some(), "Should find fragment_textured");

        let ycbcr_fn = library.get_function("fragment_ycbcr");
        assert!(ycbcr_fn.is_some(), "Should find fragment_ycbcr");

        // Non-existent function should return None
        let missing_fn = library.get_function("nonexistent_function");
        assert!(missing_fn.is_none());
    }

    #[test]
    fn test_metal_function_debug() {
        use screencapturekit::metal::SHADER_SOURCE;

        let device = MetalDevice::system_default().expect("No Metal device");
        let library = device
            .create_library_with_source(SHADER_SOURCE)
            .expect("Failed to compile shaders");
        let function = library
            .get_function("vertex_fullscreen")
            .expect("No function");

        let debug_str = format!("{:?}", function);
        assert!(debug_str.contains("MetalFunction"));
    }

    #[test]
    fn test_metal_command_buffer() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let queue = device.create_command_queue().expect("No command queue");
        let buffer = queue.command_buffer();
        assert!(buffer.is_some(), "Should create command buffer");
    }

    #[test]
    fn test_metal_command_buffer_commit() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let queue = device.create_command_queue().expect("No command queue");
        let buffer = queue.command_buffer().expect("No command buffer");

        // Commit should not panic
        buffer.commit();
    }
}

// ============================================================================
// Metal Texture Tests (requires IOSurface + GPU)
// ============================================================================

mod metal_texture_tests {
    use screencapturekit::cm::IOSurface;
    use screencapturekit::metal::MetalDevice;

    // Note: IOSurface textures require 16-byte aligned bytesPerRow
    // For BGRA (4 bytes per pixel), width must be multiple of 4

    #[test]
    fn test_create_metal_textures_from_iosurface() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let surface =
            IOSurface::create(128, 128, 0x42475241, 4).expect("Failed to create IOSurface");

        let textures = surface.create_metal_textures(&device);
        assert!(textures.is_some(), "Should create textures");

        let textures = textures.unwrap();
        assert_eq!(textures.width, 128);
        assert_eq!(textures.height, 128);
        assert!(textures.plane1.is_none(), "BGRA should be single-plane");
        assert!(!textures.is_ycbcr());
    }

    #[test]
    fn test_metal_texture_dimensions() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let surface =
            IOSurface::create(256, 192, 0x42475241, 4).expect("Failed to create IOSurface");

        let textures = surface
            .create_metal_textures(&device)
            .expect("Failed to create textures");

        assert_eq!(textures.plane0.width(), 256);
        assert_eq!(textures.plane0.height(), 192);
    }

    #[test]
    fn test_metal_texture_pixel_format() {
        use screencapturekit::metal::MetalPixelFormat;

        let device = MetalDevice::system_default().expect("No Metal device");
        let surface = IOSurface::create(64, 64, 0x42475241, 4).expect("Failed to create IOSurface");

        let textures = surface
            .create_metal_textures(&device)
            .expect("Failed to create textures");

        // BGRA should use BGRA8Unorm
        assert_eq!(textures.plane0.pixel_format(), MetalPixelFormat::BGRA8Unorm);
    }

    #[test]
    fn test_metal_texture_as_ptr() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let surface = IOSurface::create(64, 64, 0x42475241, 4).expect("Failed to create IOSurface");

        let textures = surface
            .create_metal_textures(&device)
            .expect("Failed to create textures");

        let ptr = textures.plane0.as_ptr();
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_metal_texture_debug() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let surface = IOSurface::create(64, 64, 0x42475241, 4).expect("Failed to create IOSurface");

        let textures = surface
            .create_metal_textures(&device)
            .expect("Failed to create textures");

        let debug_str = format!("{:?}", textures.plane0);
        assert!(debug_str.contains("MetalTexture"));
    }

    #[test]
    fn test_captured_textures_debug() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let surface = IOSurface::create(64, 64, 0x42475241, 4).expect("Failed to create IOSurface");

        let textures = surface
            .create_metal_textures(&device)
            .expect("Failed to create textures");

        let debug_str = format!("{:?}", textures);
        assert!(debug_str.contains("CapturedTextures"));
    }

    #[test]
    fn test_texture_params() {
        let surface =
            IOSurface::create(128, 128, 0x42475241, 4).expect("Failed to create IOSurface");

        let params = surface.texture_params();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].width, 128);
        assert_eq!(params[0].height, 128);
        assert_eq!(params[0].plane, 0);
    }

    #[test]
    fn test_texture_params_metal_pixel_format() {
        use screencapturekit::metal::MetalPixelFormat;

        let surface =
            IOSurface::create(128, 128, 0x42475241, 4).expect("Failed to create IOSurface");

        let params = surface.texture_params();
        assert_eq!(
            params[0].metal_pixel_format(),
            MetalPixelFormat::BGRA8Unorm.raw()
        );
    }

    #[test]
    fn test_uniforms_from_captured_textures() {
        use screencapturekit::metal::Uniforms;

        let device = MetalDevice::system_default().expect("No Metal device");
        let surface =
            IOSurface::create(1920, 1080, 0x42475241, 4).expect("Failed to create IOSurface");

        let textures = surface
            .create_metal_textures(&device)
            .expect("Failed to create textures");

        let uniforms = Uniforms::from_captured_textures(1920.0, 1080.0, &textures);

        assert!((uniforms.viewport_size[0] - 1920.0).abs() < f32::EPSILON);
        assert!((uniforms.viewport_size[1] - 1080.0).abs() < f32::EPSILON);
        assert!((uniforms.texture_size[0] - 1920.0).abs() < f32::EPSILON);
        assert!((uniforms.texture_size[1] - 1080.0).abs() < f32::EPSILON);
        assert_eq!(uniforms.pixel_format, 0x42475241);
    }
}

// ============================================================================
// Render Pipeline Tests
// ============================================================================

mod metal_pipeline_tests {
    use screencapturekit::metal::{
        MTLPixelFormat, MetalDevice, MetalRenderPipelineDescriptor, SHADER_SOURCE,
    };

    #[test]
    fn test_render_pipeline_descriptor_creation() {
        let desc = MetalRenderPipelineDescriptor::new();
        let debug_str = format!("{:?}", desc);
        assert!(debug_str.contains("MetalRenderPipelineDescriptor"));
    }

    #[test]
    fn test_render_pipeline_descriptor_with_functions() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let library = device
            .create_library_with_source(SHADER_SOURCE)
            .expect("Failed to compile shaders");

        let vertex_fn = library
            .get_function("vertex_fullscreen")
            .expect("No vertex function");
        let fragment_fn = library
            .get_function("fragment_textured")
            .expect("No fragment function");

        let desc = MetalRenderPipelineDescriptor::new();
        desc.set_vertex_function(&vertex_fn);
        desc.set_fragment_function(&fragment_fn);

        let debug_str = format!("{:?}", desc);
        assert!(debug_str.contains("MetalRenderPipelineDescriptor"));
    }

    #[test]
    fn test_render_pipeline_state_creation() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let library = device
            .create_library_with_source(SHADER_SOURCE)
            .expect("Failed to compile shaders");

        let vertex_fn = library
            .get_function("vertex_fullscreen")
            .expect("No vertex function");
        let fragment_fn = library
            .get_function("fragment_textured")
            .expect("No fragment function");

        let desc = MetalRenderPipelineDescriptor::new();
        desc.set_vertex_function(&vertex_fn);
        desc.set_fragment_function(&fragment_fn);
        desc.set_color_attachment_pixel_format(0, MTLPixelFormat::BGRA8Unorm);

        let pipeline = device.create_render_pipeline_state(&desc);
        assert!(pipeline.is_some(), "Should create pipeline state");
    }

    #[test]
    fn test_render_pipeline_state_debug() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let library = device
            .create_library_with_source(SHADER_SOURCE)
            .expect("Failed to compile shaders");

        let vertex_fn = library
            .get_function("vertex_fullscreen")
            .expect("No vertex function");
        let fragment_fn = library
            .get_function("fragment_textured")
            .expect("No fragment function");

        let desc = MetalRenderPipelineDescriptor::new();
        desc.set_vertex_function(&vertex_fn);
        desc.set_fragment_function(&fragment_fn);
        desc.set_color_attachment_pixel_format(0, MTLPixelFormat::BGRA8Unorm);

        let pipeline = device
            .create_render_pipeline_state(&desc)
            .expect("No pipeline");
        let debug_str = format!("{:?}", pipeline);
        assert!(debug_str.contains("MetalRenderPipelineState"));
    }
}

// ============================================================================
// Metal Layer Tests
// ============================================================================

mod metal_layer_tests {
    use screencapturekit::metal::{MTLPixelFormat, MetalDevice, MetalLayer};

    #[test]
    fn test_metal_layer_creation() {
        let layer = MetalLayer::new();
        let debug_str = format!("{:?}", layer);
        assert!(debug_str.contains("MetalLayer"));
    }

    #[test]
    fn test_metal_layer_default() {
        let layer = MetalLayer::default();
        let ptr = layer.as_ptr();
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_metal_layer_set_device() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let layer = MetalLayer::new();
        layer.set_device(&device);
        // No assertion needed - just verify it doesn't crash
    }

    #[test]
    fn test_metal_layer_set_pixel_format() {
        let layer = MetalLayer::new();
        layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
    }

    #[test]
    fn test_metal_layer_set_drawable_size() {
        let layer = MetalLayer::new();
        layer.set_drawable_size(1920.0, 1080.0);
    }

    #[test]
    fn test_metal_layer_set_presents_with_transaction() {
        let layer = MetalLayer::new();
        layer.set_presents_with_transaction(true);
        layer.set_presents_with_transaction(false);
    }

    #[test]
    fn test_metal_layer_as_ptr() {
        let layer = MetalLayer::new();
        let ptr = layer.as_ptr();
        assert!(!ptr.is_null());
    }
}

// ============================================================================
// Metal Render Pass Descriptor Tests
// ============================================================================

mod metal_render_pass_tests {
    use screencapturekit::cm::IOSurface;
    use screencapturekit::metal::{
        MTLLoadAction, MTLStoreAction, MetalDevice, MetalRenderPassDescriptor,
    };

    #[test]
    fn test_render_pass_descriptor_creation() {
        let desc = MetalRenderPassDescriptor::new();
        let debug_str = format!("{:?}", desc);
        assert!(debug_str.contains("MetalRenderPassDescriptor"));
    }

    #[test]
    fn test_render_pass_descriptor_default() {
        let desc = MetalRenderPassDescriptor::default();
        let ptr = desc.as_ptr();
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_render_pass_set_load_action() {
        let desc = MetalRenderPassDescriptor::new();
        desc.set_color_attachment_load_action(0, MTLLoadAction::Clear);
        desc.set_color_attachment_load_action(0, MTLLoadAction::Load);
        desc.set_color_attachment_load_action(0, MTLLoadAction::DontCare);
    }

    #[test]
    fn test_render_pass_set_store_action() {
        let desc = MetalRenderPassDescriptor::new();
        desc.set_color_attachment_store_action(0, MTLStoreAction::Store);
        desc.set_color_attachment_store_action(0, MTLStoreAction::DontCare);
    }

    #[test]
    fn test_render_pass_set_clear_color() {
        let desc = MetalRenderPassDescriptor::new();
        desc.set_color_attachment_clear_color(0, 0.0, 0.0, 0.0, 1.0);
        desc.set_color_attachment_clear_color(0, 1.0, 0.5, 0.25, 0.75);
    }

    #[test]
    fn test_render_pass_with_texture() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let surface = IOSurface::create(64, 64, 0x42475241, 4).expect("Failed to create IOSurface");

        let textures = surface
            .create_metal_textures(&device)
            .expect("Failed to create textures");

        let desc = MetalRenderPassDescriptor::new();
        desc.set_color_attachment_texture(0, &textures.plane0);
        desc.set_color_attachment_load_action(0, MTLLoadAction::Clear);
        desc.set_color_attachment_store_action(0, MTLStoreAction::Store);
        desc.set_color_attachment_clear_color(0, 0.0, 0.0, 0.0, 1.0);
    }
}

// ============================================================================
// Metal Textures Closure API Tests
// ============================================================================

mod metal_textures_closure_tests {
    use screencapturekit::cm::IOSurface;

    #[test]
    fn test_metal_textures_with_closure() {
        let surface =
            IOSurface::create(128, 128, 0x42475241, 4).expect("Failed to create IOSurface");

        // Use the closure-based API
        let textures = surface.metal_textures(|params, iosurface_ptr| {
            assert_eq!(params.width, 128);
            assert_eq!(params.height, 128);
            assert!(!iosurface_ptr.is_null());
            Some(()) // Return unit as placeholder
        });

        assert!(textures.is_some());
        let textures = textures.unwrap();
        assert_eq!(textures.width, 128);
        assert_eq!(textures.height, 128);
        assert!(textures.plane1.is_none());
    }

    #[test]
    fn test_metal_textures_closure_returns_none() {
        let surface =
            IOSurface::create(128, 128, 0x42475241, 4).expect("Failed to create IOSurface");

        // Closure returns None
        let textures: Option<screencapturekit::metal::CapturedTextures<()>> =
            surface.metal_textures(|_params, _ptr| None);

        assert!(textures.is_none());
    }

    #[test]
    fn test_iosurface_is_ycbcr_biplanar() {
        // BGRA is not YCbCr
        let surface =
            IOSurface::create(128, 128, 0x42475241, 4).expect("Failed to create IOSurface");
        assert!(!surface.is_ycbcr_biplanar());
    }
}

// ============================================================================
// Metal Buffer Tests
// ============================================================================

mod metal_buffer_tests {
    use screencapturekit::metal::{MetalDevice, ResourceOptions};

    #[test]
    fn test_create_buffer() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let buffer = device.create_buffer(256, ResourceOptions::CPU_CACHE_MODE_DEFAULT_CACHE);
        assert!(buffer.is_some());
    }

    #[test]
    fn test_buffer_length() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let buffer = device
            .create_buffer(512, ResourceOptions::CPU_CACHE_MODE_DEFAULT_CACHE)
            .expect("Failed to create buffer");
        assert_eq!(buffer.length(), 512);
    }

    #[test]
    fn test_buffer_contents() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let buffer = device
            .create_buffer(64, ResourceOptions::CPU_CACHE_MODE_DEFAULT_CACHE)
            .expect("Failed to create buffer");
        let ptr = buffer.contents();
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_buffer_as_ptr() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let buffer = device
            .create_buffer(64, ResourceOptions::CPU_CACHE_MODE_DEFAULT_CACHE)
            .expect("Failed to create buffer");
        let ptr = buffer.as_ptr();
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_buffer_debug() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let buffer = device
            .create_buffer(64, ResourceOptions::CPU_CACHE_MODE_DEFAULT_CACHE)
            .expect("Failed to create buffer");
        let debug_str = format!("{:?}", buffer);
        assert!(debug_str.contains("MetalBuffer"));
    }

    #[test]
    fn test_create_buffer_with_data() {
        use screencapturekit::metal::Uniforms;

        let device = MetalDevice::system_default().expect("No Metal device");
        let uniforms = Uniforms::new(1920.0, 1080.0, 1920.0, 1080.0);
        let buffer = device.create_buffer_with_data(&uniforms);
        assert!(buffer.is_some());
        assert!(buffer.unwrap().length() >= std::mem::size_of::<Uniforms>());
    }

    #[test]
    fn test_resource_options_combinations() {
        let device = MetalDevice::system_default().expect("No Metal device");

        // Test different options
        let buffer1 = device.create_buffer(64, ResourceOptions::STORAGE_MODE_SHARED);
        assert!(buffer1.is_some());

        let buffer2 = device.create_buffer(64, ResourceOptions::CPU_CACHE_MODE_DEFAULT_CACHE);
        assert!(buffer2.is_some());
    }
}

// ============================================================================
// Command Buffer and Encoder Tests
// ============================================================================

mod metal_command_tests {
    use screencapturekit::metal::{MTLPixelFormat, MetalDevice, SHADER_SOURCE};

    #[test]
    fn test_command_buffer_debug() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let queue = device.create_command_queue().expect("No command queue");
        let buffer = queue.command_buffer().expect("No command buffer");
        let debug_str = format!("{:?}", buffer);
        assert!(debug_str.contains("MetalCommandBuffer"));
    }

    #[test]
    fn test_command_buffer_as_ptr() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let queue = device.create_command_queue().expect("No command queue");
        let buffer = queue.command_buffer().expect("No command buffer");
        let ptr = buffer.as_ptr();
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_library_as_ptr() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let library = device
            .create_library_with_source(SHADER_SOURCE)
            .expect("Failed to compile shaders");
        let ptr = library.as_ptr();
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_library_debug() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let library = device
            .create_library_with_source(SHADER_SOURCE)
            .expect("Failed to compile shaders");
        let debug_str = format!("{:?}", library);
        assert!(debug_str.contains("MetalLibrary"));
    }

    #[test]
    fn test_function_as_ptr() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let library = device
            .create_library_with_source(SHADER_SOURCE)
            .expect("Failed to compile shaders");
        let function = library
            .get_function("vertex_fullscreen")
            .expect("No function");
        let ptr = function.as_ptr();
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_pipeline_state_as_ptr() {
        use screencapturekit::metal::MetalRenderPipelineDescriptor;

        let device = MetalDevice::system_default().expect("No Metal device");
        let library = device
            .create_library_with_source(SHADER_SOURCE)
            .expect("Failed to compile shaders");

        let vertex_fn = library
            .get_function("vertex_fullscreen")
            .expect("No vertex function");
        let fragment_fn = library
            .get_function("fragment_textured")
            .expect("No fragment function");

        let desc = MetalRenderPipelineDescriptor::new();
        desc.set_vertex_function(&vertex_fn);
        desc.set_fragment_function(&fragment_fn);
        desc.set_color_attachment_pixel_format(0, MTLPixelFormat::BGRA8Unorm);

        let pipeline = device
            .create_render_pipeline_state(&desc)
            .expect("No pipeline");
        let ptr = pipeline.as_ptr();
        assert!(!ptr.is_null());
    }
}

// ============================================================================
// Vertex Descriptor Tests
// ============================================================================

mod metal_vertex_descriptor_tests {
    use screencapturekit::metal::{MTLVertexFormat, MTLVertexStepFunction, MetalVertexDescriptor};

    #[test]
    fn test_vertex_descriptor_creation() {
        let desc = MetalVertexDescriptor::new();
        let debug_str = format!("{:?}", desc);
        assert!(debug_str.contains("MetalVertexDescriptor"));
    }

    #[test]
    fn test_vertex_descriptor_default() {
        let desc = MetalVertexDescriptor::default();
        let ptr = desc.as_ptr();
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_vertex_descriptor_set_attribute() {
        let desc = MetalVertexDescriptor::new();
        // Set position attribute: Float4 at offset 0, buffer 0
        desc.set_attribute(0, MTLVertexFormat::Float4, 0, 0);
        // Set texcoord attribute: Float2 at offset 16, buffer 0
        desc.set_attribute(1, MTLVertexFormat::Float2, 16, 0);
    }

    #[test]
    fn test_vertex_descriptor_set_layout() {
        let desc = MetalVertexDescriptor::new();
        // Set buffer 0 layout: stride 24, per-vertex
        desc.set_layout(0, 24, MTLVertexStepFunction::PerVertex);
        // Set buffer 1 layout: stride 64, per-instance
        desc.set_layout(1, 64, MTLVertexStepFunction::PerInstance);
    }

    #[test]
    fn test_vertex_descriptor_full_setup() {
        let desc = MetalVertexDescriptor::new();

        // Position (Float4) at attribute 0
        desc.set_attribute(0, MTLVertexFormat::Float4, 0, 0);
        // Color (Float4) at attribute 1
        desc.set_attribute(1, MTLVertexFormat::Float4, 16, 0);
        // TexCoord (Float2) at attribute 2
        desc.set_attribute(2, MTLVertexFormat::Float2, 32, 0);

        // Layout for buffer 0
        desc.set_layout(0, 40, MTLVertexStepFunction::PerVertex);

        let ptr = desc.as_ptr();
        assert!(!ptr.is_null());
    }
}

// ============================================================================
// Render Encoder Tests (using CAMetalLayer drawable)
// ============================================================================

mod metal_render_encoder_tests {
    use screencapturekit::metal::{
        MTLLoadAction, MTLPixelFormat, MTLPrimitiveType, MTLStoreAction, MetalDevice, MetalLayer,
        MetalRenderPassDescriptor, MetalRenderPipelineDescriptor, ResourceOptions, Uniforms,
        SHADER_SOURCE,
    };

    #[test]
    fn test_layer_drawable_and_texture() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let layer = MetalLayer::new();
        layer.set_device(&device);
        layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        layer.set_drawable_size(64.0, 64.0);

        // Get drawable from layer
        let drawable = layer.next_drawable();
        assert!(drawable.is_some(), "Should get drawable from layer");

        let drawable = drawable.unwrap();
        let texture = drawable.texture();

        assert_eq!(texture.width(), 64);
        assert_eq!(texture.height(), 64);
    }

    #[test]
    fn test_render_command_encoder_creation() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let layer = MetalLayer::new();
        layer.set_device(&device);
        layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        layer.set_drawable_size(64.0, 64.0);

        let drawable = layer.next_drawable().expect("No drawable");
        let texture = drawable.texture();

        let queue = device.create_command_queue().expect("No queue");
        let cmd_buffer = queue.command_buffer().expect("No command buffer");

        let render_pass = MetalRenderPassDescriptor::new();
        render_pass.set_color_attachment_texture(0, &texture);
        render_pass.set_color_attachment_load_action(0, MTLLoadAction::Clear);
        render_pass.set_color_attachment_store_action(0, MTLStoreAction::Store);
        render_pass.set_color_attachment_clear_color(0, 0.0, 0.0, 0.0, 1.0);

        let encoder = cmd_buffer.render_command_encoder(&render_pass);
        assert!(encoder.is_some(), "Should create render encoder");

        let encoder = encoder.unwrap();
        encoder.end_encoding();
    }

    #[test]
    fn test_render_encoder_set_pipeline_state() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let layer = MetalLayer::new();
        layer.set_device(&device);
        layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        layer.set_drawable_size(64.0, 64.0);

        let drawable = layer.next_drawable().expect("No drawable");
        let texture = drawable.texture();

        // Create pipeline
        let library = device
            .create_library_with_source(SHADER_SOURCE)
            .expect("Shader compile failed");
        let vertex_fn = library.get_function("vertex_fullscreen").expect("No fn");
        let fragment_fn = library.get_function("fragment_textured").expect("No fn");

        let pipeline_desc = MetalRenderPipelineDescriptor::new();
        pipeline_desc.set_vertex_function(&vertex_fn);
        pipeline_desc.set_fragment_function(&fragment_fn);
        pipeline_desc.set_color_attachment_pixel_format(0, MTLPixelFormat::BGRA8Unorm);

        let pipeline = device
            .create_render_pipeline_state(&pipeline_desc)
            .expect("No pipeline");

        // Create encoder and set pipeline
        let queue = device.create_command_queue().expect("No queue");
        let cmd_buffer = queue.command_buffer().expect("No buffer");

        let render_pass = MetalRenderPassDescriptor::new();
        render_pass.set_color_attachment_texture(0, &texture);
        render_pass.set_color_attachment_load_action(0, MTLLoadAction::Clear);
        render_pass.set_color_attachment_store_action(0, MTLStoreAction::Store);

        let encoder = cmd_buffer
            .render_command_encoder(&render_pass)
            .expect("No encoder");
        encoder.set_render_pipeline_state(&pipeline);
        encoder.end_encoding();

        cmd_buffer.commit();
    }

    #[test]
    fn test_render_encoder_set_buffers() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let layer = MetalLayer::new();
        layer.set_device(&device);
        layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        layer.set_drawable_size(64.0, 64.0);

        let drawable = layer.next_drawable().expect("No drawable");
        let texture = drawable.texture();

        // Create buffers
        let uniforms = Uniforms::new(64.0, 64.0, 64.0, 64.0);
        let uniform_buffer = device
            .create_buffer_with_data(&uniforms)
            .expect("No buffer");

        let vertex_data = [0.0f32; 24]; // 6 vertices * 4 floats
        let vertex_buffer = device
            .create_buffer(
                std::mem::size_of_val(&vertex_data),
                ResourceOptions::CPU_CACHE_MODE_DEFAULT_CACHE,
            )
            .expect("No buffer");

        // Create encoder
        let queue = device.create_command_queue().expect("No queue");
        let cmd_buffer = queue.command_buffer().expect("No buffer");

        let render_pass = MetalRenderPassDescriptor::new();
        render_pass.set_color_attachment_texture(0, &texture);
        render_pass.set_color_attachment_load_action(0, MTLLoadAction::Clear);
        render_pass.set_color_attachment_store_action(0, MTLStoreAction::Store);

        let encoder = cmd_buffer
            .render_command_encoder(&render_pass)
            .expect("No encoder");

        encoder.set_vertex_buffer(&vertex_buffer, 0, 0);
        encoder.set_fragment_buffer(&uniform_buffer, 0, 0);
        encoder.end_encoding();

        cmd_buffer.commit();
    }

    #[test]
    fn test_render_encoder_set_fragment_texture() {
        use screencapturekit::cm::IOSurface;

        let device = MetalDevice::system_default().expect("No Metal device");
        let layer = MetalLayer::new();
        layer.set_device(&device);
        layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        layer.set_drawable_size(64.0, 64.0);

        let drawable = layer.next_drawable().expect("No drawable");
        let target_texture = drawable.texture();

        // Create source texture from IOSurface
        let surface = IOSurface::create(64, 64, 0x42475241, 4).expect("Failed to create IOSurface");
        let source_textures = surface.create_metal_textures(&device).expect("No textures");

        // Create encoder
        let queue = device.create_command_queue().expect("No queue");
        let cmd_buffer = queue.command_buffer().expect("No buffer");

        let render_pass = MetalRenderPassDescriptor::new();
        render_pass.set_color_attachment_texture(0, &target_texture);
        render_pass.set_color_attachment_load_action(0, MTLLoadAction::Clear);
        render_pass.set_color_attachment_store_action(0, MTLStoreAction::Store);

        let encoder = cmd_buffer
            .render_command_encoder(&render_pass)
            .expect("No encoder");

        encoder.set_fragment_texture(&source_textures.plane0, 0);
        encoder.end_encoding();

        cmd_buffer.commit();
    }

    #[test]
    fn test_render_encoder_draw_primitives() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let layer = MetalLayer::new();
        layer.set_device(&device);
        layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        layer.set_drawable_size(64.0, 64.0);

        let drawable = layer.next_drawable().expect("No drawable");
        let texture = drawable.texture();

        // Create pipeline
        let library = device
            .create_library_with_source(SHADER_SOURCE)
            .expect("Shader compile failed");
        let vertex_fn = library.get_function("vertex_fullscreen").expect("No fn");
        let fragment_fn = library.get_function("fragment_textured").expect("No fn");

        let pipeline_desc = MetalRenderPipelineDescriptor::new();
        pipeline_desc.set_vertex_function(&vertex_fn);
        pipeline_desc.set_fragment_function(&fragment_fn);
        pipeline_desc.set_color_attachment_pixel_format(0, MTLPixelFormat::BGRA8Unorm);

        let pipeline = device
            .create_render_pipeline_state(&pipeline_desc)
            .expect("No pipeline");

        // Create encoder and draw
        let queue = device.create_command_queue().expect("No queue");
        let cmd_buffer = queue.command_buffer().expect("No buffer");

        let render_pass = MetalRenderPassDescriptor::new();
        render_pass.set_color_attachment_texture(0, &texture);
        render_pass.set_color_attachment_load_action(0, MTLLoadAction::Clear);
        render_pass.set_color_attachment_store_action(0, MTLStoreAction::Store);

        let encoder = cmd_buffer
            .render_command_encoder(&render_pass)
            .expect("No encoder");

        encoder.set_render_pipeline_state(&pipeline);
        encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, 6);
        encoder.end_encoding();

        cmd_buffer.commit();
    }

    #[test]
    fn test_drawable_debug() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let layer = MetalLayer::new();
        layer.set_device(&device);
        layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        layer.set_drawable_size(32.0, 32.0);

        let drawable = layer.next_drawable().expect("No drawable");
        let debug_str = format!("{:?}", drawable);
        assert!(debug_str.contains("MetalDrawable"));
    }

    #[test]
    fn test_render_encoder_debug() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let layer = MetalLayer::new();
        layer.set_device(&device);
        layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        layer.set_drawable_size(32.0, 32.0);

        let drawable = layer.next_drawable().expect("No drawable");
        let texture = drawable.texture();

        let queue = device.create_command_queue().expect("No queue");
        let cmd_buffer = queue.command_buffer().expect("No buffer");

        let render_pass = MetalRenderPassDescriptor::new();
        render_pass.set_color_attachment_texture(0, &texture);
        render_pass.set_color_attachment_load_action(0, MTLLoadAction::Clear);
        render_pass.set_color_attachment_store_action(0, MTLStoreAction::Store);

        let encoder = cmd_buffer
            .render_command_encoder(&render_pass)
            .expect("No encoder");

        let debug_str = format!("{:?}", encoder);
        assert!(debug_str.contains("MetalRenderCommandEncoder"));

        encoder.end_encoding();
    }

    #[test]
    fn test_render_encoder_as_ptr() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let layer = MetalLayer::new();
        layer.set_device(&device);
        layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        layer.set_drawable_size(32.0, 32.0);

        let drawable = layer.next_drawable().expect("No drawable");
        let texture = drawable.texture();

        let queue = device.create_command_queue().expect("No queue");
        let cmd_buffer = queue.command_buffer().expect("No buffer");

        let render_pass = MetalRenderPassDescriptor::new();
        render_pass.set_color_attachment_texture(0, &texture);
        render_pass.set_color_attachment_load_action(0, MTLLoadAction::Clear);
        render_pass.set_color_attachment_store_action(0, MTLStoreAction::Store);

        let encoder = cmd_buffer
            .render_command_encoder(&render_pass)
            .expect("No encoder");

        let ptr = encoder.as_ptr();
        assert!(!ptr.is_null());

        encoder.end_encoding();
    }
}

#[test]
fn test_iosurface_info() {
    use screencapturekit::cm::IOSurface;

    let surface = IOSurface::create(200, 100, 0x42475241, 4).expect("Failed to create IOSurface");
    let info = surface.info();

    assert_eq!(info.width, 200);
    assert_eq!(info.height, 100);
    assert!(info.bytes_per_row >= 800);
    assert_eq!(info.pixel_format.as_u32(), 0x42475241);
    assert_eq!(info.plane_count, 0);
    assert!(info.planes.is_empty());
}

#[test]
fn test_iosurface_info_debug() {
    use screencapturekit::cm::IOSurface;

    let surface = IOSurface::create(50, 50, 0x42475241, 4).expect("Failed to create IOSurface");
    let info = surface.info();

    let debug_str = format!("{:?}", info);
    assert!(debug_str.contains("IOSurfaceInfo"));
    assert!(debug_str.contains("width"));
    assert!(debug_str.contains("height"));
}

#[test]
fn test_iosurface_info_clone() {
    use screencapturekit::cm::IOSurface;

    let surface = IOSurface::create(100, 100, 0x42475241, 4).expect("Failed to create IOSurface");
    let info = surface.info();
    let cloned = info.clone();

    assert_eq!(info.width, cloned.width);
    assert_eq!(info.height, cloned.height);
}

#[test]
fn test_plane_info_debug() {
    use screencapturekit::metal::PlaneInfo;

    let plane = PlaneInfo {
        index: 0,
        width: 100,
        height: 100,
        bytes_per_row: 400,
    };

    let debug_str = format!("{:?}", plane);
    assert!(debug_str.contains("PlaneInfo"));
    assert!(debug_str.contains("index"));
}

#[test]
fn test_pixel_format_all_constants() {
    assert_eq!(pixel_format::BGRA.as_u32(), u32::from_be_bytes(*b"BGRA"));
    assert_eq!(pixel_format::L10R.as_u32(), u32::from_be_bytes(*b"l10r"));
    assert_eq!(
        pixel_format::YCBCR_420V.as_u32(),
        u32::from_be_bytes(*b"420v")
    );
    assert_eq!(
        pixel_format::YCBCR_420F.as_u32(),
        u32::from_be_bytes(*b"420f")
    );
}

#[test]
fn test_pixel_format_is_ycbcr_with_raw_u32() {
    // Test with raw u32 values
    let ycbcr_420v_raw = u32::from_be_bytes(*b"420v");
    let ycbcr_420f_raw = u32::from_be_bytes(*b"420f");
    let bgra_raw = u32::from_be_bytes(*b"BGRA");

    assert!(pixel_format::is_ycbcr_biplanar(ycbcr_420v_raw));
    assert!(pixel_format::is_ycbcr_biplanar(ycbcr_420f_raw));
    assert!(!pixel_format::is_ycbcr_biplanar(bgra_raw));
}

#[test]
fn test_pixel_format_l10r_not_ycbcr() {
    assert!(!pixel_format::is_ycbcr_biplanar(pixel_format::L10R));
}

#[test]
fn test_metal_pixel_format_default() {
    let default = MTLPixelFormat::default();
    // Default is BGRA8Unorm
    assert_eq!(default.raw(), 80);
}

#[test]
fn test_metal_pixel_format_debug() {
    let format = MTLPixelFormat::BGRA8Unorm;
    let debug_str = format!("{:?}", format);
    assert!(debug_str.contains("BGRA8Unorm"));
}

#[test]
fn test_metal_pixel_format_copy() {
    let format = MetalPixelFormat::BGRA8Unorm;
    let copy = format;
    assert_eq!(format, copy);
}

#[test]
fn test_uniforms_size() {
    // Uniforms should be a known size for GPU buffer alignment
    let size = std::mem::size_of::<Uniforms>();
    // Minimum: 2 floats viewport + 2 floats texture + 1 float time + 1 u32 format = 24 bytes
    assert!(size >= 24);
}

#[test]
fn test_mtl_vertex_format_copy() {
    let format = MTLVertexFormat::Float4;
    let copy = format;
    assert_eq!(format.raw(), copy.raw());
}

#[test]
fn test_mtl_primitive_type_copy() {
    let ptype = MTLPrimitiveType::Triangle;
    let copy = ptype;
    assert_eq!(ptype.raw(), copy.raw());
}

// YCbCr surface and Metal texture tests
mod ycbcr_tests {
    use screencapturekit::cm::{IOSurface, PlaneProperties};
    use screencapturekit::metal::MetalDevice;

    fn create_ycbcr_surface(width: usize, height: usize) -> Option<IOSurface> {
        let y_bytes_per_row = (width + 63) & !63;
        let y_size = y_bytes_per_row * height;

        let uv_width = width / 2;
        let uv_height = height / 2;
        let uv_bytes_per_row = ((uv_width * 2) + 63) & !63;
        let uv_size = uv_bytes_per_row * uv_height;

        let alloc_size = y_size + uv_size;

        let planes = [
            PlaneProperties {
                width,
                height,
                bytes_per_row: y_bytes_per_row,
                bytes_per_element: 1,
                offset: 0,
                size: y_size,
            },
            PlaneProperties {
                width: uv_width,
                height: uv_height,
                bytes_per_row: uv_bytes_per_row,
                bytes_per_element: 2,
                offset: y_size,
                size: uv_size,
            },
        ];

        IOSurface::create_with_properties(
            width,
            height,
            0x34323076, // '420v'
            1,
            y_bytes_per_row,
            alloc_size,
            Some(&planes),
        )
    }

    #[test]
    fn test_ycbcr_surface_texture_params() {
        let width = 128;
        let height = 128;

        if let Some(surface) = create_ycbcr_surface(width, height) {
            let params = surface.texture_params();

            // YCbCr should have 2 texture params
            assert_eq!(params.len(), 2);

            // First plane (Y) should be full size
            assert_eq!(params[0].width, width);
            assert_eq!(params[0].height, height);
            assert_eq!(params[0].plane, 0);

            // Second plane (UV) should be half size
            assert_eq!(params[1].width, width / 2);
            assert_eq!(params[1].height, height / 2);
            assert_eq!(params[1].plane, 1);
        }
    }

    #[test]
    fn test_ycbcr_metal_textures() {
        if let Some(_device) = MetalDevice::system_default() {
            if let Some(surface) = create_ycbcr_surface(64, 64) {
                // Use metal_textures with a simple closure
                let textures =
                    surface.metal_textures(|_params, _iosurface_ptr| -> Option<()> { None });
                assert!(textures.is_none());
            }
        }
    }

    #[test]
    fn test_ycbcr_420f_surface() {
        let width = 64;
        let height = 64;

        let y_bytes_per_row = (width + 63) & !63;
        let y_size = y_bytes_per_row * height;
        let uv_width = width / 2;
        let uv_height = height / 2;
        let uv_bytes_per_row = ((uv_width * 2) + 63) & !63;
        let uv_size = uv_bytes_per_row * uv_height;
        let alloc_size = y_size + uv_size;

        let planes = [
            PlaneProperties {
                width,
                height,
                bytes_per_row: y_bytes_per_row,
                bytes_per_element: 1,
                offset: 0,
                size: y_size,
            },
            PlaneProperties {
                width: uv_width,
                height: uv_height,
                bytes_per_row: uv_bytes_per_row,
                bytes_per_element: 2,
                offset: y_size,
                size: uv_size,
            },
        ];

        if let Some(surface) = IOSurface::create_with_properties(
            width,
            height,
            0x34323066, // '420f'
            1,
            y_bytes_per_row,
            alloc_size,
            Some(&planes),
        ) {
            let params = surface.texture_params();
            assert_eq!(params.len(), 2);
            assert!(surface.is_ycbcr_biplanar());
        }
    }

    #[test]
    fn test_unknown_format_fallback() {
        let width = 64;
        let height = 64;
        let bytes_per_row = (width * 4 + 63) & !63;
        let alloc_size = bytes_per_row * height;

        if let Some(surface) = IOSurface::create_with_properties(
            width,
            height,
            0x12345678, // Unknown format
            4,
            bytes_per_row,
            alloc_size,
            None,
        ) {
            let params = surface.texture_params();
            // Should fall back to BGRA single plane
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].plane, 0);
        }
    }

    #[test]
    fn test_iosurface_info() {
        let width = 128;
        let height = 96;

        if let Some(surface) = create_ycbcr_surface(width, height) {
            let info = surface.info();

            assert_eq!(info.width, width);
            assert_eq!(info.height, height);
            assert_eq!(info.plane_count, 2);
            assert!(info.planes.len() >= 2);
        }
    }
}
