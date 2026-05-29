// swift-tools-version:5.9
import PackageDescription

// Swift compiler defines (SCREENCAPTUREKIT_HAS_MACOS15_SDK, SCREENCAPTUREKIT_HAS_MACOS26_SDK)
// are passed via -Xswiftc flags from build.rs based on Cargo feature flags (macos_15_0, macos_26_0).
//
// Note: CoreGraphicsBridge / CoreVideoBridge / IOSurfaceBridge / DispatchBridge
// targets that used to live here were extracted into apple-cf-rs's bridge.
// CoreMediaBridge here keeps only the SCStreamFrameInfo attachment readers
// and a few generic accessors with frame-info-specific signatures —
// everything else comes from apple-cf's CoreMediaBridge target.

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
        // Main ScreenCaptureKit bindings.
        .target(
            name: "ScreenCaptureKitBridge",
            dependencies: ["CoreMediaBridge", "MetalBridge"],
            path: "Sources/ScreenCaptureKitBridge"),
        // CoreMedia bindings — only the SCStreamFrameInfo attachment readers
        // and SC-specific sample-buffer helpers. Generic CMSampleBuffer /
        // CMBlockBuffer / CMFormatDescription accessors come from
        // apple-cf-rs's CoreMediaBridge.
        .target(
            name: "CoreMediaBridge",
            path: "Sources/CoreMedia"),
        // Metal framework bindings (MTLDevice, MTLTexture, etc.) — apple-cf
        // doesn't provide a metal module yet so this stays local.
        .target(
            name: "MetalBridge",
            path: "Sources/Metal")
    ]
)
