//! Tests for Metal integration and pixel format utilities

use screencapturekit::output::metal::pixel_format;
use screencapturekit::output::metal::Uniforms;

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
