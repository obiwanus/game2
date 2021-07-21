use egui::{CentralPanel, ClippedMesh, CtxRef, RawInput};
use epaint::Color32;
use memoffset::offset_of;

use crate::{
    opengl::buffers::{Buffer, VertexArray},
    texture::Texture,
};

pub struct Gui {
    window_width: f32,
    window_height: f32,

    ctx: CtxRef,
    egui_texture: Texture,
    egui_texture_version: Option<u64>,

    vao: VertexArray,
    vbo: Buffer,
    ebo: Buffer,
}

impl Gui {
    // Note: assuming non-resizable window for now
    pub fn new(window_width: f32, window_height: f32) -> Gui {
        let vao = VertexArray::new();
        vao.bind();
        let vbo = Buffer::new();
        vbo.bind_as(gl::ARRAY_BUFFER);
        let ebo = Buffer::new();
        ebo.bind_as(gl::ELEMENT_ARRAY_BUFFER);

        unsafe {
            use gl::types::*;
            // Position
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<Vertex>() as i32,
                offset_of!(Vertex, pos) as *const GLvoid,
            );
            gl::EnableVertexAttribArray(0);

            // UV
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<Vertex>() as i32,
                offset_of!(Vertex, uv) as *const GLvoid,
            );
            gl::EnableVertexAttribArray(1);

            // Color
            gl::VertexAttribPointer(
                2,
                4,
                gl::UNSIGNED_BYTE,
                gl::FALSE,
                std::mem::size_of::<Vertex>() as i32,
                offset_of!(Vertex, srgba) as *const GLvoid,
            );
            gl::EnableVertexAttribArray(2);
        }

        let egui_texture = Texture::new();
        unsafe {
            use gl::types::*;
            gl::BindTexture(gl::TEXTURE_2D, egui_texture.id);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        }

        Gui {
            window_width,
            window_height,

            ctx: CtxRef::default(),
            egui_texture,
            egui_texture_version: None,

            vao,
            vbo,
            ebo,
        }
    }

    pub fn draw(&mut self) {
        let gui_input = RawInput::default();
        self.ctx.begin_frame(gui_input);

        CentralPanel::default().show(&self.ctx, |ui| {
            ui.label("Hello world");
            if ui.button("Click me").clicked() {
                println!("hey!");
            }
        });

        let (output, shapes) = self.ctx.end_frame();
        // TODO: handle output

        let pixels_per_point = self.ctx.pixels_per_point();
        let texture = &self.ctx.texture();
        self.upload_egui_texture(texture);

        let clipped_meshes = self.ctx.tessellate(shapes);

        // TODO: bind texture

        for ClippedMesh(clip_rect, mesh) in clipped_meshes {
            // Upload vertices
            let vertices = mesh
                .vertices
                .iter()
                .map(|v| Vertex {
                    pos: [v.pos.x, v.pos.y],
                    uv: [v.uv.x, v.uv.y],
                    srgba: v.color.to_array(),
                })
                .collect::<Vec<Vertex>>();
            self.vbo.bind_as(gl::ARRAY_BUFFER);
            Buffer::send_stream_data(gl::ARRAY_BUFFER, &vertices);

            // Upload indices
            self.ebo.bind_as(gl::ELEMENT_ARRAY_BUFFER);
            Buffer::send_stream_data(gl::ELEMENT_ARRAY_BUFFER, &mesh.indices);

            // TODO: draw (blend, shaders etc)
        }
    }

    fn upload_egui_texture(&mut self, texture: &egui::Texture) {
        if self.egui_texture_version == Some(texture.version) {
            return; // No change
        }
        self.egui_texture_version = Some(texture.version);

        let pixels: Vec<Vec<(u8, u8, u8, u8)>> = texture
            .pixels
            .chunks(texture.width)
            .map(|row| {
                row.iter()
                    .map(|&a| Color32::from_white_alpha(a).to_tuple())
                    .collect()
            })
            .collect();

        unsafe {
            use gl::types::*;
            gl::BindTexture(gl::TEXTURE_2D, self.egui_texture.id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::SRGB8_ALPHA8 as GLint,
                texture.width as GLint,
                texture.height as GLint,
                0,
                gl::SRGB8_ALPHA8,
                gl::UNSIGNED_BYTE,
                pixels.as_ptr() as *const std::ffi::c_void,
            );
        }
    }
}

struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
    srgba: [u8; 4],
}

// ==================================== TODO =================================================

// [package]
// name = "egui"
// version = "0.1.0"
// edition = "2018"

// # See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

// [dependencies]
// egui_glium = ""
// glium = ""
// egui = ""

// ==================================== example =================================================

// //! Example how to use pure `egui_glium` without [`epi`].
// use glium::glutin;

// fn create_display(event_loop: &glutin::event_loop::EventLoop<()>) -> glium::Display {
//     let window_builder = glutin::window::WindowBuilder::new()
//         .with_resizable(true)
//         .with_inner_size(glutin::dpi::LogicalSize {
//             width: 800.0,
//             height: 600.0,
//         })
//         .with_title("egui_glium example");

//     let context_builder = glutin::ContextBuilder::new()
//         .with_depth_buffer(0)
//         .with_srgb(true)
//         .with_stencil_buffer(0)
//         .with_vsync(true);

//     glium::Display::new(window_builder, context_builder, event_loop).unwrap()
// }

// fn main() {
//     let event_loop = glutin::event_loop::EventLoop::with_user_event();
//     let display = create_display(&event_loop);

//     let mut egui = egui_glium::EguiGlium::new(&display);

//     event_loop.run(move |event, _, control_flow| {
//         let mut redraw = || {
//             egui.begin_frame(&display);

//             let mut quit = false;

//             egui::SidePanel::left("my_side_panel").show(egui.ctx(), |ui| {
//                 ui.heading("Водка водка!");
//                 if ui.button("Quit").clicked() {
//                     quit = true;
//                 }

//                 egui::ComboBox::from_label("Version")
//                     .width(350.0)
//                     .selected_text("foo")
//                     .show_ui(ui, |ui| {
//                         egui::CollapsingHeader::new("Dev")
//                             .default_open(true)
//                             .show(ui, |ui| {
//                                 ui.label("contents");
//                             });
//                     });
//             });

//             let (needs_repaint, shapes) = egui.end_frame(&display);

//             *control_flow = if quit {
//                 glutin::event_loop::ControlFlow::Exit
//             } else if needs_repaint {
//                 display.gl_window().window().request_redraw();
//                 glutin::event_loop::ControlFlow::Poll
//             } else {
//                 glutin::event_loop::ControlFlow::Wait
//             };

//             {
//                 use glium::Surface as _;
//                 let mut target = display.draw();

//                 let clear_color = egui::Rgba::from_rgb(0.1, 0.3, 0.2);
//                 target.clear_color(
//                     clear_color[0],
//                     clear_color[1],
//                     clear_color[2],
//                     clear_color[3],
//                 );

//                 // draw things behind egui here

//                 egui.paint(&display, &mut target, shapes);

//                 // draw things on top of egui here

//                 target.finish().unwrap();
//             }
//         };

//         match event {
//             // Platform-dependent event handlers to workaround a winit bug
//             // See: https://github.com/rust-windowing/winit/issues/987
//             // See: https://github.com/rust-windowing/winit/issues/1619
//             glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
//             glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),

//             glutin::event::Event::WindowEvent { event, .. } => {
//                 if egui.is_quit_event(&event) {
//                     *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
//                 }

//                 egui.on_event(&event);

//                 display.gl_window().window().request_redraw(); // TODO: ask egui if the events warrants a repaint instead
//             }

//             _ => (),
//         }
//     });
// }
