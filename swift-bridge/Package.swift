// swift-tools-version:5.9
import PackageDescription
import Foundation

// Detect SDK version to enable version-gated APIs
// The SDK version determines what APIs are available at compile time
func detectSDKMajorVersion() -> Int {
    // Try to detect SDK version via xcrun
    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/usr/bin/xcrun")
    process.arguments = ["--show-sdk-version"]
    
    let pipe = Pipe()
    process.standardOutput = pipe
    process.standardError = FileHandle.nullDevice
    
    do {
        try process.run()
        process.waitUntilExit()
        
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        if let versionString = String(data: data, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines) {
            // Parse version like "15.0" or "14.5"
            let components = versionString.split(separator: ".")
            if let major = components.first, let majorInt = Int(major) {
                return majorInt
            }
        }
    } catch {
        // Fall back to checking ProcessInfo
    }
    
    // Fallback: check if we're on macOS at build time
    return ProcessInfo.processInfo.operatingSystemVersion.majorVersion
}

let sdkMajorVersion = detectSDKMajorVersion()

var swiftSettings: [SwiftSetting] = []
let env = ProcessInfo.processInfo.environment
let enableMacOS15 = env["CARGO_FEATURE_MACOS_15_0"] != nil && sdkMajorVersion >= 15
let enableMacOS26 = env["CARGO_FEATURE_MACOS_26_0"] != nil && sdkMajorVersion >= 26

if enableMacOS15 {
    swiftSettings.append(.define("SCREENCAPTUREKIT_HAS_MACOS15_SDK"))
}
if enableMacOS26 {
    swiftSettings.append(.define("SCREENCAPTUREKIT_HAS_MACOS26_SDK"))
}

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
            publicHeadersPath: "include",
            swiftSettings: swiftSettings),
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
