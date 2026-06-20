use std::sync::Arc;

use anyhow::Result;
use log::debug;
use wgpu::CurrentSurfaceTexture;
use winit::{
    dpi::PhysicalSize,
    window::{Window, WindowId},
};

use crate::{
    input::InputService,
    renderer::{GpuContext, Renderer},
    scene::Scene,
    window::{WindowService, WindowState},
};

pub struct Engine<'window> {
    pub ctx: GpuContext,
    pub windows: WindowService<'window>,
    pub input: InputService,
    pub scene: Scene,
}

impl Engine<'static> {
    pub async fn new(first_window: Arc<Window>) -> Result<Self> {
        let (ctx, renderer) = GpuContext::new(first_window.clone()).await?;
        let scene = Scene::default_instanced(
            &ctx,
            renderer.texture_bind_group_layout(),
            renderer.surface.aspect(),
        )?;
        let mut windows = WindowService::new();
        windows.insert(WindowState::new(first_window, renderer));

        Ok(Self {
            ctx,
            windows,
            input: InputService::new(),
            scene,
        })
    }

    pub fn add_window(&mut self, window: Arc<Window>) -> Result<WindowId> {
        let renderer = Renderer::new(&self.ctx, window.clone())?;
        Ok(self.windows.insert(WindowState::new(window, renderer)))
    }
}

impl<'window> Engine<'window> {
    pub fn resize_window(&mut self, id: WindowId, size: PhysicalSize<u32>) {
        if let Some(window) = self.windows.get_mut(id) {
            window.resize(&self.ctx, size);
            if window.renderer.surface.is_configured {
                self.scene
                    .set_active_camera_aspect(window.renderer.surface.aspect());
            }
        }
    }

    pub fn set_window_focused(&mut self, id: WindowId, focused: bool) {
        self.windows.set_focused(id, focused);
    }

    pub fn render_window(&mut self, id: WindowId) -> Option<CurrentSurfaceTexture> {
        let Some(window) = self.windows.get_mut(id) else {
            debug!("missing window id {:#?}", id);
            return None;
        };

        window.renderer.render(&self.ctx, &self.scene)
    }

    pub fn rebuild_scene_render_batches(&mut self) {
        self.scene.rebuild_render_batches(&self.ctx);
    }
}
