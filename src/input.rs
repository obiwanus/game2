use glam::Vec2;

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
