use std::{collections::HashMap, sync::Arc};
use winit::{
    dpi::PhysicalSize,
    window::{Window, WindowId},
};

use crate::renderer::{GpuContext, Renderer};

pub struct WindowService<'windows> {
    pub windows: HashMap<WindowId, WindowState<'windows>>,
    pub focused: Option<WindowId>,
}

pub struct WindowState<'window> {
    pub window: Arc<Window>,
    pub renderer: Renderer<'window>,
    pub focused: bool,
    pub size: PhysicalSize<u32>,
}

impl<'windows> WindowService<'windows> {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            focused: None,
        }
    }

    pub fn insert(&mut self, window: WindowState<'windows>) -> WindowId {
        let id = window.id();
        if window.focused {
            self.focused = Some(id);
        }
        self.windows.insert(id, window);
        id
    }

    pub fn get(&self, id: WindowId) -> Option<&WindowState<'windows>> {
        self.windows.get(&id)
    }

    pub fn get_mut(&mut self, id: WindowId) -> Option<&mut WindowState<'windows>> {
        self.windows.get_mut(&id)
    }

    pub fn remove(&mut self, id: WindowId) -> Option<WindowState<'windows>> {
        let removed = self.windows.remove(&id);
        if self.focused == Some(id) {
            self.focused = None;
        }
        removed
    }

    pub fn set_focused(&mut self, id: WindowId, focused: bool) {
        if let Some(previous_id) = self.focused.filter(|previous_id| *previous_id != id) {
            if let Some(previous) = self.windows.get_mut(&previous_id) {
                previous.focused = false;
            }
        }

        if let Some(window) = self.windows.get_mut(&id) {
            window.focused = focused;
            if focused {
                self.focused = Some(id);
            } else if self.focused == Some(id) {
                self.focused = None;
            }
        }
    }

    pub fn focused(&self) -> Option<&WindowState<'windows>> {
        match self.focused {
            Some(id) => self.windows.get(&id),
            None => None,
        }
    }

    pub fn focused_mut(&mut self) -> Option<&mut WindowState<'windows>> {
        match self.focused {
            Some(id) => self.windows.get_mut(&id),
            None => None,
        }
    }
}

impl Default for WindowService<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'window> WindowState<'window> {
    pub fn new(window: Arc<Window>, renderer: Renderer<'window>) -> Self {
        let size = window.inner_size();
        Self {
            window,
            renderer,
            focused: false,
            size,
        }
    }

    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    pub fn resize(&mut self, ctx: &GpuContext, size: PhysicalSize<u32>) {
        self.size = size;
        self.renderer.resize(ctx, size);
    }
}
