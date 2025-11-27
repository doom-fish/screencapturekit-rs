import CoreGraphics
import Foundation

// MARK: - CGRect Bridge

public struct CGRectBridge {
    public var x: Double
    public var y: Double
    public var width: Double
    public var height: Double

    public init(x: Double, y: Double, width: Double, height: Double) {
        self.x = x
        self.y = y
        self.width = width
        self.height = height
    }

    public init(rect: CGRect) {
        self.x = Double(rect.origin.x)
        self.y = Double(rect.origin.y)
        self.width = Double(rect.size.width)
        self.height = Double(rect.size.height)
    }

    public func toCGRect() -> CGRect {
        CGRect(x: x, y: y, width: width, height: height)
    }
}

// MARK: - CGSize Bridge

public struct CGSizeBridge {
    public var width: Double
    public var height: Double

    public init(width: Double, height: Double) {
        self.width = width
        self.height = height
    }

    public init(size: CGSize) {
        self.width = Double(size.width)
        self.height = Double(size.height)
    }

    public func toCGSize() -> CGSize {
        CGSize(width: width, height: height)
    }
}

// MARK: - CGPoint Bridge

public struct CGPointBridge {
    public var x: Double
    public var y: Double

    public init(x: Double, y: Double) {
        self.x = x
        self.y = y
    }

    public init(point: CGPoint) {
        self.x = Double(point.x)
        self.y = Double(point.y)
    }

    public func toCGPoint() -> CGPoint {
        CGPoint(x: x, y: y)
    }
}
