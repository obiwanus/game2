use std::ffi::CString;
use std::fs;
use std::io;

use gl::types::*;
use glam::Vec2;
use glam::{Mat4, Vec3};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShaderError {
    #[error("Failed to compile {name}: {message}")]
    CompileError { name: String, message: String },
    #[error("Failed to link program: {0}")]
    LinkError(String),
    #[error("Couldn't get uniform location for '{name}'")]
    UniformLocationNotFound { name: String },
    #[error("Couldn't get uniform block index for '{name}'")]
    UniformBlockIndexNotFound { name: String },
}

pub type Result<T> = std::result::Result<T, ShaderError>;

pub struct Program {
    id: GLuint,
}

impl Program {
    pub fn new() -> Self {
        let id = unsafe { gl::CreateProgram() };
        Program { id }
    }

    fn attach_shader(&self, code: &str, kind: GLenum) -> Result<()> {
        let shader = Shader::new(kind, code)?;
        unsafe {
            gl::AttachShader(self.id, shader.id());
        }
        Ok(())
    }

    pub fn vertex_shader(self, code: &str) -> Result<Self> {
        self.attach_shader(code, gl::VERTEX_SHADER)?;
        Ok(self)
    }

    pub fn fragment_shader(self, code: &str) -> Result<Self> {
        self.attach_shader(code, gl::FRAGMENT_SHADER)?;
        Ok(self)
    }

    pub fn tess_control_shader(self, code: &str) -> Result<Self> {
        self.attach_shader(code, gl::TESS_CONTROL_SHADER)?;
        Ok(self)
    }

    pub fn tess_evaluation_shader(self, code: &str) -> Result<Self> {
        self.attach_shader(code, gl::TESS_EVALUATION_SHADER)?;
        Ok(self)
    }

    pub fn link(self) -> Result<Self> {
        unsafe {
            gl::LinkProgram(self.id);
        }
        let mut success: GLint = 1;
        unsafe {
            gl::GetProgramiv(self.id, gl::LINK_STATUS, &mut success);
        }
        if success == 0 {
            let mut len: GLint = 0;
            unsafe {
                gl::GetProgramiv(self.id, gl::INFO_LOG_LENGTH, &mut len);
            }
            let error = new_cstring(len as usize);
            unsafe {
                gl::GetProgramInfoLog(
                    self.id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut GLchar,
                )
            }
            return Err(ShaderError::LinkError(error.to_string_lossy().into_owned()));
        }

        Ok(self)
    }

    pub fn set_used(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    // @Speed: don't get uniform location every time
    pub fn get_uniform_location(&self, name: &str) -> Result<GLint> {
        let name_cstr = CString::new(name).unwrap();
        let location =
            unsafe { gl::GetUniformLocation(self.id, name_cstr.as_ptr() as *const GLchar) };
        if location < 0 {
            return Err(ShaderError::UniformLocationNotFound {
                name: name.to_owned(),
            });
        }
        Ok(location)
    }

    fn get_uniform_block_index(&self, name: &str) -> Result<GLuint> {
        let name_cstr = CString::new(name).unwrap();
        let index = unsafe { gl::GetUniformBlockIndex(self.id, name_cstr.as_ptr() as *const _) };
        if index == gl::INVALID_INDEX {
            return Err(ShaderError::UniformBlockIndexNotFound {
                name: name.to_owned(),
            });
        }
        Ok(index)
    }

    pub fn bind_uniform_block(&self, name: &str, binding: u32) -> Result<()> {
        let index = self.get_uniform_block_index(name)?;
        unsafe {
            gl::UniformBlockBinding(self.id, index, binding);
        }
        Ok(())
    }

    /// Assigns a name from the shader to a texture unit
    pub fn set_texture_unit(&self, name: &str, unit: i32) -> Result<()> {
        let location = self.get_uniform_location(name)?;
        unsafe {
            gl::Uniform1i(location, unit);
        }
        Ok(())
    }

    /// Sets a vec2 uniform
    pub fn set_vec2(&self, name: &str, vec: &Vec2) -> Result<()> {
        let location = self.get_uniform_location(name)?;
        unsafe {
            gl::Uniform2fv(location, 1, vec.to_array().as_ptr());
        }
        Ok(())
    }

    /// Sets a vec3 uniform
    pub fn set_vec3(&self, name: &str, vec: &Vec3) -> Result<()> {
        let location = self.get_uniform_location(name)?;
        unsafe {
            gl::Uniform3fv(location, 1, vec.to_array().as_ptr());
        }
        Ok(())
    }

    /// Sets a [f32; 3] uniform
    pub fn set_float3(&self, name: &str, vec: &[f32]) -> Result<()> {
        let location = self.get_uniform_location(name)?;
        unsafe {
            gl::Uniform3fv(location, 1, vec.as_ptr());
        }
        Ok(())
    }

    /// Sets a mat4 uniform
    pub fn set_mat4(&self, name: &str, mat: &Mat4) -> Result<()> {
        let location = self.get_uniform_location(name)?;
        unsafe {
            gl::UniformMatrix4fv(location, 1, gl::FALSE, mat.to_cols_array().as_ptr());
        }
        Ok(())
    }

    /// Sets a float uniform
    pub fn set_f32(&self, name: &str, value: f32) -> Result<()> {
        let location = self.get_uniform_location(name)?;
        unsafe {
            gl::Uniform1fv(location, 1, &value as *const f32);
        }
        Ok(())
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

struct Shader {
    id: GLuint,
}

impl Shader {
    pub fn new(kind: GLenum, code: &str) -> Result<Self> {
        let source = CString::new(code).unwrap();
        let id = unsafe { gl::CreateShader(kind) };
        unsafe {
            gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
            gl::CompileShader(id);
        }
        let mut success: GLint = 1;
        unsafe {
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        }
        if success == 0 {
            let mut len: GLint = 0;
            unsafe {
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }
            let error = new_cstring(len as usize);
            unsafe {
                gl::GetShaderInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar);
            }
            let name = match kind {
                gl::VERTEX_SHADER => "vertex shader",
                gl::FRAGMENT_SHADER => "fragment shader",
                gl::TESS_CONTROL_SHADER => "tessellation control shader",
                gl::TESS_EVALUATION_SHADER => "tessellation evaluation shader",
                _ => panic!("Unknown shader type"),
            };
            return Err(ShaderError::CompileError {
                name: name.to_owned(),
                message: error.to_string_lossy().into_owned(),
            });
        }
        Ok(Shader { id })
    }

    pub fn id(&self) -> GLuint {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

fn new_cstring(len: usize) -> CString {
    let buffer: Vec<u8> = vec![0; len];
    unsafe { CString::from_vec_unchecked(buffer) }
}
