//! Recording capture logic (macOS 15.0+)

#[cfg(feature = "macos_15_0")]
use screencapturekit::recording_output::{
    SCRecordingOutput, SCRecordingOutputCodec, SCRecordingOutputConfiguration,
    SCRecordingOutputFileType,
};
#[cfg(feature = "macos_15_0")]
use screencapturekit::stream::sc_stream::SCStream;
#[cfg(feature = "macos_15_0")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "macos_15_0")]
use std::sync::Arc;

/// Recording configuration state
#[cfg(feature = "macos_15_0")]
#[derive(Debug, Clone)]
pub struct RecordingConfig {
    pub codec: SCRecordingOutputCodec,
    pub file_type: SCRecordingOutputFileType,
}

#[cfg(feature = "macos_15_0")]
impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            codec: SCRecordingOutputCodec::H264,
            file_type: SCRecordingOutputFileType::MP4,
        }
    }
}

#[cfg(feature = "macos_15_0")]
impl RecordingConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply this config to a recording output configuration
    pub fn apply_to(
        &self,
        config: SCRecordingOutputConfiguration,
    ) -> SCRecordingOutputConfiguration {
        config
            .with_video_codec(self.codec)
            .with_output_file_type(self.file_type)
    }

    /// Get file extension based on file type
    pub const fn file_extension(&self) -> &'static str {
        match self.file_type {
            SCRecordingOutputFileType::MP4 => "mp4",
            SCRecordingOutputFileType::MOV => "mov",
        }
    }
}

/// Recording state manager
#[cfg(feature = "macos_15_0")]
pub struct RecordingState {
    pub output: Option<SCRecordingOutput>,
    pub path: Option<String>,
    pub is_recording: Arc<AtomicBool>,
}

#[cfg(feature = "macos_15_0")]
impl RecordingState {
    pub fn new() -> Self {
        Self {
            output: None,
            path: None,
            is_recording: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if currently recording
    pub fn is_active(&self) -> bool {
        self.is_recording.load(Ordering::Relaxed)
    }

    /// Start recording to a file
    pub fn start(&mut self, stream: &SCStream, config: &RecordingConfig) -> Result<String, String> {
        if self.is_active() {
            return Err("Already recording".to_string());
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let path = format!("/tmp/recording_{}.{}", timestamp, config.file_extension());

        let rec_config = config.apply_to(
            SCRecordingOutputConfiguration::new().with_output_url(std::path::Path::new(&path)),
        );

        match SCRecordingOutput::new(&rec_config) {
            Some(rec) => match stream.add_recording_output(&rec) {
                Ok(()) => {
                    println!("🔴 Recording to: {path}");
                    self.is_recording.store(true, Ordering::Relaxed);
                    self.output = Some(rec);
                    self.path = Some(path.clone());
                    Ok(path)
                }
                Err(e) => Err(format!("Failed to start recording: {e:?}")),
            },
            None => Err("Failed to create recording output".to_string()),
        }
    }

    /// Stop recording and return the file path
    pub fn stop(&mut self, stream: &SCStream) -> Option<String> {
        if !self.is_active() {
            return None;
        }

        if let Some(ref rec) = self.output {
            println!("⏹️  Stopping recording...");
            let _ = stream.remove_recording_output(rec);
        }

        self.is_recording.store(false, Ordering::Relaxed);
        self.output = None;

        let path = self.path.take();
        if let Some(ref p) = path {
            println!("✅ Recording saved: {p}");
            let _ = std::process::Command::new("open").arg(p).spawn();
        }
        path
    }

    /// Get the recording flag for UI display
    pub fn recording_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.is_recording)
    }
}

#[cfg(feature = "macos_15_0")]
impl Default for RecordingState {
    fn default() -> Self {
        Self::new()
    }
}

/// Recording config menu
#[cfg(feature = "macos_15_0")]
pub struct RecordingConfigMenu;

#[cfg(feature = "macos_15_0")]
impl RecordingConfigMenu {
    pub const OPTIONS: &'static [&'static str] = &["Video Codec", "File Type"];

    pub const fn option_count() -> usize {
        Self::OPTIONS.len()
    }

    pub fn option_name(idx: usize) -> &'static str {
        Self::OPTIONS.get(idx).unwrap_or(&"?")
    }

    pub fn option_value(config: &RecordingConfig, idx: usize) -> String {
        match idx {
            0 => match config.codec {
                SCRecordingOutputCodec::H264 => "H.264".to_string(),
                SCRecordingOutputCodec::HEVC => "HEVC".to_string(),
            },
            1 => match config.file_type {
                SCRecordingOutputFileType::MP4 => "MP4".to_string(),
                SCRecordingOutputFileType::MOV => "MOV".to_string(),
            },
            _ => "?".to_string(),
        }
    }

    pub fn toggle_or_adjust(config: &mut RecordingConfig, idx: usize, _increase: bool) {
        match idx {
            0 => {
                // Toggle codec
                config.codec = match config.codec {
                    SCRecordingOutputCodec::H264 => SCRecordingOutputCodec::HEVC,
                    SCRecordingOutputCodec::HEVC => SCRecordingOutputCodec::H264,
                };
            }
            1 => {
                // Toggle file type
                config.file_type = match config.file_type {
                    SCRecordingOutputFileType::MP4 => SCRecordingOutputFileType::MOV,
                    SCRecordingOutputFileType::MOV => SCRecordingOutputFileType::MP4,
                };
            }
            _ => {}
        }
    }
}
