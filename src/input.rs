use std::{
    convert::{TryFrom, TryInto},
    time::Instant,
};

use egui::RawInput as EguiInput;
use glam::{DVec2, Vec2};
use glutin::event::VirtualKeyCode;

#[derive(Clone, Debug)]
pub struct RawInput {
    /// The point in time when the game started (don't change it once set)
    pub game_start: Instant,

    /// The start time of the current frame (must monotonically increase)
    pub frame_start: Instant,

    /// Seconds passed since last frame started
    pub delta_time: f32,

    /// In logical pixels
    pub screen_size: Vec2,

    /// In physical pixels
    pub pointer_pos: Vec2,

    /// In physical pixels
    pub pointer_delta: DVec2,

    /// Screen scale factor
    pub scale_factor: f64,

    /// In logical pixels
    pub scroll_delta: Vec2,

    /// The state of modifier keys
    pub modifiers: Modifiers,
    // In-order events received since last frame
    // pub events: Vec<Event>,
}

impl RawInput {
    pub fn new(now: Instant, screen_size: Vec2, scale_factor: f64) -> Self {
        RawInput {
            game_start: now,
            frame_start: now,
            delta_time: 0.0,
            screen_size,
            pointer_pos: Vec2::new(f32::INFINITY, f32::INFINITY),
            pointer_delta: Default::default(),
            scale_factor,
            scroll_delta: Vec2::new(0.0, 0.0),
            modifiers: Default::default(),
            // events: Vec::new(),
        }
    }

    pub fn take(&mut self) -> Self {
        let now = Instant::now();
        let delta_time = now.duration_since(self.frame_start).as_secs_f32();
        RawInput {
            game_start: self.game_start,
            frame_start: std::mem::replace(&mut self.frame_start, now),
            delta_time: std::mem::replace(&mut self.delta_time, delta_time),
            screen_size: self.screen_size,
            pointer_pos: self.pointer_pos,
            pointer_delta: std::mem::take(&mut self.pointer_delta),
            scale_factor: self.scale_factor,
            scroll_delta: std::mem::take(&mut self.scroll_delta),
            modifiers: self.modifiers,
            // events: std::mem::take(&mut self.events),
        }
    }

    pub fn into_egui_input(&self) -> EguiInput {
        EguiInput::default()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Modifiers {
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
    pub logo: bool,
}

#[derive(Default)]
pub struct Input {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,

    pub pointer: Vec2,
    pub pointer_moved: bool,
    pub left_mouse_button_pressed: bool,

    pub wasd_mode: bool,
    pub should_exit: bool,
}

pub fn vec2_to_egui_vec2(vec2: Vec2) -> egui::Vec2 {
    egui::Vec2 {
        x: vec2.x,
        y: vec2.y,
    }
}

pub fn vec2_to_egui_pos2(vec2: Vec2) -> egui::Pos2 {
    egui::Pos2 {
        x: vec2.x,
        y: vec2.y,
    }
}

pub fn vkeycode_to_egui_key(virtual_key_code: VirtualKeyCode) -> Option<egui::Key> {
    let key = match virtual_key_code {
        VirtualKeyCode::Down => egui::Key::ArrowDown,
        VirtualKeyCode::Left => egui::Key::ArrowLeft,
        VirtualKeyCode::Right => egui::Key::ArrowRight,
        VirtualKeyCode::Up => egui::Key::ArrowUp,

        VirtualKeyCode::Escape => egui::Key::Escape,
        VirtualKeyCode::Tab => egui::Key::Tab,
        VirtualKeyCode::Back => egui::Key::Backspace,
        VirtualKeyCode::Return => egui::Key::Enter,
        VirtualKeyCode::Space => egui::Key::Space,

        VirtualKeyCode::Insert => egui::Key::Insert,
        VirtualKeyCode::Delete => egui::Key::Delete,
        VirtualKeyCode::Home => egui::Key::Home,
        VirtualKeyCode::End => egui::Key::End,
        VirtualKeyCode::PageUp => egui::Key::PageUp,
        VirtualKeyCode::PageDown => egui::Key::PageDown,

        VirtualKeyCode::Numpad0 => egui::Key::Num0,
        VirtualKeyCode::Numpad1 => egui::Key::Num1,
        VirtualKeyCode::Numpad2 => egui::Key::Num2,
        VirtualKeyCode::Numpad3 => egui::Key::Num3,
        VirtualKeyCode::Numpad4 => egui::Key::Num4,
        VirtualKeyCode::Numpad5 => egui::Key::Num5,
        VirtualKeyCode::Numpad6 => egui::Key::Num6,
        VirtualKeyCode::Numpad7 => egui::Key::Num7,
        VirtualKeyCode::Numpad8 => egui::Key::Num8,
        VirtualKeyCode::Numpad9 => egui::Key::Num9,

        VirtualKeyCode::Key0 => egui::Key::Num0,
        VirtualKeyCode::Key1 => egui::Key::Num1,
        VirtualKeyCode::Key2 => egui::Key::Num2,
        VirtualKeyCode::Key3 => egui::Key::Num3,
        VirtualKeyCode::Key4 => egui::Key::Num4,
        VirtualKeyCode::Key5 => egui::Key::Num5,
        VirtualKeyCode::Key6 => egui::Key::Num6,
        VirtualKeyCode::Key7 => egui::Key::Num7,
        VirtualKeyCode::Key8 => egui::Key::Num8,
        VirtualKeyCode::Key9 => egui::Key::Num9,

        VirtualKeyCode::A => egui::Key::A,
        VirtualKeyCode::B => egui::Key::B,
        VirtualKeyCode::C => egui::Key::C,
        VirtualKeyCode::D => egui::Key::D,
        VirtualKeyCode::E => egui::Key::E,
        VirtualKeyCode::F => egui::Key::F,
        VirtualKeyCode::G => egui::Key::G,
        VirtualKeyCode::H => egui::Key::H,
        VirtualKeyCode::I => egui::Key::I,
        VirtualKeyCode::J => egui::Key::J,
        VirtualKeyCode::K => egui::Key::K,
        VirtualKeyCode::L => egui::Key::L,
        VirtualKeyCode::M => egui::Key::M,
        VirtualKeyCode::N => egui::Key::N,
        VirtualKeyCode::O => egui::Key::O,
        VirtualKeyCode::P => egui::Key::P,
        VirtualKeyCode::Q => egui::Key::Q,
        VirtualKeyCode::R => egui::Key::R,
        VirtualKeyCode::S => egui::Key::S,
        VirtualKeyCode::T => egui::Key::T,
        VirtualKeyCode::U => egui::Key::U,
        VirtualKeyCode::V => egui::Key::V,
        VirtualKeyCode::W => egui::Key::W,
        VirtualKeyCode::X => egui::Key::X,
        VirtualKeyCode::Y => egui::Key::Y,
        VirtualKeyCode::Z => egui::Key::Z,

        _ => return None,
    };

    Some(key)
}

pub fn key_to_string(key: egui::Key, shift: bool) -> Option<String> {
    let str = match key {
        egui::Key::A => "A",
        egui::Key::B => "B",
        egui::Key::C => "C",
        egui::Key::D => "D",
        egui::Key::E => "E",
        egui::Key::F => "F",
        egui::Key::G => "G",
        egui::Key::H => "H",
        egui::Key::I => "I",
        egui::Key::J => "J",
        egui::Key::K => "K",
        egui::Key::L => "L",
        egui::Key::M => "M",
        egui::Key::N => "N",
        egui::Key::O => "O",
        egui::Key::P => "P",
        egui::Key::Q => "Q",
        egui::Key::R => "R",
        egui::Key::S => "S",
        egui::Key::T => "T",
        egui::Key::U => "U",
        egui::Key::V => "V",
        egui::Key::W => "W",
        egui::Key::X => "X",
        egui::Key::Y => "Y",
        egui::Key::Z => "Z",

        egui::Key::Num0 => "0",
        egui::Key::Num1 => "1",
        egui::Key::Num2 => "2",
        egui::Key::Num3 => "3",
        egui::Key::Num4 => "4",
        egui::Key::Num5 => "5",
        egui::Key::Num6 => "6",
        egui::Key::Num7 => "7",
        egui::Key::Num8 => "8",
        egui::Key::Num9 => "9",

        egui::Key::Space => " ",

        _ => return None,
    };

    Some(if shift {
        str.to_owned()
    } else {
        str.to_ascii_lowercase()
    })
}
