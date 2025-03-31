use std::io;
use std::io::Write;
use wgpu::{Device, PipelineLayoutDescriptor, TextureFormat};
use winit::{
    self,
    dpi::PhysicalSize,
    event::*,
    event_loop::*,
    keyboard::{Key, NamedKey},
    window::*,
};

struct State<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: &'window Window,

    // SELF
    color: wgpu::Color,

    // Pipeline
    pipeline: wgpu::RenderPipeline,

    // Pipeline
    pipeline2: wgpu::RenderPipeline,

    current_pipeline: u8,
}

fn read_file(name: &str) -> io::Result<String> {
    std::fs::read_to_string(name)
}

fn create_pipeline(name: &str, shader: &str, device: &Device, format: TextureFormat) -> wgpu::RenderPipeline {
    let shader_source = match read_file(shader) {
        Ok(source) => source,
        Err(err) => panic!("Failed to read shader: {}\n{}", shader, err),
    };

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor{
        label: Some(name),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor{
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState{
            module: &shader,
            entry_point: "vs_main", // 1
            buffers: &[], // 2
        },
        fragment: Some(wgpu::FragmentState{
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState{
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })]
        }),
        primitive: wgpu::PrimitiveState{
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: None, // 1
        multisample: wgpu::MultisampleState{
            count: 1, // 2
            mask: !0, // 3
            alpha_to_coverage_enabled: false, // 4
        },
        multiview: None,
    });

    render_pipeline
}

impl<'window> State<'window> {
    async fn new(window: &'window Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window, so this should be safe.
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
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);

        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            // present_mode: surface_capabilities.present_modes[0], // VSync on
            present_mode: wgpu::PresentMode::AutoNoVsync, // VSync off
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 0,
        };

        surface.configure(&device, &config);

        let render_pipeline = create_pipeline("Shader 1", "./src/shader.wgsl", &device, surface_format);
        let render_pipeline_2 = create_pipeline("Shader 2", "./src/shader2.wgsl", &device, surface_format);

        State {
            surface,
            device,
            queue,
            config,
            size,
            window,

            color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },

            pipeline: render_pipeline,
            pipeline2: render_pipeline_2,

            current_pipeline: 1,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if (new_size.width > 0 && new_size.height > 0) && (new_size != self.size) {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseInput {   .. } => true,
            WindowEvent::CursorMoved { position, .. } => {
                self.color = wgpu::Color {
                    r: position.x / self.size.width as f64,
                    g: 1.0 - (position.x / self.size.width as f64),
                    b: position.y / self.size.height as f64,
                    a: 1.0,
                };
                self.window.request_redraw();
                true
            }
            _ => false,
        }
    }

    fn update(&mut self) {
        // todo!()
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // get a frame to render to, -> SurfaceTexture
        let output = self.surface.get_current_texture()?;

        // create default texture view, we'll manipulate it later
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // create the command encoder, this builds a command buffer to send commands to the GPU
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {

            // create the render pass, this is the actual command to the GPU
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(self.color),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let pipeline = match self.current_pipeline {
                1 => &self.pipeline,
                2 => &self.pipeline2,
                _ => &self.pipeline,
            };
            render_pass.set_pipeline(pipeline); // 2
            render_pass.draw(0..3, 0..1); // 3
        }

        // submit it to the queue
        self.queue.submit(std::iter::once(encoder.finish()));

        // present the output
        output.present();

        Ok(())
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window).await;

    event_loop
        .run(move |event, target| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                            state.window().request_redraw();
                        }
                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            state.resize(PhysicalSize::new(
                                (state.size.width as f64 * *scale_factor) as u32,
                                (state.size.height as f64 * *scale_factor) as u32,
                            ));
                            state.window().request_redraw();
                        }
                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(_) => (),
                                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                                Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        WindowEvent::KeyboardInput {
                            event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                logical_key,
                                ..
                            },
                            ..
                        } => {
                            match logical_key {
                                Key::Named(NamedKey::Space) => {
                                    if state.current_pipeline == 1 {
                                        state.current_pipeline = 2;
                                    } else {
                                        state.current_pipeline = 1;
                                    }
                                    state.window.request_redraw();
                                    println!("Updated current pipeline: {}", state.current_pipeline);
                                },
                                key => {
                                    match key.to_text() {
                                        Some(text) => print!("{}", text),
                                        None => print!("{:?}", key),
                                    }
                                }
                            }

                            match io::stdout().flush() {
                                Ok(_) => (),
                                Err(_) => target.exit(),
                            }
                        },
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    logical_key: Key::Named(NamedKey::Escape),
                                    state: ElementState::Released,
                                    ..
                                },
                            ..
                        } => target.exit(),
                        _ => {}
                    }
                }
            }

            _ => {}
        })
        .unwrap();
}
