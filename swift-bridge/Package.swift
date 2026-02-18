// swift-tools-version:5.9
import PackageDescription

// Swift compiler defines (SCREENCAPTUREKIT_HAS_MACOS15_SDK, SCREENCAPTUREKIT_HAS_MACOS26_SDK)
// are passed via -Xswiftc flags from build.rs based on Cargo feature flags (macos_15_0, macos_26_0).

let package = Package(
    name: "ScreenCaptureKitBridge",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "ScreenCaptureKitBridge",
            type: .static,
            targets: ["ScreenCaptureKitBridge"])
    ],
    targets: [
        // Main ScreenCaptureKit bindings
        .target(
            name: "ScreenCaptureKitBridge",
            dependencies: ["CoreMediaBridge", "CoreVideoBridge", "CoreGraphicsBridge", "IOSurfaceBridge", "DispatchBridge", "MetalBridge"],
            path: "Sources/ScreenCaptureKitBridge",
            publicHeadersPath: "include"),
        // CoreMedia framework bindings (CMSampleBuffer, CMTime, CMFormatDescription)
        .target(
            name: "CoreMediaBridge",
            path: "Sources/CoreMedia"),
        // CoreVideo framework bindings (CVPixelBuffer, CVPixelBufferPool)
        .target(
            name: "CoreVideoBridge",
            path: "Sources/CoreVideo"),
        // CoreGraphics framework bindings (CGRect, CGSize, CGPoint, CGImage)
        .target(
            name: "CoreGraphicsBridge",
            path: "Sources/CoreGraphics"),
        // IOSurface framework bindings
        .target(
            name: "IOSurfaceBridge",
            path: "Sources/IOSurface"),
        // Dispatch framework bindings (DispatchQueue)
        .target(
            name: "DispatchBridge",
            path: "Sources/Dispatch"),
        // Metal framework bindings (MTLDevice, MTLTexture, etc.)
        .target(
            name: "MetalBridge",
            path: "Sources/Metal")
    ]
)
