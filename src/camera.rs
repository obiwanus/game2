#![allow(dead_code)]

use std::f32::consts::PI;

use glam::{Mat4, Vec3};

use super::utils::clamp;

const FOV_MIN: f32 = 0.01 * PI;
const FOV_MAX: f32 = 0.5 * PI;

const ZOOM_MIN: f32 = 1.0;
const ZOOM_MAX: f32 = 100.0;
const ZOOM_DEFAULT: f32 = 30.0;

const PITCH_MIN: f32 = -0.49 * PI;
const PITCH_MAX: f32 = 0.49 * PI;

pub enum Movement {
    Forward,
    Backward,
    Left,
    Right,
}

#[derive(Debug)]
pub struct Camera {
    pub position: Vec3,
    direction: Vec3,
    up: Vec3,
    right: Vec3,

    yaw: f32,
    pitch: f32,

    movement_speed: f32,
    sensitivity: f32,
    zoom: f32,
    pub aspect_ratio: f32,
    locked: bool, // whether to allow flying
    pub moved: bool,
}

impl Camera {
    pub fn new(position: Vec3, up: Vec3, look_at: Vec3, aspect_ratio: f32) -> Self {
        let mut camera = Camera {
            position,
            direction: position + look_at,
            up,
            right: Vec3::new(1.0, 0.0, 0.0),

            yaw: 0.0,
            pitch: 0.0,

            movement_speed: 15.0,
            sensitivity: 0.0015,
            zoom: ZOOM_DEFAULT,
            aspect_ratio,
            locked: false,
            moved: true,
        };
        camera.look_at(look_at);
        camera
    }

    /// Point the camera at the target.
    /// Sets direction, right and Euler angles accordingly
    pub fn look_at(&mut self, target: Vec3) {
        self.direction = (target - self.position).normalize();
        let (x, y, z) = (self.direction.x, self.direction.y, self.direction.z);
        self.pitch = y.asin();
        self.pitch = clamp(self.pitch, PITCH_MIN, PITCH_MAX);
        self.yaw = (z / x).atan();
        self.right = self.recalculate_right();
    }

    /// Move the camera
    pub fn go(&mut self, direction: Movement, delta_time: f32) {
        let speed = self.movement_speed * delta_time;
        let projected_direction = if self.locked {
            Vec3::new(self.direction.x, 0.0, self.direction.z)
        } else {
            self.direction
        };
        match direction {
            Movement::Forward => self.position += speed * projected_direction,
            Movement::Backward => self.position -= speed * projected_direction,
            Movement::Left => self.position -= speed * self.right,
            Movement::Right => self.position += speed * self.right,
        }
        self.moved = true;
    }

    /// Zoom is used to calculate the vertical FOV:
    ///
    /// 1.0 corresponds to FOV_MAX,
    /// 100.0 corresponds to FOV_MIN.
    pub fn adjust_zoom(&mut self, delta: i32) {
        self.zoom += delta as f32;
        self.zoom = clamp(self.zoom, ZOOM_MIN, ZOOM_MAX);
    }

    pub fn rotate(&mut self, yaw_delta: f64, pitch_delta: f64) {
        // Adjust Euler angles
        self.pitch -= pitch_delta as f32 * self.sensitivity;
        self.pitch = clamp(self.pitch, PITCH_MIN, PITCH_MAX);
        self.yaw += yaw_delta as f32 * self.sensitivity;

        // Recalculate direction
        self.direction = Vec3::new(
            self.pitch.cos() * self.yaw.cos(),
            self.pitch.sin(),
            self.pitch.cos() * self.yaw.sin(),
        )
        .normalize();
        self.right = self.recalculate_right();

        self.moved = true;
    }

    fn recalculate_right(&self) -> Vec3 {
        self.direction.cross(self.up).normalize()
    }

    /// Calculate vertical FOV based on zoom level
    pub fn fov(&self) -> f32 {
        let t = (self.zoom - ZOOM_MIN) / (ZOOM_MAX - ZOOM_MIN);
        (1.0 - t) * FOV_MAX + t * FOV_MIN
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.position + self.direction, self.up)
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        let mut proj = Mat4::perspective_rh(self.fov(), self.aspect_ratio, 0.1, 100.0);
        proj.y_axis.y *= -1.0; // account for the Vulkan coordinate system
        proj
    }
}
