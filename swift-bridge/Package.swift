// swift-tools-version:5.9
import PackageDescription
import Foundation

// Detect SDK version to enable macOS 15+ APIs (SCRecordingOutput, etc.)
// The SDK version determines what APIs are available at compile time
func detectMacOS15SDK() -> Bool {
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
                return majorInt >= 15
            }
        }
    } catch {
        // Fall back to checking ProcessInfo
    }
    
    // Fallback: check if we're on macOS 15+ at build time
    let osVersion = ProcessInfo.processInfo.operatingSystemVersion
    return osVersion.majorVersion >= 15
}

let hasMacOS15SDK = detectMacOS15SDK()

var swiftSettings: [SwiftSetting] = []
if hasMacOS15SDK {
    swiftSettings.append(.define("SCREENCAPTUREKIT_HAS_MACOS15_SDK"))
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
            dependencies: ["CoreMediaBridge", "CoreVideoBridge", "CoreGraphicsBridge", "IOSurfaceBridge", "DispatchBridge"],
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
            path: "Sources/Dispatch")
    ]
)
