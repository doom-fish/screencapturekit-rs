//! Full Metal Application
//!
//! A complete macOS application demonstrating the full `ScreenCaptureKit` API:
//!
//! - **Metal GPU Rendering** - Hardware-accelerated graphics with runtime shader compilation
//! - **Screen Capture** - Real-time display/window capture via `ScreenCaptureKit`
//! - **Content Picker** - System UI for selecting capture source (macOS 14.0+)
//! - **Audio Visualization** - Real-time waveform display with VU meters
//! - **Screenshot Capture** - Single-frame capture with HDR support (macOS 14.0+/26.0+)
//! - **Video Recording** - Direct-to-file recording (macOS 15.0+)
//! - **Microphone Capture** - Audio input with device selection (macOS 15.0+)
//! - **Bitmap Font Rendering** - Custom 8x8 pixel glyph overlay text
//! - **Interactive Menu** - Keyboard-navigable settings UI
//!
//! ## Running
//!
//! ```bash
//! # Basic (macOS 14.0+)
//! cargo run --example 16_full_metal_app --features macos_14_0
//!
//! # With recording support (macOS 15.0+)
//! cargo run --example 16_full_metal_app --features macos_15_0
//!
//! # With HDR screenshots (macOS 26.0+)
//! cargo run --example 16_full_metal_app --features macos_26_0
//! ```
//!
//! ## Controls
//!
//! **Initial Menu** (before picking a source):
//! - `↑`/`↓` - Navigate menu items
//! - `Enter` - Pick Source / Quit
//!
//! **Main Menu** (after picking - capture auto-starts):
//! - `↑`/`↓` - Navigate menu items
//! - `Enter` - Select (Stop/Start, Screenshot, Record, Config, Change Source, Quit)
//! - `Esc`/`H` - Hide menu
//!
//! **Direct Controls** (when menu hidden):
//! - `P` - Open content picker
//! - `Space` - Start/stop capture
//! - `S` - Take screenshot
//! - `R` - Start/stop recording (macOS 15.0+)
//! - `W` - Toggle waveform display
//! - `M` - Toggle microphone
//! - `C` - Open config menu
//! - `H` - Show menu
//! - `Q`/`Esc` - Quit

#![allow(deprecated)] // Suppress cocoa crate deprecation warnings
#![allow(
    clippy::too_many_lines,
    clippy::useless_transmute,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::option_if_let_else
)]

mod capture;
mod font;
mod input;
mod overlay;
#[cfg(feature = "macos_15_0")]
mod recording;
mod renderer;
mod screenshot;
mod ui;
mod vertex;
mod waveform;

use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use raw_window_handle::HasWindowHandle;
use screencapturekit::content_sharing_picker::SCPickedSource;
use screencapturekit::metal::{autoreleasepool, setup_metal_view};
use screencapturekit::prelude::*;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use capture::CaptureState;
use font::BitmapFont;
use input::{
    format_picked_source, open_picker, open_picker_for_stream, start_capture, stop_capture,
    PickerResult,
};
use overlay::{default_stream_config, ConfigMenu, OverlayState};
#[cfg(feature = "macos_15_0")]
use recording::{RecordingConfig, RecordingState};
use renderer::{
    create_pipeline, CaptureTextures, MTLLoadAction, MTLPixelFormat, MTLPrimitiveType,
    MTLStoreAction, MetalDevice, MetalLayer, MetalRenderPassDescriptor,
    MetalRenderPipelineDescriptor, SHADER_SOURCE,
};
use screenshot::take_screenshot;
use vertex::{Uniforms, Vertex, VertexBufferBuilder};

/// Result of keyboard input handling
enum KeyAction {
    None,
    Quit,
}

/// Consolidated application state
struct AppState {
    stream: Option<SCStream>,
    current_filter: Option<SCContentFilter>,
    capture_size: (u32, u32),
    picked_source: SCPickedSource,
    stream_config: SCStreamConfiguration,
    mic_device_idx: Option<usize>,
    overlay: OverlayState,
    capture_state: Arc<CaptureState>,
    capturing: Arc<AtomicBool>,
    pending_picker: Arc<Mutex<PickerResult>>,
    #[cfg(feature = "macos_15_0")]
    recording_state: RecordingState,
    #[cfg(feature = "macos_15_0")]
    recording_config: RecordingConfig,
    recording: Arc<AtomicBool>,
}

impl AppState {
    fn new() -> Self {
        let capture_state = Arc::new(CaptureState::new());
        let capturing = Arc::new(AtomicBool::new(false));

        #[cfg(feature = "macos_15_0")]
        let recording_state = RecordingState::new();
        #[cfg(feature = "macos_15_0")]
        let recording = recording_state.recording_flag();
        #[cfg(not(feature = "macos_15_0"))]
        let recording = Arc::new(AtomicBool::new(false));

        Self {
            stream: None,
            current_filter: None,
            capture_size: (1920, 1080),
            picked_source: SCPickedSource::Unknown,
            stream_config: default_stream_config(),
            mic_device_idx: None,
            overlay: OverlayState::new(),
            capture_state,
            capturing,
            pending_picker: Arc::new(Mutex::new(None)),
            #[cfg(feature = "macos_15_0")]
            recording_state,
            #[cfg(feature = "macos_15_0")]
            recording_config: RecordingConfig::new(),
            recording,
        }
    }

    /// Check for pending picker results and handle auto-start
    fn process_pending_picker(&mut self) {
        if let Ok(mut pending) = self.pending_picker.try_lock() {
            if let Some((filter, width, height, source)) = pending.take() {
                println!(
                    "✅ Content selected: {}x{} - {}",
                    width,
                    height,
                    format_picked_source(&source)
                );
                self.capture_size = (width, height);
                self.picked_source = source;

                if self.capturing.load(Ordering::Relaxed) {
                    // Update filter live if already capturing
                    if let Some(ref s) = self.stream {
                        match s.update_content_filter(&filter) {
                            Ok(()) => println!("✅ Source updated live"),
                            Err(e) => eprintln!("❌ Failed to update source: {e:?}"),
                        }
                    }
                    self.current_filter = Some(filter);
                } else {
                    // Auto-start capture after picking
                    self.current_filter = Some(filter);
                    start_capture(
                        &mut self.stream,
                        self.current_filter.as_ref(),
                        self.capture_size,
                        &self.stream_config,
                        &self.capture_state,
                        &self.capturing,
                        false,
                    );
                }
                self.overlay.switch_to_full_menu();
            }
        }
    }

    /// Apply current configuration to running stream
    fn apply_config_to_stream(&self) {
        if self.capturing.load(Ordering::Relaxed) {
            if let Some(ref s) = self.stream {
                let mut new_config = self.stream_config.clone();
                new_config.set_width(self.capture_size.0);
                new_config.set_height(self.capture_size.1);
                if let Err(e) = s.update_configuration(&new_config) {
                    eprintln!("❌ Config update failed: {e:?}");
                }
            }
        }
    }

    /// Toggle capture on/off
    fn toggle_capture(&mut self) {
        if self.capturing.load(Ordering::Relaxed) {
            stop_capture(&mut self.stream, &self.capturing);
        } else {
            start_capture(
                &mut self.stream,
                self.current_filter.as_ref(),
                self.capture_size,
                &self.stream_config,
                &self.capture_state,
                &self.capturing,
                false,
            );
        }
    }

    /// Open content picker
    fn open_picker(&self) {
        if let Some(ref s) = self.stream {
            open_picker_for_stream(&self.pending_picker, s);
        } else {
            open_picker(&self.pending_picker);
        }
    }

    /// Take a screenshot
    fn take_screenshot(&self) {
        if let Some(ref filter) = self.current_filter {
            take_screenshot(filter, self.capture_size, &self.stream_config);
        } else {
            println!("⚠️  Select a source first");
        }
    }

    /// Toggle recording (macOS 15.0+)
    #[allow(clippy::unused_self, clippy::needless_pass_by_ref_mut)]
    fn toggle_recording(&mut self) {
        #[cfg(feature = "macos_15_0")]
        {
            if self.recording_state.is_active() {
                if let Some(ref s) = self.stream {
                    self.recording_state.stop(s);
                }
            } else if self.stream.is_some() {
                if let Some(ref s) = self.stream {
                    if let Err(e) = self.recording_state.start(s, &self.recording_config) {
                        eprintln!("❌ {e}");
                    }
                }
            } else {
                println!("⚠️  Start capture first");
            }
        }
        #[cfg(not(feature = "macos_15_0"))]
        {
            println!("⚠️  Recording requires macOS 15.0+");
        }
    }

    /// Handle menu item selection
    fn handle_menu_selection(&mut self) -> KeyAction {
        let selected_item = self.overlay.menu_items()[self.overlay.menu_selection];
        match selected_item {
            "Pick Source" | "Change Source" => self.open_picker(),
            "Capture" => self.toggle_capture(),
            "Screenshot" => self.take_screenshot(),
            "Record" => self.toggle_recording(),
            "Config" => {
                self.overlay.show_config = true;
                self.overlay.show_help = false;
            }
            "Rec Config" => {
                #[cfg(feature = "macos_15_0")]
                {
                    self.overlay.show_recording_config = true;
                    self.overlay.show_help = false;
                }
            }
            "Quit" => return KeyAction::Quit,
            _ => {}
        }
        KeyAction::None
    }

    /// Handle keyboard input in main menu
    fn handle_menu_key(&mut self, keycode: KeyCode) -> KeyAction {
        let menu_items = self.overlay.menu_items();
        match keycode {
            KeyCode::ArrowUp if self.overlay.menu_selection > 0 => {
                self.overlay.menu_selection -= 1;
                println!(
                    "⬆️  Menu selection: {} ({})",
                    self.overlay.menu_selection, menu_items[self.overlay.menu_selection]
                );
            }
            KeyCode::ArrowDown => {
                let max = self.overlay.menu_count().saturating_sub(1);
                if self.overlay.menu_selection < max {
                    self.overlay.menu_selection += 1;
                    println!(
                        "⬇️  Menu selection: {} ({})",
                        self.overlay.menu_selection, menu_items[self.overlay.menu_selection]
                    );
                }
            }
            KeyCode::Enter | KeyCode::Space => {
                return self.handle_menu_selection();
            }
            KeyCode::Escape | KeyCode::KeyH => {
                self.overlay.show_help = false;
            }
            KeyCode::KeyQ => return KeyAction::Quit,
            _ => {}
        }
        KeyAction::None
    }

    /// Handle keyboard input in config menu
    fn handle_config_key(&mut self, keycode: KeyCode) -> KeyAction {
        match keycode {
            KeyCode::ArrowUp if self.overlay.config_selection > 0 => {
                self.overlay.config_selection -= 1;
            }
            KeyCode::ArrowDown => {
                let max = ConfigMenu::option_count().saturating_sub(1);
                if self.overlay.config_selection < max {
                    self.overlay.config_selection += 1;
                }
            }
            KeyCode::ArrowLeft | KeyCode::ArrowRight => {
                ConfigMenu::toggle_or_adjust(
                    &mut self.stream_config,
                    &mut self.mic_device_idx,
                    self.overlay.config_selection,
                    keycode == KeyCode::ArrowRight,
                );
                self.apply_config_to_stream();
            }
            KeyCode::Enter | KeyCode::Space => {
                ConfigMenu::toggle_or_adjust(
                    &mut self.stream_config,
                    &mut self.mic_device_idx,
                    self.overlay.config_selection,
                    true,
                );
                self.apply_config_to_stream();
            }
            KeyCode::Escape | KeyCode::Backspace => {
                self.overlay.show_config = false;
                self.overlay.show_help = true;
            }
            KeyCode::KeyQ => return KeyAction::Quit,
            _ => {}
        }
        KeyAction::None
    }

    /// Handle keyboard input in recording config menu (macOS 15.0+)
    #[cfg(feature = "macos_15_0")]
    fn handle_recording_config_key(&mut self, keycode: KeyCode) -> KeyAction {
        use crate::recording::RecordingConfigMenu;

        match keycode {
            KeyCode::ArrowUp if self.overlay.recording_config_selection > 0 => {
                self.overlay.recording_config_selection -= 1;
            }
            KeyCode::ArrowDown => {
                let max = RecordingConfigMenu::option_count().saturating_sub(1);
                if self.overlay.recording_config_selection < max {
                    self.overlay.recording_config_selection += 1;
                }
            }
            KeyCode::ArrowLeft | KeyCode::ArrowRight => {
                RecordingConfigMenu::toggle_or_adjust(
                    &mut self.recording_config,
                    self.overlay.recording_config_selection,
                    keycode == KeyCode::ArrowRight,
                );
            }
            KeyCode::Enter | KeyCode::Space => {
                RecordingConfigMenu::toggle_or_adjust(
                    &mut self.recording_config,
                    self.overlay.recording_config_selection,
                    true,
                );
            }
            KeyCode::Escape | KeyCode::Backspace => {
                self.overlay.show_recording_config = false;
                self.overlay.show_help = true;
            }
            KeyCode::KeyQ => return KeyAction::Quit,
            _ => {}
        }
        KeyAction::None
    }

    /// Handle direct keyboard shortcuts (no menu shown)
    fn handle_direct_key(&mut self, keycode: KeyCode) -> KeyAction {
        match keycode {
            KeyCode::Space => self.toggle_capture(),
            KeyCode::KeyP => self.open_picker(),
            KeyCode::KeyW => self.overlay.show_waveform = !self.overlay.show_waveform,
            KeyCode::KeyH => self.overlay.show_help = true,
            KeyCode::KeyC => self.overlay.show_config = true,
            KeyCode::KeyM => {
                let new_val = !self.stream_config.captures_microphone();
                self.stream_config.set_captures_microphone(new_val);
                println!("🎤 Microphone: {}", if new_val { "On" } else { "Off" });
                self.apply_config_to_stream();
            }
            KeyCode::KeyS => self.take_screenshot(),
            KeyCode::KeyR => {
                #[cfg(feature = "macos_15_0")]
                {
                    if self.recording_state.is_active() {
                        if let Some(ref s) = self.stream {
                            self.recording_state.stop(s);
                        }
                    } else if self.current_filter.is_some() && self.stream.is_some() {
                        if let Some(ref s) = self.stream {
                            match self.recording_state.start(s, &self.recording_config) {
                                Ok(path) => println!("🔴 Recording to: {path}"),
                                Err(e) => eprintln!("❌ {e}"),
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
            KeyCode::Escape | KeyCode::KeyQ => return KeyAction::Quit,
            _ => {}
        }
        KeyAction::None
    }

    /// Main keyboard input handler - routes to appropriate sub-handler
    fn handle_key(&mut self, keycode: KeyCode) -> KeyAction {
        #[cfg(feature = "macos_15_0")]
        let show_any_config = self.overlay.show_config || self.overlay.show_recording_config;
        #[cfg(not(feature = "macos_15_0"))]
        let show_any_config = self.overlay.show_config;

        if self.overlay.show_help && !show_any_config {
            self.handle_menu_key(keycode)
        } else if self.overlay.show_config {
            self.handle_config_key(keycode)
        } else {
            #[cfg(feature = "macos_15_0")]
            if self.overlay.show_recording_config {
                return self.handle_recording_config_key(keycode);
            }

            if !self.overlay.show_help && !show_any_config {
                self.handle_direct_key(keycode)
            } else {
                KeyAction::None
            }
        }
    }
}

/// Application wrapper for winit 0.30 `ApplicationHandler`
#[allow(clippy::struct_field_names)]
struct App {
    window: Option<Window>,
    device: MetalDevice,
    layer: MetalLayer,
    #[allow(dead_code)]
    library: renderer::MetalLibrary,
    overlay_pipeline: renderer::MetalRenderPipelineState,
    fullscreen_pipeline: renderer::MetalRenderPipelineState,
    ycbcr_pipeline: renderer::MetalRenderPipelineState,
    command_queue: renderer::MetalCommandQueue,
    app_state: AppState,
    font: BitmapFont,
    vertex_builder: VertexBufferBuilder,
    time: f32,
}

impl App {
    fn new() -> Self {
        // Initialize Metal
        let device = MetalDevice::system_default().expect("No Metal device found");
        println!("🖥️  Metal device: {}", device.name());

        let layer = MetalLayer::new();
        layer.set_device(&device);
        layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        layer.set_presents_with_transaction(false);

        // Compile shaders
        println!("🔧 Compiling shaders...");
        let library = device
            .create_library_with_source(SHADER_SOURCE)
            .expect("Failed to compile shaders");
        println!("✅ Shaders compiled");

        // Create pipelines
        let overlay_pipeline =
            create_pipeline(&device, &library, "vertex_colored", "fragment_colored")
                .expect("Failed to create overlay pipeline");
        let fullscreen_pipeline = create_textured_pipeline(&device, &library, "fragment_textured");
        let ycbcr_pipeline = create_textured_pipeline(&device, &library, "fragment_ycbcr");
        let command_queue = device
            .create_command_queue()
            .expect("Failed to create command queue");

        Self {
            window: None,
            device,
            layer,
            library,
            overlay_pipeline,
            fullscreen_pipeline,
            ycbcr_pipeline,
            command_queue,
            app_state: AppState::new(),
            font: BitmapFont::new(),
            vertex_builder: VertexBufferBuilder::new(),
            time: 0.0,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_attrs = Window::default_attributes()
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .with_title("ScreenCaptureKit Metal Overlay");
        let window = event_loop.create_window(window_attrs).unwrap();

        // Attach layer to window
        unsafe {
            use raw_window_handle::RawWindowHandle;
            match window.window_handle().unwrap().as_raw() {
                RawWindowHandle::AppKit(handle) => {
                    setup_metal_view(handle.ns_view.as_ptr().cast(), &self.layer);
                }
                _ => panic!("Unsupported window handle"),
            }
        }

        let draw_size = window.inner_size();
        self.layer
            .set_drawable_size(f64::from(draw_size.width), f64::from(draw_size.height));

        println!("🎮 Press ENTER to pick a source");
        self.window = Some(window);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        autoreleasepool(|| {
            self.app_state.process_pending_picker();

            match event {
                WindowEvent::CloseRequested => event_loop.exit(),

                WindowEvent::Resized(size) => {
                    self.layer
                        .set_drawable_size(f64::from(size.width), f64::from(size.height));
                }

                WindowEvent::KeyboardInput {
                    event:
                        winit::event::KeyEvent {
                            physical_key: PhysicalKey::Code(keycode),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    if matches!(self.app_state.handle_key(keycode), KeyAction::Quit) {
                        event_loop.exit();
                    }
                }

                WindowEvent::RedrawRequested => {
                    self.time += 0.016;
                    let Some(window) = &self.window else { return };
                    let size = window.inner_size();
                    let (width, height) = (size.width as f32, size.height as f32);

                    // Get capture textures
                    let capture_textures = get_capture_textures(&self.app_state, &self.device);

                    // Build UI vertices
                    build_ui_vertices(
                        &mut self.vertex_builder,
                        &self.font,
                        &self.app_state,
                        capture_textures.as_ref(),
                        width,
                        height,
                    );

                    // Create GPU buffers
                    let vertex_buffer = self.vertex_builder.build(&self.device);
                    vertex_buffer.did_modify_range(
                        0..(self.vertex_builder.vertex_count() * size_of::<Vertex>()),
                    );

                    let uniforms = match &capture_textures {
                        Some(tex) => Uniforms::from_captured_textures(width, height, tex)
                            .with_time(self.time),
                        None => Uniforms::new(
                            width,
                            height,
                            self.app_state.capture_size.0 as f32,
                            self.app_state.capture_size.1 as f32,
                        )
                        .with_time(self.time),
                    };
                    let uniforms_buffer = self
                        .device
                        .create_buffer_with_data(&uniforms)
                        .expect("Failed to create uniforms buffer");

                    // Render
                    render_frame(
                        &self.layer,
                        &self.command_queue,
                        capture_textures.as_ref(),
                        &uniforms_buffer,
                        &vertex_buffer,
                        self.vertex_builder.vertex_count(),
                        &self.fullscreen_pipeline,
                        &self.ycbcr_pipeline,
                        &self.overlay_pipeline,
                    );
                }
                _ => {}
            }
        });
    }
}

fn main() {
    println!("🎮 Metal Overlay Renderer");
    println!("========================\n");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}

/// Create a textured pipeline with the specified fragment function
fn create_textured_pipeline(
    device: &MetalDevice,
    library: &renderer::MetalLibrary,
    fragment_fn: &str,
) -> renderer::MetalRenderPipelineState {
    let vert = library
        .get_function("vertex_fullscreen")
        .expect("vertex_fullscreen not found");
    let frag = library
        .get_function(fragment_fn)
        .unwrap_or_else(|| panic!("{fragment_fn} not found"));
    let desc = MetalRenderPipelineDescriptor::new();
    desc.set_vertex_function(&vert);
    desc.set_fragment_function(&frag);
    desc.set_color_attachment_pixel_format(0, MTLPixelFormat::BGRA8Unorm);
    device
        .create_render_pipeline_state(&desc)
        .expect("Failed to create pipeline")
}

/// Get capture textures from the current capture state
fn get_capture_textures(app: &AppState, device: &MetalDevice) -> Option<CaptureTextures> {
    if app.capturing.load(Ordering::Relaxed) {
        app.capture_state
            .latest_surface
            .try_lock()
            .ok()
            .and_then(|guard| guard.as_ref().and_then(|s| s.create_metal_textures(device)))
    } else {
        None
    }
}

/// Build all UI vertices for the current frame
fn build_ui_vertices(
    vertex_builder: &mut VertexBufferBuilder,
    font: &BitmapFont,
    app: &AppState,
    capture_textures: Option<&CaptureTextures>,
    width: f32,
    height: f32,
) {
    vertex_builder.clear();

    // Status bar
    vertex_builder.rect(0.0, 0.0, width, 32.0, [0.1, 0.1, 0.12, 0.9]);
    let status = build_status_text(app, capture_textures);
    vertex_builder.text(font, &status, 8.0, 8.0, 2.0, [0.2, 1.0, 0.3, 1.0]);

    // Waveform display
    if app.overlay.show_waveform && app.capturing.load(Ordering::Relaxed) {
        build_waveform_ui(vertex_builder, font, app, width);
    }

    // Menu overlays
    if app.overlay.show_help {
        let source_str = format_picked_source(&app.picked_source);
        let format_str = app.capture_state.format_info().unwrap_or_default();
        vertex_builder.help_overlay(
            font,
            width,
            height,
            app.capturing.load(Ordering::Relaxed),
            app.recording.load(Ordering::Relaxed),
            &source_str,
            &format_str,
            app.overlay.menu_selection,
            app.overlay.menu_items(),
        );
    }

    if app.overlay.show_config {
        let source_str = format_picked_source(&app.picked_source);
        vertex_builder.config_menu(
            font,
            width,
            height,
            &app.stream_config,
            app.mic_device_idx,
            app.overlay.config_selection,
            app.capturing.load(Ordering::Relaxed),
            &source_str,
        );
    }

    #[cfg(feature = "macos_15_0")]
    if app.overlay.show_recording_config {
        vertex_builder.recording_config_menu(
            font,
            width,
            height,
            &app.recording_config,
            app.overlay.recording_config_selection,
        );
    }
}

/// Build status bar text
fn build_status_text(app: &AppState, capture_textures: Option<&CaptureTextures>) -> String {
    let fps = app.capture_state.frame_count.load(Ordering::Relaxed);
    let (audio_samples, audio_peak) = app.capture_state.audio_stats();
    let (mic_samples, mic_peak) = app.capture_state.mic_stats();

    if let Some(tex) = capture_textures {
        let format_str = if tex.is_ycbcr() { "YCbCr" } else { "BGRA" };
        format!(
            "LIVE {}x{} {} F:{} A:{}k P:{:.2} M:{}k P:{:.2}",
            tex.width,
            tex.height,
            format_str,
            fps,
            audio_samples / 1000,
            audio_peak,
            mic_samples / 1000,
            mic_peak
        )
    } else if app.capturing.load(Ordering::Relaxed) {
        format!("Starting... {fps}")
    } else {
        "H=Menu".to_string()
    }
}

/// Build waveform visualization UI
fn build_waveform_ui(
    vertex_builder: &mut VertexBufferBuilder,
    font: &BitmapFont,
    app: &AppState,
    width: f32,
) {
    // Layout constants
    const WAVE_HEIGHT: f32 = 40.0;
    const WAVE_SPACING: f32 = 4.0;
    const BAR_Y: f32 = 36.0;
    const METER_WIDTH: f32 = 24.0;
    const PADDING: f32 = 8.0;
    const LABEL_WIDTH: f32 = 24.0;

    // Colors
    const AUDIO_COLOR: [f32; 4] = [0.0, 0.9, 0.8, 0.9];
    const AUDIO_LABEL: [f32; 4] = [0.0, 0.9, 0.8, 0.7];
    const MIC_COLOR: [f32; 4] = [1.0, 0.3, 0.7, 0.9];
    const MIC_LABEL: [f32; 4] = [1.0, 0.3, 0.7, 0.7];
    const BG_COLOR: [f32; 4] = [0.08, 0.08, 0.1, 0.9];

    let total_wave_h = WAVE_HEIGHT.mul_add(2.0, WAVE_SPACING);
    let meters_space = METER_WIDTH.mul_add(2.0, PADDING * 3.0);
    let wave_w = width - meters_space - PADDING - LABEL_WIDTH;
    let wave_x = PADDING + LABEL_WIDTH;
    let audio_wave_y = BAR_Y + 4.0;
    let mic_wave_y = audio_wave_y + WAVE_HEIGHT + WAVE_SPACING;
    let meters_x = width - meters_space + PADDING;

    // Background
    vertex_builder.rect(0.0, BAR_Y, width, total_wave_h + 12.0, BG_COLOR);

    // System audio waveform
    vertex_builder.text(
        font,
        "SYS",
        PADDING,
        audio_wave_y + WAVE_HEIGHT / 2.0 - 4.0,
        1.0,
        AUDIO_LABEL,
    );
    let audio_samples = app.capture_state.audio_display_samples(512);
    vertex_builder.waveform(
        &audio_samples,
        wave_x,
        audio_wave_y,
        wave_w,
        WAVE_HEIGHT,
        AUDIO_COLOR,
    );

    // Microphone waveform
    vertex_builder.text(
        font,
        "MIC",
        PADDING,
        mic_wave_y + WAVE_HEIGHT / 2.0 - 4.0,
        1.0,
        MIC_LABEL,
    );
    let mic_samples = app.capture_state.mic_display_samples(512);
    vertex_builder.waveform(
        &mic_samples,
        wave_x,
        mic_wave_y,
        wave_w,
        WAVE_HEIGHT,
        MIC_COLOR,
    );

    // VU meters
    let audio_level = app.capture_state.audio_rms(2048);
    vertex_builder.vu_meter_vertical(
        audio_level,
        meters_x,
        audio_wave_y,
        METER_WIDTH,
        total_wave_h,
        "S",
        font,
    );

    let mic_level = app.capture_state.mic_rms(2048);
    vertex_builder.vu_meter_vertical(
        mic_level,
        meters_x + METER_WIDTH + PADDING,
        audio_wave_y,
        METER_WIDTH,
        total_wave_h,
        "M",
        font,
    );
}

/// Render the frame to the Metal layer
#[allow(clippy::too_many_arguments)]
fn render_frame(
    layer: &MetalLayer,
    command_queue: &renderer::MetalCommandQueue,
    capture_textures: Option<&CaptureTextures>,
    uniforms_buffer: &renderer::MetalBuffer,
    vertex_buffer: &renderer::MetalBuffer,
    vertex_count: usize,
    fullscreen_pipeline: &renderer::MetalRenderPipelineState,
    ycbcr_pipeline: &renderer::MetalRenderPipelineState,
    overlay_pipeline: &renderer::MetalRenderPipelineState,
) {
    let Some(drawable) = layer.next_drawable() else {
        return;
    };

    let render_pass = MetalRenderPassDescriptor::new();
    let drawable_texture = drawable.texture();
    render_pass.set_color_attachment_texture(0, &drawable_texture);
    render_pass.set_color_attachment_load_action(0, MTLLoadAction::Clear);
    render_pass.set_color_attachment_clear_color(0, 0.08, 0.08, 0.1, 1.0);
    render_pass.set_color_attachment_store_action(0, MTLStoreAction::Store);

    let Some(cmd_buffer) = command_queue.command_buffer() else {
        return;
    };
    let Some(encoder) = cmd_buffer.render_command_encoder(&render_pass) else {
        return;
    };

    // Draw captured frame as background
    if let Some(textures) = capture_textures {
        if textures.is_ycbcr() {
            if let Some(ref plane1) = textures.plane1 {
                encoder.set_render_pipeline_state(ycbcr_pipeline);
                encoder.set_vertex_buffer(uniforms_buffer, 0, 0);
                encoder.set_fragment_texture(&textures.plane0, 0);
                encoder.set_fragment_texture(plane1, 1);
                encoder.set_fragment_buffer(uniforms_buffer, 0, 0);
            }
        } else {
            encoder.set_render_pipeline_state(fullscreen_pipeline);
            encoder.set_vertex_buffer(uniforms_buffer, 0, 0);
            encoder.set_fragment_texture(&textures.plane0, 0);
        }
        encoder.draw_primitives(MTLPrimitiveType::TriangleStrip, 0, 4);
    }

    // Draw overlay UI
    encoder.set_render_pipeline_state(overlay_pipeline);
    encoder.set_vertex_buffer(vertex_buffer, 0, 0);
    encoder.set_vertex_buffer(uniforms_buffer, 0, 1);
    encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, vertex_count);
    encoder.end_encoding();

    cmd_buffer.present_drawable(&drawable);
    cmd_buffer.commit();
}
