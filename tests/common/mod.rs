extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
}

pub fn screen_capture_allowed() -> bool {
    let allowed = unsafe { CGPreflightScreenCaptureAccess() };
    if !allowed {
        eprintln!("skip: Screen Recording permission is not granted to this process");
    }
    allowed
}
