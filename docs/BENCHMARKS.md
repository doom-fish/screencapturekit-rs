# Performance Benchmarks

This document describes the performance benchmarks available in `screencapturekit-rs` and how to interpret the results.

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench -- api/
cargo bench -- throughput/
cargo bench -- latency/

# Run with macOS 14.0+ screenshot benchmarks
cargo bench --features macos_14_0
```

> **Note:** Benchmarks require screen recording permission and a display to capture.

## Benchmark Categories

### 1. API Overhead (`api/`)

Measures the cost of creating core objects. These are typically one-time costs at initialization.

| Benchmark | Description | Typical Range |
|-----------|-------------|---------------|
| `SCShareableContent::get` | Query available displays/windows | 5-15ms |
| `SCShareableContent::create` | Query with filter options | 5-15ms |
| `SCContentFilter::create` | Create content filter | 10-50µs |
| `SCStreamConfiguration::new` | Create stream config | 1-5µs |
| `SCStream::new` | Create stream instance | 50-200µs |

### 2. Frame Throughput (`throughput/`)

Measures frame capture performance at various resolutions. Reports pixels processed per second.

| Resolution | Expected FPS | Notes |
|------------|--------------|-------|
| 480p (640×480) | 60-120 FPS | Minimal GPU load |
| 720p (1280×720) | 60-90 FPS | Good for preview |
| 1080p (1920×1080) | 30-60 FPS | Standard capture |
| 4K (3840×2160) | 15-30 FPS | High GPU load |

Throughput depends on:
- GPU capabilities (Apple Silicon vs Intel)
- Display refresh rate
- Pixel format (BGRA vs YCbCr)
- Other GPU workloads

### 3. Frame Latency (`latency/`)

Measures time from `start_capture()` to first frame callback.

| Benchmark | Description | Typical Range |
|-----------|-------------|---------------|
| `first_frame` | Time to first frame | 30-100ms |

First-frame latency includes:
- Stream initialization
- GPU pipeline setup
- First vsync wait

### 4. Data Access (`data_access/`)

Measures pixel buffer and IOSurface access patterns.

| Benchmark | Description | Typical Range |
|-----------|-------------|---------------|
| `pixel_buffer/lock_unlock` | Lock/unlock cycle | 1-10µs |
| `pixel_buffer/read_first_pixel` | Read single pixel | 2-15µs |
| `pixel_buffer/read_all_pixels` | Process full frame | 1-5ms |
| `iosurface/lock_unlock` | IOSurface lock cycle | 1-5µs |
| `iosurface/get_properties` | Read surface metadata | <1µs |

### 5. Screenshot (`screenshot/`) - macOS 14.0+

Measures single-frame capture performance.

| Benchmark | Description | Typical Range |
|-----------|-------------|---------------|
| `capture_image/1080p` | CGImage capture | 20-50ms |
| `capture_sample_buffer/1080p` | CMSampleBuffer capture | 15-40ms |

Screenshot capture is slower than streaming but simpler for single frames.

### 6. Stream Lifecycle (`lifecycle/`)

Measures stream start/stop overhead.

| Benchmark | Description | Typical Range |
|-----------|-------------|---------------|
| `start_stop_cycle` | Full start→stop cycle | 50-150ms |
| `start_capture_only` | Just start_capture() | 20-80ms |

### 7. Configuration Updates (`updates/`)

Measures runtime configuration changes.

| Benchmark | Description | Typical Range |
|-----------|-------------|---------------|
| `update_configuration` | Change resolution mid-stream | 10-30ms |

## Optimization Tips

### For Lowest Latency

```rust
let config = SCStreamConfiguration::new()
    .with_minimum_frame_interval(&CMTime::new(1, 120))  // 120 FPS cap
    .with_queue_depth(3);  // Smaller buffer
```

### For Highest Throughput

```rust
let config = SCStreamConfiguration::new()
    .with_pixel_format(PixelFormat::YCbCr420v)  // Hardware-native format
    .with_queue_depth(8);  // Larger buffer
```

### For Lowest Memory

```rust
let config = SCStreamConfiguration::new()
    .with_width(1280)
    .with_height(720)  // Lower resolution
    .with_queue_depth(3);
```

### Zero-Copy GPU Access

Use IOSurface for Metal/GPU pipelines to avoid CPU copies:

```rust
if let Some(surface) = pixel_buffer.io_surface() {
    // Create Metal texture directly from IOSurface
    let texture = device.create_texture_from_iosurface(&surface);
}
```

## Hardware Considerations

### Apple Silicon (M1/M2/M3)

- Unified memory eliminates GPU↔CPU copies
- Hardware video encoder for recording
- Excellent 4K performance

### Intel Macs

- Discrete GPU may have memory transfer overhead
- Lower 4K throughput
- Consider 1080p for real-time processing

## Comparing Results

When comparing benchmark results:

1. **Same hardware** - GPU, RAM, and display resolution affect results
2. **Same macOS version** - Performance varies between releases
3. **Same workload** - Close other GPU-intensive apps
4. **Multiple runs** - Use `cargo bench` statistical analysis

## Profiling

For deeper analysis, use Instruments:

```bash
# CPU profiling
xcrun xctrace record --template 'Time Profiler' --launch -- cargo bench

# GPU profiling  
xcrun xctrace record --template 'Metal System Trace' --launch -- cargo bench
```
