//! `ScreenCaptureKit` CLI - A comprehensive command-line tool for screen capture
//!
//! This example demonstrates the full capabilities of screencapturekit-rs:
//! - Listing displays, windows, and applications
//! - Capturing screen content (video and/or audio)
//! - Taking screenshots (using `SCScreenshotManager` on macOS 14.0+)
//! - Saving captured frames to disk
//! - Configuring capture settings
//! - Interactive content selection with system picker UI (macOS 14.0+)
//!
//! Usage examples:
//!   cargo run --example cli -- list displays
//!   cargo run --example cli -- list windows
//!   cargo run --example cli -- list apps
//!   cargo run --example cli -- capture --display 0 --duration 5 --output ./frames
//!   cargo run --example cli -- screenshot --display 0 --output screenshot.png
//!   cargo run --example cli --features `macos_14_0` -- screenshot --display 0 --output shot.png
//!   cargo run --example cli -- capture --window "Terminal" --with-audio --duration 10
//!   cargo run --example cli --features `macos_14_0` -- picker
//!   cargo run --example cli --features `macos_14_0` -- capture --picker --duration 5
//!
//! Note: The `--features macos_14_0` flag enables:
//! - Content picker UI for interactive selection
//! - Optimized screenshot capture using `SCScreenshotManager`

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::significant_drop_tightening)]



use screencapturekit::{
    cm::CMSampleBuffer,
    output::{CVImageBufferLockExt, PixelBufferLockFlags},
    shareable_content::SCShareableContent,
    stream::{
        configuration::{PixelFormat, SCStreamConfiguration},
        content_filter::SCContentFilter,
        output_trait::SCStreamOutputTrait,
        output_type::SCStreamOutputType,
        SCStream,
    },
};

#[cfg(feature = "macos_14_0")]
use screencapturekit::{
    content_sharing_picker::{
        SCContentSharingPicker, SCContentSharingPickerConfiguration, SCContentSharingPickerMode,
        SCContentSharingPickerResult,
    },
    screenshot_manager::SCScreenshotManager,
};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

#[cfg(not(feature = "macos_14_0"))]
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::{Duration, Instant};

// Simple argument parser
struct CliArgs {
    command: Command,
}

enum Command {
    List { what: ListTarget },
    Capture(CaptureConfig),
    Screenshot(ScreenshotConfig),
    #[cfg(feature = "macos_14_0")]
    Picker,
    Help,
}

enum ListTarget {
    Displays,
    Windows,
    Apps,
}

struct CaptureConfig {
    display_id: Option<u32>,
    window_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    duration: u32,
    output_dir: PathBuf,
    with_audio: bool,
    fps: Option<u32>,
    save_frames: bool,
    pixel_format: PixelFormat,
    use_picker: bool,
}

struct ScreenshotConfig {
    display_id: u32,
    output_path: PathBuf,
}

impl CliArgs {
    fn parse() -> Result<Self, String> {
        let args: Vec<String> = env::args().collect();
        
        if args.len() < 2 {
            return Ok(Self {
                command: Command::Help,
            });
        }

        match args[1].as_str() {
            "list" => {
                if args.len() < 3 {
                    return Err("Usage: list <displays|windows|apps>".to_string());
                }
                let what = match args[2].as_str() {
                    "displays" => ListTarget::Displays,
                    "windows" => ListTarget::Windows,
                    "apps" => ListTarget::Apps,
                    _ => return Err("Invalid list target. Use: displays, windows, or apps".to_string()),
                };
                Ok(Self {
                    command: Command::List { what },
                })
            }
            "capture" => {
                let mut config = CaptureConfig {
                    display_id: None,
                    window_name: None,
                    width: Some(1920),
                    height: Some(1080),
                    duration: 5,
                    output_dir: PathBuf::from("./capture_output"),
                    with_audio: false,
                    fps: None,
                    save_frames: false,
                    pixel_format: PixelFormat::BGRA,
                    use_picker: false,
                };

                let mut i = 2;
                while i < args.len() {
                    match args[i].as_str() {
                        "--display" => {
                            i += 1;
                            if i >= args.len() {
                                return Err("--display requires a value".to_string());
                            }
                            config.display_id = Some(args[i].parse()
                                .map_err(|_| "Invalid display ID".to_string())?);
                        }
                        "--window" => {
                            i += 1;
                            if i >= args.len() {
                                return Err("--window requires a value".to_string());
                            }
                            config.window_name = Some(args[i].clone());
                        }
                        "--width" => {
                            i += 1;
                            if i >= args.len() {
                                return Err("--width requires a value".to_string());
                            }
                            config.width = Some(args[i].parse()
                                .map_err(|_| "Invalid width".to_string())?);
                        }
                        "--height" => {
                            i += 1;
                            if i >= args.len() {
                                return Err("--height requires a value".to_string());
                            }
                            config.height = Some(args[i].parse()
                                .map_err(|_| "Invalid height".to_string())?);
                        }
                        "--duration" => {
                            i += 1;
                            if i >= args.len() {
                                return Err("--duration requires a value".to_string());
                            }
                            config.duration = args[i].parse()
                                .map_err(|_| "Invalid duration".to_string())?;
                        }
                        "--output" => {
                            i += 1;
                            if i >= args.len() {
                                return Err("--output requires a value".to_string());
                            }
                            config.output_dir = PathBuf::from(&args[i]);
                        }
                        "--with-audio" => {
                            config.with_audio = true;
                        }
                        "--fps" => {
                            i += 1;
                            if i >= args.len() {
                                return Err("--fps requires a value".to_string());
                            }
                            config.fps = Some(args[i].parse()
                                .map_err(|_| "Invalid FPS".to_string())?);
                        }
                        "--save-frames" => {
                            config.save_frames = true;
                        }
                        "--yuv" => {
                            config.pixel_format = PixelFormat::YCbCr_420v;
                        }
                        "--picker" => {
                            config.use_picker = true;
                        }
                        _ => return Err(format!("Unknown option: {}", args[i])),
                    }
                    i += 1;
                }

                Ok(Self {
                    command: Command::Capture(config),
                })
            }
            "screenshot" => {
                let mut display_id = 0;
                let mut output_path = PathBuf::from("screenshot.png");

                let mut i = 2;
                while i < args.len() {
                    match args[i].as_str() {
                        "--display" => {
                            i += 1;
                            if i >= args.len() {
                                return Err("--display requires a value".to_string());
                            }
                            display_id = args[i].parse()
                                .map_err(|_| "Invalid display ID".to_string())?;
                        }
                        "--output" => {
                            i += 1;
                            if i >= args.len() {
                                return Err("--output requires a value".to_string());
                            }
                            output_path = PathBuf::from(&args[i]);
                        }
                        _ => return Err(format!("Unknown option: {}", args[i])),
                    }
                    i += 1;
                }

                Ok(Self {
                    command: Command::Screenshot(ScreenshotConfig {
                        display_id,
                        output_path,
                    }),
                })
            }
            #[cfg(feature = "macos_14_0")]
            "picker" => Ok(Self {
                command: Command::Picker,
            }),
            "help" | "--help" | "-h" => Ok(Self {
                command: Command::Help,
            }),
            _ => Err(format!("Unknown command: {}", args[1])),
        }
    }
}

fn print_help() {
    println!(r#"
ScreenCaptureKit CLI - Screen capture utility

USAGE:
    cli <COMMAND> [OPTIONS]

COMMANDS:
    list <TARGET>              List available capture sources
        displays               List all displays
        windows                List all windows
        apps                   List all applications

    capture [OPTIONS]          Capture screen content
        --display <ID>         Display ID to capture (default: 0)
        --window <NAME>        Window name to capture (instead of display)
        --width <WIDTH>        Output width (default: 1920)
        --height <HEIGHT>      Output height (default: 1080)
        --duration <SECS>      Capture duration in seconds (default: 5)
        --output <PATH>        Output directory (default: ./capture_output)
        --with-audio           Enable audio capture
        --fps <FPS>            Target frame rate
        --save-frames          Save individual frames as PNG files
        --yuv                  Use YUV pixel format instead of BGRA
        --picker               Use system picker UI to select content (macOS 14.0+)

    screenshot [OPTIONS]       Take a single screenshot
        --display <ID>         Display ID to capture (default: 0)
        --output <PATH>        Output file path (default: screenshot.png)
                               Uses SCScreenshotManager (macOS 14.0+) or SCStream fallback

    picker                     Show system picker UI to select content (macOS 14.0+)

    help                       Show this help message

EXAMPLES:
    # List all displays
    cli list displays

    # List all windows
    cli list windows

    # Use picker to select what to capture
    cli picker

    # Capture display 0 for 10 seconds
    cli capture --display 0 --duration 10

    # Capture with picker UI and save frames
    cli capture --picker --duration 5 --save-frames

    # Capture with audio and save frames
    cli capture --display 0 --duration 5 --with-audio --save-frames

    # Capture a specific window
    cli capture --window "Safari" --duration 10

    # Take a screenshot
    cli screenshot --display 0 --output my_screen.png

    # Capture in high quality
    cli capture --display 0 --width 3840 --height 2160 --duration 5

NOTES:
    - This tool requires Screen Recording permission
    - Enable in: System Settings → Privacy & Security → Screen Recording
    - Audio capture requires both Screen Recording and Microphone permissions
    - Picker UI requires macOS 14.0 or later
    - Screenshot uses SCScreenshotManager on macOS 14.0+ (faster, more efficient)
      or falls back to SCStream on earlier versions
"#);
}

fn list_displays() -> Result<(), Box<dyn std::error::Error>> {
    println!("📺 Listing displays...\n");
    
    let content = SCShareableContent::get()?;
    let displays = content.displays();
    
    if displays.is_empty() {
        println!("No displays found.");
        return Ok(());
    }

    for (i, display) in displays.iter().enumerate() {
        println!("Display #{i}");
        println!("  ID: {}", display.display_id());
        println!("  Resolution: {}x{}", display.width(), display.height());
        let frame = display.frame();
        println!("  Frame: origin({:.0}, {:.0}) size({:.0}, {:.0})",
            frame.origin().x, frame.origin().y,
            frame.size().width, frame.size().height);
        println!();
    }

    Ok(())
}

fn list_windows() -> Result<(), Box<dyn std::error::Error>> {
    println!("🪟 Listing windows...\n");
    
    let content = SCShareableContent::get()?;
    let windows = content.windows();
    
    if windows.is_empty() {
        println!("No windows found.");
        return Ok(());
    }

    let on_screen: Vec<_> = windows.iter().filter(|w| w.is_on_screen()).collect();
    
    println!("Total windows: {} (On-screen: {})\n", windows.len(), on_screen.len());

    for (i, window) in on_screen.iter().take(50).enumerate() {
        let title = window.title().unwrap_or_else(|| "<untitled>".to_string());
        let frame = window.frame();
        
        println!("Window #{}", i + 1);
        println!("  ID: {}", window.window_id());
        println!("  Title: {title}");
        println!("  Layer: {}", window.window_layer());
        println!("  Frame: {}x{} at ({:.0}, {:.0})",
            frame.size().width as i32, frame.size().height as i32,
            frame.origin().x, frame.origin().y);
        
        if let Some(app) = window.owning_application() {
            println!("  App: {} ({})",
                app.application_name(),
                app.bundle_identifier());
        }
        println!();
    }

    if on_screen.len() > 50 {
        println!("... and {} more windows", on_screen.len() - 50);
    }

    Ok(())
}

fn list_apps() -> Result<(), Box<dyn std::error::Error>> {
    println!("🖥️  Listing applications...\n");
    
    let content = SCShareableContent::get()?;
    let apps = content.applications();
    
    if apps.is_empty() {
        println!("No applications found.");
        return Ok(());
    }

    println!("Total applications: {}\n", apps.len());

    for (i, app) in apps.iter().enumerate() {
        println!("App #{}", i + 1);
        println!("  Name: {}", app.application_name());
        println!("  Bundle ID: {}", app.bundle_identifier());
        println!("  PID: {}", app.process_id());
        println!();
    }

    Ok(())
}

struct CaptureHandler {
    frame_count: Arc<AtomicUsize>,
    audio_count: Arc<AtomicUsize>,
    start_time: Instant,
    save_frames: bool,
    output_dir: PathBuf,
    saved_frames: Arc<Mutex<Vec<String>>>,
}

// Wrapper types to allow the same handler to be registered multiple times
struct ScreenHandlerWrapper(Arc<CaptureHandler>);
struct AudioHandlerWrapper(Arc<CaptureHandler>);

impl SCStreamOutputTrait for ScreenHandlerWrapper {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        self.0.did_output_sample_buffer(sample, of_type);
    }
}

impl SCStreamOutputTrait for AudioHandlerWrapper {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        self.0.did_output_sample_buffer(sample, of_type);
    }
}

impl SCStreamOutputTrait for CaptureHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        match of_type {
            SCStreamOutputType::Screen => {
                let count = self.frame_count.fetch_add(1, Ordering::Relaxed);
                let elapsed = self.start_time.elapsed().as_secs_f64();
                
                // Get frame status
                let status = sample.get_frame_status();
                
                if count % 30 == 0 {
                    let fps = count as f64 / elapsed;
                    let status_str = status.map(|s| format!(" [{s}]")).unwrap_or_default();
                    println!("📹 Frame {count:5} ({fps:6.2} fps, {elapsed:6.2}s){status_str}");
                }

                // Only try to save frames that have image buffers and are complete
                if self.save_frames && count % 30 == 0 {
                    // Check if this sample has an image buffer before trying to save
                    if sample.get_image_buffer().is_some() {
                        if let Err(e) = self.save_frame(&sample, count) {
                            eprintln!("⚠️  Failed to save frame {count}: {e}");
                        }
                    }
                }
            }
            SCStreamOutputType::Audio => {
                let count = self.audio_count.fetch_add(1, Ordering::Relaxed);
                if count % 100 == 0 {
                    let elapsed = self.start_time.elapsed().as_secs_f64();
                    println!("🔊 Audio sample {count:5} ({elapsed:6.2}s)");
                }
            }
            SCStreamOutputType::Microphone => {
                // Handle microphone audio (macOS 15.0+)
                let count = self.audio_count.fetch_add(1, Ordering::Relaxed);
                if count % 100 == 0 {
                    let elapsed = self.start_time.elapsed().as_secs_f64();
                    println!("🎤 Microphone sample {count:5} ({elapsed:6.2}s)");
                }
            }
        }
    }
}

impl CaptureHandler {
    fn save_frame(&self, sample: &CMSampleBuffer, frame_num: usize) -> Result<(), Box<dyn std::error::Error>> {
        let image_buffer = sample.get_image_buffer()
            .ok_or("No image buffer")?;
        
        let lock_guard = CVImageBufferLockExt::lock(&image_buffer, PixelBufferLockFlags::ReadOnly)?;
        
        let _width = lock_guard.width();  // Not used, actual width from bytes_per_row
        let height = lock_guard.height();
        let bytes_per_row = lock_guard.bytes_per_row();
        let data = lock_guard.as_slice();
        
        // Detect actual pixels per row from stride
        let actual_width = bytes_per_row / 4;  // BGRA = 4 bytes per pixel
        
        // Convert BGRA to RGB using actual dimensions from buffer
        let mut rgb_data = Vec::with_capacity(actual_width * height * 3);
        
        for y in 0..height {
            let row_start = y * bytes_per_row;
            
            for x in 0..actual_width {
                let pixel_offset = row_start + (x * 4);
                if pixel_offset + 3 < data.len() {
                    rgb_data.push(data[pixel_offset + 2]); // R
                    rgb_data.push(data[pixel_offset + 1]); // G
                    rgb_data.push(data[pixel_offset]); // B
                }
            }
        }
        
        let filename = format!("frame_{frame_num:06}.png");
        let path = self.output_dir.join(&filename);
        
        let file = fs::File::create(&path)?;
        let mut encoder = png::Encoder::new(file, actual_width as u32, height as u32);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&rgb_data)?;
        
        if let Ok(mut saved) = self.saved_frames.lock() {
            saved.push(filename);
        }
        
        Ok(())
    }
}

fn run_capture(config: CaptureConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎬 Starting capture...\n");
    println!("Configuration:");
    println!("  Duration: {} seconds", config.duration);
    println!("  Output: {}", config.output_dir.display());
    println!("  Audio: {}", if config.with_audio { "enabled" } else { "disabled" });
    println!("  Save frames: {}", if config.save_frames { "yes" } else { "no" });
    println!("  Pixel format: {:?}", config.pixel_format);
    println!();

    let content = SCShareableContent::get()?;
    
    // Get the content filter
    #[cfg(feature = "macos_14_0")]
    let filter = if config.use_picker {
        println!("🎯 Opening system picker...");
        let mut picker_config = SCContentSharingPickerConfiguration::new();
        picker_config.set_allowed_picker_modes(&[
            SCContentSharingPickerMode::SingleWindow,
            SCContentSharingPickerMode::SingleDisplay,
        ]);

        match SCContentSharingPicker::show(&picker_config) {
            SCContentSharingPickerResult::Display(display) => {
                println!("Selected display: {}x{}", display.width(), display.height());
                #[allow(deprecated)]
                SCContentFilter::new().with_display_excluding_windows(&display, &[])
            }
            SCContentSharingPickerResult::Window(window) => {
                println!("Selected window: {} (ID: {})", 
                    window.title().unwrap_or_else(|| "Untitled".to_string()), 
                    window.window_id());
                #[allow(deprecated)]
                SCContentFilter::new().with_desktop_independent_window(&window)
            }
            SCContentSharingPickerResult::Application(app) => {
                println!("Selected application: {}", app.application_name());
                return Err("Application selection not yet implemented. Please select a window or display.".into());
            }
            SCContentSharingPickerResult::Cancelled => {
                println!("Picker cancelled by user");
                return Ok(());
            }
            SCContentSharingPickerResult::Error(err) => {
                return Err(format!("Picker error: {err}").into());
            }
        }
    } else if let Some(window_name) = &config.window_name {
        println!("Searching for window: {window_name}");
        let windows = content.windows();
        let window = windows.iter()
            .find(|w| {
                if let Some(title) = w.title() {
                    title.contains(window_name)
                } else {
                    false
                }
            })
            .ok_or_else(|| format!("Window '{window_name}' not found"))?;
        
        println!("Found window: {} (ID: {})", window.title().unwrap_or_default(), window.window_id());
        #[allow(deprecated)]
        SCContentFilter::new().with_desktop_independent_window(window)
    } else {
        let mut displays = content.displays();
        if displays.is_empty() {
            return Err("No displays available".into());
        }
        
        let display_idx = config.display_id.unwrap_or(0) as usize;
        if display_idx >= displays.len() {
            return Err(format!("Display {display_idx} not found").into());
        }
        
        let display = displays.remove(display_idx);
        println!("Capturing display: {}x{}", display.width(), display.height());
        
        #[allow(deprecated)]
        SCContentFilter::new().with_display_excluding_windows(&display, &[])
    };

    #[cfg(not(feature = "macos_14_0"))]
    let filter = if config.use_picker {
        return Err("Picker functionality requires the 'macos_14_0' feature flag. Build with: cargo build --example cli --features macos_14_0".into());
    } else if let Some(window_name) = &config.window_name {
        println!("Searching for window: {window_name}");
        let windows = content.windows();
        let window = windows.iter()
            .find(|w| {
                if let Some(title) = w.title() {
                    title.contains(window_name)
                } else {
                    false
                }
            })
            .ok_or_else(|| format!("Window '{window_name}' not found"))?;
        
        println!("Found window: {} (ID: {})", window.title().unwrap_or_default(), window.window_id());
        #[allow(deprecated)]
        SCContentFilter::new().with_desktop_independent_window(window)
    } else {
        let mut displays = content.displays();
        if displays.is_empty() {
            return Err("No displays available".into());
        }
        
        let display_idx = config.display_id.unwrap_or(0) as usize;
        if display_idx >= displays.len() {
            return Err(format!("Display {display_idx} not found").into());
        }
        
        let display = displays.remove(display_idx);
        println!("Capturing display: {}x{}", display.width(), display.height());
        
        #[allow(deprecated)]
        SCContentFilter::new().with_display_excluding_windows(&display, &[])
    };

    // Create configuration
    let mut stream_config = SCStreamConfiguration::build();
    
    if let Some(width) = config.width {
        stream_config = stream_config.set_width(width)?;
    }
    if let Some(height) = config.height {
        stream_config = stream_config.set_height(height)?;
    }
    
    stream_config = stream_config.set_pixel_format(config.pixel_format)?;
    
    if config.with_audio {
        stream_config = stream_config.set_captures_audio(true)?;
        stream_config = stream_config.set_sample_rate(48000)?;
        stream_config = stream_config.set_channel_count(2)?;
    }

    // Create output directory if saving frames
    if config.save_frames {
        fs::create_dir_all(&config.output_dir)?;
        println!("Created output directory: {}", config.output_dir.display());
    }

    // Create shared state for handler
    let frame_count = Arc::new(AtomicUsize::new(0));
    let audio_count = Arc::new(AtomicUsize::new(0));
    let saved_frames = Arc::new(Mutex::new(Vec::new()));

    // Create handler (wrapped in Arc so we can use it for both Screen and Audio)
    let handler = Arc::new(CaptureHandler {
        frame_count: Arc::clone(&frame_count),
        audio_count: Arc::clone(&audio_count),
        start_time: Instant::now(),
        save_frames: config.save_frames,
        output_dir: config.output_dir.clone(),
        saved_frames: Arc::clone(&saved_frames),
    });

    // Create and start stream
    let mut stream = SCStream::new(&filter, &stream_config);
    
    // Need to clone the Arc-wrapped handler for each output type
    let screen_handler = ScreenHandlerWrapper(Arc::clone(&handler));
    stream.add_output_handler(screen_handler, SCStreamOutputType::Screen);
    
    if config.with_audio {
        let audio_handler = AudioHandlerWrapper(Arc::clone(&handler));
        stream.add_output_handler(audio_handler, SCStreamOutputType::Audio);
    }

    println!("\n✅ Capture started!\n");
    stream.start_capture()?;

    // Wait for duration
    thread::sleep(Duration::from_secs(u64::from(config.duration)));

    // Stop capture
    stream.stop_capture()?;

    let final_frames = frame_count.load(Ordering::Relaxed);
    let final_audio = audio_count.load(Ordering::Relaxed);

    println!("\n🎬 Capture complete!");
    println!("\nStatistics:");
    println!("  Video frames: {final_frames}");
    if config.with_audio {
        println!("  Audio samples: {final_audio}");
    }
    
    if config.save_frames {
        if let Ok(saved) = saved_frames.lock() {
            println!("  Saved frames: {}", saved.len());
            if !saved.is_empty() {
                println!("  Output directory: {}", config.output_dir.display());
                println!("  First frame: {}", saved[0]);
                if saved.len() > 1 {
                    println!("  Last frame: {}", saved[saved.len() - 1]);
                }
                println!("\n💡 Tip: View frames with: open {}", config.output_dir.display());
            }
        }
    }

    Ok(())
}

fn take_screenshot(config: ScreenshotConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("📸 Taking screenshot...\n");

    #[cfg(feature = "macos_14_0")]
    {
        use_screenshot_manager(config)
    }

    #[cfg(not(feature = "macos_14_0"))]
    {
        use_stream_screenshot(config)
    }
}

#[cfg(feature = "macos_14_0")]
fn use_screenshot_manager(config: ScreenshotConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("Using SCScreenshotManager (macOS 14.0+)...\n");

    let content = SCShareableContent::get()?;
    let mut displays = content.displays();
    
    if displays.is_empty() {
        return Err("No displays available".into());
    }
    
    if config.display_id as usize >= displays.len() {
        return Err(format!("Display {} not found", config.display_id).into());
    }
    
    let display = displays.remove(config.display_id as usize);
    let display_width = display.width();
    let display_height = display.height();
    println!("Display: {display_width}x{display_height}");

    #[allow(deprecated)]
    let filter = SCContentFilter::new().with_display_excluding_windows(&display, &[]);
    
    // Use the actual display resolution
    let stream_config = SCStreamConfiguration::build()
        .set_width(display_width)?
        .set_height(display_height)?;

    println!("Capturing screenshot...");
    let image = SCScreenshotManager::capture_image(&filter, &stream_config)?;
    
    let width = image.width();
    let height = image.height();
    println!("📷 Captured: {width}x{height}");
    
    // Get RGBA data
    let rgba_pixels = image.get_rgba_data()?;
    
    // Convert RGBA to RGB for PNG
    let mut rgb_data = Vec::with_capacity(width * height * 3);
    for chunk in rgba_pixels.chunks_exact(4) {
        rgb_data.push(chunk[0]); // R
        rgb_data.push(chunk[1]); // G
        rgb_data.push(chunk[2]); // B
        // Skip alpha channel (chunk[3])
    }
    
    // Save as PNG
    let file = fs::File::create(&config.output_path)?;
    let mut encoder = png::Encoder::new(file, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&rgb_data)?;
    
    println!("✅ Screenshot saved: {}", config.output_path.display());
    println!("   Resolution: {width}x{height}");
    
    Ok(())
}

#[cfg(not(feature = "macos_14_0"))]
fn use_stream_screenshot(config: ScreenshotConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("Using SCStream for screenshot (fallback for macOS < 14.0)...\n");

    let content = SCShareableContent::get()?;
    let mut displays = content.displays();
    
    if displays.is_empty() {
        return Err("No displays available".into());
    }
    
    if config.display_id as usize >= displays.len() {
        return Err(format!("Display {} not found", config.display_id).into());
    }
    
    let display = displays.remove(config.display_id as usize);
    let display_width = display.width();
    let display_height = display.height();
    println!("Display: {display_width}x{display_height}");

    #[allow(deprecated)]
    let filter = SCContentFilter::new().with_display_excluding_windows(&display, &[]);
    
    // Use the actual display resolution
    let stream_config = SCStreamConfiguration::build()
        .set_width(display_width)?
        .set_height(display_height)?;

    let frame_captured = Arc::new(AtomicBool::new(false));
    let frame_data: Arc<Mutex<Option<Vec<u8>>>> = Arc::new(Mutex::new(None));
    let frame_width = Arc::new(Mutex::new(0));
    let frame_height = Arc::new(Mutex::new(0));

    struct ScreenshotHandler {
        captured: Arc<AtomicBool>,
        data: Arc<Mutex<Option<Vec<u8>>>>,
        width: Arc<Mutex<usize>>,
        height: Arc<Mutex<usize>>,
    }

    impl SCStreamOutputTrait for ScreenshotHandler {
        fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
            if matches!(of_type, SCStreamOutputType::Screen) && !self.captured.load(Ordering::Relaxed) {
                if let Some(image_buffer) = sample.get_image_buffer() {
                    if let Ok(lock_guard) = CVImageBufferLockExt::lock(&image_buffer, PixelBufferLockFlags::ReadOnly) {
                        let w = lock_guard.width();
                        let h = lock_guard.height();
                        let bytes_per_row = lock_guard.bytes_per_row();
                        let pixels = lock_guard.as_slice();
                        
                        println!("📷 Actual frame: {}x{}, {} bytes_per_row, {} total bytes", w, h, bytes_per_row, pixels.len());
                        
                        // Detect actual pixels per row from stride
                        let actual_width = bytes_per_row / 4;  // BGRA = 4 bytes per pixel
                        
                        // Convert BGRA to RGB using actual dimensions from buffer
                        let mut rgb_data = Vec::with_capacity(actual_width * h * 3);
                        
                        for y in 0..h {
                            let row_start = y * bytes_per_row;
                            
                            for x in 0..actual_width {
                                let pixel_offset = row_start + (x * 4);
                                if pixel_offset + 3 < pixels.len() {
                                    rgb_data.push(pixels[pixel_offset + 2]); // R
                                    rgb_data.push(pixels[pixel_offset + 1]); // G
                                    rgb_data.push(pixels[pixel_offset]); // B
                                }
                            }
                        }
                        
                        *self.data.lock().unwrap() = Some(rgb_data);
                        *self.width.lock().unwrap() = actual_width;
                        *self.height.lock().unwrap() = h;
                        self.captured.store(true, Ordering::Relaxed);
                    }
                }
            }
        }
    }

    let handler = ScreenshotHandler {
        captured: Arc::clone(&frame_captured),
        data: Arc::clone(&frame_data),
        width: Arc::clone(&frame_width),
        height: Arc::clone(&frame_height),
    };

    let mut stream = SCStream::new(&filter, &stream_config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);
    
    stream.start_capture()?;
    
    // Wait for one frame
    let mut attempts = 0;
    while !frame_captured.load(Ordering::Relaxed) && attempts < 100 {
        thread::sleep(Duration::from_millis(50));
        attempts += 1;
    }
    
    stream.stop_capture()?;

    if !frame_captured.load(Ordering::Relaxed) {
        return Err("Failed to capture frame".into());
    }

    // Save the image
    let data = frame_data.lock().unwrap();
    let width = *frame_width.lock().unwrap();
    let height = *frame_height.lock().unwrap();

    if let Some(rgb_data) = data.as_ref() {
        let file = fs::File::create(&config.output_path)?;
        let mut encoder = png::Encoder::new(file, width as u32, height as u32);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        
        let mut writer = encoder.write_header()?;
        writer.write_image_data(rgb_data)?;
        
        println!("✅ Screenshot saved: {}", config.output_path.display());
        println!("   Resolution: {width}x{height}");
    } else {
        return Err("No frame data captured".into());
    }

    Ok(())
}

#[cfg(feature = "macos_14_0")]
fn show_picker() {
    println!("🎯 Opening content sharing picker...\n");
    println!("Select the content you want to share from the system UI.");
    println!("This demonstrates the macOS 14.0+ picker functionality.\n");

    let mut config = SCContentSharingPickerConfiguration::new();
    config.set_allowed_picker_modes(&[
        SCContentSharingPickerMode::SingleWindow,
        SCContentSharingPickerMode::SingleDisplay,
    ]);

    match SCContentSharingPicker::show(&config) {
        SCContentSharingPickerResult::Display(display) => {
            println!("✅ You selected a display:");
            println!("   ID: {}", display.display_id());
            println!("   Resolution: {}x{}", display.width(), display.height());
            let frame = display.frame();
            println!("   Frame: origin({:.0}, {:.0}) size({:.0}, {:.0})",
                frame.origin().x, frame.origin().y,
                frame.size().width, frame.size().height);
        }
        SCContentSharingPickerResult::Window(window) => {
            println!("✅ You selected a window:");
            println!("   ID: {}", window.window_id());
            println!("   Title: {}", window.title().unwrap_or_else(|| "<untitled>".to_string()));
            let frame = window.frame();
            println!("   Size: {}x{}", frame.size().width as i32, frame.size().height as i32);
            if let Some(app) = window.owning_application() {
                println!("   App: {} ({})", app.application_name(), app.bundle_identifier());
            }
        }
        SCContentSharingPickerResult::Application(app) => {
            println!("✅ You selected an application:");
            println!("   Name: {}", app.application_name());
            println!("   Bundle ID: {}", app.bundle_identifier());
            println!("   PID: {}", app.process_id());
        }
        SCContentSharingPickerResult::Cancelled => {
            println!("❌ Picker was cancelled");
        }
        SCContentSharingPickerResult::Error(err) => {
            println!("❌ Picker error: {err}");
        }
    }
}

#[cfg(feature = "macos_14_0")]
fn main() {
    let args = match CliArgs::parse() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {e}\n");
            print_help();
            std::process::exit(1);
        }
    };

    let result = match args.command {
        Command::Help => {
            print_help();
            Ok(())
        }
        Command::List { what } => match what {
            ListTarget::Displays => list_displays(),
            ListTarget::Windows => list_windows(),
            ListTarget::Apps => list_apps(),
        },
        Command::Capture(config) => run_capture(config),
        Command::Screenshot(config) => take_screenshot(config),
        #[cfg(feature = "macos_14_0")]
        Command::Picker => {
            show_picker();
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("\n❌ Error: {e}");
        eprintln!("\nNote: This tool requires Screen Recording permission.");
        eprintln!("Enable in: System Settings → Privacy & Security → Screen Recording");
        std::process::exit(1);
    }
}

#[cfg(not(feature = "macos_14_0"))]
fn main() {
    eprintln!("This example requires the macos_14_0 feature flag.");
    eprintln!("Run with: cargo run --example cli --features macos_14_0");
    std::process::exit(1);
}
