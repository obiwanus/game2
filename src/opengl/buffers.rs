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

    pub fn send_data<T>(&self, data: &[T]) {
        let length = std::mem::size_of::<T>() * data.len();
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                length as isize,
                data.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
        }
    }

    pub fn bind_as(&self, target: GLenum) {
        unsafe {
            gl::BindBuffer(target, self.id);
        }
    }
}

// ==================================== ElementBuffer =============================================

#[derive(Debug)]
pub struct ElementBuffer {
    pub num_elements: usize,
    pub buffer_offset: usize,
    pub element_type: GLenum,
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
