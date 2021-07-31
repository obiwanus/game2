#![allow(dead_code)]
#![allow(unused)]

extern crate gl as opengl_lib;

mod camera;
mod editor;
mod input;
mod opengl;
mod ray;
mod skybox;
mod terrain;
mod texture;
mod utils;

use std::convert::TryInto;
use std::error::Error;
use std::time::Instant;

use egui::{Event as GuiEvent, PointerButton, Pos2, RawInput as EguiInput, Rect};
use glam::{DVec2, Vec2, Vec3, Vec4};
use glutin::event::{
    DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode,
    WindowEvent,
};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{Api, GlProfile, GlRequest};
use glutin::{PossiblyCurrent, WindowedContext};

use camera::Camera;
use editor::gui::Gui;
use editor::Brush;
use input::{
    vec2_to_egui_pos2, vec2_to_egui_vec2, Event as InputEvent, Input, Modifiers, RawInput,
};
use opengl::buffers::Buffer;
use opengl::shader::Program;
use skybox::Skybox;
use terrain::Terrain;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

// ==================================== Main loop =================================================

fn main() {
    let event_loop = EventLoop::new();
    let mut game = Game::new(&event_loop).unwrap_or_else(|error| {
        eprintln!("{}", error);
        std::process::exit(1);
    });

    event_loop.run(move |event, _, control_flow| {
        if let Err(error) = game.process_event(event, control_flow) {
            eprint!("{}", error);
            std::process::exit(1);
        };
    });
}

// ==================================== Game ======================================================

struct DirectionalLight {
    color: Vec3,
    direction: Vec3,
}

struct Game {
    windowed_context: WindowedContext<PossiblyCurrent>,

    prev_raw_input: RawInput,
    raw_input: RawInput,

    gui_input: EguiInput,
    gui: Gui,

    camera: Camera,
    in_focus: bool,

    terrain: Terrain,
    skybox: Skybox,
}

impl Game {
    /// Creates a window and inits a new game
    fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        // Create window

        #[cfg(all(windows))]
        let window_builder = {
            let monitor = event_loop.primary_monitor().unwrap_or_else(|| {
                event_loop
                    .available_monitors()
                    .next()
                    .expect("Couldn't find monitor")
            });
            let inner_size = glutin::dpi::LogicalSize::new(
                monitor.size().width - 100,
                monitor.size().height - 100,
            );

            WindowBuilder::new()
                .with_title("Game 2")
                .with_resizable(false)
                .with_position(glutin::dpi::LogicalPosition::new(70, 10))
                .with_inner_size(inner_size)
        };

        #[cfg(not(windows))]
        let window_builder = WindowBuilder::new()
            .with_title("Game 2")
            .with_position(glutin::dpi::LogicalPosition::new(70, 10))
            // .with_fullscreen(Some(glutin::window::Fullscreen::Borderless(
            //     event_loop.primary_monitor(),
            // )))
            .with_inner_size(glutin::dpi::LogicalSize::new(1920, 1080))
            .with_resizable(false);

        let gl_request = GlRequest::Specific(Api::OpenGl, (3, 3));
        let gl_profile = GlProfile::Core;
        let windowed_context = glutin::ContextBuilder::new()
            .with_gl(gl_request)
            .with_gl_profile(gl_profile)
            .with_srgb(true)
            .with_double_buffer(Some(true))
            .with_depth_buffer(16)
            .with_vsync(true)
            .build_windowed(window_builder, &event_loop)?;

        // Set up OpenGL
        let windowed_context = unsafe { windowed_context.make_current().unwrap() };
        gl::load_with(|s| windowed_context.get_proc_address(s) as *const _);
        let window = windowed_context.window();
        // window.set_cursor_grab(true)?;
        // window.set_cursor_visible(false);
        let window_size = window.inner_size();
        unsafe {
            gl::Viewport(0, 0, window_size.width as i32, window_size.height as i32);
            gl::ClearColor(0.05, 0.05, 0.05, 1.0);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::FRAMEBUFFER_SRGB);

            #[cfg(not(target_os = "macos"))]
            {
                // MacOS deprecated OpenGL, which is stuck at 4.1 so no debug callbacks here :(
                gl::Enable(gl::DEBUG_OUTPUT);
                gl::DebugMessageCallback(Some(opengl::debug_callback), std::ptr::null());
            }
        }

        // // Directional light
        // let light_color = Vec3::new(1.0, 0.7, 0.7);
        // shader.set_vec3("directional_light.ambient", &(0.2f32 * light_color))?;
        // shader.set_vec3("directional_light.diffuse", &(0.5f32 * light_color))?;
        // shader.set_vec3("directional_light.specular", &(1.0f32 * light_color))?;

        // // Default terrain material
        // shader.set_vec3("material.specular", &Vec3::new(0.4, 0.4, 0.4))?;
        // shader.set_float("material.shininess", 10.0)?;

        // Set up camera
        let camera = Camera::new(
            Vec3::new(20.0, 20.0, 35.0),
            Vec3::new(0.0, 0.0, 0.0),
            window_size.width,
            window_size.height,
        );

        let terrain = Terrain::new(60.0, 120)?;

        let skybox = Skybox::from([
            "textures/skybox/right.jpg",
            "textures/skybox/left.jpg",
            "textures/skybox/top.jpg",
            "textures/skybox/bottom.jpg",
            "textures/skybox/front.jpg",
            "textures/skybox/back.jpg",
        ])?;

        let screen_size_physical = Vec2::new(window_size.width as f32, window_size.height as f32);
        let screen_size_logical = screen_size_physical / window.scale_factor() as f32;

        let now = Instant::now();
        let input = RawInput::new(Instant::now(), screen_size_logical, window.scale_factor());
        let prev_input = input.clone();

        // Gui and its initial input
        let gui = Gui::new(screen_size_physical)?;
        let gui_input = EguiInput {
            screen_rect: Some(Rect::from_min_max(
                Pos2::new(0.0, 0.0),
                Pos2::new(screen_size_logical.x, screen_size_logical.y),
            )),
            pixels_per_point: Some(window.scale_factor() as f32),
            time: Some(0.0),

            ..Default::default()
        };

        Ok(Game {
            windowed_context,

            raw_input: input,
            prev_raw_input: prev_input,

            gui_input,
            gui,

            camera,
            in_focus: true,

            terrain,
            skybox,
        })
    }

    fn process_event(&mut self, event: Event<()>, control_flow: &mut ControlFlow) -> Result<()> {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::ScaleFactorChanged {
                    scale_factor,
                    new_inner_size: _,
                } => {
                    self.raw_input.scale_factor = scale_factor;
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let pointer = Vec2::new(position.x as f32, position.y as f32);
                    self.raw_input.pointer_pos = pointer;
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    let pressed = state == ElementState::Pressed;
                    self.raw_input.events.push(InputEvent::MouseButtonPressed {
                        pos: self.raw_input.pointer_pos,
                        button: match button {
                            MouseButton::Left => input::MouseButton::Primary,
                            MouseButton::Right => input::MouseButton::Secondary,
                            MouseButton::Middle => input::MouseButton::Middle,
                            _ => input::MouseButton::Unknown,
                        },
                        pressed,
                        modifiers: self.raw_input.modifiers,
                    });
                }
                WindowEvent::Focused(focused) => {
                    self.in_focus = focused;
                    // Try using Poll here?
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(virtual_key_code),
                            ..
                        },
                    ..
                } => {
                    let key: input::Key = virtual_key_code.into();
                    if key != input::Key::Unknown {
                        self.raw_input.events.push(InputEvent::Key {
                            key,
                            pressed: state == ElementState::Pressed,
                            modifiers: self.raw_input.modifiers,
                        });
                    }
                }
                WindowEvent::ModifiersChanged(state) => {
                    self.raw_input.modifiers = Modifiers {
                        alt: state.alt(),
                        ctrl: state.ctrl(),
                        shift: state.shift(),
                        logo: state.logo(),
                    };
                }
                _ => {}
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } if self.in_focus => {
                    let (x, y) = delta;
                    self.raw_input.pointer_delta = DVec2::new(x, y);
                }
                DeviceEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(x, y),
                } => {
                    // TODO:
                    // self.terrain.brush.size = (self.terrain.brush.size - y * 0.5).clamp(0.1, 20.0);

                    self.raw_input.scroll_delta = Vec2::new(x, y);
                }
                _ => {}
            },
            Event::MainEventsCleared => self.update_and_render()?,
            _ => {}
        };
        Ok(())
    }

    fn derive_gui_input(input: &RawInput, prev_input: &RawInput) -> EguiInput {
        EguiInput {
            scroll_delta: vec2_to_egui_vec2(input.scroll_delta),
            screen_rect: if input.screen_size != prev_input.screen_size {
                Some(egui::Rect {
                    min: Default::default(),
                    max: vec2_to_egui_pos2(input.screen_size),
                })
            } else {
                None
            },
            pixels_per_point: if (input.scale_factor - prev_input.scale_factor).abs() < f64::EPSILON
            {
                Some(input.scale_factor as f32)
            } else {
                None
            },
            time: Some(
                input
                    .frame_start
                    .duration_since(input.game_start)
                    .as_secs_f64(),
            ),
            modifiers: input.modifiers.into(),
            events: input
                .events
                .iter()
                .filter_map(|e| e.try_into().ok())
                .collect(),
            ..Default::default()
        }
    }

    fn derive_game_input(&self, raw_input: &RawInput, prev_raw_input: &RawInput) -> Input {
        let mut input = Input {
            pointer: raw_input.pointer_pos,
            ..Default::default()
        };

        for event in self.raw_input.events.iter() {
            match *event {
                InputEvent::Key {
                    key,
                    pressed,
                    modifiers: _,
                } => {
                    use input::Key::*;
                    match key {
                        W => input.forward = pressed,
                        A => input.left = pressed,
                        S => input.back = pressed,
                        D => input.right = pressed,
                        Escape => input.should_exit = true,
                        _ => {}
                    }
                }
                InputEvent::MouseButtonPressed {
                    pos,
                    button,
                    pressed,
                    modifiers: _,
                } => {
                    use input::MouseButton::*;
                    match button {
                        Primary => input.left_mouse_button_pressed = pressed,
                        Secondary => input.wasd_mode = pressed,
                        _ => {}
                    }
                }
                InputEvent::PointerMoved(pos) => {
                    input.pointer_moved = true;
                    input.pointer = pos;
                }
                _ => {}
            }
        }

        input
    }

    fn update_and_render(&mut self) -> Result<()> {
        let input = self.derive_game_input(&self.raw_input, &self.prev_raw_input);
        let delta_time = self.raw_input.delta_time;

        // Move camera
        if input.wasd_mode {
            use camera::Movement::*;
            if input.forward {
                self.camera.go(Forward, delta_time);
            }
            if input.left {
                self.camera.go(Left, delta_time);
            }
            if input.back {
                self.camera.go(Backward, delta_time);
            }
            if input.right {
                self.camera.go(Right, delta_time);
            }
        }

        if input.pointer_moved || self.camera.moved {
            let cursor = {
                let ray = self.camera.get_ray_through_pixel(input.pointer);
                let mut hit = f32::INFINITY;
                for (a, b, c) in self.terrain.triangles() {
                    let new_hit = ray.hits_triangle(a, b, c);
                    if new_hit.t < hit {
                        hit = new_hit.t;
                    }
                }
                self.windowed_context
                    .window()
                    .set_cursor_visible(hit == f32::INFINITY);
                ray.get_point_at(hit)
            };

            self.terrain.cursor = cursor;
        }

        // Shape the terrain
        if input.left_mouse_button_pressed {
            let brush_size_squared = self.terrain.brush.size * self.terrain.brush.size;
            for v in self.terrain.vertices.iter_mut() {
                let dist_sq = (v.pos - self.terrain.cursor).length_squared();
                if dist_sq < brush_size_squared {
                    v.pos.y += 5.0 * delta_time;
                }
            }
            self.terrain.send_vertex_buffer();
        }

        // Draw
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        self.terrain.draw(&self.camera)?;
        self.skybox.draw(&self.camera)?; // draw skybox last

        self.gui_input = Game::derive_gui_input(&self.raw_input, &self.prev_raw_input);
        self.gui.interact_and_draw(self.gui_input.clone());

        self.windowed_context.swap_buffers()?;

        // Clear input and save as old
        self.prev_raw_input = self.raw_input.renew();

        if self.camera.moved {
            // TODO: move to input and clear automatically
            self.camera.moved = false;
        }

        Ok(())
    }
}
