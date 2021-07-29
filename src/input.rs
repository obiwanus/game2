use std::time::Instant;

use glam::Vec2;

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
