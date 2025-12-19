use std::process::Command;

fn main() {
    tauri_build::build();
    
    // Add rpath for Swift runtime libraries (needed for screencapturekit)
    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
    
    // Add rpath for Xcode Swift runtime (needed for Swift Concurrency)
    if let Ok(output) = Command::new("xcode-select").arg("-p").output() {
        if output.status.success() {
            let xcode_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let swift_lib_path = format!("{}/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.5/macosx", xcode_path);
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", swift_lib_path);
            let swift_lib_path_new = format!("{}/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx", xcode_path);
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", swift_lib_path_new);
        }
    }
}
