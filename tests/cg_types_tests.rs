//! Core Graphics types tests

use screencapturekit::cg::{CGPoint, CGRect, CGSize};

#[test]
fn test_cgpoint_new() {
    let point = CGPoint::new(10.0, 20.0);
    assert_eq!(point.x, 10.0);
    assert_eq!(point.y, 20.0);
}

#[test]
fn test_cgpoint_zero() {
    let point = CGPoint::zero();
    assert!(point.is_zero());
    assert_eq!(point.x, 0.0);
    assert_eq!(point.y, 0.0);
}

#[test]
fn test_cgpoint_distance() {
    let p1 = CGPoint::new(0.0, 0.0);
    let p2 = CGPoint::new(3.0, 4.0);
    assert_eq!(p1.distance_to(&p2), 5.0);
}

#[test]
fn test_cgpoint_distance_squared() {
    let p1 = CGPoint::new(0.0, 0.0);
    let p2 = CGPoint::new(3.0, 4.0);
    assert_eq!(p1.distance_squared_to(&p2), 25.0);
}

#[test]
fn test_cgpoint_display() {
    let point = CGPoint::new(100.5, 200.5);
    let display = format!("{}", point);
    assert!(display.contains("100.5"));
    assert!(display.contains("200.5"));
}

#[test]
fn test_cgsize_new() {
    let size = CGSize::new(1920.0, 1080.0);
    assert_eq!(size.width, 1920.0);
    assert_eq!(size.height, 1080.0);
}

#[test]
fn test_cgsize_zero() {
    let size = CGSize::zero();
    assert!(size.is_null());
    assert_eq!(size.width, 0.0);
    assert_eq!(size.height, 0.0);
}

#[test]
fn test_cgsize_area() {
    let size = CGSize::new(100.0, 50.0);
    assert_eq!(size.area(), 5000.0);
}

#[test]
fn test_cgsize_aspect_ratio() {
    let size = CGSize::new(1920.0, 1080.0);
    let ratio = size.aspect_ratio();
    assert!((ratio - 16.0 / 9.0).abs() < 0.01);
}

#[test]
fn test_cgsize_aspect_ratio_zero_height() {
    let size = CGSize::new(100.0, 0.0);
    assert_eq!(size.aspect_ratio(), 0.0);
}

#[test]
fn test_cgsize_is_square() {
    let square = CGSize::new(100.0, 100.0);
    let rect = CGSize::new(100.0, 200.0);
    assert!(square.is_square());
    assert!(!rect.is_square());
}

#[test]
fn test_cgsize_is_empty() {
    let empty1 = CGSize::new(0.0, 100.0);
    let empty2 = CGSize::new(100.0, 0.0);
    let empty3 = CGSize::new(-10.0, 100.0);
    let valid = CGSize::new(100.0, 100.0);

    assert!(empty1.is_empty());
    assert!(empty2.is_empty());
    assert!(empty3.is_empty());
    assert!(!valid.is_empty());
}

#[test]
fn test_cgsize_display() {
    let size = CGSize::new(1920.0, 1080.0);
    let display = format!("{}", size);
    assert_eq!(display, "1920x1080");
}

#[test]
fn test_cgrect_new() {
    let rect = CGRect::new(10.0, 20.0, 100.0, 200.0);
    assert_eq!(rect.x, 10.0);
    assert_eq!(rect.y, 20.0);
    assert_eq!(rect.width, 100.0);
    assert_eq!(rect.height, 200.0);
}

#[test]
fn test_cgrect_zero() {
    let rect = CGRect::zero();
    assert!(rect.is_null());
}

#[test]
fn test_cgrect_with_origin_and_size() {
    let origin = CGPoint::new(10.0, 20.0);
    let size = CGSize::new(100.0, 200.0);
    let rect = CGRect::with_origin_and_size(origin, size);

    assert_eq!(rect.x, 10.0);
    assert_eq!(rect.y, 20.0);
    assert_eq!(rect.width, 100.0);
    assert_eq!(rect.height, 200.0);
}

#[test]
fn test_cgrect_origin_and_size() {
    let rect = CGRect::new(10.0, 20.0, 100.0, 200.0);

    let origin = rect.origin();
    assert_eq!(origin.x, 10.0);
    assert_eq!(origin.y, 20.0);

    let size = rect.size();
    assert_eq!(size.width, 100.0);
    assert_eq!(size.height, 200.0);
}

#[test]
fn test_cgrect_center() {
    let rect = CGRect::new(0.0, 0.0, 100.0, 200.0);
    let center = rect.center();
    assert_eq!(center.x, 50.0);
    assert_eq!(center.y, 100.0);
}

#[test]
fn test_cgrect_min_max() {
    let rect = CGRect::new(10.0, 20.0, 100.0, 200.0);
    assert_eq!(rect.min_x(), 10.0);
    assert_eq!(rect.min_y(), 20.0);
    assert_eq!(rect.max_x(), 110.0);
    assert_eq!(rect.max_y(), 220.0);
}

#[test]
fn test_cgrect_mid() {
    let rect = CGRect::new(0.0, 0.0, 100.0, 200.0);
    assert_eq!(rect.mid_x(), 50.0);
    assert_eq!(rect.mid_y(), 100.0);
}

#[test]
fn test_cgrect_is_empty() {
    let empty1 = CGRect::new(0.0, 0.0, 0.0, 100.0);
    let empty2 = CGRect::new(0.0, 0.0, 100.0, 0.0);
    let empty3 = CGRect::new(0.0, 0.0, -10.0, 100.0);
    let valid = CGRect::new(0.0, 0.0, 100.0, 100.0);

    assert!(empty1.is_empty());
    assert!(empty2.is_empty());
    assert!(empty3.is_empty());
    assert!(!valid.is_empty());
}

#[test]
fn test_cgrect_display() {
    let rect = CGRect::new(10.0, 20.0, 100.0, 200.0);
    let display = format!("{}", rect);
    assert!(display.contains("10"));
    assert!(display.contains("20"));
    assert!(display.contains("100"));
    assert!(display.contains("200"));
}

#[test]
fn test_cg_types_default() {
    let point: CGPoint = Default::default();
    let size: CGSize = Default::default();
    let rect: CGRect = Default::default();

    assert!(point.is_zero());
    assert!(size.is_null());
    assert!(rect.is_null());
}

#[test]
fn test_cg_types_hash() {
    use std::collections::HashSet;

    let mut points = HashSet::new();
    points.insert(CGPoint::new(10.0, 20.0));
    points.insert(CGPoint::new(30.0, 40.0));
    assert_eq!(points.len(), 2);

    let mut sizes = HashSet::new();
    sizes.insert(CGSize::new(100.0, 200.0));
    sizes.insert(CGSize::new(300.0, 400.0));
    assert_eq!(sizes.len(), 2);

    let mut rects = HashSet::new();
    rects.insert(CGRect::new(0.0, 0.0, 100.0, 100.0));
    rects.insert(CGRect::new(10.0, 10.0, 200.0, 200.0));
    assert_eq!(rects.len(), 2);
}

#[test]
fn test_cg_types_clone() {
    let point = CGPoint::new(10.0, 20.0);
    let cloned_point = point;
    assert_eq!(point, cloned_point);

    let size = CGSize::new(100.0, 200.0);
    let cloned_size = size;
    assert_eq!(size, cloned_size);

    let rect = CGRect::new(0.0, 0.0, 100.0, 200.0);
    let cloned_rect = rect;
    assert_eq!(rect, cloned_rect);
}
