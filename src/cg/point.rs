//! `CGPoint` type for 2D coordinates

use std::fmt;

/// `CGPoint` representation
///
/// Represents a point in 2D space.
///
/// # Examples
///
/// ```
/// use screencapturekit::cg::CGPoint;
///
/// let p1 = CGPoint::new(0.0, 0.0);
/// let p2 = CGPoint::new(3.0, 4.0);
/// assert_eq!(p1.distance_to(&p2), 5.0);
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CGPoint {
    pub x: f64,
    pub y: f64,
}

impl std::hash::Hash for CGPoint {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

impl Eq for CGPoint {}

impl CGPoint {
    /// Create a new point
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::cg::CGPoint;
    ///
    /// let point = CGPoint::new(100.0, 200.0);
    /// assert_eq!(point.x, 100.0);
    /// ```
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Create a point at origin (0, 0)
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::cg::CGPoint;
    ///
    /// let point = CGPoint::zero();
    /// assert!(point.is_zero());
    /// ```
    pub const fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    /// Check if point is at origin (0, 0)
    pub const fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }

    /// Calculate distance to another point
    pub fn distance_to(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx.hypot(dy)
    }

    /// Calculate squared distance to another point (faster than `distance_to`)
    pub const fn distance_squared_to(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

impl Default for CGPoint {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for CGPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
