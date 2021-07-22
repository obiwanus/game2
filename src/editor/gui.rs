use std::error::Error;

use egui::{CentralPanel, ClippedMesh, CtxRef, RawInput};
use epaint::Color32;
use memoffset::offset_of;

use crate::{
    opengl::{
        buffers::{Buffer, VertexArray},
        shader::Program,
    },
    texture::Texture,
};

pub struct Gui {
    window_width: f32,
    window_height: f32,

    ctx: CtxRef,
    egui_texture: Texture,
    egui_texture_version: Option<u64>,

    shader: Program,
    vao: VertexArray,
    vbo: Buffer,
    ebo: Buffer,
}

impl Gui {
    // Note: assuming non-resizable window for now
    pub fn new(window_width: f32, window_height: f32) -> Result<Gui, Box<dyn Error>> {
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

        let shader = Program::new()
            .vertex_shader("shaders/editor/gui.vert")?
            .fragment_shader("shaders/editor/gui.frag")?
            .link()?;
        shader.set_used();
        shader.se

        Ok(Gui {
            window_width,
            window_height,

            ctx: CtxRef::default(),
            egui_texture,
            egui_texture_version: None,

            shader,

            vao,
            vbo,
            ebo,
        })
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
