use std::mem::size_of;

use gl::types::*;
use thiserror::Error;

use crate::opengl::shader::{Program, ShaderError};
use crate::utils::size_of_slice;

#[derive(Debug, Error)]
pub enum SkyboxError {
    #[error("Skybox shader error: {0}")]
    Shader(#[from] ShaderError),
}

pub struct Skybox {
    id: GLuint,
    shader: Program,
    vao: GLuint,
    vbo: GLuint,
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
            let img = image::open(path)
                .expect("Can't load skybox image")
                .into_rgb8();
            let (width, height) = img.dimensions();
            unsafe {
                // Send to GPU
                gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                    0,
                    gl::SRGB8 as GLint,
                    width as GLint,
                    height as GLint,
                    0,
                    gl::RGB,
                    gl::UNSIGNED_BYTE,
                    img.as_raw().as_ptr() as *const std::ffi::c_void,
                );
            }
        }

        // Create shader
        let shader = Program::new()
            .vertex_shader(include_str!("shaders/skybox/skybox.vert"))?
            .fragment_shader(include_str!("shaders/skybox/skybox.frag"))?
            .link()?;
        shader.set_used();

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

        // Init buffers
        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;
        unsafe {
            gl::CreateVertexArrays(1, &mut vao);
            gl::CreateBuffers(1, &mut vbo);

            // Upload vertices
            gl::NamedBufferStorage(
                vbo,
                size_of_slice(&vertices) as isize,
                vertices.as_ptr() as *const _,
                0,
            );

            // Describe vertex buffer
            gl::VertexArrayVertexBuffer(vao, 0, vbo, 0, (size_of::<f32>() * 3) as i32);
            gl::VertexArrayAttribFormat(vao, 0, 3, gl::FLOAT, gl::FALSE, 0);
            gl::EnableVertexArrayAttrib(vao, 0);
        }

        Ok(Skybox {
            id,
            shader,
            vao,
            vbo,
        })
    }

    pub fn draw(&self) {
        unsafe {
            gl::DepthFunc(gl::LEQUAL);
        }
        self.shader.set_used();

        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.id);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::DepthFunc(gl::LESS);
        }
    }
}

impl Drop for Skybox {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo as *const _);
            gl::DeleteVertexArrays(1, &self.vao as *const _);
        }
    }
}
