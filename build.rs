use std::process::Command;

fn main() {
    println!("cargo:rustc-link-lib=framework=ScreenCaptureKit");
    
    // Build the Swift bridge
    let swift_dir = "swift-bridge";
    
    println!("cargo:rerun-if-changed={swift_dir}");
    
    // Run swiftlint if available (non-strict mode, don't fail build)
    if let Ok(output) = Command::new("swiftlint")
        .args(["lint"])
        .current_dir(swift_dir)
        .output()
    {
        if !output.status.success() {
            eprintln!("SwiftLint warnings:\n{}", String::from_utf8_lossy(&output.stdout));
        }
    }
    
    // Build Swift package
    let output = Command::new("swift")
        .args(["build", "-c", "release", "--package-path", swift_dir])
        .output()
        .expect("Failed to build Swift bridge");
    
    // Swift build outputs warnings to stderr even on success, check exit code only
    if !output.status.success() {
        eprintln!("Swift build STDOUT:\n{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("Swift build STDERR:\n{}", String::from_utf8_lossy(&output.stderr));
        panic!("Swift build failed with exit code: {:?}", output.status.code());
    }
    
    // Link the Swift library
    println!("cargo:rustc-link-search=native={swift_dir}/.build/release");
    println!("cargo:rustc-link-lib=static=ScreenCaptureKitBridge");
    
    // Link required frameworks
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=CoreGraphics");
    println!("cargo:rustc-link-lib=framework=CoreMedia");
    println!("cargo:rustc-link-lib=framework=IOSurface");
    
    // Add rpath for Swift runtime libraries
    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
}
