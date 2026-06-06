use std::env;
use std::process::Command;

/// Detect the macOS SDK major version via `xcrun --sdk macosx --show-sdk-version`.
///
/// Returns `None` if detection fails.
///
/// **Why `--sdk macosx` is required**: bare `xcrun --show-sdk-version` follows
/// xcrun's notion of the "active developer dir" plus the embedded "default
/// SDK" preference, which can land on `/Library/Developer/CommandLineTools/
/// SDKs/MacOSX.sdk` even when `xcode-select -p` correctly points at a full
/// Xcode install. On machines where Command Line Tools is registered but its
/// SDK directory is missing or stale, the bare invocation fails with
/// `xcodebuild: error: SDK "/Library/Developer/CommandLineTools/SDKs/
/// MacOSX.sdk" cannot be located`. Forcing `--sdk macosx` resolves the SDK
/// from the active Xcode toolchain instead, which is what every other Apple
/// build system does (`CMake`, Swift PM, etc.).
fn detect_sdk_major_version() -> Option<u32> {
    let output = Command::new("xcrun")
        .args(["--sdk", "macosx", "--show-sdk-version"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let version_str = String::from_utf8_lossy(&output.stdout);
    let major = version_str.trim().split('.').next()?;
    major.parse().ok()
}

/// Resolve which `-D<MACRO>` flags to pass to the Swift compiler for each
/// enabled `macos_*` Cargo feature.
///
/// Iterates the `version_features` table; for each enabled feature, emits
/// the corresponding `-D` flag if the host SDK is recent enough, otherwise
/// records the feature in `stubbed_features` so the caller can warn (or
/// fail) about stub mode.
///
/// Behaviour for stubbed features:
/// * If the SDK was detected but is just too old for the feature, the
///   user has consciously requested stub mode — warn only.
/// * If SDK detection failed entirely (`xcrun` returned non-parseable
///   output), the build fails by default to prevent shipping a binary
///   with silently-stubbed APIs. The user can opt back in via
///   `SCREENCAPTUREKIT_ALLOW_STUBBED_BUILD=1`.
fn configure_swift_version_defines(sdk_version: Option<u32>) -> Vec<String> {
    // The Swift bridge consults the MACOS14, MACOS15, and MACOS26 defines today
    // (see ScreenshotManager.swift, StreamConfiguration.swift, Stream.swift).
    // The remaining defines are passed through for symmetry so that future
    // version-gated Swift APIs don't silently drop their feature gate.
    //
    // (cargo_feature, min_sdk_major, swift_define)
    let version_features: [(&str, u32, &str); 7] = [
        (
            "CARGO_FEATURE_MACOS_13_0",
            13,
            "SCREENCAPTUREKIT_HAS_MACOS13_SDK",
        ),
        (
            "CARGO_FEATURE_MACOS_14_0",
            14,
            "SCREENCAPTUREKIT_HAS_MACOS14_SDK",
        ),
        (
            "CARGO_FEATURE_MACOS_14_2",
            14,
            "SCREENCAPTUREKIT_HAS_MACOS14_2_SDK",
        ),
        (
            "CARGO_FEATURE_MACOS_14_4",
            14,
            "SCREENCAPTUREKIT_HAS_MACOS14_4_SDK",
        ),
        (
            "CARGO_FEATURE_MACOS_15_0",
            15,
            "SCREENCAPTUREKIT_HAS_MACOS15_SDK",
        ),
        (
            "CARGO_FEATURE_MACOS_15_2",
            15,
            "SCREENCAPTUREKIT_HAS_MACOS15_2_SDK",
        ),
        (
            "CARGO_FEATURE_MACOS_26_0",
            26,
            "SCREENCAPTUREKIT_HAS_MACOS26_SDK",
        ),
    ];

    let sdk_at_least = |min: u32| sdk_version.is_some_and(|v| v >= min);

    let mut define_flags: Vec<String> = Vec::new();
    let mut stubbed_features: Vec<&str> = Vec::new();
    for (cargo_feature, min_sdk, swift_define) in version_features {
        if env::var(cargo_feature).is_err() {
            continue;
        }
        if sdk_at_least(min_sdk) {
            define_flags.push(format!("-D{swift_define}"));
        } else {
            // Strip the CARGO_FEATURE_ prefix so the warning names the
            // Cargo feature the user actually enabled.
            stubbed_features.push(cargo_feature.trim_start_matches("CARGO_FEATURE_"));
        }
    }

    if !stubbed_features.is_empty() {
        warn_or_fail_for_stub_mode(sdk_version, &stubbed_features);
    }

    define_flags
}

/// Issue the user-facing warning (or panic) when one or more requested
/// Cargo features cannot be satisfied by the detected SDK. See the
/// docs on [`configure_swift_version_defines`] for the policy.
fn warn_or_fail_for_stub_mode(sdk_version: Option<u32>, stubbed_features: &[&str]) {
    let opt_out = env::var("SCREENCAPTUREKIT_ALLOW_STUBBED_BUILD").is_ok();
    let detection_failed = sdk_version.is_none();
    let feature_list = stubbed_features.join(", ").to_lowercase();

    assert!(
        !detection_failed || opt_out,
        "screencapturekit: SDK version detection failed (`xcrun --show-sdk-version` \
         returned non-parseable output) but the following version feature(s) were \
         enabled in Cargo.toml: [{feature_list}]. Building would silently produce a \
         binary whose macOS-version-gated APIs are stubbed out and fail at runtime. \
         Resolve this by:\n\
         \n\
           1. Installing the full Xcode (not just Command Line Tools) and \
              ensuring `xcode-select -p` points at it; or\n\
           2. Setting DEVELOPER_DIR to a valid Xcode path; or\n\
           3. Removing the unused version feature(s) from your Cargo.toml; or\n\
           4. Setting SCREENCAPTUREKIT_ALLOW_STUBBED_BUILD=1 to opt into the \
              stubbed-API build (only useful for `cargo doc`/`cargo check` runs).",
    );

    let suffix = if detection_failed {
        " (suppressed via SCREENCAPTUREKIT_ALLOW_STUBBED_BUILD)"
    } else {
        ""
    };
    let detected = sdk_version.map_or_else(|| "unknown".to_string(), |v| v.to_string());
    println!(
        "cargo:warning=Cargo feature(s) [{feature_list}] requested but SDK major \
         version ({detected}) is too old{suffix}; the corresponding Swift APIs will \
         be stubbed out.",
    );
}

fn main() {
    // Re-run this build script if the build script itself changes, or if
    // any environment variable that affects its decisions changes.
    // (Cargo's default rerun-if-changed already covers crate sources.)
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    println!("cargo:rerun-if-env-changed=DEVELOPER_DIR");
    println!("cargo:rerun-if-env-changed=SDKROOT");
    println!("cargo:rerun-if-env-changed=SCREENCAPTUREKIT_ALLOW_STUBBED_BUILD");

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

    let sdk_version = detect_sdk_major_version();

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

    let mut swift_args: Vec<&str> = vec![
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

    // Resolve which `-DSCREENCAPTUREKIT_HAS_MACOS<X>_SDK` flags to add
    // and emit any warnings/failures for stubbed builds. The owned
    // `define_flags` strings live as long as `swift_args`'s borrows.
    let define_flags = configure_swift_version_defines(sdk_version);
    for flag in &define_flags {
        swift_args.push("-Xswiftc");
        swift_args.push(flag);
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
    match Command::new("xcode-select").arg("-p").output() {
        Ok(output) if output.status.success() => {
            let xcode_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let swift_lib_path = format!(
                "{xcode_path}/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.5/macosx"
            );
            println!("cargo:rustc-link-arg=-Wl,-rpath,{swift_lib_path}");
            let swift_lib_path_new =
                format!("{xcode_path}/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx");
            println!("cargo:rustc-link-arg=-Wl,-rpath,{swift_lib_path_new}");
        }
        Ok(output) => {
            // xcode-select ran but reported failure (e.g. exit code != 0).
            println!(
                "cargo:warning=`xcode-select -p` exited non-zero (status={:?}); \
                 the Swift Concurrency rpath will not be baked in. The resulting \
                 binary may fail at load time with `dyld: Library not loaded` \
                 unless Swift's concurrency runtime is on the system search \
                 path. Install the full Xcode (not just Command Line Tools), \
                 or set DEVELOPER_DIR to a valid Xcode path.",
                output.status.code()
            );
        }
        Err(err) => {
            // xcode-select binary missing or not executable.
            println!(
                "cargo:warning=`xcode-select` could not be invoked ({err}); \
                 the Swift Concurrency rpath will not be baked in. The \
                 resulting binary may fail at load time with `dyld: Library \
                 not loaded`. Install Xcode and ensure xcode-select is on PATH."
            );
        }
    }
}
