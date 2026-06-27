use dear_imgui_rs::Condition;
use dear_imgui_wgpu::WgpuRenderer;
use dear_imgui_winit::WinitPlatform;
use log::debug;
use std::default::Default;
use std::{path::PathBuf, sync::Arc, time::Instant};

use crate::{
    camera::CameraUniform,
    scene::{InstanceRaw, Scene, Vertex},
    world::World,
};
use anyhow::{Context, Result};
use wgpu::{
    BackendOptions, Dx12BackendOptions, ExperimentalFeatures, GlBackendOptions, InstanceFlags,
    MemoryBudgetThresholds, NoopBackendOptions, SurfaceConfiguration, TextureFormat,
};
use wgpu::{CurrentSurfaceTexture, util::DeviceExt};
use winit::event::WindowEvent;
use winit::{dpi::PhysicalSize, window::Window};

pub struct Renderer<'window> {
    pub window: Arc<winit::window::Window>,
    pub surface: RenderSurface<'window>,
    pub resources: RenderResources,
    pub imgui: ImguiState,
}

impl Renderer<'static> {
    pub fn new(ctx: &GpuContext, window: Arc<Window>) -> Result<Self> {
        let surface = ctx
            .instance
            .create_surface(window.clone())
            .context("failed to create render surface for window")?;

        Self::from_surface(ctx, surface, window.clone(), window.inner_size())
    }
}

impl<'window> Renderer<'window> {
    pub fn from_surface(
        ctx: &GpuContext,
        surface: wgpu::Surface<'window>,
        window: Arc<winit::window::Window>,
        size: PhysicalSize<u32>,
    ) -> Result<Self> {
        let surface = RenderSurface::new(ctx, surface, size)?;
        let resources = RenderResources::new(ctx, surface.surface_format);

        // Setup ImGui immediately
        let mut context = dear_imgui_rs::Context::create();
        context.set_ini_filename(None::<String>).unwrap();
        let ini_path = std::env::temp_dir().join("dear-imgui-wgpu_basic.ini");
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Err(e) = context.load_ini_settings_from_disk(ini_path.clone()) {
                debug!("Failed to load ini settings from {:?}: {e}", ini_path);
            }
        }

        let mut platform = WinitPlatform::new(&mut context);
        platform.attach_window(
            window.as_ref(),
            dear_imgui_winit::HiDpiMode::Default,
            &mut context,
        );

        // Method 1: One-step initialization (recommended)
        let init_info = dear_imgui_wgpu::WgpuInitInfo::new(
            ctx.device.clone(),
            ctx.queue.clone(),
            surface.config.format,
        );
        let mut renderer =
            WgpuRenderer::new(init_info, &mut context).expect("Failed to initialize WGPU renderer");
        // Unify visuals (sRGB): auto gamma by format, matches official practice
        renderer.set_gamma_mode(dear_imgui_wgpu::GammaMode::Auto);

        // Log successful initialization
        dear_imgui_rs::logging::log_context_created();
        dear_imgui_rs::logging::log_platform_init("Winit");
        dear_imgui_rs::logging::log_renderer_init("WGPU");

        let imgui = ImguiState {
            context,
            platform,
            renderer,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            demo_open: true,
            last_frame: Instant::now(),
            ini_path,
            log_counter: 0,
            frame_count: 0,
            total_frame_time: 0.0,
        };

        Ok(Self {
            window,
            surface,
            resources,
            imgui,
        })
    }

    pub fn texture_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.resources.texture_bind_group_layout
    }

    pub fn resize(&mut self, ctx: &GpuContext, size: PhysicalSize<u32>) {
        self.surface.resize(ctx, size);
    }

    // pub fn rebuild_render_batches(&mut self, ctx: &GpuContext) {
    //     let mut grouped_instances: BTreeMap<(MeshHandle, MaterialHandle), Vec<InstanceRaw>> =
    //         BTreeMap::new();

    //     for (entity, renderer) in &self.mesh_renderers {
    //         let Some(transform) = self.transforms.get(entity) else {
    //             continue;
    //         };

    //         grouped_instances
    //             .entry((renderer.mesh, renderer.material))
    //             .or_default()
    //             .push(InstanceRaw::from_transform(transform));
    //     }

    //     self.render_batches = grouped_instances
    //         .into_iter()
    //         .filter_map(|((mesh, material), instances)| {
    //             if instances.is_empty() {
    //                 return None;
    //             }

    //             let instance_buffer =
    //                 ctx.device
    //                     .create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //                         label: Some("Instance Buffer"),
    //                         contents: bytemuck::cast_slice(&instances),
    //                         usage: wgpu::BufferUsages::VERTEX,
    //                     });

    //             Some(RenderBatch {
    //                 mesh,
    //                 material,
    //                 instance_buffer,
    //                 instance_count: instances.len() as u32,
    //             })
    //         })
    //         .collect();
    // }

    pub fn render(&mut self, ctx: &GpuContext, world: &World) -> Option<CurrentSurfaceTexture> {
        if !self.surface.is_configured {
            return None;
        }

        if let Some(camera_uniform) = world.active_camera_uniform() {
            ctx.queue.write_buffer(
                &self.resources.camera_buffer,
                0,
                bytemuck::bytes_of(&camera_uniform),
            );
        }

        let output = match self.surface.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(current_texture) => current_texture,
            _ => return None,
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.imgui.render(ctx, &mut self.surface, &mut self.window);

        self.imgui
            .platform
            .prepare_frame(&self.window, &mut self.imgui.context);
        let ui = self.imgui.context.frame();
        {
            // Main window content

            ui.window("Hello, Dear ImGui!")
                .size([400.0, 300.0], Condition::FirstUseEver)
                .build(|| {
                    ui.text("Welcome to Dear ImGui Rust bindings!");
                    ui.separator();

                    ui.text(format!(
                        "Application average {:.3} ms/frame ({:.1} FPS)",
                        1000.0 / ui.io().framerate(),
                        ui.io().framerate()
                    ));
                });
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                multiview_mask: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                // depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                //     view: &self.surface.depth_texture.view,
                //     depth_ops: Some(wgpu::Operations {
                //         load: wgpu::LoadOp::Clear(1.0),
                //         store: wgpu::StoreOp::Store,
                //     }),
                //     stencil_ops: None,
                // }),
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.resources.render_pipeline);
            render_pass.set_bind_group(1, &self.resources.camera_bind_group, &[]);

            // for batch in &world.render_batches {
            //     let Some(mesh) = world.mesh(batch.mesh) else {
            //         continue;
            //     };
            //     let Some(material) = world.material(batch.material) else {
            //         continue;
            //     };

            //     render_pass.set_bind_group(0, &material.bind_group, &[]);
            //     render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            //     render_pass.set_vertex_buffer(1, batch.instance_buffer.slice(..));
            //     render_pass
            //         .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            //     render_pass.draw_indexed(0..mesh.index_count, 0, 0..batch.instance_count);
            // }

            // Call new_frame before rendering
            self.imgui
                .renderer
                .new_frame()
                .expect("Failed to prepare new frame");

            self.imgui
                .renderer
                .render_context(&mut self.imgui.context, &mut render_pass);
        }

        ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        None
    }
}

pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GpuContext {
    pub async fn new(window: Arc<Window>) -> Result<(Self, Renderer<'static>)> {
        let instance = Self::create_instance();
        let surface = instance
            .create_surface(window.clone())
            .context("failed to create initial render surface")?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("failed to find a compatible GPU adapter")?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
                experimental_features: ExperimentalFeatures::disabled(),
            })
            .await
            .context("failed to create logical GPU device")?;

        let ctx = Self {
            instance,
            adapter,
            device,
            queue,
        };
        let renderer = Renderer::from_surface(&ctx, surface, window.clone(), window.inner_size())?;

        Ok((ctx, renderer))
    }

    fn create_instance() -> wgpu::Instance {
        wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            display: None,
            backend_options: BackendOptions {
                dx12: Dx12BackendOptions::default(),
                gl: GlBackendOptions::default(),
                noop: NoopBackendOptions { enable: true },
            },
            flags: InstanceFlags::default(),
            memory_budget_thresholds: MemoryBudgetThresholds::default(),
        })
    }
}

pub struct ImguiState {
    context: dear_imgui_rs::Context,
    platform: WinitPlatform,
    renderer: WgpuRenderer,
    clear_color: wgpu::Color,
    demo_open: bool,
    last_frame: Instant,
    ini_path: PathBuf,
    // Logging demo state
    log_counter: i32,
    frame_count: u64,
    total_frame_time: f32,
}

impl<'window> ImguiState {
    pub fn render(
        &mut self,
        ctx: &GpuContext,
        surface: &mut RenderSurface<'window>,
        window: &mut Arc<Window>,
    ) -> std::result::Result<(), wgpu::Error> {
        let now = Instant::now();
        let delta_time = now - self.last_frame;
        let delta_secs = delta_time.as_secs_f32();

        // Update frame statistics
        self.frame_count += 1;
        self.total_frame_time += delta_secs;

        // Log frame statistics every 60 frames
        if self.frame_count % 60 == 0 {
            let avg_frame_time = self.total_frame_time / 60.0;
            dear_imgui_rs::logging::log_frame_stats(avg_frame_time, 1.0 / avg_frame_time);
            self.total_frame_time = 0.0;
        }

        self.context.io_mut().set_delta_time(delta_secs);
        self.last_frame = now;

        self.platform.prepare_frame(&window, &mut self.context);
        Ok(())
    }

    pub fn handle_window_event(&mut self, window: &Window, event: &WindowEvent) {
        self.platform
            .handle_window_event(&mut self.context, window, event);
    }
}

pub struct RenderResources {
    pub render_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
}

impl RenderResources {
    pub fn new(ctx: &GpuContext, target_format: wgpu::TextureFormat) -> Self {
        let texture_bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    label: Some("texture_bind_group_layout"),
                });

        let camera_uniform = CameraUniform::new();
        let camera_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::bytes_of(&camera_uniform),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("camera_bind_group_layout"),
                });

        let camera_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

        let render_pipeline_layout =
            ctx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    immediate_size: 0,
                    bind_group_layouts: &[
                        Some(&texture_bind_group_layout),
                        Some(&camera_bind_group_layout),
                    ],
                });

        let render_pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                multiview_mask: None,
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex::desc(), InstanceRaw::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: target_format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                // depth_stencil: Some(wgpu::DepthStencilState {
                //     format: crate::texture::Texture::DEPTH_FORMAT,
                //     depth_write_enabled: Some(true),
                //     depth_compare: Some(wgpu::CompareFunction::Less),
                //     stencil: wgpu::StencilState::default(),
                //     bias: wgpu::DepthBiasState::default(),
                // }),
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                cache: None,
            });

        Self {
            render_pipeline,
            texture_bind_group_layout,
            camera_bind_group_layout,
            camera_buffer,
            camera_bind_group,
        }
    }
}

pub struct RenderSurface<'window> {
    pub surface: wgpu::Surface<'window>,
    pub config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    pub depth_texture: crate::texture::Texture,
    pub surface_format: wgpu::TextureFormat,
    pub is_configured: bool,
}

impl<'window> RenderSurface<'window> {
    pub fn new(
        ctx: &GpuContext,
        surface: wgpu::Surface<'window>,
        size: PhysicalSize<u32>,
    ) -> Result<Self> {
        let surface_caps = surface.get_capabilities(&ctx.adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .or_else(|| surface_caps.formats.first().copied())
            .context("surface reports no supported texture formats")?;
        let present_mode = surface_caps
            .present_modes
            .first()
            .copied()
            .context("surface reports no supported present modes")?;
        let alpha_mode = surface_caps
            .alpha_modes
            .first()
            .copied()
            .context("surface reports no supported alpha modes")?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let depth_texture =
            crate::texture::Texture::create_depth_texture(&ctx.device, &config, "depth_texture");
        let mut render_surface = Self {
            surface,
            config,
            size,
            depth_texture,
            surface_format,
            is_configured: false,
        };

        if size.width > 0 && size.height > 0 {
            render_surface.configure(ctx);
        }

        Ok(render_surface)
    }

    pub fn configure(&mut self, ctx: &GpuContext) {
        self.surface.configure(&ctx.device, &self.config);
        self.is_configured = true;
    }

    pub fn aspect(&self) -> f32 {
        self.config.width as f32 / self.config.height as f32
    }

    pub fn resize(&mut self, ctx: &GpuContext, size: PhysicalSize<u32>) {
        self.size = size;
        if size.width == 0 || size.height == 0 {
            self.is_configured = false;
            return;
        }

        self.config.width = size.width;
        self.config.height = size.height;
        self.depth_texture = crate::texture::Texture::create_depth_texture(
            &ctx.device,
            &self.config,
            "depth_texture",
        );
        self.configure(ctx);
    }
}
