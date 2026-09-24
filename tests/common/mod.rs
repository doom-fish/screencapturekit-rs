extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
}

pub fn screen_capture_allowed() -> bool {
    if std::env::var("SCREENCAPTUREKIT_LIVE_TESTS").as_deref() != Ok("1") {
        eprintln!("skip: set SCREENCAPTUREKIT_LIVE_TESTS=1 to run live capture tests");
        return false;
    }
    let allowed = unsafe { CGPreflightScreenCaptureAccess() };
    if !allowed {
        eprintln!("skip: Screen Recording permission is not granted to this process");
    }
    allowed
}
