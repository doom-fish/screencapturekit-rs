extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
}

pub fn screen_capture_allowed() -> bool {
    if std::env::var_os("SCREENCAPTUREKIT_SKIP_LIVE_TESTS").is_some() {
        eprintln!("skip: SCREENCAPTUREKIT_SKIP_LIVE_TESTS is set");
        return false;
    }
    let allowed = unsafe { CGPreflightScreenCaptureAccess() };
    if !allowed {
        eprintln!("skip: Screen Recording permission is not granted to this process");
    }
    allowed
}
