use glam::Vec2;
use glutin::event::VirtualKeyCode;

#[derive(Default, Clone)]
pub struct Input {
    // Raw
    pub pointer: Vec2,
    pub pointer_moved: bool,
    pub pointer_delta: Vec2,
    pub scrolled: bool,
    pub scroll_delta: Vec2,
    pub modifiers: Modifiers,
    pub mouse_buttons: MouseButtons,
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub time: f32,

    // Processed
    pub should_exit: bool,
    pub camera_moved: bool,
}

impl Input {
    /// Clear volatiles, persist everything else
    pub fn renew(&mut self) -> Input {
        let old_input = self.clone();
        *self = Input {
            pointer: self.pointer,
            mouse_buttons: self.mouse_buttons,
            forward: self.forward,
            back: self.back,
            left: self.left,
            right: self.right,
            modifiers: self.modifiers,
            should_exit: self.should_exit,
            ..Default::default()
        };
        old_input
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MouseButtons {
    pub primary: bool,
    pub middle: bool,
    pub secondary: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Modifiers {
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
    pub logo: bool,
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
