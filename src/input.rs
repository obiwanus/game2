use std::time::Instant;

use egui::RawInput as EguiInput;
use glam::Vec2;
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

    /// Screen scale factor
    pub scale_factor: f64,

    /// In logical pixels
    pub scroll_delta: Vec2,

    /// The state of modifier keys
    pub modifiers: Modifiers,

    /// In-order events received since last frame
    pub events: Vec<Event>,
}

impl RawInput {
    pub fn new(now: Instant, screen_size: Vec2, scale_factor: f64) -> Self {
        RawInput {
            game_start: now,
            frame_start: now,
            delta_time: 0.0,
            screen_size,
            scale_factor,
            scroll_delta: Vec2::new(0.0, 0.0),
            modifiers: Default::default(),
            events: Vec::new(),
        }
    }

    /// To be used an the end of the frame to get a fresh input
    /// Returns old input
    pub fn renew(&mut self) -> Self {
        let now = Instant::now();
        let delta_time = now.duration_since(self.frame_start).as_secs_f32();
        RawInput {
            game_start: self.game_start,
            frame_start: std::mem::replace(&mut self.frame_start, now),
            delta_time: std::mem::replace(&mut self.delta_time, delta_time),
            screen_size: self.screen_size,
            scale_factor: self.scale_factor,
            scroll_delta: std::mem::take(&mut self.scroll_delta),
            modifiers: self.modifiers,
            events: std::mem::take(&mut self.events),
        }
    }

    pub fn into_egui_input(&self) -> EguiInput {
        EguiInput::default()
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    Copy,
    Cut,
    Key {
        key: Key,
        pressed: bool,
        modifiers: Modifiers,
    },
    PointerMoved(Vec2),
    MouseButton {
        pos: Vec2,
        button: MouseButton,
        pressed: bool,
        modifiers: Modifiers,
    },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Modifiers {
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
    pub logo: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MouseButton {
    Primary = 0,
    Secondary = 1,
    Middle = 2,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum Key {
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,

    Escape,
    Tab,
    Backspace,
    Enter,
    Space,

    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,

    // Numpad digits
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,

    // Keyboard digits
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    Unknown,
}

impl From<VirtualKeyCode> for Key {
    fn from(virtual_key_code: VirtualKeyCode) -> Self {
        use glutin::event::VirtualKeyCode::*;

        match virtual_key_code {
            A => Key::A,
            B => Key::B,
            C => Key::C,
            D => Key::D,
            E => Key::E,
            F => Key::F,
            G => Key::G,
            H => Key::H,
            I => Key::I,
            J => Key::J,
            K => Key::K,
            L => Key::L,
            M => Key::M,
            N => Key::N,
            O => Key::O,
            P => Key::P,
            Q => Key::Q,
            R => Key::R,
            S => Key::S,
            T => Key::T,
            U => Key::U,
            V => Key::V,
            W => Key::W,
            X => Key::X,
            Y => Key::Y,
            Z => Key::Z,

            F1 => Key::F1,
            F2 => Key::F2,
            F3 => Key::F3,
            F4 => Key::F4,
            F5 => Key::F5,
            F6 => Key::F6,
            F7 => Key::F7,
            F8 => Key::F8,
            F9 => Key::F9,
            F10 => Key::F10,
            F11 => Key::F11,
            F12 => Key::F12,
            F13 => Key::F13,
            F14 => Key::F14,
            F15 => Key::F15,
            F16 => Key::F16,
            F17 => Key::F17,
            F18 => Key::F18,
            F19 => Key::F19,
            F20 => Key::F20,
            F21 => Key::F21,
            F22 => Key::F22,
            F23 => Key::F23,
            F24 => Key::F24,

            Escape => Key::Escape,
            Insert => Key::Insert,
            Home => Key::Home,
            Delete => Key::Delete,
            End => Key::End,
            PageDown => Key::PageDown,
            PageUp => Key::PageUp,

            Left => Key::ArrowLeft,
            Up => Key::ArrowUp,
            Right => Key::ArrowRight,
            Down => Key::ArrowDown,

            Back => Key::Backspace,
            Return => Key::Enter,
            Space => Key::Space,

            Key1 => Key::Key1,
            Key2 => Key::Key2,
            Key3 => Key::Key3,
            Key4 => Key::Key4,
            Key5 => Key::Key5,
            Key6 => Key::Key6,
            Key7 => Key::Key7,
            Key8 => Key::Key8,
            Key9 => Key::Key9,
            Key0 => Key::Key0,

            Numpad0 => Key::Num0,
            Numpad1 => Key::Num1,
            Numpad2 => Key::Num2,
            Numpad3 => Key::Num3,
            Numpad4 => Key::Num4,
            Numpad5 => Key::Num5,
            Numpad6 => Key::Num6,
            Numpad7 => Key::Num7,
            Numpad8 => Key::Num8,
            Numpad9 => Key::Num9,

            _ => Key::Unknown,
        }
    }
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
    pub brush_size_changed: bool,

    pub wasd_mode: bool,
}
