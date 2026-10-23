// swift-tools-version:5.9
import PackageDescription

// Swift compiler defines (SCREENCAPTUREKIT_HAS_MACOS14_SDK,
// SCREENCAPTUREKIT_HAS_MACOS14_4_SDK, SCREENCAPTUREKIT_HAS_MACOS15_SDK,
// SCREENCAPTUREKIT_HAS_MACOS15_2_SDK, SCREENCAPTUREKIT_HAS_MACOS26_SDK) are
// passed via -Xswiftc flags from build.rs based on the enabled Cargo features.
//
// Note: CoreGraphicsBridge / CoreVideoBridge / IOSurfaceBridge / DispatchBridge
// targets that used to live here were extracted into apple-cf-rs's bridge.
// ScreenCaptureKitCoreMediaBridge keeps only the SCStreamFrameInfo attachment
// readers and a few generic accessors with frame-info-specific signatures.
// Generic CoreMedia bindings come from apple-cf's CoreMediaBridge target.

let package = Package(
    name: "ScreenCaptureKitBridge",
    platforms: [
        .macOS("12.3")
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
            dependencies: ["ScreenCaptureKitCoreMediaBridge", "MetalBridge"],
            path: "Sources/ScreenCaptureKitBridge"),
        // SC-specific CoreMedia bindings: the SCStreamFrameInfo attachment
        // readers and sample-buffer helpers with frame-info-specific
        // signatures. Generic CMSampleBuffer / CMBlockBuffer /
        // CMFormatDescription accessors come from apple-cf-rs instead.
        //
        // Named `ScreenCaptureKitCoreMediaBridge`, not `CoreMediaBridge`:
        // apple-cf-rs publishes a target with the latter name and both static
        // archives are linked into the same Rust binary. Two Swift modules with
        // the same name produce colliding `.swiftmodule` names and duplicate
        // metadata symbols at link time.
        .target(
            name: "ScreenCaptureKitCoreMediaBridge",
            path: "Sources/CoreMedia"),
        // Metal framework bindings (MTLDevice, MTLTexture, etc.) — apple-cf
        // doesn't provide a metal module yet so this stays local.
        .target(
            name: "MetalBridge",
            path: "Sources/Metal")
    ]
)
