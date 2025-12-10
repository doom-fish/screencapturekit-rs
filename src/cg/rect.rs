//! `CGRect` type for 2D rectangles

use std::fmt;

use super::{CGPoint, CGSize};

/// `CGRect` representation
///
/// Represents a rectangle with origin (x, y) and dimensions (width, height).
///
/// # Examples
///
/// ```
/// use screencapturekit::cg::CGRect;
///
/// let rect = CGRect::new(10.0, 20.0, 100.0, 200.0);
/// assert_eq!(rect.x, 10.0);
/// assert_eq!(rect.width, 100.0);
/// assert_eq!(rect.max_x(), 110.0);
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CGRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl std::hash::Hash for CGRect {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
        self.width.to_bits().hash(state);
        self.height.to_bits().hash(state);
    }
}

impl Eq for CGRect {}

impl CGRect {
    /// Create a new rectangle
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::cg::CGRect;
    ///
    /// let rect = CGRect::new(0.0, 0.0, 1920.0, 1080.0);
    /// assert_eq!(rect.width, 1920.0);
    /// ```
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create a zero-sized rectangle at origin
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::cg::CGRect;
    ///
    /// let rect = CGRect::zero();
    /// assert!(rect.is_null());
    /// ```
    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    /// Create a rect with origin and size
    pub const fn with_origin_and_size(origin: CGPoint, size: CGSize) -> Self {
        Self {
            x: origin.x,
            y: origin.y,
            width: size.width,
            height: size.height,
        }
    }

    /// Get the origin point
    pub const fn origin(&self) -> CGPoint {
        CGPoint::new(self.x, self.y)
    }

    /// Get the size
    pub const fn size(&self) -> CGSize {
        CGSize::new(self.width, self.height)
    }

    /// Get the center point
    pub const fn center(&self) -> CGPoint {
        CGPoint::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Get the minimum X coordinate
    pub const fn min_x(&self) -> f64 {
        self.x
    }

    /// Get the minimum Y coordinate
    pub const fn min_y(&self) -> f64 {
        self.y
    }

    /// Get the maximum X coordinate
    pub const fn max_x(&self) -> f64 {
        self.x + self.width
    }

    /// Get the maximum Y coordinate
    pub const fn max_y(&self) -> f64 {
        self.y + self.height
    }

    /// Get the mid X coordinate
    pub const fn mid_x(&self) -> f64 {
        self.x + self.width / 2.0
    }

    /// Get the mid Y coordinate
    pub const fn mid_y(&self) -> f64 {
        self.y + self.height / 2.0
    }

    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    /// Check if rect is null (both position and size are zero)
    pub const fn is_null(&self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.width == 0.0 && self.height == 0.0
    }
}

impl Default for CGRect {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for CGRect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}, {}, {}, {})",
            self.x, self.y, self.width, self.height
        )
    }
}
