// Verify that macOS version-gated ObjC symbols only appear in the Swift bridge
// static library when the corresponding Cargo feature is enabled.
//
// This is a regression test for https://github.com/doom-fish/screencapturekit-rs/issues/127
// where building on macOS 26 SDK without the `macos_26_0` feature caused
// `_OBJC_CLASS_$_SCScreenshotConfiguration` to leak into the binary, crashing
// on macOS 15 at launch with `dyld: Symbol not found`.

use std::process::Command;

/// Read undefined symbols from the Swift bridge static library.
fn get_undefined_symbols() -> String {
    let lib_path = env!("SWIFT_BRIDGE_LIB_PATH");
    let output = Command::new("/usr/bin/nm")
        .args(["-u", lib_path])
        .output()
        .expect("failed to run nm on Swift bridge library");
    assert!(
        output.status.success(),
        "nm failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn has_objc_class(symbols: &str, class_name: &str) -> bool {
    symbols.contains(&format!("_OBJC_CLASS_$_{class_name}"))
}

// -- macOS 26 symbols ----------------------------------------------------------

#[test]
#[cfg(not(feature = "macos_26_0"))]
fn macos26_symbols_absent_without_feature() {
    let symbols = get_undefined_symbols();
    assert!(
        !has_objc_class(&symbols, "SCScreenshotConfiguration"),
        "SCScreenshotConfiguration found in binary WITHOUT macos_26_0 feature! \
         This will cause a dyld crash on macOS 15. See issue #127."
    );
}

#[test]
#[cfg(feature = "macos_26_0")]
fn macos26_symbols_present_with_feature() {
    let symbols = get_undefined_symbols();
    // This may legitimately be absent if the SDK is too old (build.rs skips the define).
    // Only assert presence when we expect the SDK to support it.
    let sdk_version: Option<u32> =
        option_env!("MACOS_SDK_VERSION").and_then(|v| v.split('.').next()?.parse().ok());

    // If we can't determine SDK version, skip — the build.rs warning covers this.
    if sdk_version.is_none_or(|v| v >= 26) {
        assert!(
            has_objc_class(&symbols, "SCScreenshotConfiguration"),
            "SCScreenshotConfiguration NOT found despite macos_26_0 feature being enabled \
             (SDK should support it). The Swift #if guard may not be receiving the define."
        );
    }
}
