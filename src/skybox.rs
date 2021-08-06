use gl::types::*;
use gltf::json::image;
use thiserror::Error;

use crate::camera::Camera;
use crate::opengl::buffers::{Buffer, VertexArray};
use crate::opengl::shader::{Program, ShaderError};
use crate::texture::{load_image, TextureError};

#[derive(Debug, Error)]
pub enum SkyboxError {
    #[error("Skybox texture error: {0}")]
    Texture(#[from] TextureError),
    #[error("Skybox shader error: {0}")]
    Shader(#[from] ShaderError),
}

pub struct Skybox {
    texture_id: GLuint,
    shader: Program,
    vao: VertexArray,
    _vbo: Buffer,
}

impl Skybox {
    /// right, left, top, bottom, front, back
    pub fn from(path: &str) -> Result<Self, SkyboxError> {
        // Generate texture
        let image = load_image(path, true)?;
        debug_assert_eq!(image.height, 1, "Only 1D textures are supported");

        let mut texture_id: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_1D, texture_id);
            gl::TexParameteri(
                gl::TEXTURE_1D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_1D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(gl::TEXTURE_1D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_1D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);

            // Send to GPU
            gl::TexImage1D(
                gl::TEXTURE_1D,
                0,
                gl::SRGB8 as GLint,
                image.width as GLint,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                image.data.as_ptr() as *const std::ffi::c_void,
            );
        }

        // Create shader
        let shader = Program::new()
            .vertex_shader(include_str!("shaders/skybox/skybox.vert"))?
            .fragment_shader(include_str!("shaders/skybox/skybox.frag"))?
            .link()?;
        shader.set_used();
        shader.set_texture_unit("SkyTexture", 0)?;

        #[rustfmt::skip]
        let vertices = [
            // x, y, z
            -1.0f32,  1.0, -1.0,
            -1.0, -1.0, -1.0,
            1.0, -1.0, -1.0,
            1.0, -1.0, -1.0,
            1.0,  1.0, -1.0,
            -1.0,  1.0, -1.0,

            -1.0, -1.0,  1.0,
            -1.0, -1.0, -1.0,
            -1.0,  1.0, -1.0,
            -1.0,  1.0, -1.0,
            -1.0,  1.0,  1.0,
            -1.0, -1.0,  1.0,

            1.0, -1.0, -1.0,
            1.0, -1.0,  1.0,
            1.0,  1.0,  1.0,
            1.0,  1.0,  1.0,
            1.0,  1.0, -1.0,
            1.0, -1.0, -1.0,

            -1.0, -1.0,  1.0,
            -1.0,  1.0,  1.0,
            1.0,  1.0,  1.0,
            1.0,  1.0,  1.0,
            1.0, -1.0,  1.0,
            -1.0, -1.0,  1.0,

            -1.0,  1.0, -1.0,
            1.0,  1.0, -1.0,
            1.0,  1.0,  1.0,
            1.0,  1.0,  1.0,
            -1.0,  1.0,  1.0,
            -1.0,  1.0, -1.0,

            -1.0, -1.0, -1.0,
            -1.0, -1.0,  1.0,
            1.0, -1.0, -1.0,
            1.0, -1.0, -1.0,
            -1.0, -1.0,  1.0,
            1.0, -1.0,  1.0,
        ];
        let vao = VertexArray::new();
        vao.bind();
        let vbo = Buffer::new();
        vbo.bind_as(gl::ARRAY_BUFFER);
        Buffer::send_static_data(gl::ARRAY_BUFFER, &vertices);
        unsafe {
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                0,
                std::ptr::null() as *const GLvoid,
            );
            gl::EnableVertexAttribArray(0);
        }
        vao.unbind();

        Ok(Skybox {
            texture_id,
            shader,
            vao,
            _vbo: vbo,
        })
    }

    pub fn draw(&self, camera: &Camera, camera_moved: bool) -> Result<(), SkyboxError> {
        self.shader.set_used();
        // @tmp
        if camera_moved {
            let proj = camera.get_projection_matrix();
            let view = camera.get_view_matrix();
            self.shader.set_mat4("proj", &proj)?;
            self.shader.set_mat4("view", &view)?;
        }
        self.vao.bind();

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_1D, self.texture_id);
            gl::DepthFunc(gl::LEQUAL);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::DepthFunc(gl::LESS);
        }

        Ok(())
    }
}
