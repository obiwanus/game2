use gl::types::*;

// ==================================== Buffer ====================================================

pub struct StaticBuffer {
    buffer: Buffer,
}

impl StaticBuffer {
    /// Initialise with static data
    pub fn init<T>(data: &[T]) -> Self {
        let size = std::mem::size_of::<T>() * data.len();
        let mut buffer = Buffer::new();
        buffer.size = Some(size);
        unsafe {
            gl::NamedBufferStorage(buffer.id, size as isize, data.as_ptr() as *const _, 0);
        }
        StaticBuffer { buffer }
    }

    #[inline(always)]
    pub fn id(&self) -> GLuint {
        self.buffer.id
    }
}

#[derive(Debug)]
struct Buffer {
    id: GLuint,
    size: Option<usize>,
}

// @abstraction: need to think more about this

impl Buffer {
    pub fn new() -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::CreateBuffers(1, &mut id);
        }
        Buffer { id, size: None }
    }

    pub fn allocate_dynamic_data<T>(&mut self, target: GLenum, data: &[T]) {
        let size = std::mem::size_of::<T>() * data.len();
        unsafe {
            gl::BufferData(
                target,
                size as isize,
                std::ptr::null() as *const GLvoid,
                gl::DYNAMIC_DRAW,
            );
        }
        self.size = size;
    }

    pub fn send_dynamic_data<T>(&self, target: GLenum, offset: usize, data: &[T]) {
        let size = std::mem::size_of::<T>() * data.len();
        assert!(
            (size + offset) <= self.size,
            "Attempting to update buffer past its end"
        );

        unsafe {
            gl::BufferSubData(
                target,
                offset as isize,
                size as isize,
                data.as_ptr() as *const GLvoid,
            );
        }
    }

    pub fn send_stream_data<T>(target: GLenum, data: &[T]) {
        let size = std::mem::size_of::<T>() * data.len();
        unsafe {
            gl::BufferData(
                target,
                size as isize,
                data.as_ptr() as *const GLvoid,
                gl::STREAM_DRAW,
            );
        }
    }

    pub fn bind_as(&self, target: GLenum) {
        unsafe {
            gl::BindBuffer(target, self.id);
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let ids = [self.id];
        unsafe {
            gl::DeleteBuffers(1, ids.as_ptr() as *const GLuint);
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

impl Drop for VertexArray {
    fn drop(&mut self) {
        let ids = [self.id];
        unsafe {
            gl::DeleteVertexArrays(1, ids.as_ptr() as *const GLuint);
        }
    }
}
