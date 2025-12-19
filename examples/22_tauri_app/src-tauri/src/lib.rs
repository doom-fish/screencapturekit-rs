//! Tauri commands for screen capture using screencapturekit-rs
//!
//! This example demonstrates integrating screencapturekit-rs with Tauri 2.0
//! to build a cross-platform (macOS) screen capture application.

use base64::{engine::general_purpose::STANDARD, Engine};
use screencapturekit::prelude::*;
use screencapturekit::screenshot_manager::SCScreenshotManager;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

/// Display information returned to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub id: u32,
    pub width: usize,
    pub height: usize,
    pub frame_x: f64,
    pub frame_y: f64,
}

/// Window information returned to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: u32,
    pub title: Option<String>,
    pub app_name: Option<String>,
    pub bundle_id: Option<String>,
    pub is_on_screen: bool,
    pub frame_x: f64,
    pub frame_y: f64,
    pub width: f64,
    pub height: f64,
}

/// Application information returned to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub bundle_id: String,
    pub app_name: String,
    pub pid: i32,
}

/// Screenshot result with base64-encoded RGBA data for WebGL rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotResult {
    pub data: String, // Base64-encoded RGBA pixels
    pub width: usize,
    pub height: usize,
}

/// Shared state for recording
pub struct RecordingState {
    is_recording: Mutex<bool>,
}

impl Default for RecordingState {
    fn default() -> Self {
        Self {
            is_recording: Mutex::new(false),
        }
    }
}

/// List all available displays
#[tauri::command]
fn list_displays() -> Result<Vec<DisplayInfo>, String> {
    let content = SCShareableContent::get().map_err(|e| format!("Failed to get content: {}", e))?;

    let displays: Vec<DisplayInfo> = content
        .displays()
        .iter()
        .map(|d| DisplayInfo {
            id: d.display_id(),
            width: d.width() as usize,
            height: d.height() as usize,
            frame_x: d.frame().x,
            frame_y: d.frame().y,
        })
        .collect();

    Ok(displays)
}

/// List all available windows
#[tauri::command]
fn list_windows() -> Result<Vec<WindowInfo>, String> {
    let content = SCShareableContent::get().map_err(|e| format!("Failed to get content: {}", e))?;

    let windows: Vec<WindowInfo> = content
        .windows()
        .iter()
        .filter(|w| w.is_on_screen()) // Only on-screen windows
        .map(|w| {
            let frame = w.frame();
            WindowInfo {
                id: w.window_id(),
                title: w.title(),
                app_name: w.owning_application().map(|a| a.application_name()),
                bundle_id: w.owning_application().map(|a| a.bundle_identifier()),
                is_on_screen: w.is_on_screen(),
                frame_x: frame.x,
                frame_y: frame.y,
                width: frame.width,
                height: frame.height,
            }
        })
        .collect();

    Ok(windows)
}

/// List all running applications
#[tauri::command]
fn list_applications() -> Result<Vec<AppInfo>, String> {
    let content = SCShareableContent::get().map_err(|e| format!("Failed to get content: {}", e))?;

    let apps: Vec<AppInfo> = content
        .applications()
        .iter()
        .map(|a| AppInfo {
            bundle_id: a.bundle_identifier(),
            app_name: a.application_name(),
            pid: a.process_id(),
        })
        .collect();

    Ok(apps)
}

/// Take a screenshot of the primary display - returns RGBA data for WebGL
#[tauri::command]
fn take_screenshot_display(display_id: Option<u32>) -> Result<ScreenshotResult, String> {
    let content = SCShareableContent::get().map_err(|e| format!("Failed to get content: {}", e))?;

    // Find display by ID or use the first one
    let display = if let Some(id) = display_id {
        content
            .displays()
            .iter()
            .find(|d| d.display_id() == id)
            .cloned()
            .ok_or_else(|| format!("Display {} not found", id))?
    } else {
        content
            .displays()
            .first()
            .cloned()
            .ok_or("No displays available")?
    };

    let filter = SCContentFilter::with()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(display.width() as u32)
        .with_height(display.height() as u32)
        .with_pixel_format(PixelFormat::BGRA);

    let image = SCScreenshotManager::capture_image(&filter, &config)
        .map_err(|e| format!("Screenshot failed: {}", e))?;

    // Get RGBA data for WebGL rendering
    let rgba_data = image
        .rgba_data()
        .map_err(|e| format!("Failed to get pixel data: {}", e))?;

    Ok(ScreenshotResult {
        data: STANDARD.encode(&rgba_data),
        width: image.width(),
        height: image.height(),
    })
}

/// Take a screenshot of a specific window - returns RGBA data for WebGL
#[tauri::command]
fn take_screenshot_window(window_id: u32) -> Result<ScreenshotResult, String> {
    let content = SCShareableContent::get().map_err(|e| format!("Failed to get content: {}", e))?;

    let window = content
        .windows()
        .iter()
        .find(|w| w.window_id() == window_id)
        .cloned()
        .ok_or_else(|| format!("Window {} not found", window_id))?;

    let filter = SCContentFilter::with().with_window(&window).build();

    let frame = window.frame();
    let config = SCStreamConfiguration::new()
        .with_width(frame.width as u32)
        .with_height(frame.height as u32)
        .with_pixel_format(PixelFormat::BGRA);

    let image = SCScreenshotManager::capture_image(&filter, &config)
        .map_err(|e| format!("Screenshot failed: {}", e))?;

    // Get RGBA data for WebGL rendering
    let rgba_data = image
        .rgba_data()
        .map_err(|e| format!("Failed to get pixel data: {}", e))?;

    Ok(ScreenshotResult {
        data: STANDARD.encode(&rgba_data),
        width: image.width(),
        height: image.height(),
    })
}

/// Check if recording is active
#[tauri::command]
fn is_recording(state: State<RecordingState>) -> bool {
    *state.is_recording.lock().unwrap()
}

/// Get recording status
#[tauri::command]
fn get_status() -> Result<String, String> {
    let content = SCShareableContent::get().map_err(|e| format!("Failed to get content: {}", e))?;

    Ok(format!(
        "Ready - {} displays, {} windows, {} apps available",
        content.displays().len(),
        content
            .windows()
            .iter()
            .filter(|w| w.is_on_screen())
            .count(),
        content.applications().len()
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(RecordingState::default())
        .invoke_handler(tauri::generate_handler![
            list_displays,
            list_windows,
            list_applications,
            take_screenshot_display,
            take_screenshot_window,
            is_recording,
            get_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
