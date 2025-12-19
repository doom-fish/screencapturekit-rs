//! wgpu Integration (Zero-Copy)
//!
//! Demonstrates real-time screen capture display using wgpu.
//! This example shows:
//! - Creating a wgpu rendering pipeline
//! - Uploading captured frames to GPU textures
//! - Rendering fullscreen quad with captured content
//!
//! Run with: `cargo run --example 18_wgpu_integration`

use screencapturekit::cv::CVPixelBufferLockFlags;
use screencapturekit::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// Shared frame data between capture and render
struct SharedFrame {
    data: Vec<u8>,
    width: u32,
    height: u32,
    bytes_per_row: u32,
    dirty: AtomicBool,
    frame_count: AtomicUsize,
}

impl SharedFrame {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            width: 0,
            height: 0,
            bytes_per_row: 0,
            dirty: AtomicBool::new(false),
            frame_count: AtomicUsize::new(0),
        }
    }
}

struct FrameHandler {
    frame: Arc<Mutex<SharedFrame>>,
}

impl SCStreamOutputTrait for FrameHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, output_type: SCStreamOutputType) {
        if !matches!(output_type, SCStreamOutputType::Screen) {
            return;
        }

        let Some(pixel_buffer) = sample.image_buffer() else {
            return;
        };

        let Ok(guard) = pixel_buffer.lock(CVPixelBufferLockFlags::READ_ONLY) else {
            return;
        };

        let width = pixel_buffer.width() as u32;
        let height = pixel_buffer.height() as u32;
        let bytes_per_row = pixel_buffer.bytes_per_row() as u32;
        let src_data = guard.as_slice();

        // Remove row padding if present - wgpu expects tightly packed rows
        let expected_bytes_per_row = width * 4;
        let data = if bytes_per_row == expected_bytes_per_row {
            src_data.to_vec()
        } else {
            // Strip padding from each row
            let mut packed = Vec::with_capacity((width * height * 4) as usize);
            for row in 0..height {
                let start = (row * bytes_per_row) as usize;
                let end = start + (width * 4) as usize;
                if end <= src_data.len() {
                    packed.extend_from_slice(&src_data[start..end]);
                }
            }
            packed
        };

        if let Ok(mut frame) = self.frame.lock() {
            frame.data = data;
            frame.width = width;
            frame.height = height;
            frame.bytes_per_row = width * 4; // Now always tightly packed
            frame.frame_count.fetch_add(1, Ordering::Relaxed);
            frame.dirty.store(true, Ordering::Release);
        }
    }
}

/// wgpu renderer state
struct Renderer<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    texture: Option<wgpu::Texture>,
    texture_view: Option<wgpu::TextureView>,
    bind_group: Option<wgpu::BindGroup>,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl<'a> Renderer<'a> {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::METAL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("wgpu_shader.wgsl").into()),
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            texture: None,
            texture_view: None,
            bind_group: None,
            bind_group_layout,
            sampler,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn update_texture(&mut self, data: &[u8], width: u32, height: u32) {
        // Recreate texture if size changed
        let needs_new_texture = self
            .texture
            .as_ref()
            .map(|t| t.width() != width || t.height() != height)
            .unwrap_or(true);

        if needs_new_texture {
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Screen Capture Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // Use sRGB format to match screen capture data (already gamma-encoded)
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

            self.texture = Some(texture);
            self.texture_view = Some(view);
            self.bind_group = Some(bind_group);
        }

        // Upload texture data
        if let Some(ref texture) = self.texture {
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            if let Some(ref bind_group) = self.bind_group {
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.draw(0..6, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

struct App<'a> {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer<'a>>,
    shared_frame: Arc<Mutex<SharedFrame>>,
    stream: Option<SCStream>,
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("wgpu Screen Capture")
            .with_inner_size(winit::dpi::LogicalSize::new(1300, 800));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());

        // Create renderer
        let renderer = pollster::block_on(Renderer::new(window));
        self.renderer = Some(renderer);

        // Start capture
        if let Ok(content) = SCShareableContent::get() {
            if let Some(display) = content.displays().into_iter().next() {
                let filter = SCContentFilter::with()
                    .with_display(&display)
                    .with_excluding_windows(&[])
                    .build();

                let config = SCStreamConfiguration::new()
                    .with_width(1280)
                    .with_height(720)
                    .with_pixel_format(PixelFormat::BGRA)
                    .with_minimum_frame_interval(&CMTime::new(1, 30));

                let handler = FrameHandler {
                    frame: self.shared_frame.clone(),
                };

                let mut stream = SCStream::new(&filter, &config);
                stream.add_output_handler(handler, SCStreamOutputType::Screen);
                let _ = stream.start_capture();
                self.stream = Some(stream);

                println!("✅ Capture started");
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                if let Some(ref mut stream) = self.stream {
                    let _ = stream.stop_capture();
                }
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(ref mut renderer) = self.renderer {
                    renderer.resize(physical_size);
                }
            }
            WindowEvent::RedrawRequested => {
                // Check for new frame
                if let Ok(frame) = self.shared_frame.lock() {
                    if frame.dirty.swap(false, Ordering::AcqRel) && !frame.data.is_empty() {
                        if let Some(ref mut renderer) = self.renderer {
                            renderer.update_texture(&frame.data, frame.width, frame.height);
                        }
                    }
                }

                // Render
                if let Some(ref mut renderer) = self.renderer {
                    match renderer.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            renderer.resize(winit::dpi::PhysicalSize {
                                width: renderer.config.width,
                                height: renderer.config.height,
                            })
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("Render error: {:?}", e),
                    }
                }

                // Request next frame
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 wgpu Screen Capture Viewer\n");

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        renderer: None,
        shared_frame: Arc::new(Mutex::new(SharedFrame::new())),
        stream: None,
    };

    event_loop.run_app(&mut app)?;

    Ok(())
}
