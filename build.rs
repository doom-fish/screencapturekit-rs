use std::env;
use std::process::Command;

fn main() {
    println!("cargo:rustc-link-lib=framework=ScreenCaptureKit");

    // Build the Swift bridge
    let swift_dir = "swift-bridge";
    let out_dir = env::var("OUT_DIR").unwrap();
    let swift_build_dir = format!("{out_dir}/swift-build");

    println!("cargo:rerun-if-changed={swift_dir}");

    // Run swiftlint if available (non-strict mode, don't fail build)
    if let Ok(output) = Command::new("swiftlint")
        .args(["lint"])
        .current_dir(swift_dir)
        .output()
    {
        if !output.status.success() {
            eprintln!(
                "SwiftLint warnings:\n{}",
                String::from_utf8_lossy(&output.stdout)
            );
        }
    }

    let feature_open_15 = env::var("CARGO_FEATURE_MACOS_15_0").is_ok();
    let feature_open_26 = env::var("CARGO_FEATURE_MACOS_26_0").is_ok();
    let mut args = vec![
        "build",
        "-c",
        "release",
        "--package-path",
        swift_dir,
        "--scratch-path",
        &swift_build_dir,
    ];

    if feature_open_15 {
        args.push("--features");
        args.push("macos_15_0");
    }
    if feature_open_26 {
        args.push("--features");
        args.push("macos_26_0");
    }

    // Build Swift package with build directory in OUT_DIR
    let output = Command::new("swift")
        .args(args)
        .output()
        .expect("Failed to build Swift bridge");

    // Swift build outputs warnings to stderr even on success, check exit code only
    if !output.status.success() {
        eprintln!(
            "Swift build STDOUT:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
        eprintln!(
            "Swift build STDERR:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        panic!(
            "Swift build failed with exit code: {:?}",
            output.status.code()
        );
    }

    // Link the Swift library from OUT_DIR
    println!("cargo:rustc-link-search=native={swift_build_dir}/release");
    println!("cargo:rustc-link-lib=static=ScreenCaptureKitBridge");

    // Link required frameworks
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=CoreGraphics");
    println!("cargo:rustc-link-lib=framework=CoreMedia");
    println!("cargo:rustc-link-lib=framework=IOSurface");

    // Add rpath for Swift runtime libraries
    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");

    // Add rpath for Xcode Swift runtime (needed for Swift Concurrency)
    if let Ok(output) = Command::new("xcode-select").arg("-p").output() {
        if output.status.success() {
            let xcode_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let swift_lib_path = format!(
                "{xcode_path}/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.5/macosx"
            );
            println!("cargo:rustc-link-arg=-Wl,-rpath,{swift_lib_path}");
            // Also add the newer swift path
            let swift_lib_path_new =
                format!("{xcode_path}/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx");
            println!("cargo:rustc-link-arg=-Wl,-rpath,{swift_lib_path_new}");
        }
    }
}
