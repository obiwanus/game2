use std::mem::size_of;

use egui::{Align2, ClippedMesh, CtxRef, RawInput};
use egui_gizmo::{Gizmo, GizmoMode, GizmoOrientation, GizmoVisuals};
use epaint::Color32;
use gl::types::*;
use glam::{Mat4, Vec2};
use memoffset::offset_of;

use crate::{opengl::shader::Program, texture::unit_to_gl_const, utils::size_of_slice, Result};

/// An action to take as a result of interacting with the GUI
pub enum Action {
    SaveTerrain,
    SaveCamera,
    Quit,
}

pub struct Gui {
    screen_size: Vec2,

    ctx: CtxRef,
    egui_texture: GLuint,
    egui_texture_version: Option<u64>,

    shader: Program,

    // OpenGL buffers
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
    vertex_buffer_size: usize,
    index_buffer_size: usize,
    index_count: i32,
}

impl Gui {
    // Note: assuming non-resizable window for now
    pub fn new(screen_size: Vec2) -> Result<Gui> {
        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;
        let mut ebo: GLuint = 0;

        // Initial size
        let vertex_buffer_size = 1024 * 1024;
        let index_buffer_size = 1024 * 1024;

        unsafe {
            // Create objects
            gl::CreateVertexArrays(1, &mut vao);
            gl::CreateBuffers(1, &mut vbo);
            gl::CreateBuffers(1, &mut ebo);

            // Attach buffers to vao
            gl::VertexArrayVertexBuffer(vao, 0, vbo, 0, size_of::<Vertex>() as i32);
            gl::VertexArrayElementBuffer(vao, ebo);

            // Allocate some initial storage for the buffers with the hope that
            // it won't have to reallocate often
            gl::NamedBufferStorage(
                vbo,
                vertex_buffer_size as isize,
                std::ptr::null(),
                gl::DYNAMIC_STORAGE_BIT,
            );
            gl::NamedBufferStorage(
                ebo,
                index_buffer_size as isize,
                std::ptr::null(),
                gl::DYNAMIC_STORAGE_BIT,
            );

            // Position
            gl::VertexArrayAttribFormat(
                vao,
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                offset_of!(Vertex, pos) as u32,
            );

            // UV
            gl::VertexArrayAttribFormat(
                vao,
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                offset_of!(Vertex, uv) as u32,
            );

            // Color
            gl::VertexArrayAttribFormat(
                vao,
                2,
                4,
                gl::UNSIGNED_BYTE,
                gl::FALSE,
                offset_of!(Vertex, srgba) as u32,
            );

            gl::EnableVertexArrayAttrib(vao, 0);
            gl::EnableVertexArrayAttrib(vao, 1);
            gl::EnableVertexArrayAttrib(vao, 2);

            gl::VertexArrayAttribBinding(vao, 0, 0);
            gl::VertexArrayAttribBinding(vao, 1, 0);
            gl::VertexArrayAttribBinding(vao, 2, 0);
        }

        let shader = Program::new()
            .vertex_shader(include_str!("../shaders/editor/gui.vert"))?
            .fragment_shader(include_str!("../shaders/editor/gui.frag"))?
            .link()?;

        Ok(Gui {
            screen_size,

            ctx: CtxRef::default(),
            egui_texture: 0, // will be created before draw
            egui_texture_version: None,

            shader,

            vao,
            vbo,
            ebo,
            vertex_buffer_size,
            index_buffer_size,
            index_count: 0,
        })
    }

    pub fn ctx(&self) -> &CtxRef {
        &self.ctx
    }

    pub fn wants_input(&self) -> bool {
        self.ctx.wants_pointer_input() || self.ctx.wants_keyboard_input()
    }

    pub fn layout_and_interact(
        &mut self,
        input: RawInput,
        drag: bool,
        view_matrix: &Mat4,
        projection_matrix: &Mat4,
        model_matrix: &mut Mat4,
    ) -> Vec<Action> {
        self.ctx.begin_frame(input);
        let mut actions = vec![];

        // ================== GUI starts ========================

        egui::Window::new("Tools")
            .anchor(Align2::RIGHT_TOP, egui::Vec2::new(-10.0, 10.0))
            .resizable(false)
            .show(&self.ctx, |ui| {
                if ui.button("Save terrain").clicked() {
                    actions.push(Action::SaveTerrain);
                }

                if ui.button("Save camera position").clicked() {
                    actions.push(Action::SaveCamera);
                }
            });

        egui::Area::new("Viewport")
            .fixed_pos((0.0, 0.0))
            .show(&self.ctx, |ui| {
                let visuals = GizmoVisuals {
                    gizmo_size: 100.0,
                    ..Default::default()
                };
                let gizmo = Gizmo::new("gizmo")
                    .view_matrix(view_matrix.to_cols_array_2d())
                    .projection_matrix(projection_matrix.to_cols_array_2d())
                    .model_matrix(model_matrix.to_cols_array_2d())
                    .mode(GizmoMode::Translate)
                    .orientation(GizmoOrientation::Global)
                    .visuals(visuals);

                if drag {
                    let a = 1;
                }

                if let Some(gizmo_result) = gizmo.interact(ui) {
                    *model_matrix = Mat4::from_cols_array_2d(&gizmo_result.transform);
                }
            });

        // ================== GUI ends ===========================

        // TODO: handle output
        let (_output, shapes) = self.ctx.end_frame();

        // Send meshes and texture to GPU
        self.upload_egui_texture();

        let clipped_meshes = self.ctx.tessellate(shapes);

        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut vertex_count = 0;

        for ClippedMesh(_clip_rect, mesh) in clipped_meshes {
            vertices.extend(mesh.vertices.iter().map(|v| Vertex {
                pos: [v.pos.x, v.pos.y],
                uv: [v.uv.x, v.uv.y],
                srgba: v.color.to_array(),
            }));
            indices.extend(mesh.indices.iter().map(|&i| i + vertex_count));
            vertex_count = vertices.len() as u32;
        }
        self.index_count = indices.len() as i32;

        // Fill vertex buffer with data, reallocating if necessary
        let required_size = size_of_slice(&vertices);
        if self.vertex_buffer_size < required_size {
            unsafe {
                gl::DeleteBuffers(1, &self.vbo);
                gl::CreateBuffers(1, &mut self.vbo);
                gl::VertexArrayVertexBuffer(self.vao, 0, self.vbo, 0, size_of::<Vertex>() as i32);
                gl::NamedBufferStorage(
                    self.vbo,
                    required_size as isize,
                    vertices.as_ptr() as *const _,
                    gl::DYNAMIC_STORAGE_BIT,
                );
            }
            self.vertex_buffer_size = required_size;
            println!("Reallocating vertex buffer to {}", required_size);
        } else {
            unsafe {
                gl::NamedBufferSubData(
                    self.vbo,
                    0,
                    required_size as isize,
                    vertices.as_ptr() as *const _,
                )
            }
        }

        // Fill index buffer with data, reallocating if necessary
        let required_size = size_of_slice(&indices);
        if self.index_buffer_size < required_size {
            unsafe {
                gl::DeleteBuffers(1, &self.ebo);
                gl::CreateBuffers(1, &mut self.ebo);
                gl::VertexArrayElementBuffer(self.vao, self.ebo);
                gl::NamedBufferStorage(
                    self.ebo,
                    required_size as isize,
                    indices.as_ptr() as *const _,
                    gl::DYNAMIC_STORAGE_BIT,
                );
            }
            self.index_buffer_size = required_size;
            println!("Reallocating index buffer to {}", required_size);
        } else {
            unsafe {
                gl::NamedBufferSubData(
                    self.ebo,
                    0,
                    required_size as isize,
                    indices.as_ptr() as *const _,
                )
            }
        }

        actions
    }

    pub fn draw(&mut self) {
        let pixels_per_point = self.ctx.pixels_per_point();
        let screen_size_in_points = self.screen_size / pixels_per_point;

        self.shader.set_used();
        self.shader
            .set_vec2("u_screen_size", &screen_size_in_points)
            .unwrap();
        unsafe {
            gl::ActiveTexture(unit_to_gl_const(0));
            gl::BindTexture(gl::TEXTURE_2D, self.egui_texture);

            gl::BindVertexArray(self.vao);
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::CULL_FACE);
            gl::Enable(gl::BLEND);
            gl::BlendFuncSeparate(
                gl::ONE,
                gl::ONE_MINUS_SRC_ALPHA,
                gl::ONE_MINUS_DST_ALPHA,
                gl::ONE,
            );

            gl::DrawElements(
                gl::TRIANGLES,
                self.index_count,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );

            gl::Disable(gl::BLEND);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
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

        let mut new_texture: GLuint = 0;
        unsafe {
            if self.egui_texture != 0 {
                // Delete old texture
                gl::DeleteTextures(1, &self.egui_texture);
            }
            gl::CreateTextures(gl::TEXTURE_2D, 1, &mut new_texture);
            gl::TextureParameteri(new_texture, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl::TextureParameteri(new_texture, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
            gl::TextureParameteri(new_texture, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TextureParameteri(new_texture, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl::TextureStorage2D(
                new_texture,
                1,
                gl::SRGB8_ALPHA8,
                texture.width as GLint,
                texture.height as GLint,
            );
            gl::TextureSubImage2D(
                new_texture,
                0,
                0,
                0,
                texture.width as i32,
                texture.height as i32,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixels.as_ptr() as *const _,
            );
        }
        self.egui_texture = new_texture;
    }
}

impl Drop for Gui {
    fn drop(&mut self) {
        unsafe {
            let buffers = [self.vbo, self.ebo];
            gl::DeleteBuffers(1, buffers.as_ptr());

            let arrays = [self.vao];
            gl::DeleteVertexArrays(1, arrays.as_ptr());
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
