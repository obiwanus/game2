extern crate gl as opengl_lib;

mod camera;
mod opengl;
mod ray;
mod skybox;
mod terrain;
mod texture;
mod utils;

use std::error::Error;
use std::time::Instant;

use glam::{Vec2, Vec3};
use glutin::event::{
    DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent,
};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{Api, GlProfile, GlRequest};
use glutin::{PossiblyCurrent, WindowedContext};

use camera::Camera;
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
        if let Err(error) = game.main_loop(event, control_flow) {
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

    cursor: Vec2,
    cursor_moved: bool,

    wasd_mode: bool,
}

struct Game {
    windowed_context: WindowedContext<PossiblyCurrent>,
    input: Input,
    camera: Camera,
    in_focus: bool,
    frame_start: Instant,

    shader: Program,
    terrain: Terrain,
    skybox: Skybox,
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
        }

        let shader = Program::new()
            .vertex_shader("shaders/basic/basic.vert")?
            .fragment_shader("shaders/basic/basic.frag")?
            .link()?;
        shader.set_used();

        // Set up camera
        let camera = Camera::new(
            Vec3::new(0.0, 5.0, 30.0),
            Vec3::new(0.0, 0.0, 0.0),
            window_size.width,
            window_size.height,
        );

        let terrain = Terrain::new(40.0, 40);

        let skybox = Skybox::from([
            "textures/skybox/right.jpg",
            "textures/skybox/left.jpg",
            "textures/skybox/top.jpg",
            "textures/skybox/bottom.jpg",
            "textures/skybox/front.jpg",
            "textures/skybox/back.jpg",
        ])?;

        Ok(Game {
            windowed_context,
            input: Input::default(),
            camera,
            in_focus: true,
            frame_start: Instant::now(),

            shader,
            terrain,
            skybox,
        })
    }

    fn main_loop(
        &mut self,
        event: Event<()>,
        control_flow: &mut ControlFlow,
    ) -> Result<(), Box<dyn Error>> {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state: ElementState::Pressed,
                    ..
                } => {
                    // Mouse button click
                }
                WindowEvent::MouseInput {
                    button: MouseButton::Right,
                    state,
                    ..
                } => self.input.wasd_mode = state == ElementState::Pressed,
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
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.input.cursor = Vec2::new(position.x as f32, position.y as f32);
                    self.input.cursor_moved = true;
                }
                _ => {}
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } if self.in_focus && self.input.wasd_mode => {
                    let (yaw_delta, pitch_delta) = delta;
                    self.camera.rotate(yaw_delta, pitch_delta);
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

        if self.input.cursor_moved || self.camera.moved {
            self.input.cursor_moved = false;

            let highlighted_triangle = {
                let ray = self.camera.get_ray_through_pixel(self.input.cursor);
                let mut previous_hit = f32::INFINITY;
                let mut triangle_index = None;
                for (index, a, b, c) in self.terrain.triangles() {
                    let hit = ray.hits_triangle(a, b, c);
                    if hit.t < previous_hit {
                        previous_hit = hit.t;
                        triangle_index = Some(index);
                    }
                }
                triangle_index
            };

            self.terrain.highlighted_triangle = highlighted_triangle;
        }

        let proj = self.camera.get_projection_matrix();
        let view = self.camera.get_view_matrix();

        self.shader.set_used();
        self.shader.set_mat4("proj", &proj)?;
        self.shader.set_mat4("view", &view)?;

        // Draw
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        self.terrain.draw();

        self.skybox.draw(&proj, &view)?; // draw skybox last

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
