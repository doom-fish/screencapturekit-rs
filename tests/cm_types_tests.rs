#![allow(clippy::pedantic, clippy::nursery)]
//! Core Media types tests
//!
//! Tests for CMTime, CMSampleTimingInfo, and related types

use screencapturekit::cm::{CMSampleTimingInfo, CMTime};

#[test]
fn test_cmtime_creation() {
    let time = CMTime::new(1, 30);
    assert!(time.is_valid());
    
    let zero = CMTime::ZERO;
    assert!(zero.is_zero());
    assert!(zero.is_valid());
    
    let invalid = CMTime::INVALID;
    assert!(!invalid.is_valid());
}

#[test]
fn test_cmtime_special_values() {
    let pos_inf = CMTime::positive_infinity();
    assert!(pos_inf.is_positive_infinity());
    assert!(!pos_inf.is_negative_infinity());
    assert!(!pos_inf.is_indefinite());
    
    let neg_inf = CMTime::negative_infinity();
    assert!(neg_inf.is_negative_infinity());
    assert!(!neg_inf.is_positive_infinity());
    
    let indefinite = CMTime::indefinite();
    assert!(indefinite.is_indefinite());
    assert!(!indefinite.is_positive_infinity());
}

#[test]
fn test_cmtime_const_functions() {
    const FRAME_DURATION: CMTime = CMTime::new(1, 30);
    const ZERO: CMTime = CMTime::ZERO;
    const INVALID: CMTime = CMTime::INVALID;
    
    assert!(FRAME_DURATION.is_valid());
    assert!(ZERO.is_zero());
    assert!(!INVALID.is_valid());
}

#[test]
fn test_cmtime_const_equality() {
    const TIME1: CMTime = CMTime::new(1, 30);
    const TIME2: CMTime = CMTime::new(1, 30);
    const TIME3: CMTime = CMTime::new(2, 30);
    
    assert!(TIME1.equals(&TIME2));
    assert!(!TIME1.equals(&TIME3));
}

#[test]
fn test_cmtime_display() {
    let time = CMTime::new(1, 30);
    let display = format!("{:?}", time);
    // Just verify it doesn't crash
    assert!(!display.is_empty());
    
    let zero = CMTime::ZERO;
    let zero_display = format!("{:?}", zero);
    assert!(!zero_display.is_empty());
}

#[test]
fn test_cmsample_timing_info_creation() {
    let timing = CMSampleTimingInfo::with_times(
        CMTime::new(1, 30),
        CMTime::new(0, 30),
        CMTime::new(1, 30),
    );
    
    // Verify valid times
    assert!(timing.has_valid_presentation_time());
    assert!(timing.has_valid_decode_time());
}

#[test]
fn test_cmsample_timing_info_invalid() {
    let timing = CMSampleTimingInfo::with_times(
        CMTime::INVALID,
        CMTime::INVALID,
        CMTime::INVALID,
    );
    
    assert!(!timing.has_valid_presentation_time());
    assert!(!timing.has_valid_decode_time());
}

#[test]
fn test_cmsample_timing_info_display() {
    let timing = CMSampleTimingInfo::with_times(
        CMTime::new(1, 30),
        CMTime::new(0, 30),
        CMTime::new(1, 30),
    );
    
    let display = format!("{:?}", timing);
    // Just verify it doesn't crash
    assert!(!display.is_empty());
}

#[test]
fn test_cmsample_timing_default() {
    let timing = CMSampleTimingInfo::with_times(
        CMTime::ZERO,
        CMTime::ZERO,
        CMTime::ZERO,
    );
    
    assert!(timing.is_valid());
}
