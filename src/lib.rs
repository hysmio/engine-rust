use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::engine::Engine;

pub mod camera;
pub mod engine;
pub mod input;
pub mod renderer;
pub mod scene;
pub mod texture;
pub mod window;

pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<Engine<'static>>>,
    engine: Option<Engine<'static>>,
}

impl App {
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<Engine<'static>>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());

        Self {
            engine: None,
            #[cfg(target_arch = "wasm32")]
            proxy,
        }
    }
}

impl ApplicationHandler<Engine<'static>> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.engine.is_some() {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        if self.proxy.is_none() {
            return;
        }

        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            let window_id = window.id();
            let engine = pollster::block_on(Engine::new(window)).unwrap();
            if let Some(window) = engine.windows.get(window_id) {
                window.window.request_redraw();
            }
            self.engine = Some(engine);
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(proxy
                        .send_event(
                            Engine::new(window)
                                .await
                                .expect("Unable to create canvas!!!"),
                        )
                        .is_ok())
                });
            }
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: Engine<'static>) {
        #[cfg(target_arch = "wasm32")]
        {
            let window_id = event
                .windows
                .focused
                .or_else(|| event.windows.windows.keys().next().copied());
            if let Some(window_id) = window_id {
                let size = event.windows.get(window_id).map(|window| {
                    window.window.request_redraw();
                    window.window.inner_size()
                });
                if let Some(size) = size {
                    event.resize_window(window_id, size);
                }
            }
        }
        self.engine = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let engine = match &mut self.engine {
            Some(engine) => engine,
            None => return,
        };

        // Handle the event with ImGui first (window-local path)
        let window = match engine.windows.get_mut(window_id) {
            Some(window) => window,
            None => return,
        };

        window.handle_event(event.clone());

        match event {
            WindowEvent::CloseRequested => {
                engine.windows.remove(window_id);
                if engine.windows.windows.is_empty() {
                    event_loop.exit();
                }
            }
            WindowEvent::Focused(focused) => engine.set_window_focused(window_id, focused),
            WindowEvent::Resized(size) => engine.resize_window(window_id, size),
            WindowEvent::RedrawRequested => match engine.render_window(window_id) {
                Some(wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost) => {
                    let size = engine
                        .windows
                        .get(window_id)
                        .map(|window| window.window.inner_size());
                    if let Some(size) = size {
                        engine.resize_window(window_id, size);
                    }
                }
                Some(_) => {
                    if let Some(window) = engine.windows.get(window_id) {
                        window.window.request_redraw();
                    }
                }
                None => {
                    if let Some(window) = engine.windows.get(window_id) {
                        window.window.request_redraw();
                    }
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                engine.input.set_cursor_position(position);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                engine.input.set_mouse_button(button, state.is_pressed());
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                let pressed = key_state.is_pressed();
                engine.input.set_key(code, pressed);
                if code == KeyCode::Escape && pressed {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );
    event_loop.run_app(&mut app)?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run().unwrap_throw();

    Ok(())
}
