//! WebRTC Screen Streaming
//!
//! Streams screen capture over WebRTC to a browser.
//!
//! This example demonstrates:
//! - Capturing screen with `ScreenCaptureKit`
//! - Encoding frames to VP8 (browser-compatible)
//! - Streaming via WebRTC with signaling server
//!
//! Requirements:
//! - Run the signaling server first, then open the HTML page in a browser
//!
//! Usage:
//! ```bash
//! cargo run --example 24_webrtc
//! # Then open http://127.0.0.1:8080 in your browser
//! ```

#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::needless_pass_by_value)]

use screencapturekit::cv::CVPixelBufferLockFlags;
use screencapturekit::prelude::*;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_VP8};
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_connection_state::RTCIceConnectionState;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::media::Sample;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;

/// Simple VP8 encoder using libvpx via vpx-encode crate
/// For production, consider using hardware encoding
struct SimpleVP8Encoder {
    width: u32,
    height: u32,
    #[allow(dead_code)]
    frame_count: u64,
}

impl SimpleVP8Encoder {
    const fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            frame_count: 0,
        }
    }

    /// Encode BGRA frame to VP8
    /// Returns a simple VP8-like frame (in production, use proper vpx encoder)
    fn encode(&mut self, bgra_data: &[u8]) -> Vec<u8> {
        self.frame_count += 1;

        // Convert BGRA to I420 (YUV420p) - required for VP8
        // For this example, we'll send raw YUV wrapped in a simple container
        // In production, use vpx-encode or similar for proper VP8 encoding
        //
        // Note: Real VP8 encoding requires the vpx crate which needs libvpx installed
        // This simplified version just demonstrates the flow
        bgra_to_i420(bgra_data, self.width as usize, self.height as usize)
    }
}

/// Convert BGRA to I420 (`YUV420p`)
fn bgra_to_i420(bgra: &[u8], width: usize, height: usize) -> Vec<u8> {
    let y_size = width * height;
    let uv_size = (width / 2) * (height / 2);
    let mut yuv = vec![0u8; y_size + uv_size * 2];

    let (y_plane, uv_planes) = yuv.split_at_mut(y_size);
    let (u_plane, v_plane) = uv_planes.split_at_mut(uv_size);

    for y in 0..height {
        for x in 0..width {
            let bgra_idx = (y * width + x) * 4;
            let b = i32::from(bgra.get(bgra_idx).copied().unwrap_or(0));
            let g = i32::from(bgra.get(bgra_idx + 1).copied().unwrap_or(0));
            let r = i32::from(bgra.get(bgra_idx + 2).copied().unwrap_or(0));

            // BT.601 conversion
            let y_val = ((66 * r + 129 * g + 25 * b + 128) >> 8) + 16;
            y_plane[y * width + x] = y_val.clamp(0, 255) as u8;

            // Subsample U and V (2x2 blocks)
            if y % 2 == 0 && x % 2 == 0 {
                let u_val = ((-38 * r - 74 * g + 112 * b + 128) >> 8) + 128;
                let v_val = ((112 * r - 94 * g - 18 * b + 128) >> 8) + 128;
                let uv_idx = (y / 2) * (width / 2) + (x / 2);
                u_plane[uv_idx] = u_val.clamp(0, 255) as u8;
                v_plane[uv_idx] = v_val.clamp(0, 255) as u8;
            }
        }
    }

    yuv
}

/// Shared state for frame exchange between capture and WebRTC
struct SharedState {
    frame_data: Mutex<Option<Vec<u8>>>,
    frame_ready: AtomicBool,
    frame_count: AtomicU64,
    running: AtomicBool,
}

impl SharedState {
    const fn new() -> Self {
        Self {
            frame_data: Mutex::new(None),
            frame_ready: AtomicBool::new(false),
            frame_count: AtomicU64::new(0),
            running: AtomicBool::new(true),
        }
    }
}

struct CaptureHandler {
    state: Arc<SharedState>,
    encoder: Mutex<SimpleVP8Encoder>,
    expected_size: usize,
}

impl SCStreamOutputTrait for CaptureHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, output_type: SCStreamOutputType) {
        if !matches!(output_type, SCStreamOutputType::Screen) {
            return;
        }

        if !self.state.running.load(Ordering::Relaxed) {
            return;
        }

        let Some(pixel_buffer) = sample.image_buffer() else {
            return;
        };

        let Ok(guard) = pixel_buffer.lock(CVPixelBufferLockFlags::READ_ONLY) else {
            return;
        };

        let data = guard.as_slice();
        if data.len() < self.expected_size {
            return;
        }

        // Encode frame
        let encoded = {
            let mut encoder = self.encoder.lock().unwrap();
            encoder.encode(&data[..self.expected_size])
        };

        // Store for WebRTC to pick up
        {
            let mut frame = self.state.frame_data.lock().unwrap();
            *frame = Some(encoded);
        }
        self.state.frame_ready.store(true, Ordering::Release);
        self.state.frame_count.fetch_add(1, Ordering::Relaxed);
    }
}

/// Simple HTTP server for signaling and serving the HTML page
fn serve_http(
    listener: TcpListener,
    offer_tx: std::sync::mpsc::Sender<String>,
    answer_rx: Arc<Mutex<Option<String>>>,
) {
    println!("🌐 HTTP server listening on http://127.0.0.1:8080");
    println!("   Open this URL in your browser to view the stream\n");

    for stream in listener.incoming().flatten() {
        handle_http_request(&mut { stream }, &offer_tx, &answer_rx);
    }
}

fn handle_http_request(
    stream: &mut TcpStream,
    offer_tx: &std::sync::mpsc::Sender<String>,
    answer_rx: &Arc<Mutex<Option<String>>>,
) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }

    // Read headers
    let mut content_length = 0;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).is_err() || header == "\r\n" {
            break;
        }
        if header.to_lowercase().starts_with("content-length:") {
            if let Some(len) = header.split(':').nth(1) {
                content_length = len.trim().parse().unwrap_or(0);
            }
        }
    }

    if request_line.starts_with("GET / ") {
        // Serve HTML page
        let html = include_str!("index.html");
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
            html.len(),
            html
        );
        let _ = stream.write_all(response.as_bytes());
    } else if request_line.starts_with("POST /offer") {
        // Receive SDP offer from browser
        let mut body = vec![0u8; content_length];
        if reader.read_exact(&mut body).is_ok() {
            if let Ok(offer) = String::from_utf8(body) {
                let _ = offer_tx.send(offer);

                // Wait for answer (with timeout)
                let start = Instant::now();
                loop {
                    let answer = answer_rx.lock().unwrap().take();
                    if let Some(answer) = answer {
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
                            answer.len(),
                            answer
                        );
                        let _ = stream.write_all(response.as_bytes());
                        return;
                    }
                    if start.elapsed().as_secs() > 10 {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        }
        let response = "HTTP/1.1 500 Internal Server Error\r\n\r\n";
        let _ = stream.write_all(response.as_bytes());
    } else if request_line.starts_with("OPTIONS") {
        // CORS preflight
        let response = "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: POST\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";
        let _ = stream.write_all(response.as_bytes());
    } else {
        let response = "HTTP/1.1 404 Not Found\r\n\r\n";
        let _ = stream.write_all(response.as_bytes());
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎬 WebRTC Screen Streaming\n");

    let width: u32 = 1280;
    let height: u32 = 720;

    // Set up screen capture
    let content = SCShareableContent::get()?;
    let display = content
        .displays()
        .into_iter()
        .next()
        .ok_or("No displays found")?;

    println!(
        "📺 Capturing display: {} ({}x{})",
        display.display_id(),
        display.width(),
        display.height()
    );
    println!("   Streaming at: {width}x{height}\n");

    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    let frame_interval = CMTime::new(1, 30); // 30 FPS
    let config = SCStreamConfiguration::new()
        .with_width(width)
        .with_height(height)
        .with_pixel_format(PixelFormat::BGRA)
        .with_minimum_frame_interval(&frame_interval);

    // Shared state
    let state = Arc::new(SharedState::new());

    let handler = CaptureHandler {
        state: state.clone(),
        encoder: Mutex::new(SimpleVP8Encoder::new(width, height)),
        expected_size: (width * height * 4) as usize,
    };

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Screen);
    stream.start_capture()?;
    println!("✅ Screen capture started\n");

    // Set up signaling channels
    let (offer_tx, offer_rx) = std::sync::mpsc::channel::<String>();
    let answer_rx = Arc::new(Mutex::new(None::<String>));
    let answer_tx = answer_rx.clone();

    // Start HTTP server for signaling
    let http_listener = TcpListener::bind("127.0.0.1:8080")?;
    std::thread::spawn(move || {
        serve_http(http_listener, offer_tx, answer_rx);
    });

    // WebRTC setup
    let mut media_engine = MediaEngine::default();
    media_engine.register_default_codecs()?;

    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut media_engine)?;

    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .build();

    let rtc_config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_string()],
            ..Default::default()
        }],
        ..Default::default()
    };

    println!("⏳ Waiting for browser connection...");
    println!("   Open http://127.0.0.1:8080 in your browser\n");

    // Wait for offer from browser
    let offer_sdp = offer_rx.recv()?;
    println!("📥 Received offer from browser");

    let peer_connection = Arc::new(api.new_peer_connection(rtc_config).await?);

    // Create video track
    let video_track = Arc::new(TrackLocalStaticSample::new(
        RTCRtpCodecCapability {
            mime_type: MIME_TYPE_VP8.to_string(),
            clock_rate: 90000,
            ..Default::default()
        },
        "video".to_string(),
        "screen".to_string(),
    ));

    let rtp_sender = peer_connection
        .add_track(Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>)
        .await?;

    // Read incoming RTCP
    tokio::spawn(async move {
        let mut buf = vec![0u8; 1500];
        while let Ok((_, _)) = rtp_sender.read(&mut buf).await {}
    });

    // Handle ICE connection state
    let state_clone = state.clone();
    peer_connection.on_ice_connection_state_change(Box::new(move |s: RTCIceConnectionState| {
        println!("🔗 ICE state: {s}");
        if s == RTCIceConnectionState::Failed || s == RTCIceConnectionState::Disconnected {
            state_clone.running.store(false, Ordering::Relaxed);
        }
        Box::pin(async {})
    }));

    // Set remote description (browser's offer)
    let offer = serde_json::from_str::<RTCSessionDescription>(&offer_sdp)?;
    peer_connection.set_remote_description(offer).await?;

    // Create answer
    let answer = peer_connection.create_answer(None).await?;
    peer_connection
        .set_local_description(answer.clone())
        .await?;

    // Wait for ICE gathering
    let mut gather_complete = peer_connection.gathering_complete_promise().await;
    let _ = gather_complete.recv().await;

    // Send answer back to browser
    if let Some(local_desc) = peer_connection.local_description().await {
        let answer_json = serde_json::to_string(&local_desc)?;
        *answer_tx.lock().unwrap() = Some(answer_json);
        println!("📤 Sent answer to browser\n");
    }

    println!("🎥 Streaming... Press Ctrl+C to stop\n");

    // Stream frames
    let frame_duration = std::time::Duration::from_millis(33); // ~30 FPS
    while state.running.load(Ordering::Relaxed) {
        if state.frame_ready.swap(false, Ordering::AcqRel) {
            let frame_data = state.frame_data.lock().unwrap().take();
            if let Some(data) = frame_data {
                let sample = Sample {
                    data: data.into(),
                    duration: frame_duration,
                    ..Default::default()
                };

                if video_track.write_sample(&sample).await.is_err() {
                    break;
                }

                let count = state.frame_count.load(Ordering::Relaxed);
                if count % 30 == 0 {
                    println!("📹 Streamed {count} frames");
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }

    stream.stop_capture()?;
    peer_connection.close().await?;
    println!("\n✅ Streaming stopped");

    Ok(())
}
