// Add after the SCShareableContent struct definition
unsafe impl Send for SCShareableContent {}
unsafe impl Sync for SCShareableContent {}
