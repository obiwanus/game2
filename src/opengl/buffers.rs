use gl::types::*;

// ==================================== Buffer ====================================================

#[derive(Debug)]
pub struct Buffer {
    id: GLuint,
}

impl Buffer {
    pub fn new() -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        Buffer { id }
    }

    pub fn send_data<T>(target: GLenum, data: &[T], usage: GLenum) {
        let length = std::mem::size_of::<T>() * data.len();
        unsafe {
            gl::BufferData(
                target,
                length as isize,
                data.as_ptr() as *const GLvoid,
                usage,
            );
        }
    }

    pub fn bind_as(&self, target: GLenum) {
        unsafe {
            gl::BindBuffer(target, self.id);
        }
    }
}

// ==================================== VertexArray ===============================================

#[derive(Debug)]
pub struct VertexArray {
    id: GLuint,
}

impl VertexArray {
    pub fn new() -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }
        VertexArray { id }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}
