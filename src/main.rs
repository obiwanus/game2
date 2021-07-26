#![allow(dead_code)]
#![allow(unused)]

extern crate gl as opengl_lib;

mod camera;
mod editor;
mod opengl;
mod ray;
mod skybox;
mod terrain;
mod texture;
mod utils;

use std::error::Error;
use std::time::Instant;

use egui::{Event as GuiEvent, Modifiers, PointerButton, Pos2, RawInput, Rect};
use glam::{Vec2, Vec3, Vec4};
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
use opengl::buffers::Buffer;
use opengl::shader::Program;
use skybox::Skybox;
use terrain::Terrain;

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

#[derive(Default)]
struct Input {
    forward: bool,
    back: bool,
    left: bool,
    right: bool,

    pointer: Vec2,
    pointer_moved: bool,
    left_mouse_button_pressed: bool,
    brush_size_changed: bool,

    wasd_mode: bool,
}

struct DirectionalLight {
    color: Vec3,
    direction: Vec3,
}

struct Game {
    game_start: Instant,
    frame_start: Instant,

    windowed_context: WindowedContext<PossiblyCurrent>,
    input: Input,
    gui_input: RawInput,
    camera: Camera,
    in_focus: bool,
    gui: Gui,

    shader: Program,
    terrain: Terrain,
    skybox: Skybox,

    // tmp
    brush: Brush,
}

impl Game {
    /// Creates a window and inits a new game
    fn new(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn Error>> {
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
            .with_resizable(false)
            .with_position(glutin::dpi::LogicalPosition::new(70, 10))
            .with_fullscreen(Some(glutin::window::Fullscreen::Borderless(
                event_loop.primary_monitor(),
            )));

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

            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(Some(opengl::debug_callback), std::ptr::null());
        }

        let shader = Program::new()
            .vertex_shader("shaders/basic/basic.vert")?
            .fragment_shader("shaders/basic/basic.frag")?
            .link()?;
        shader.set_used();

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

        let terrain = Terrain::new(60.0, 120);

        let brush = Brush::new("src/editor/brushes/brush1.png");
        shader.set_float("brush_size", brush.size)?;

        let skybox = Skybox::from([
            "textures/skybox/right.jpg",
            "textures/skybox/left.jpg",
            "textures/skybox/top.jpg",
            "textures/skybox/bottom.jpg",
            "textures/skybox/front.jpg",
            "textures/skybox/back.jpg",
        ])?;

        let screen_size_physical = Vec2::new(window_size.width as f32, window_size.height as f32);
        let gui = Gui::new(screen_size_physical)?;

        // Set some initial parameters which will then be used every frame
        let gui_input = {
            let screen_size_logical = screen_size_physical / window.scale_factor() as f32;
            RawInput {
                screen_rect: Some(Rect::from_min_max(
                    Pos2::new(0.0, 0.0),
                    Pos2::new(screen_size_logical.x, screen_size_logical.y),
                )),
                pixels_per_point: Some(window.scale_factor() as f32),
                time: Some(0.0),

                ..Default::default()
            }
        };

        Ok(Game {
            game_start: Instant::now(),
            frame_start: Instant::now(),

            windowed_context,
            input: Input::default(),
            gui_input,
            camera,
            in_focus: true,
            gui,

            shader,
            terrain,
            skybox,

            brush,
        })
    }

    fn process_event(
        &mut self,
        event: Event<()>,
        control_flow: &mut ControlFlow,
    ) -> Result<(), Box<dyn Error>> {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::ScaleFactorChanged {
                    scale_factor,
                    new_inner_size: _,
                } => {
                    self.gui_input.pixels_per_point = Some(scale_factor as f32);
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    let pressed = state == ElementState::Pressed;

                    if button == MouseButton::Left {
                        self.input.left_mouse_button_pressed = pressed;
                    } else if button == MouseButton::Right {
                        self.input.wasd_mode = pressed;
                    }

                    let pointer_button = match button {
                        MouseButton::Left => Some(PointerButton::Primary),
                        MouseButton::Middle => Some(PointerButton::Middle),
                        MouseButton::Right => Some(PointerButton::Secondary),
                        _ => None,
                    };
                    if let Some(button) = pointer_button {
                        self.gui_input.events.push(GuiEvent::PointerButton {
                            pos: Pos2::new(self.input.pointer.x, self.input.pointer.y),
                            button,
                            pressed,
                            modifiers: self.gui_input.modifiers,
                        });
                    }
                }
                WindowEvent::Focused(focused) => {
                    self.in_focus = focused;
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                } => {
                    // Key press
                    match key {
                        VirtualKeyCode::W => self.input.forward = state == ElementState::Pressed,
                        VirtualKeyCode::A => self.input.left = state == ElementState::Pressed,
                        VirtualKeyCode::S => self.input.back = state == ElementState::Pressed,
                        VirtualKeyCode::D => self.input.right = state == ElementState::Pressed,
                        VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
                WindowEvent::ModifiersChanged(state) => {
                    self.camera.speed_boost = state.shift();
                    self.gui_input.modifiers = Modifiers {
                        alt: state.alt(),
                        ctrl: state.ctrl(),
                        shift: state.shift(),
                        command: state.ctrl(),
                        mac_cmd: false, // must be false unless on Mac
                    };
                    #[cfg(target_os = "macos")]
                    {
                        self.gui_input.modifiers.command = state.logo();
                        self.gui_input.modifiers.mac_cmd = state.logo();
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let pointer = Vec2::new(position.x as f32, position.y as f32);
                    self.input.pointer = pointer;
                    self.input.pointer_moved = true;

                    self.gui_input
                        .events
                        .push(GuiEvent::PointerMoved(Pos2::new(pointer.x, pointer.y)))
                }
                _ => {}
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } if self.in_focus && self.input.wasd_mode => {
                    let (yaw_delta, pitch_delta) = delta;
                    self.camera.rotate(yaw_delta, pitch_delta);
                }
                DeviceEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(x, y),
                } => {
                    self.brush.size = (self.brush.size - y * 0.5).clamp(0.1, 20.0);
                    self.input.brush_size_changed = true;

                    self.gui_input.scroll_delta = egui::emath::vec2(x, y);
                }
                _ => {}
            },
            Event::MainEventsCleared => self.update_and_render()?,
            _ => {}
        };
        Ok(())
    }

    fn update_and_render(&mut self) -> Result<(), Box<dyn Error>> {
        // Application code
        let now = Instant::now();
        let delta_time = now.duration_since(self.frame_start).as_secs_f32();
        self.frame_start = now;

        // For GUI animations
        self.gui_input.time = Some(now.duration_since(self.game_start).as_secs_f64());

        // Move camera
        if self.input.wasd_mode {
            use camera::Movement::*;
            if self.input.forward {
                self.camera.go(Forward, delta_time);
            }
            if self.input.left {
                self.camera.go(Left, delta_time);
            }
            if self.input.back {
                self.camera.go(Backward, delta_time);
            }
            if self.input.right {
                self.camera.go(Right, delta_time);
            }
        }

        let proj = self.camera.get_projection_matrix();
        let view = self.camera.get_view_matrix();

        self.shader.set_used();

        // let light_dir = view * Vec4::new(0.37f32, -0.56, 0.75, 0.0);
        // let light_dir: Vec3 = light_dir.into();

        // self.shader
        //     .set_vec3("directional_light.direction", &light_dir)?;

        if self.input.brush_size_changed {
            self.shader.set_float("brush_size", self.brush.size)?;
        }

        if self.camera.moved {
            self.shader.set_mat4("proj", &proj)?;
            self.shader.set_mat4("view", &view)?;
        }

        if self.input.pointer_moved || self.camera.moved {
            self.input.pointer_moved = false;

            let cursor = {
                let ray = self.camera.get_ray_through_pixel(self.input.pointer);
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

            self.shader.set_vec3("cursor", &cursor)?;
            self.terrain.cursor = cursor;
        }

        // Shape the terrain
        if self.input.left_mouse_button_pressed {
            let brush_size_squared = self.brush.size * self.brush.size;
            for v in self.terrain.vertices.iter_mut() {
                let dist_sq = (v.pos - self.terrain.cursor).length_squared();
                if dist_sq < brush_size_squared {
                    v.pos.y += 5.0 * delta_time;
                }
            }
            self.terrain.send_vertex_buffer();
        }

        if self.camera.moved {
            // should we include it into input?
            self.camera.moved = false;
        }

        // Draw
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        self.terrain.draw();
        self.skybox.draw(&proj, &view)?; // draw skybox last

        self.gui.interact_and_draw(self.gui_input.clone());
        self.gui_input = RawInput::default();

        self.windowed_context.swap_buffers()?;

        #[cfg(feature = "debug")]
        {
            // Display frame time
            let frame_ms =
                Instant::now().duration_since(self.frame_start).as_micros() as f32 / 1000.0;
            println!("frame: {} ms", frame_ms);
        }

        Ok(())
    }
}
