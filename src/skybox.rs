use gl::types::*;
use glam::Mat4;
use thiserror::Error;

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
    id: GLuint,
    shader: Program,
    vao: VertexArray,
    _vbo: Buffer,
}

impl Skybox {
    /// right, left, top, bottom, front, back
    pub fn from(paths: [&str; 6]) -> Result<Self, SkyboxError> {
        // Generate texture
        let mut id: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, id);

            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_R,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as GLint,
            );
        }

        // Load images
        for (i, path) in paths.iter().enumerate() {
            let img = load_image(path, false)?;
            unsafe {
                // Send to GPU
                gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                    0,
                    gl::SRGB8 as GLint,
                    img.width as GLint,
                    img.height as GLint,
                    0,
                    gl::RGB,
                    gl::UNSIGNED_BYTE,
                    img.data.as_ptr() as *const std::ffi::c_void,
                );
            }
        }

        // Create shader
        let shader = Program::new()
            .vertex_shader("shaders/skybox/skybox.vert")?
            .fragment_shader("shaders/skybox/skybox.frag")?
            .link()?;
        shader.set_used();
        shader.set_texture_unit("skybox", 0)?;

        #[rustfmt::skip]
        let vertices = [
            // positions
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
            id,
            shader,
            vao,
            _vbo: vbo,
        })
    }

    pub fn draw(&self, proj: &Mat4, view: &Mat4) -> Result<(), SkyboxError> {
        unsafe {
            gl::DepthFunc(gl::LEQUAL);
        }
        self.shader.set_used();
        self.shader.set_mat4("proj", proj)?;
        self.shader.set_mat4("view", view)?;
        self.vao.bind();

        unsafe {
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.id);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::DepthFunc(gl::LESS);
        }

        Ok(())
    }
}
