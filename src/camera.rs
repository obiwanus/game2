#![allow(dead_code)]

use std::f32::consts::PI;

use glam::{const_vec3, Mat4, Vec2, Vec3};

use crate::ray::Ray;

const FOV_MIN: f32 = 0.01 * PI;
const FOV_MAX: f32 = 0.5 * PI;

const ZOOM_MIN: f32 = 1.0;
const ZOOM_MAX: f32 = 100.0;
const ZOOM_DEFAULT: f32 = 30.0;

const PITCH_MIN: f32 = -0.49 * PI;
const PITCH_MAX: f32 = 0.49 * PI;

const TRUE_UP: Vec3 = const_vec3!([0.0, 1.0, 0.0]); // Y UP

pub enum Movement {
    Forward,
    Backward,
    Left,
    Right,
}

#[derive(Debug, Default)]
pub struct Camera {
    position: Vec3,
    direction: Vec3,
    up: Vec3,
    right: Vec3,

    yaw: f32,
    pitch: f32,

    movement_speed: f32,
    sensitivity: f32,
    zoom: f32,
    screen_dimensions: Vec2,
    aspect_ratio: f32,
    v_fov: f32,
    locked: bool, // whether to allow flying

    pub speed_boost: bool,
}

impl Camera {
    pub fn new(position: Vec3, target: Vec3, screen_width: u32, screen_height: u32) -> Self {
        let screen_dimensions = Vec2::new(screen_width as f32, screen_height as f32);
        let aspect_ratio = screen_dimensions.x / screen_dimensions.y;
        let zoom = ZOOM_DEFAULT;
        let v_fov = Camera::calculate_vert_fov(zoom);

        // Camera basis
        let direction = (target - position).normalize();
        let right = direction.cross(TRUE_UP).normalize();
        let up = right.cross(direction).normalize();

        // Euler angles
        let (pitch, yaw) = {
            // @hacky: maybe could be done simpler without special cases
            let (x, y, z) = (direction.x, direction.y, direction.z);
            let pitch = y.asin();
            let pitch = pitch.clamp(PITCH_MIN, PITCH_MAX);
            let yaw = if z < 0.0 {
                (-x / z).atan()
            } else if z > 0.0 {
                (-x / z).atan() + std::f32::consts::PI
            } else {
                // z == 0
                if x > 0.0 {
                    std::f32::consts::PI / 2.0
                } else {
                    -std::f32::consts::PI / 2.0
                }
            };
            (pitch, yaw)
        };

        Camera {
            position,
            up,
            right,
            movement_speed: 10.0,
            speed_boost: false,
            sensitivity: 0.0015,
            zoom,
            v_fov,
            screen_dimensions,
            aspect_ratio,
            locked: false,
            pitch,
            yaw,
            direction,
        }
    }

    /// Move the camera
    pub fn go(&mut self, direction: Movement, delta_time: f32) {
        let speed = if self.speed_boost {
            self.movement_speed * 2.0
        } else {
            self.movement_speed
        };
        let speed = speed * delta_time;

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
    }

    /// Zoom is used to calculate the vertical FOV:
    ///
    /// 1.0 corresponds to FOV_MAX,
    /// 100.0 corresponds to FOV_MIN.
    pub fn adjust_zoom(&mut self, delta: i32) {
        self.zoom += delta as f32;
        self.zoom = self.zoom.clamp(ZOOM_MIN, ZOOM_MAX);
    }

    pub fn rotate(&mut self, yaw_delta: f32, pitch_delta: f32) {
        // Adjust Euler angles
        self.pitch -= pitch_delta * self.sensitivity;
        self.pitch = self.pitch.clamp(PITCH_MIN, PITCH_MAX);
        self.yaw += yaw_delta * self.sensitivity;

        // Recalculate direction
        self.direction = Vec3::new(
            self.pitch.cos() * self.yaw.sin(),
            self.pitch.sin(),
            self.pitch.cos() * (-self.yaw.cos()),
        )
        .normalize();
        self.right = self.direction.cross(TRUE_UP).normalize();
        self.up = self.right.cross(self.direction).normalize();
    }

    pub fn calculate_vert_fov(zoom: f32) -> f32 {
        let t = (zoom - ZOOM_MIN) / (ZOOM_MAX - ZOOM_MIN);
        (1.0 - t) * FOV_MAX + t * FOV_MIN
    }

    /// pixel has coordinates relative to the top left corner
    pub fn get_ray_through_pixel(&self, pixel: Vec2) -> Ray {
        let half_height = (self.v_fov / 2.0).tan();
        let half_width = half_height * self.aspect_ratio;
        let top_left_corner =
            self.position + self.direction - half_width * self.right + half_height * self.up;
        let pixel_width = 2.0 * half_width / self.screen_dimensions.x;
        let pixel_height = 2.0 * half_height / self.screen_dimensions.y;

        let pixel =
            top_left_corner + self.right * pixel_width * pixel.x - self.up * pixel_height * pixel.y;
        let direction = pixel - self.position;

        // TODO
        Ray::new(self.position, direction)
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        // Camera never turns upside down so true up is fixed
        Mat4::look_at_rh(self.position, self.position + self.direction, TRUE_UP)
    }

    // // For Vulkan:
    // pub fn get_projection_matrix(&self) -> Mat4 {
    //     let mut proj = Mat4::perspective_rh(self.fov(), self.aspect_ratio, 0.5, 400.0);
    //     proj.y_axis.y *= -1.0; // account for the Vulkan coordinate system
    //     proj
    // }

    // For OpenGL:
    pub fn get_projection_matrix(&self) -> Mat4 {
        // Mat4::perspective_rh(self.v_fov, self.aspect_ratio, 0.5, 400.0)
        // @explore: try setting different clip planes every frame based on z-buffer (glReadPixels)?
        Mat4::perspective_infinite_rh(self.v_fov, self.aspect_ratio, 0.5)
    }
}
