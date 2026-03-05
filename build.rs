use std::env;
use std::process::Command;

/// Detect the macOS SDK major version via `xcrun --show-sdk-version`.
/// Returns `None` if detection fails.
fn detect_sdk_major_version() -> Option<u32> {
    let output = Command::new("xcrun")
        .args(["--show-sdk-version"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let version_str = String::from_utf8_lossy(&output.stdout);
    let major = version_str.trim().split('.').next()?;
    major.parse().ok()
}

fn main() {
    // docs.rs builds on Linux where Swift toolchain and macOS frameworks are
    // unavailable. Skip native compilation – rustdoc only needs type info.
    if env::var("DOCS_RS").is_ok() {
        return;
    }

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

    // Build Swift package with build directory in OUT_DIR
    // Pass Cargo feature flags as Swift compiler defines so the Swift bridge
    // only compiles version-gated APIs that the crate consumer opted into.
    // We intersect the requested Cargo feature with the actual SDK version:
    // a define is only passed when the feature is enabled AND the SDK supports it.
    let sdk_version = detect_sdk_major_version();
    let sdk_at_least = |min: u32| sdk_version.is_some_and(|v| v >= min);

    // Determine Swift triple from Cargo's target arch so cross-compilation
    // works (e.g. building x86_64 on Apple Silicon). Without --triple,
    // Swift PM defaults to the host architecture and the linker fails with
    // "symbol(s) not found" for the target arch.
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let swift_triple = match target_arch.as_str() {
        "x86_64" => "x86_64-apple-macosx",
        "aarch64" => "arm64-apple-macosx",
        other => panic!(
            "screencapturekit: unsupported target arch '{other}'. \
             Expected x86_64 or aarch64."
        ),
    };

    let mut swift_args = vec![
        "build",
        "-c",
        "release",
        "--triple",
        swift_triple,
        "--package-path",
        swift_dir,
        "--scratch-path",
        &swift_build_dir,
    ];
    if env::var("CARGO_FEATURE_MACOS_15_0").is_ok() {
        if sdk_at_least(15) {
            swift_args.extend(["-Xswiftc", "-DSCREENCAPTUREKIT_HAS_MACOS15_SDK"]);
        } else {
            println!(
                "cargo:warning=Feature macos_15_0 enabled but SDK version ({}) < 15; \
                 macOS 15+ APIs will be stubbed out",
                sdk_version.map_or_else(|| "unknown".to_string(), |v| v.to_string())
            );
        }
    }
    if env::var("CARGO_FEATURE_MACOS_26_0").is_ok() {
        if sdk_at_least(26) {
            swift_args.extend(["-Xswiftc", "-DSCREENCAPTUREKIT_HAS_MACOS26_SDK"]);
        } else {
            println!(
                "cargo:warning=Feature macos_26_0 enabled but SDK version ({}) < 26; \
                 macOS 26+ APIs will be stubbed out",
                sdk_version.map_or_else(|| "unknown".to_string(), |v| v.to_string())
            );
        }
    }
    let output = Command::new("swift")
        .args(&swift_args)
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

    link_swift_bridge(&swift_build_dir);
}

fn link_swift_bridge(swift_build_dir: &str) {
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
            let swift_lib_path_new =
                format!("{xcode_path}/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx");
            println!("cargo:rustc-link-arg=-Wl,-rpath,{swift_lib_path_new}");
        }
    }
}
