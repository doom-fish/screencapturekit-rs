//! Menu state and configuration

use screencapturekit::prelude::*;

/// Menu mode - determines which menu items are shown
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuMode {
    /// Initial state - only Pick Source and Quit
    Initial,
    /// After source is picked - full menu with capture controls
    Full,
}

#[allow(clippy::struct_excessive_bools)]
pub struct OverlayState {
    pub show_help: bool,
    pub show_waveform: bool,
    pub show_config: bool,
    #[cfg(feature = "macos_15_0")]
    pub show_recording_config: bool,
    pub config_selection: usize,
    #[cfg(feature = "macos_15_0")]
    pub recording_config_selection: usize,
    pub menu_selection: usize,
    pub menu_mode: MenuMode,
}

impl OverlayState {
    pub const fn new() -> Self {
        Self {
            show_help: true,
            show_waveform: true,
            show_config: false,
            #[cfg(feature = "macos_15_0")]
            show_recording_config: false,
            config_selection: 0,
            #[cfg(feature = "macos_15_0")]
            recording_config_selection: 0,
            menu_selection: 0,
            menu_mode: MenuMode::Initial,
        }
    }

    /// Menu items for initial state (no source picked)
    pub const MENU_ITEMS_INITIAL: &'static [&'static str] = &["Pick Source", "Quit"];

    /// Menu items for full state (after source picked) - macOS 15.0+
    #[cfg(feature = "macos_15_0")]
    pub const MENU_ITEMS_FULL: &'static [&'static str] = &[
        "Capture",
        "Screenshot",
        "Record",
        "Config",
        "Rec Config",
        "Change Source",
        "Quit",
    ];

    /// Menu items for full state (after source picked) - pre-macOS 15.0
    #[cfg(not(feature = "macos_15_0"))]
    pub const MENU_ITEMS_FULL: &'static [&'static str] =
        &["Capture", "Screenshot", "Config", "Change Source", "Quit"];

    /// Get current menu items based on mode
    pub const fn menu_items(&self) -> &'static [&'static str] {
        match self.menu_mode {
            MenuMode::Initial => Self::MENU_ITEMS_INITIAL,
            MenuMode::Full => Self::MENU_ITEMS_FULL,
        }
    }

    /// Get current menu item count
    pub const fn menu_count(&self) -> usize {
        self.menu_items().len()
    }

    /// Switch to full menu mode (called after picking a source)
    pub fn switch_to_full_menu(&mut self) {
        self.menu_mode = MenuMode::Full;
        self.menu_selection = 0; // Reset selection to first item (Capture)
    }
}

pub struct ConfigMenu;

impl ConfigMenu {
    pub const OPTIONS: &'static [&'static str] = &[
        "FPS",
        "Cursor",
        "Audio",
        "Mic",
        "Mic Device",
        "Exclude Self",
        "Sample Rate",
        "Channels",
        "Scale Fit",
        "Aspect",
        "Opaque",
        "Shadows Only",
        "Ignore Shadows",
        "Pixel Format",
        "Queue",
    ];
    pub const FPS_OPTIONS: [u32; 5] = [15, 30, 60, 120, 240];
    pub const QUEUE_OPTIONS: [u32; 4] = [3, 5, 8, 12];
    pub const SAMPLE_RATE_OPTIONS: [i32; 3] = [44100, 48000, 96000];
    pub const CHANNEL_OPTIONS: [i32; 2] = [1, 2];

    pub const fn option_count() -> usize {
        Self::OPTIONS.len()
    }

    pub fn option_name(idx: usize) -> &'static str {
        Self::OPTIONS.get(idx).unwrap_or(&"?")
    }

    pub fn option_value(
        config: &SCStreamConfiguration,
        mic_device_idx: Option<usize>,
        idx: usize,
    ) -> String {
        match idx {
            0 => format!("{}", config.fps()),
            1 => if config.shows_cursor() { "On" } else { "Off" }.to_string(),
            2 => if config.captures_audio() { "On" } else { "Off" }.to_string(),
            3 => if config.captures_microphone() {
                "On"
            } else {
                "Off"
            }
            .to_string(),
            4 => mic_device_idx.map_or_else(
                || "Default".to_string(),
                |idx| {
                    AudioInputDevice::list()
                        .get(idx)
                        .map_or_else(|| "?".to_string(), |d| d.name.chars().take(10).collect())
                },
            ),
            5 => if config.excludes_current_process_audio() {
                "On"
            } else {
                "Off"
            }
            .to_string(),
            6 => format!("{}Hz", config.sample_rate()),
            7 => format!("{}ch", config.channel_count()),
            8 => if config.scales_to_fit() { "On" } else { "Off" }.to_string(),
            9 => if config.preserves_aspect_ratio() {
                "On"
            } else {
                "Off"
            }
            .to_string(),
            10 => if config.should_be_opaque() {
                "On"
            } else {
                "Off"
            }
            .to_string(),
            11 => if config.captures_shadows_only() {
                "On"
            } else {
                "Off"
            }
            .to_string(),
            12 => if config.ignores_shadows_display() {
                "On"
            } else {
                "Off"
            }
            .to_string(),
            13 => format!("{:?}", config.pixel_format()),
            14 => format!("{}", config.queue_depth()),
            _ => "?".to_string(),
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn toggle_or_adjust(
        config: &mut SCStreamConfiguration,
        mic_device_idx: &mut Option<usize>,
        idx: usize,
        increase: bool,
    ) {
        use screencapturekit::stream::configuration::pixel_format::PixelFormat;
        const PIXEL_FORMATS: [PixelFormat; 4] = [
            PixelFormat::BGRA,
            PixelFormat::l10r,
            PixelFormat::YCbCr_420v,
            PixelFormat::YCbCr_420f,
        ];

        match idx {
            0 => {
                // FPS
                let current_fps = config.fps();
                let current_idx = Self::FPS_OPTIONS
                    .iter()
                    .position(|&f| f == current_fps)
                    .unwrap_or(2);
                let new_idx = if increase {
                    (current_idx + 1) % Self::FPS_OPTIONS.len()
                } else {
                    (current_idx + Self::FPS_OPTIONS.len() - 1) % Self::FPS_OPTIONS.len()
                };
                config.set_fps(Self::FPS_OPTIONS[new_idx]);
            }
            1 => {
                config.set_shows_cursor(!config.shows_cursor());
            }
            2 => {
                config.set_captures_audio(!config.captures_audio());
            }
            3 => {
                let new_val = !config.captures_microphone();
                config.set_captures_microphone(new_val);
                // Also clear device ID when disabling (as per Apple's sample)
                if !new_val {
                    config.clear_microphone_capture_device_id();
                }
            }
            4 => {
                // Mic Device
                let devices = AudioInputDevice::list();
                if devices.is_empty() {
                    return;
                }
                match *mic_device_idx {
                    None => *mic_device_idx = Some(if increase { 0 } else { devices.len() - 1 }),
                    Some(idx) => {
                        if increase {
                            *mic_device_idx = if idx + 1 >= devices.len() {
                                None
                            } else {
                                Some(idx + 1)
                            };
                        } else {
                            *mic_device_idx = if idx == 0 { None } else { Some(idx - 1) };
                        }
                    }
                }
                if let Some(idx) = *mic_device_idx {
                    if let Some(device) = devices.get(idx) {
                        config.set_microphone_capture_device_id(&device.id);
                    }
                }
            }
            5 => {
                config.set_excludes_current_process_audio(!config.excludes_current_process_audio());
            }
            6 => {
                // Sample Rate
                let current = config.sample_rate();
                let current_idx = Self::SAMPLE_RATE_OPTIONS
                    .iter()
                    .position(|&r| r == current)
                    .unwrap_or(1);
                let new_idx = if increase {
                    (current_idx + 1) % Self::SAMPLE_RATE_OPTIONS.len()
                } else {
                    (current_idx + Self::SAMPLE_RATE_OPTIONS.len() - 1)
                        % Self::SAMPLE_RATE_OPTIONS.len()
                };
                config.set_sample_rate(Self::SAMPLE_RATE_OPTIONS[new_idx]);
            }
            7 => {
                // Channels
                let current = config.channel_count();
                let current_idx = Self::CHANNEL_OPTIONS
                    .iter()
                    .position(|&c| c == current)
                    .unwrap_or(1);
                let new_idx = if increase {
                    (current_idx + 1) % Self::CHANNEL_OPTIONS.len()
                } else {
                    (current_idx + Self::CHANNEL_OPTIONS.len() - 1) % Self::CHANNEL_OPTIONS.len()
                };
                config.set_channel_count(Self::CHANNEL_OPTIONS[new_idx]);
            }
            8 => {
                config.set_scales_to_fit(!config.scales_to_fit());
            }
            9 => {
                config.set_preserves_aspect_ratio(!config.preserves_aspect_ratio());
            }
            10 => {
                config.set_should_be_opaque(!config.should_be_opaque());
            }
            11 => {
                config.set_captures_shadows_only(!config.captures_shadows_only());
            }
            12 => {
                config.set_ignores_shadows_display(!config.ignores_shadows_display());
            }
            13 => {
                // Pixel Format
                let current = config.pixel_format();
                let current_idx = PIXEL_FORMATS
                    .iter()
                    .position(|&f| f == current)
                    .unwrap_or(0);
                let new_idx = if increase {
                    (current_idx + 1) % PIXEL_FORMATS.len()
                } else {
                    (current_idx + PIXEL_FORMATS.len() - 1) % PIXEL_FORMATS.len()
                };
                config.set_pixel_format(PIXEL_FORMATS[new_idx]);
            }
            14 => {
                // Queue
                let current_depth = config.queue_depth();
                let current_idx = Self::QUEUE_OPTIONS
                    .iter()
                    .position(|&q| q == current_depth)
                    .unwrap_or(2);
                let new_idx = if increase {
                    (current_idx + 1) % Self::QUEUE_OPTIONS.len()
                } else {
                    (current_idx + Self::QUEUE_OPTIONS.len() - 1) % Self::QUEUE_OPTIONS.len()
                };
                config.set_queue_depth(Self::QUEUE_OPTIONS[new_idx]);
            }
            _ => {}
        }
    }
}

pub fn default_stream_config() -> SCStreamConfiguration {
    SCStreamConfiguration::new()
        .with_width(1920)
        .with_height(1080)
        .with_fps(60)
        .with_shows_cursor(true)
        .with_captures_audio(true)
        .with_excludes_current_process_audio(true)
        .with_captures_microphone(false) // Microphone off by default, press 'M' to enable
        .with_channel_count(2)
        .with_sample_rate(48000)
        .with_scales_to_fit(true)
        .with_preserves_aspect_ratio(true)
        .with_queue_depth(8)
        .with_pixel_format(screencapturekit::stream::configuration::PixelFormat::BGRA)
}
