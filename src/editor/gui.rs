use std::error::Error;

use egui::{ClippedMesh, CtxRef, RawInput};
use epaint::Color32;
use glam::Vec2;
use memoffset::offset_of;

use crate::{
    opengl::{
        buffers::{Buffer, VertexArray},
        shader::Program,
    },
    texture::Texture,
    Input,
};

pub struct Gui {
    screen_size: Vec2,

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
    pub fn new(screen_size: Vec2) -> Result<Gui, Box<dyn Error>> {
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

        Ok(Gui {
            screen_size,

            ctx: CtxRef::default(),
            egui_texture,
            egui_texture_version: None,

            shader,

            vao,
            vbo,
            ebo,
        })
    }

    pub fn interact_and_draw(&mut self, game_input: &Input) {
        self.ctx.begin_frame(game_input.into());

        // ================== GUI starts ========================
        egui::SidePanel::left("my_side_panel").show(&self.ctx, |ui| {
            ui.heading("Водка водка!");
            if ui.button("Quit").clicked() {
                println!("Quit clicked");
            }

            egui::ComboBox::from_label("Version")
                .width(350.0)
                .selected_text("foo")
                .show_ui(ui, |ui| {
                    egui::CollapsingHeader::new("Dev")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.label("contents");
                        });
                });
        });
        // ================== GUI ends ===========================

        let (output, shapes) = self.ctx.end_frame();
        // TODO: handle output

        let pixels_per_point = self.ctx.pixels_per_point();
        let screen_size_in_points = self.screen_size / pixels_per_point;

        self.upload_egui_texture();

        let clipped_meshes = self.ctx.tessellate(shapes);

        self.vao.bind();
        self.shader.set_used();
        self.shader
            .set_vec2("u_screen_size", &screen_size_in_points)
            .unwrap();
        self.shader.set_texture_unit("u_sampler", 0).unwrap();
        self.egui_texture.bind_2d(0);

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

            unsafe {
                gl::Disable(gl::DEPTH_TEST);
                gl::Enable(gl::BLEND);
                gl::BlendEquation(gl::FUNC_ADD);
                gl::BlendFuncSeparate(
                    gl::ONE,
                    gl::ONE_MINUS_SRC_ALPHA,
                    gl::ONE_MINUS_DST_ALPHA,
                    gl::ONE,
                );
                gl::DrawElements(
                    gl::TRIANGLES,
                    mesh.indices.len() as i32,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                );
                gl::Disable(gl::BLEND);
                gl::Enable(gl::DEPTH_TEST);
            }
        }
    }

    fn upload_egui_texture(&mut self) {
        let texture = self.ctx.texture();
        if self.egui_texture_version == Some(texture.version) {
            return; // No change
        }
        self.egui_texture_version = Some(texture.version);

        let pixels: Vec<(u8, u8, u8, u8)> = texture
            .pixels
            .iter()
            .map(|&a| Color32::from_white_alpha(a).to_tuple())
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
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixels.as_ptr() as *const std::ffi::c_void,
            );
        }
    }
}

#[derive(Debug)]
#[repr(C)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
    srgba: [u8; 4],
}

impl From<&Input> for RawInput {
    fn from(input: &Input) -> Self {
        RawInput::default()
    }
}
