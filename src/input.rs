use std::collections::{HashMap, HashSet};

use winit::{event::MouseButton, keyboard::KeyCode};
use winit::dpi::PhysicalPosition;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ControllerId(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ControllerButton {
    South,
    East,
    West,
    North,
    LeftShoulder,
    RightShoulder,
    Select,
    Start,
    LeftStick,
    RightStick,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

#[derive(Clone, Debug, Default)]
pub struct ControllerState {
    pub buttons: HashSet<ControllerButton>,
    pub left_stick: [f32; 2],
    pub right_stick: [f32; 2],
    pub left_trigger: f32,
    pub right_trigger: f32,
}

impl ControllerState {
    pub fn set_button(&mut self, button: ControllerButton, pressed: bool) {
        if pressed {
            self.buttons.insert(button);
        } else {
            self.buttons.remove(&button);
        }
    }

    pub fn is_button_pressed(&self, button: ControllerButton) -> bool {
        self.buttons.contains(&button)
    }
}

#[derive(Default)]
pub struct InputService {
    keys: HashSet<KeyCode>,
    mouse_buttons: HashSet<MouseButton>,
    controllers: HashMap<ControllerId, ControllerState>,
    position: PhysicalPosition<f64>,
}

impl InputService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_key(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            self.keys.insert(key);
        } else {
            self.keys.remove(&key);
        }
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys.contains(&key)
    }

    pub fn clear_keys(&mut self) {
        self.keys.clear();
    }

    pub fn set_mouse_button(&mut self, button: MouseButton, pressed: bool) {
        if pressed {
            self.mouse_buttons.insert(button);
        } else {
            self.mouse_buttons.remove(&button);
        }
    }

    pub fn set_cursor_position(&mut self, position: PhysicalPosition<f64>) {
        self.position = position;
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons.contains(&button)
    }

    pub fn controller(&self, id: ControllerId) -> Option<&ControllerState> {
        self.controllers.get(&id)
    }

    pub fn controller_mut(&mut self, id: ControllerId) -> &mut ControllerState {
        self.controllers.entry(id).or_default()
    }

    pub fn set_controller_button(
        &mut self,
        id: ControllerId,
        button: ControllerButton,
        pressed: bool,
    ) {
        self.controller_mut(id).set_button(button, pressed);
    }

    pub fn set_left_stick(&mut self, id: ControllerId, x: f32, y: f32) {
        self.controller_mut(id).left_stick = [x, y];
    }

    pub fn set_right_stick(&mut self, id: ControllerId, x: f32, y: f32) {
        self.controller_mut(id).right_stick = [x, y];
    }

    pub fn set_triggers(&mut self, id: ControllerId, left: f32, right: f32) {
        let controller = self.controller_mut(id);
        controller.left_trigger = left;
        controller.right_trigger = right;
    }
}
