//! Metal Renderer with Overlay UI
//!
//! A real GUI application demonstrating:
//! - Metal rendering with compiled shaders
//! - Screen capture via ScreenCaptureKit with zero-copy IOSurface textures
//! - System content picker (macOS 14.0+) for user-selected capture
//! - Interactive overlay menu with bitmap font rendering
//! - Real-time audio waveform visualization with vertical gain meters
//! - Screenshot capture (macOS 14.0+)
//! - Video recording (macOS 15.0+)
//!
//! ## Controls
//!
//! Menu navigation (when menu visible):
//! - `UP/DOWN` - Navigate menu items
//! - `SPACE/ENTER` - Select item
//! - `ESC/H` - Hide menu
//!
//! Direct controls (when menu hidden):
//! - `P` - Open content picker
//! - `SPACE` - Start/stop capture
//! - `S` - Take screenshot (when source selected)
//! - `R` - Start/stop recording (macOS 15.0+, requires active capture)
//! - `W` - Toggle waveform display
//! - `C` - Open config menu
//! - `M` - Toggle microphone
//! - `H` - Show menu
//! - `Q/ESC` - Quit
//!
//! ## Run
//!
//! ```bash
//! cargo run --example metal_overlay --features macos_14_0
//! ```

mod capture;
mod font;
mod overlay;
mod renderer;
mod ui;
mod vertex;
mod waveform;

use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use cocoa::appkit::NSView;
use cocoa::base::id as cocoa_id;
use core_graphics_types::geometry::CGSize;
use metal::*;
use objc::rc::autoreleasepool;
use objc::runtime::YES;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use screencapturekit::content_sharing_picker::{
    SCContentSharingPicker, SCContentSharingPickerConfiguration, SCContentSharingPickerMode,
    SCPickedSource, SCPickerOutcome,
};
use screencapturekit::prelude::*;
#[cfg(feature = "macos_15_0")]
use screencapturekit::recording_output::{SCRecordingOutput, SCRecordingOutputConfiguration};
use screencapturekit::screenshot_manager::SCScreenshotManager;
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;

use capture::{CaptureHandler, CaptureState};
use font::BitmapFont;
use overlay::{default_stream_config, ConfigMenu, OverlayState};
use renderer::{
    create_pipeline, create_textures_from_iosurface, CaptureTextures, PIXEL_FORMAT_420F,
    PIXEL_FORMAT_420V, SHADER_SOURCE,
};
use vertex::{Uniforms, Vertex, VertexBufferBuilder};

/// Take a screenshot using the best available API
/// - macOS 26.0+: Uses SCScreenshotConfiguration with native file saving
/// - macOS 14.0+: Uses SCStreamConfiguration and CGImage::save_png()
fn take_screenshot(
    filter: &SCContentFilter,
    capture_size: (u32, u32),
    stream_config: &SCStreamConfiguration,
) {
    println!("📸 Taking screenshot...");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = format!("/tmp/screenshot_{}.png", timestamp);

    #[cfg(feature = "macos_26_0")]
    {
        use screencapturekit::screenshot_manager::SCScreenshotConfiguration;
        
        // Use the new macOS 26.0 API with native file saving
        let config = SCScreenshotConfiguration::new()
            .with_width(capture_size.0 as usize)
            .with_height(capture_size.1 as usize)
            .with_shows_cursor(stream_config.shows_cursor())
            .with_file_path(&path);
        
        match SCScreenshotManager::capture_screenshot(filter, &config) {
            Ok(output) => {
                if let Some(url) = output.file_url() {
                    println!("✅ Screenshot saved to {}", url);
                    let _ = std::process::Command::new("open").arg(&url).spawn();
                } else if let Some(image) = output.sdr_image() {
                    println!("✅ Screenshot captured: {}x{}", image.width(), image.height());
                    match image.save_png(&path) {
                        Ok(()) => {
                            println!("📁 Saved to {}", path);
                            let _ = std::process::Command::new("open").arg(&path).spawn();
                        }
                        Err(e) => eprintln!("❌ Failed to save: {:?}", e),
                    }
                }
            }
            Err(e) => eprintln!("❌ Screenshot failed: {:?}", e),
        }
    }

    #[cfg(not(feature = "macos_26_0"))]
    {
        // Use macOS 14.0+ API
        let screenshot_config = SCStreamConfiguration::new()
            .with_width(capture_size.0)
            .with_height(capture_size.1)
            .with_shows_cursor(stream_config.shows_cursor());
        
        match SCScreenshotManager::capture_image(filter, &screenshot_config) {
            Ok(image) => {
                println!("✅ Screenshot captured: {}x{}", image.width(), image.height());
                match image.save_png(&path) {
                    Ok(()) => {
                        println!("📁 Saved to {}", path);
                        let _ = std::process::Command::new("open").arg(&path).spawn();
                    }
                    Err(e) => eprintln!("❌ Failed to save: {:?}", e),
                }
            }
            Err(e) => eprintln!("❌ Screenshot failed: {:?}", e),
        }
    }
}

fn main() {
    println!("🎮 Metal Overlay Renderer");
    println!("========================\n");

    // Create window
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .with_title("ScreenCaptureKit Metal Overlay")
        .build(&event_loop)
        .unwrap();

    // Initialize Metal
    let device = Device::system_default().expect("No Metal device found");
    println!("🖥️  Metal device: {}", device.name());

    let mut layer = MetalLayer::new();
    layer.set_device(&device);
    layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
    layer.set_presents_with_transaction(false);

    // Attach layer to window
    unsafe {
        match window.raw_window_handle() {
            RawWindowHandle::AppKit(handle) => {
                let view = handle.ns_view as cocoa_id;
                view.setWantsLayer(YES);
                view.setLayer(std::mem::transmute(layer.as_mut()));
            }
            _ => panic!("Unsupported window handle"),
        }
    }

    let draw_size = window.inner_size();
    layer.set_drawable_size(CGSize::new(draw_size.width as f64, draw_size.height as f64));

    // Compile shaders at runtime from embedded source
    println!("🔧 Compiling shaders...");
    let compile_options = CompileOptions::new();
    let library = device
        .new_library_with_source(SHADER_SOURCE, &compile_options)
        .expect("Failed to compile shaders");
    println!("✅ Shaders compiled");

    let overlay_pipeline = create_pipeline(&device, &library, "vertex_colored", "fragment_colored");

    // Create fullscreen textured pipeline (no blending for background) - for BGRA/RGB formats
    let fullscreen_pipeline = {
        let vert = library.get_function("vertex_fullscreen", None).unwrap();
        let frag = library.get_function("fragment_textured", None).unwrap();
        let desc = RenderPipelineDescriptor::new();
        desc.set_vertex_function(Some(&vert));
        desc.set_fragment_function(Some(&frag));
        desc.color_attachments()
            .object_at(0)
            .unwrap()
            .set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        device.new_render_pipeline_state(&desc).unwrap()
    };

    // Create YCbCr pipeline for biplanar YCbCr formats (420v/420f)
    let ycbcr_pipeline = {
        let vert = library.get_function("vertex_fullscreen", None).unwrap();
        let frag = library.get_function("fragment_ycbcr", None).unwrap();
        let desc = RenderPipelineDescriptor::new();
        desc.set_vertex_function(Some(&vert));
        desc.set_fragment_function(Some(&frag));
        desc.color_attachments()
            .object_at(0)
            .unwrap()
            .set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        device.new_render_pipeline_state(&desc).unwrap()
    };

    let command_queue = device.new_command_queue();

    // Create shared capture state
    let capture_state = Arc::new(CaptureState::new());
    let font = BitmapFont::new();
    let mut overlay = OverlayState::new();
    let capturing = Arc::new(AtomicBool::new(false));
    let mut stream_config = default_stream_config();
    let mut mic_device_idx: Option<usize> = None; // Track mic device selection separately

    // Helper to format picked source for display
    fn format_picked_source(source: &SCPickedSource) -> String {
        match source {
            SCPickedSource::Window(name) => {
                format!("[W] {}", name.chars().take(20).collect::<String>())
            }
            SCPickedSource::Display(id) => format!("[D] Display {}", id),
            SCPickedSource::Application(name) => {
                format!("[A] {}", name.chars().take(20).collect::<String>())
            }
            SCPickedSource::Unknown => "None".to_string(),
        }
    }

    // Screen capture setup
    let mut stream: Option<SCStream> = None;
    let mut current_filter: Option<SCContentFilter> = None;
    let mut capture_size: (u32, u32) = (1920, 1080);
    let mut picked_source = SCPickedSource::Unknown;

    // Recording state (macOS 15.0+)
    #[cfg(feature = "macos_15_0")]
    let mut recording_output: Option<SCRecordingOutput> = None;
    #[cfg(feature = "macos_15_0")]
    let mut recording_path: Option<String> = None;
    let recording = Arc::new(AtomicBool::new(false));

    // Shared state for picker callback results (filter, width, height, source info)
    type PickerResult = Option<(SCContentFilter, u32, u32, SCPickedSource)>;
    let pending_picker: Arc<Mutex<PickerResult>> = Arc::new(Mutex::new(None));

    let mut vertex_builder = VertexBufferBuilder::new();
    let mut time = 0.0f32;

    println!("🎮 Press SPACE to open content picker");

    // Event loop
    event_loop.run(move |event, _, control_flow| {
        autoreleasepool(|| {
            *control_flow = ControlFlow::Poll;

            // Check for pending picker results - update filter if capturing, otherwise just store
            if let Ok(mut pending) = pending_picker.try_lock() {
                if let Some((filter, width, height, source)) = pending.take() {
                    println!(
                        "✅ Content selected: {}x{} - {}",
                        width,
                        height,
                        format_picked_source(&source)
                    );
                    capture_size = (width, height);
                    picked_source = source;

                    // If already capturing, update the filter live
                    if capturing.load(Ordering::Relaxed) {
                        if let Some(ref s) = stream {
                            match s.update_content_filter(&filter) {
                                Ok(()) => println!("✅ Source updated live"),
                                Err(e) => eprintln!("❌ Failed to update source: {:?}", e),
                            }
                        }
                    }
                    current_filter = Some(filter);
                }
            }

            match event {
                Event::MainEventsCleared => window.request_redraw(),

                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::ExitWithCode(0),

                    WindowEvent::Resized(size) => {
                        layer.set_drawable_size(CGSize::new(size.width as f64, size.height as f64));
                    }

                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(keycode),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        // Helper closure to open picker (without stream - for initial selection)
                        let open_picker_no_stream = |pending_picker: &Arc<Mutex<PickerResult>>| {
                            println!("📺 Opening content picker...");
                            let mut config = SCContentSharingPickerConfiguration::new();
                            config.set_allowed_picker_modes(&[
                                SCContentSharingPickerMode::SingleWindow,
                                SCContentSharingPickerMode::MultipleWindows,
                                SCContentSharingPickerMode::SingleDisplay,
                                SCContentSharingPickerMode::SingleApplication,
                                SCContentSharingPickerMode::MultipleApplications,
                            ]);
                            let pending = Arc::clone(pending_picker);

                            SCContentSharingPicker::show(&config, move |outcome| {
                                match outcome {
                                    SCPickerOutcome::Picked(result) => {
                                        let (width, height) = result.pixel_size();
                                        let filter = result.filter();
                                        let source = result.source();

                                        if let Ok(mut pending) = pending.lock() {
                                            *pending = Some((filter, width, height, source));
                                        }
                                    }
                                    SCPickerOutcome::Cancelled => {
                                        println!("⚠️  Picker cancelled");
                                    }
                                    SCPickerOutcome::Error(e) => {
                                        eprintln!("❌ Picker error: {}", e);
                                    }
                                }
                            });
                        };

                        // Helper closure to open picker for existing stream
                        let open_picker_for_stream =
                            |pending_picker: &Arc<Mutex<PickerResult>>, stream: &SCStream| {
                                println!("📺 Opening content picker for stream...");
                                let mut config = SCContentSharingPickerConfiguration::new();
                                config.set_allowed_picker_modes(&[
                                    SCContentSharingPickerMode::SingleWindow,
                                    SCContentSharingPickerMode::MultipleWindows,
                                    SCContentSharingPickerMode::SingleDisplay,
                                    SCContentSharingPickerMode::SingleApplication,
                                    SCContentSharingPickerMode::MultipleApplications,
                                ]);
                                let pending = Arc::clone(pending_picker);

                                SCContentSharingPicker::show_for_stream(
                                    &config,
                                    stream,
                                    move |outcome| match outcome {
                                        SCPickerOutcome::Picked(result) => {
                                            let (width, height) = result.pixel_size();
                                            let filter = result.filter();
                                            let source = result.source();

                                            if let Ok(mut pending) = pending.lock() {
                                                *pending = Some((filter, width, height, source));
                                            }
                                        }
                                        SCPickerOutcome::Cancelled => {
                                            println!("⚠️  Picker cancelled");
                                        }
                                        SCPickerOutcome::Error(e) => {
                                            eprintln!("❌ Picker error: {}", e);
                                        }
                                    },
                                );
                            };

                        // Helper closure to start capture with existing filter
                        let start_capture = |stream: &mut Option<SCStream>,
                                             current_filter: &Option<SCContentFilter>,
                                             capture_size: (u32, u32),
                                             stream_config: &SCStreamConfiguration,
                                             capture_state: &Arc<CaptureState>,
                                             capturing: &Arc<AtomicBool>,
                                             mic_only: bool| {
                            // Get the filter to use
                            let filter_to_use = if let Some(ref filter) = current_filter {
                                filter.clone()
                            } else if mic_only {
                                // For mic-only capture, we still need a valid display filter
                                // macOS requires a content filter even for audio-only capture
                                println!("🎤 Starting mic-only capture (using main display)");
                                match screencapturekit::shareable_content::SCShareableContent::get()
                                {
                                    Ok(content) => {
                                        let displays = content.displays();
                                        if let Some(display) = displays.first() {
                                            SCContentFilter::builder().display(display).build()
                                        } else {
                                            println!(
                                                "❌ No displays available for mic-only capture"
                                            );
                                            return;
                                        }
                                    }
                                    Err(e) => {
                                        println!("❌ Failed to get shareable content: {:?}", e);
                                        return;
                                    }
                                }
                            } else {
                                println!("⚠️  No content selected. Open picker first.");
                                return;
                            };

                            let (width, height) = capture_size;
                            // Clone config and update dimensions
                            let mut sc_config = stream_config.clone();
                            sc_config.set_width(width);
                            sc_config.set_height(height);

                            let handler = CaptureHandler {
                                state: Arc::clone(capture_state),
                            };

                            let mut s = SCStream::new(&filter_to_use, &sc_config);
                            if !mic_only {
                                s.add_output_handler(handler.clone(), SCStreamOutputType::Screen);
                                s.add_output_handler(handler.clone(), SCStreamOutputType::Audio);
                            }
                            s.add_output_handler(handler, SCStreamOutputType::Microphone);

                            match s.start_capture() {
                                Ok(()) => {
                                    capturing.store(true, Ordering::Relaxed);
                                    *stream = Some(s);
                                    println!("✅ Capture started");
                                }
                                Err(e) => {
                                    eprintln!("❌ Failed to start capture: {:?}", e);
                                }
                            }
                        };

                        // Helper closure to stop capture
                        let stop_capture =
                            |stream: &mut Option<SCStream>, capturing: &Arc<AtomicBool>| {
                                println!("⏹️  Stopping capture...");
                                if let Some(ref mut s) = stream {
                                    let _ = s.stop_capture();
                                }
                                *stream = None;
                                capturing.store(false, Ordering::Relaxed);
                                println!("✅ Capture stopped");
                            };

                        // Handle menu navigation when help is shown
                        if overlay.show_help && !overlay.show_config {
                            match keycode {
                                VirtualKeyCode::Up => {
                                    if overlay.menu_selection > 0 {
                                        overlay.menu_selection -= 1;
                                        println!(
                                            "⬆️  Menu selection: {} ({})",
                                            overlay.menu_selection,
                                            OverlayState::MENU_ITEMS[overlay.menu_selection]
                                        );
                                    }
                                }
                                VirtualKeyCode::Down => {
                                    let max = OverlayState::menu_count().saturating_sub(1);
                                    if overlay.menu_selection < max {
                                        overlay.menu_selection += 1;
                                        println!(
                                            "⬇️  Menu selection: {} ({})",
                                            overlay.menu_selection,
                                            OverlayState::MENU_ITEMS[overlay.menu_selection]
                                        );
                                    }
                                }
                                VirtualKeyCode::Return | VirtualKeyCode::Space => {
                                    match overlay.menu_selection {
                                        0 => {
                                            // Picker
                                            if let Some(ref s) = stream {
                                                open_picker_for_stream(&pending_picker, s);
                                            } else {
                                                open_picker_no_stream(&pending_picker);
                                            }
                                        }
                                        1 => {
                                            // Capture start/stop (works with or without source)
                                            if capturing.load(Ordering::Relaxed) {
                                                stop_capture(&mut stream, &capturing);
                                            } else {
                                                // If we have a filter, use it; otherwise capture mic-only
                                                let mic_only = current_filter.is_none();
                                                start_capture(
                                                    &mut stream,
                                                    &current_filter,
                                                    capture_size,
                                                    &stream_config,
                                                    &capture_state,
                                                    &capturing,
                                                    mic_only,
                                                );
                                            }
                                        }
                                        2 => {
                                            // Screenshot
                                            if let Some(ref filter) = current_filter {
                                                take_screenshot(filter, capture_size, &stream_config);
                                            } else {
                                                println!("⚠️  Select a source first with Picker");
                                            }
                                        }
                                        3 => {
                                            // Record (macOS 15.0+)
                                            #[cfg(feature = "macos_15_0")]
                                            {
                                                if recording.load(Ordering::Relaxed) {
                                                    // Stop recording
                                                    if let Some(ref s) = stream {
                                                        if let Some(ref rec) = recording_output {
                                                            println!("⏹️  Stopping recording...");
                                                            let _ = s.remove_recording_output(rec);
                                                            recording.store(false, Ordering::Relaxed);
                                                            println!("✅ Recording stopped");
                                                            // Open the recorded file
                                                            if let Some(ref path) = recording_path {
                                                                println!("📁 Saved to {}", path);
                                                                let _ = std::process::Command::new("open").arg(path).spawn();
                                                            }
                                                        }
                                                    }
                                                    recording_output = None;
                                                    recording_path = None;
                                                } else if current_filter.is_some() {
                                                    // Start recording
                                                    let timestamp = std::time::SystemTime::now()
                                                        .duration_since(std::time::UNIX_EPOCH)
                                                        .map(|d| d.as_secs())
                                                        .unwrap_or(0);
                                                    let path = format!("/tmp/recording_{}.mp4", timestamp);
                                                    recording_path = Some(path.clone());
                                                    
                                                    let config = SCRecordingOutputConfiguration::new()
                                                        .with_output_url(std::path::Path::new(&path));
                                                    
                                                    if let Some(rec) = SCRecordingOutput::new(&config) {
                                                        if let Some(ref s) = stream {
                                                            match s.add_recording_output(&rec) {
                                                                Ok(()) => {
                                                                    println!("🔴 Recording started: {}", path);
                                                                    recording.store(true, Ordering::Relaxed);
                                                                    recording_output = Some(rec);
                                                                }
                                                                Err(e) => eprintln!("❌ Failed to start recording: {:?}", e),
                                                            }
                                                        } else {
                                                            println!("⚠️  Start capture first, then record");
                                                        }
                                                    } else {
                                                        eprintln!("❌ Failed to create recording output");
                                                    }
                                                } else {
                                                    println!("⚠️  Select a source first with Picker");
                                                }
                                            }
                                            #[cfg(not(feature = "macos_15_0"))]
                                            {
                                                println!("⚠️  Recording requires macOS 15.0+ (macos_15_0 feature)");
                                            }
                                        }
                                        4 => {
                                            // Config
                                            overlay.show_config = true;
                                            overlay.show_help = false;
                                        }
                                        5 => {
                                            // Quit
                                            *control_flow = ControlFlow::ExitWithCode(0);
                                        }
                                        _ => {}
                                    }
                                }
                                VirtualKeyCode::Escape | VirtualKeyCode::H => {
                                    overlay.show_help = false;
                                }
                                VirtualKeyCode::Q => {
                                    *control_flow = ControlFlow::ExitWithCode(0);
                                }
                                _ => {}
                            }
                        }
                        // Handle config menu navigation
                        else if overlay.show_config {
                            match keycode {
                                VirtualKeyCode::Up => {
                                    if overlay.config_selection > 0 {
                                        overlay.config_selection -= 1;
                                    }
                                }
                                VirtualKeyCode::Down => {
                                    let max = ConfigMenu::option_count().saturating_sub(1);
                                    if overlay.config_selection < max {
                                        overlay.config_selection += 1;
                                    }
                                }
                                VirtualKeyCode::Left | VirtualKeyCode::Right => {
                                    let increase = keycode == VirtualKeyCode::Right;
                                    ConfigMenu::toggle_or_adjust(
                                        &mut stream_config,
                                        &mut mic_device_idx,
                                        overlay.config_selection,
                                        increase,
                                    );
                                    // Immediately apply config to running stream
                                    if capturing.load(Ordering::Relaxed) {
                                        if let Some(ref s) = stream {
                                            let mut new_config = stream_config.clone();
                                            new_config.set_width(capture_size.0);
                                            new_config.set_height(capture_size.1);
                                            if let Err(e) = s.update_configuration(&new_config) {
                                                eprintln!("❌ Config update failed: {:?}", e);
                                            }
                                        }
                                    }
                                }
                                VirtualKeyCode::Return | VirtualKeyCode::Space => {
                                    // Toggle current option (same as Right arrow)
                                    ConfigMenu::toggle_or_adjust(
                                        &mut stream_config,
                                        &mut mic_device_idx,
                                        overlay.config_selection,
                                        true,
                                    );
                                    if capturing.load(Ordering::Relaxed) {
                                        if let Some(ref s) = stream {
                                            let mut new_config = stream_config.clone();
                                            new_config.set_width(capture_size.0);
                                            new_config.set_height(capture_size.1);
                                            if let Err(e) = s.update_configuration(&new_config) {
                                                eprintln!("❌ Config update failed: {:?}", e);
                                            }
                                        }
                                    }
                                }
                                VirtualKeyCode::Escape | VirtualKeyCode::Back => {
                                    overlay.show_config = false;
                                    overlay.show_help = true;
                                }
                                VirtualKeyCode::Q => {
                                    *control_flow = ControlFlow::ExitWithCode(0);
                                }
                                _ => {}
                            }
                        }
                        // Default key handling (no menu shown)
                        else {
                            match keycode {
                                VirtualKeyCode::Space => {
                                    // Toggle capture on/off
                                    if capturing.load(Ordering::Relaxed) {
                                        stop_capture(&mut stream, &capturing);
                                    } else {
                                        start_capture(
                                            &mut stream,
                                            &current_filter,
                                            capture_size,
                                            &stream_config,
                                            &capture_state,
                                            &capturing,
                                            false,
                                        );
                                    }
                                }
                                VirtualKeyCode::P => {
                                    if let Some(ref s) = stream {
                                        open_picker_for_stream(&pending_picker, s);
                                    } else {
                                        open_picker_no_stream(&pending_picker);
                                    }
                                }
                                VirtualKeyCode::W => {
                                    overlay.show_waveform = !overlay.show_waveform;
                                }
                                VirtualKeyCode::H => {
                                    overlay.show_help = true;
                                }
                                VirtualKeyCode::C => {
                                    overlay.show_config = true;
                                }
                                VirtualKeyCode::M => {
                                    let new_val = !stream_config.captures_microphone();
                                    stream_config.set_captures_microphone(new_val);
                                    println!(
                                        "🎤 Microphone: {}",
                                        if new_val { "On" } else { "Off" }
                                    );
                                    if capturing.load(Ordering::Relaxed) {
                                        if let Some(ref s) = stream {
                                            let mut new_config = stream_config.clone();
                                            new_config.set_width(capture_size.0);
                                            new_config.set_height(capture_size.1);
                                            match s.update_configuration(&new_config) {
                                                Ok(()) => println!("✅ Config updated"),
                                                Err(e) => {
                                                    eprintln!("❌ Config update failed: {:?}", e)
                                                }
                                            }
                                        }
                                    }
                                }
                                VirtualKeyCode::S => {
                                    // Screenshot shortcut
                                    if let Some(ref filter) = current_filter {
                                        take_screenshot(filter, capture_size, &stream_config);
                                    } else {
                                        println!("⚠️  Select a source first with P or menu");
                                    }
                                }
                                VirtualKeyCode::R => {
                                    // Recording shortcut (macOS 15.0+)
                                    #[cfg(feature = "macos_15_0")]
                                    {
                                        if recording.load(Ordering::Relaxed) {
                                            // Stop recording
                                            if let Some(ref s) = stream {
                                                if let Some(ref rec) = recording_output {
                                                    println!("⏹️  Stopping recording...");
                                                    let _ = s.remove_recording_output(rec);
                                                    recording.store(false, Ordering::Relaxed);
                                                    println!("✅ Recording stopped");
                                                    if let Some(ref path) = recording_path {
                                                        println!("📁 Saved to {}", path);
                                                        let _ = std::process::Command::new("open").arg(path).spawn();
                                                    }
                                                }
                                            }
                                            recording_output = None;
                                            recording_path = None;
                                        } else if current_filter.is_some() && stream.is_some() {
                                            // Start recording
                                            let timestamp = std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .map(|d| d.as_secs())
                                                .unwrap_or(0);
                                            let path = format!("/tmp/recording_{}.mp4", timestamp);
                                            recording_path = Some(path.clone());
                                            
                                            let config = SCRecordingOutputConfiguration::new()
                                                .with_output_url(std::path::Path::new(&path));
                                            
                                            if let Some(rec) = SCRecordingOutput::new(&config) {
                                                if let Some(ref s) = stream {
                                                    match s.add_recording_output(&rec) {
                                                        Ok(()) => {
                                                            println!("🔴 Recording started: {}", path);
                                                            recording.store(true, Ordering::Relaxed);
                                                            recording_output = Some(rec);
                                                        }
                                                        Err(e) => eprintln!("❌ Failed to start recording: {:?}", e),
                                                    }
                                                }
                                            }
                                        } else {
                                            println!("⚠️  Start capture first (P then Space), then R to record");
                                        }
                                    }
                                    #[cfg(not(feature = "macos_15_0"))]
                                    {
                                        println!("⚠️  Recording requires macOS 15.0+ (macos_15_0 feature)");
                                    }
                                }
                                VirtualKeyCode::Escape | VirtualKeyCode::Q => {
                                    *control_flow = ControlFlow::ExitWithCode(0);
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                },

                Event::RedrawRequested(_) => {
                    time += 0.016;

                    let size = window.inner_size();
                    let width = size.width as f32;
                    let height = size.height as f32;

                    // Try to get the latest IOSurface and create textures from it (zero-copy)
                    let mut capture_textures: Option<CaptureTextures> = None;
                    let mut tex_width = capture_size.0 as f32;
                    let mut tex_height = capture_size.1 as f32;
                    let mut pixel_format: u32 = 0;

                    if capturing.load(Ordering::Relaxed) {
                        if let Ok(guard) = capture_state.latest_surface.try_lock() {
                            if let Some(ref surface) = *guard {
                                tex_width = surface.width() as f32;
                                tex_height = surface.height() as f32;
                                // Create Metal textures directly from IOSurface (zero-copy)
                                capture_textures = unsafe {
                                    create_textures_from_iosurface(&device, surface.as_ptr())
                                };
                                if let Some(ref ct) = capture_textures {
                                    pixel_format = ct.pixel_format;
                                }
                            }
                        }
                    }

                    // Build vertex buffer for this frame
                    vertex_builder.clear();

                    // Status bar background
                    vertex_builder.rect(0.0, 0.0, width, 32.0, [0.1, 0.1, 0.12, 0.9]);

                    // Status text - include audio sample counts and peaks for debugging
                    let fps = capture_state.frame_count.load(Ordering::Relaxed);
                    let audio_samples_cnt =
                        capture_state.audio_waveform.lock().unwrap().sample_count();
                    let mic_samples_cnt = capture_state.mic_waveform.lock().unwrap().sample_count();
                    let audio_peak = capture_state.audio_waveform.lock().unwrap().peak(512);
                    let mic_peak = capture_state.mic_waveform.lock().unwrap().peak(512);
                    let status = if capture_textures.is_some() {
                        format!(
                            "LIVE {}x{} F:{} A:{}k P:{:.2} M:{}k P:{:.2}",
                            tex_width as u32,
                            tex_height as u32,
                            fps,
                            audio_samples_cnt / 1000,
                            audio_peak,
                            mic_samples_cnt / 1000,
                            mic_peak
                        )
                    } else if capturing.load(Ordering::Relaxed) {
                        format!("Starting... {}", fps)
                    } else {
                        "H=Menu".to_string()
                    };
                    vertex_builder.text(&font, &status, 8.0, 8.0, 2.0, [0.2, 1.0, 0.3, 1.0]);

                    // Waveform bar at top - 100% width with both system audio and mic
                    if overlay.show_waveform && capturing.load(Ordering::Relaxed) {
                        let single_wave_h = 40.0;
                        let wave_spacing = 4.0;
                        let total_wave_h = single_wave_h * 2.0 + wave_spacing;
                        let bar_y = 36.0; // Below status bar
                        let meter_w = 24.0;
                        let padding = 8.0;
                        let label_w = 24.0; // Space for labels

                        // Waveform background - full width
                        vertex_builder.rect(
                            0.0,
                            bar_y,
                            width,
                            total_wave_h + 12.0,
                            [0.08, 0.08, 0.1, 0.9],
                        );

                        // Calculate waveform area (leave space for labels on left and meters on right)
                        let meters_space = meter_w * 2.0 + padding * 3.0;
                        let wave_w = width - meters_space - padding - label_w;
                        let wave_x = padding + label_w;

                        // System audio waveform (top) - cyan/green
                        let audio_wave_y = bar_y + 4.0;
                        vertex_builder.text(
                            &font,
                            "SYS",
                            padding,
                            audio_wave_y + single_wave_h / 2.0 - 4.0,
                            1.0,
                            [0.0, 0.9, 0.8, 0.7],
                        );
                        let audio_samples =
                            capture_state.audio_waveform.lock().unwrap().display_samples(512);
                        vertex_builder.waveform(
                            &audio_samples,
                            wave_x,
                            audio_wave_y,
                            wave_w,
                            single_wave_h,
                            [0.0, 0.9, 0.8, 0.9], // Cyan (system audio)
                        );

                        // Microphone waveform (bottom) - magenta/pink
                        let mic_wave_y = audio_wave_y + single_wave_h + wave_spacing;
                        vertex_builder.text(
                            &font,
                            "MIC",
                            padding,
                            mic_wave_y + single_wave_h / 2.0 - 4.0,
                            1.0,
                            [1.0, 0.3, 0.7, 0.7],
                        );
                        let mic_samples =
                            capture_state.mic_waveform.lock().unwrap().display_samples(512);
                        vertex_builder.waveform(
                            &mic_samples,
                            wave_x,
                            mic_wave_y,
                            wave_w,
                            single_wave_h,
                            [1.0, 0.3, 0.7, 0.9], // Magenta (mic)
                        );

                        // Vertical meters on the right
                        let meters_x = width - meters_space + padding;

                        // System audio vertical meter
                        let audio_level = capture_state.audio_waveform.lock().unwrap().rms(2048);
                        vertex_builder.vu_meter_vertical(
                            audio_level,
                            meters_x,
                            audio_wave_y,
                            meter_w,
                            total_wave_h,
                            "S",
                            &font,
                        );

                        // Microphone vertical meter
                        let mic_level = capture_state.mic_waveform.lock().unwrap().rms(2048);
                        vertex_builder.vu_meter_vertical(
                            mic_level,
                            meters_x + meter_w + padding,
                            audio_wave_y,
                            meter_w,
                            total_wave_h,
                            "M",
                            &font,
                        );
                    }

                    // Help overlay - responsive centered
                    if overlay.show_help {
                        let source_str = format_picked_source(&picked_source);
                        vertex_builder.help_overlay(
                            &font,
                            width,
                            height,
                            capturing.load(Ordering::Relaxed),
                            recording.load(Ordering::Relaxed),
                            &source_str,
                            overlay.menu_selection,
                        );
                    }

                    // Config menu overlay
                    if overlay.show_config {
                        let source_str = format_picked_source(&picked_source);
                        vertex_builder.config_menu(
                            &font,
                            width,
                            height,
                            &stream_config,
                            mic_device_idx,
                            overlay.config_selection,
                            capturing.load(Ordering::Relaxed),
                            &source_str,
                        );
                    }

                    // Build GPU buffer
                    let vertex_buffer = vertex_builder.build(&device);
                    vertex_buffer.did_modify_range(metal::NSRange::new(
                        0,
                        (vertex_builder.vertex_count() * size_of::<Vertex>()) as u64,
                    ));

                    // Uniforms - pass capture texture dimensions for aspect ratio
                    let uniforms = Uniforms {
                        viewport_size: [width, height],
                        texture_size: [tex_width, tex_height],
                        time,
                        pixel_format,
                        _padding: [0.0; 2],
                    };
                    let uniforms_buffer = device.new_buffer_with_data(
                        (&uniforms as *const Uniforms).cast(),
                        size_of::<Uniforms>() as u64,
                        MTLResourceOptions::CPUCacheModeDefaultCache,
                    );

                    // Render
                    let drawable = match layer.next_drawable() {
                        Some(d) => d,
                        None => return,
                    };

                    let render_pass = RenderPassDescriptor::new();
                    let attachment = render_pass.color_attachments().object_at(0).unwrap();
                    attachment.set_texture(Some(drawable.texture()));
                    attachment.set_load_action(MTLLoadAction::Clear);
                    attachment.set_clear_color(MTLClearColor::new(0.08, 0.08, 0.1, 1.0));
                    attachment.set_store_action(MTLStoreAction::Store);

                    let cmd_buffer = command_queue.new_command_buffer();
                    let encoder = cmd_buffer.new_render_command_encoder(render_pass);

                    // First pass: Draw captured frame as background (if available)
                    if let Some(ref textures) = capture_textures {
                        let is_ycbcr = textures.pixel_format == PIXEL_FORMAT_420V
                            || textures.pixel_format == PIXEL_FORMAT_420F;

                        if is_ycbcr && textures.plane1.is_some() {
                            // Use YCbCr pipeline for biplanar formats
                            encoder.set_render_pipeline_state(&ycbcr_pipeline);
                            encoder.set_vertex_buffer(0, Some(&uniforms_buffer), 0);
                            encoder.set_fragment_texture(0, Some(&textures.plane0));
                            encoder
                                .set_fragment_texture(1, Some(textures.plane1.as_ref().unwrap()));
                            encoder.set_fragment_buffer(0, Some(&uniforms_buffer), 0);
                        } else {
                            // Use standard BGRA/RGB pipeline
                            encoder.set_render_pipeline_state(&fullscreen_pipeline);
                            encoder.set_vertex_buffer(0, Some(&uniforms_buffer), 0);
                            encoder.set_fragment_texture(0, Some(&textures.plane0));
                        }
                        encoder.draw_primitives(MTLPrimitiveType::TriangleStrip, 0, 4);
                    }

                    // Second pass: Draw overlay UI
                    encoder.set_render_pipeline_state(&overlay_pipeline);
                    encoder.set_vertex_buffer(0, Some(&vertex_buffer), 0);
                    encoder.set_vertex_buffer(1, Some(&uniforms_buffer), 0);
                    encoder.draw_primitives(
                        MTLPrimitiveType::Triangle,
                        0,
                        vertex_builder.vertex_count() as u64,
                    );
                    encoder.end_encoding();

                    cmd_buffer.present_drawable(drawable);
                    cmd_buffer.commit();
                }
                _ => {}
            }
        });
    });
}
