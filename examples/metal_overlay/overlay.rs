//! Menu state and configuration

use screencapturekit::prelude::*;
use screencapturekit::stream::configuration::PixelFormat;

pub struct OverlayState {
    pub show_help: bool,
    pub show_waveform: bool,
    pub show_config: bool,
    pub config_selection: usize,
    pub menu_selection: usize,
}

impl OverlayState {
    pub fn new() -> Self {
        Self {
            show_help: true,
            show_waveform: true,
            show_config: false,
            config_selection: 1, // Start on first non-header option
            menu_selection: 0,
        }
    }

    pub const MENU_ITEMS: &'static [&'static str] = &["Picker", "Source", "Capture", "Config", "Quit"];

    pub fn menu_count() -> usize {
        Self::MENU_ITEMS.len()
    }
}

impl Default for OverlayState {
    fn default() -> Self {
        Self::new()
    }
}

/// Config menu option type for better organization
#[derive(Clone, Copy, PartialEq)]
pub enum ConfigOptionType {
    Header,
    Toggle,
    Cycle,
    Select,
}

/// A config menu option with metadata
pub struct ConfigOption {
    pub name: &'static str,
    pub option_type: ConfigOptionType,
    pub color: [f32; 4],
}

impl ConfigOption {
    const fn new(name: &'static str, option_type: ConfigOptionType) -> Self {
        let color = match option_type {
            ConfigOptionType::Header => [0.6, 0.8, 1.0, 1.0],
            ConfigOptionType::Toggle => [1.0, 1.0, 1.0, 1.0],
            ConfigOptionType::Cycle => [1.0, 1.0, 1.0, 1.0],
            ConfigOptionType::Select => [1.0, 1.0, 1.0, 1.0],
        };
        Self { name, option_type, color }
    }
}

pub struct ConfigMenu;

impl ConfigMenu {
    // Organized options with section headers
    pub const OPTIONS: &'static [ConfigOption] = &[
        // Video section
        ConfigOption::new("-- VIDEO --", ConfigOptionType::Header),
        ConfigOption::new("Frame Rate", ConfigOptionType::Cycle),
        ConfigOption::new("Pixel Format", ConfigOptionType::Cycle),
        ConfigOption::new("Queue Depth", ConfigOptionType::Cycle),
        ConfigOption::new("Show Cursor", ConfigOptionType::Toggle),
        ConfigOption::new("Retina Scale", ConfigOptionType::Toggle),
        // Scaling section  
        ConfigOption::new("-- SCALING --", ConfigOptionType::Header),
        ConfigOption::new("Scale to Fit", ConfigOptionType::Toggle),
        ConfigOption::new("Keep Aspect", ConfigOptionType::Toggle),
        // Audio section
        ConfigOption::new("-- AUDIO --", ConfigOptionType::Header),
        ConfigOption::new("System Audio", ConfigOptionType::Toggle),
        ConfigOption::new("Exclude Self", ConfigOptionType::Toggle),
        ConfigOption::new("Microphone", ConfigOptionType::Toggle),
        ConfigOption::new("Mic Device", ConfigOptionType::Select),
        ConfigOption::new("Sample Rate", ConfigOptionType::Cycle),
        ConfigOption::new("Channels", ConfigOptionType::Cycle),
    ];
    
    pub const FPS_OPTIONS: [u32; 5] = [15, 24, 30, 60, 120];
    pub const QUEUE_OPTIONS: [u32; 5] = [3, 5, 8, 12, 16];
    pub const SAMPLE_RATE_OPTIONS: [i32; 3] = [44100, 48000, 96000];
    pub const CHANNEL_OPTIONS: [i32; 2] = [1, 2];
    pub const PIXEL_FORMAT_OPTIONS: [PixelFormat; 4] = [
        PixelFormat::BGRA,
        PixelFormat::l10r,
        PixelFormat::YCbCr_420v,
        PixelFormat::YCbCr_420f,
    ];

    pub fn option_count() -> usize {
        Self::OPTIONS.len()
    }

    pub fn option(idx: usize) -> &'static ConfigOption {
        Self::OPTIONS.get(idx).unwrap_or(&Self::OPTIONS[0])
    }

    pub fn is_header(idx: usize) -> bool {
        Self::option(idx).option_type == ConfigOptionType::Header
    }

    pub fn option_value(
        config: &SCStreamConfiguration,
        mic_device_idx: Option<usize>,
        idx: usize,
    ) -> String {
        match idx {
            0 => String::new(), // Header
            1 => format!("{} fps", config.fps()),
            2 => format!("{}", config.pixel_format()),
            3 => format!("{}", config.queue_depth()),
            4 => if config.shows_cursor() { "On" } else { "Off" }.to_string(),
            5 => if config.increase_resolution_for_retina_displays() { "On" } else { "Off" }.to_string(),
            6 => String::new(), // Header
            7 => if config.scales_to_fit() { "On" } else { "Off" }.to_string(),
            8 => if config.preserves_aspect_ratio() { "On" } else { "Off" }.to_string(),
            9 => String::new(), // Header
            10 => if config.captures_audio() { "On" } else { "Off" }.to_string(),
            11 => if config.excludes_current_process_audio() { "On" } else { "Off" }.to_string(),
            12 => if config.captures_microphone() { "On" } else { "Off" }.to_string(),
            13 => match mic_device_idx {
                None => "Default".to_string(),
                Some(idx) => AudioInputDevice::list()
                    .get(idx)
                    .map(|d| d.name.chars().take(12).collect())
                    .unwrap_or_else(|| "?".to_string()),
            },
            14 => format!("{} Hz", config.sample_rate()),
            15 => format!("{}", if config.channel_count() == 1 { "Mono" } else { "Stereo" }),
            _ => "?".to_string(),
        }
    }

    fn cycle_value<T: Copy + PartialEq>(current: T, options: &[T], increase: bool) -> T {
        let current_idx = options.iter().position(|&v| v == current).unwrap_or(0);
        let new_idx = if increase {
            (current_idx + 1) % options.len()
        } else {
            (current_idx + options.len() - 1) % options.len()
        };
        options[new_idx]
    }

    pub fn toggle_or_adjust(
        config: &mut SCStreamConfiguration,
        mic_device_idx: &mut Option<usize>,
        idx: usize,
        increase: bool,
    ) {
        match idx {
            0 | 6 | 9 => {} // Headers - do nothing
            1 => {
                let new_fps = Self::cycle_value(config.fps(), &Self::FPS_OPTIONS, increase);
                config.set_fps(new_fps);
            }
            2 => {
                let new_fmt = Self::cycle_value(config.pixel_format(), &Self::PIXEL_FORMAT_OPTIONS, increase);
                config.set_pixel_format(new_fmt);
            }
            3 => {
                let new_depth = Self::cycle_value(config.queue_depth(), &Self::QUEUE_OPTIONS, increase);
                config.set_queue_depth(new_depth);
            }
            4 => { config.set_shows_cursor(!config.shows_cursor()); }
            5 => { config.set_increase_resolution_for_retina_displays(!config.increase_resolution_for_retina_displays()); }
            7 => { config.set_scales_to_fit(!config.scales_to_fit()); }
            8 => { config.set_preserves_aspect_ratio(!config.preserves_aspect_ratio()); }
            10 => { config.set_captures_audio(!config.captures_audio()); }
            11 => { config.set_excludes_current_process_audio(!config.excludes_current_process_audio()); }
            12 => { config.set_captures_microphone(!config.captures_microphone()); }
            13 => {
                let devices = AudioInputDevice::list();
                if devices.is_empty() {
                    return;
                }
                match *mic_device_idx {
                    None => *mic_device_idx = Some(if increase { 0 } else { devices.len() - 1 }),
                    Some(idx) => {
                        if increase {
                            *mic_device_idx = if idx + 1 >= devices.len() { None } else { Some(idx + 1) };
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
            14 => {
                let new_rate = Self::cycle_value(config.sample_rate(), &Self::SAMPLE_RATE_OPTIONS, increase);
                config.set_sample_rate(new_rate);
            }
            15 => {
                let new_channels = Self::cycle_value(config.channel_count(), &Self::CHANNEL_OPTIONS, increase);
                config.set_channel_count(new_channels);
            }
            _ => {}
        }
    }
    
    /// Skip to next/previous non-header option
    pub fn next_selectable(current: usize, forward: bool) -> usize {
        let count = Self::option_count();
        let mut next = current;
        loop {
            next = if forward {
                if next + 1 >= count { 0 } else { next + 1 }
            } else {
                if next == 0 { count - 1 } else { next - 1 }
            };
            if !Self::is_header(next) || next == current {
                break;
            }
        }
        next
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
        .with_captures_microphone(true)
        .with_channel_count(2)
        .with_sample_rate(48000)
        .with_scales_to_fit(true)
        .with_preserves_aspect_ratio(true)
        .with_queue_depth(8)
        .with_pixel_format(screencapturekit::stream::configuration::PixelFormat::BGRA)
}
