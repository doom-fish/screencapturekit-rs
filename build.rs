use std::env;
use std::process::Command;

/// A parsed macOS SDK version (`major`, `minor`).
///
/// Kept as an ordered pair so feature gating can distinguish point releases:
/// `macos_15_2` needs SDK 15.2, not merely "some 15.x". The previous
/// major-only comparison accepted a 15.0 SDK for `macos_15_2` and then failed
/// with a raw Swift compile error instead of the graceful stub path.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct SdkVersion {
    major: u32,
    minor: u32,
}

impl SdkVersion {
    const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }
}

impl std::fmt::Display for SdkVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// Parse a `major[.minor[.patch]]` SDK version string.
///
/// A missing minor component is treated as `.0` (`"26"` -> `26.0`), matching
/// how Apple names its first release of a major version.
fn parse_sdk_version(raw: &str) -> Option<SdkVersion> {
    let mut parts = raw.trim().split('.');
    let major = parts.next()?.trim().parse().ok()?;
    let minor = match parts.next() {
        Some(minor) => minor.trim().parse().ok()?,
        None => 0,
    };
    Some(SdkVersion::new(major, minor))
}

/// Detect the macOS SDK version via `xcrun --sdk macosx --show-sdk-version`.
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
fn detect_sdk_version() -> Option<SdkVersion> {
    let output = Command::new("xcrun")
        .args(["--sdk", "macosx", "--show-sdk-version"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_sdk_version(&String::from_utf8_lossy(&output.stdout))
}

/// The macOS deployment target the Swift bridge is built against.
///
/// Must match `platforms: [.macOS(...)]` in `swift-bridge/Package.swift`.
/// `ScreenCaptureKit` itself starts at 12.3, but the bridge calls macOS 13.0
/// audio APIs without `#available` guards, so 13.0 is the honest floor.
const SWIFT_DEPLOYMENT_TARGET: &str = "13.0";

/// Cargo feature -> minimum macOS SDK -> Swift compile-time define.
///
/// Comparisons use the full `major.minor` SDK version so a point-release
/// feature (14.2, 14.4, 15.2) is only enabled by an SDK that actually ships
/// those symbols; a major-only comparison happily accepted a 15.0 SDK for
/// `macos_15_2` and then failed with a raw Swift compile error instead of
/// taking the graceful stub path.
///
/// A `None` define means the Swift bridge needs no conditional compilation
/// for that feature: the APIs it reaches exist in every SDK this crate can be
/// built with (the bridge's own floor is macOS 13.0 / an SDK new enough to
/// contain `ScreenCaptureKit`) and are gated at runtime with `if #available`.
/// Emitting a define nothing reads would be dead weight, and — worse — would
/// make the stub-mode warning fire for features that are not actually stubbed.
const VERSION_FEATURES: [(&str, SdkVersion, Option<&str>); 7] = [
    ("CARGO_FEATURE_MACOS_13_0", SdkVersion::new(13, 0), None),
    (
        "CARGO_FEATURE_MACOS_14_0",
        SdkVersion::new(14, 0),
        Some("SCREENCAPTUREKIT_HAS_MACOS14_SDK"),
    ),
    (
        "CARGO_FEATURE_MACOS_14_2",
        SdkVersion::new(14, 2),
        Some("SCREENCAPTUREKIT_HAS_MACOS14_2_SDK"),
    ),
    (
        "CARGO_FEATURE_MACOS_14_4",
        SdkVersion::new(14, 4),
        Some("SCREENCAPTUREKIT_HAS_MACOS14_4_SDK"),
    ),
    (
        "CARGO_FEATURE_MACOS_15_0",
        SdkVersion::new(15, 0),
        Some("SCREENCAPTUREKIT_HAS_MACOS15_SDK"),
    ),
    (
        "CARGO_FEATURE_MACOS_15_2",
        SdkVersion::new(15, 2),
        Some("SCREENCAPTUREKIT_HAS_MACOS15_2_SDK"),
    ),
    (
        "CARGO_FEATURE_MACOS_26_0",
        SdkVersion::new(26, 0),
        Some("SCREENCAPTUREKIT_HAS_MACOS26_SDK"),
    ),
];

/// Resolve which `-D<MACRO>` flags to pass to the Swift compiler for each
/// enabled `macos_*` Cargo feature.
///
/// The macOS 13.1 define is SDK-based because frame metadata added in that
/// point release is queried by an API available with every feature set.
/// The remaining defines are emitted for enabled Cargo features whose APIs
/// exist in the build SDK.
///
/// Requested APIs must exist in the build SDK. Failing here produces a clear
/// error instead of relying on incomplete Swift stubs or raw compiler errors.
fn configure_swift_version_defines(sdk_version: Option<SdkVersion>) -> Vec<String> {
    let mut define_flags: Vec<String> = Vec::new();
    if sdk_version.is_some_and(|version| version >= SdkVersion::new(13, 1)) {
        define_flags.push("-DSCREENCAPTUREKIT_HAS_MACOS13_1_SDK".to_string());
    }
    let mut stubbed_features: Vec<&str> = Vec::new();
    for (cargo_feature, min_sdk, swift_define) in VERSION_FEATURES {
        if env::var(cargo_feature).is_err() {
            continue;
        }
        if sdk_version.is_some_and(|v| v >= min_sdk) {
            if let Some(swift_define) = swift_define {
                define_flags.push(format!("-D{swift_define}"));
            }
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

/// Fail when requested Cargo features cannot be satisfied by the build SDK.
fn warn_or_fail_for_stub_mode(sdk_version: Option<SdkVersion>, stubbed_features: &[&str]) {
    let feature_list = stubbed_features.join(", ").to_lowercase();
    let detected = sdk_version.map_or_else(|| "unknown".to_string(), |v| v.to_string());
    panic!(
        "screencapturekit: Cargo feature(s) [{feature_list}] require a newer macOS SDK \
         than the detected SDK ({detected}). Install a matching Xcode, set DEVELOPER_DIR \
         or SDKROOT to it, or remove the unsupported feature(s)."
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

    // docs.rs builds on Linux where Swift toolchain and macOS frameworks are
    // unavailable. Skip native compilation – rustdoc only needs type info.
    if env::var("DOCS_RS").is_ok() {
        return;
    }

    println!("cargo:rustc-link-lib=framework=ScreenCaptureKit");

    // Build the Swift bridge
    let swift_dir = "swift-bridge";
    let out_dir = env::var("OUT_DIR").unwrap();

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

    let sdk_version = detect_sdk_version();
    assert!(
        !sdk_version.is_some_and(|version| version < SdkVersion::new(13, 0)),
        "screencapturekit: the Swift bridge requires the macOS 13.0 SDK or later; \
         install a newer Xcode or point DEVELOPER_DIR/SDKROOT at it"
    );

    // Determine Swift triple from Cargo's target arch so cross-compilation
    // works (e.g. building x86_64 on Apple Silicon). Without --triple,
    // Swift PM defaults to the host architecture and the linker fails with
    // "symbol(s) not found" for the target arch.
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let swift_arch = match target_arch.as_str() {
        "x86_64" => "x86_64",
        "aarch64" => "arm64",
        other => panic!(
            "screencapturekit: unsupported target arch '{other}'. \
             Expected x86_64 or aarch64."
        ),
    };
    // Pin the deployment target in the triple as well as in Package.swift, so
    // the emitted objects carry the same LC_BUILD_VERSION regardless of which
    // manifest Swift PM decides to honour.
    let swift_triple = format!("{swift_arch}-apple-macosx{SWIFT_DEPLOYMENT_TARGET}");

    // SwiftPM does not invalidate one scratch directory when only `-Xswiftc
    // -D...` changes. Reusing it across Cargo feature sets can therefore link
    // a stale bridge (for example, a macOS 15 build without macOS 26 symbols).
    let define_flags = configure_swift_version_defines(sdk_version);
    let variant = if define_flags.is_empty() {
        "base".to_string()
    } else {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in define_flags.iter().flat_map(|flag| flag.bytes().chain([0])) {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        format!("defines-{hash:016x}")
    };
    let sdk_key = sdk_version.map_or_else(
        || "unknown-sdk".to_string(),
        |version| format!("sdk{}-{}", version.major, version.minor),
    );
    let swift_build_dir = format!("{out_dir}/swift-build-{swift_arch}-{sdk_key}-{variant}");

    let mut swift_args: Vec<&str> = vec![
        "build",
        "-c",
        "release",
        "--triple",
        &swift_triple,
        "--package-path",
        swift_dir,
        "--scratch-path",
        &swift_build_dir,
    ];

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
