// swift-tools-version:5.9
import PackageDescription

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
        .target(
            name: "ScreenCaptureKitBridge",
            dependencies: ["SwiftBridge"],
            path: "Sources/ScreenCaptureKitBridge",
            publicHeadersPath: "include"),
        .target(
            name: "SwiftBridge",
            path: "Sources/SwiftBridge")
    ]
)
