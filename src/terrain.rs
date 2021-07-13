use gl::types::GLvoid;
use glam::Vec3;

use crate::buffers::{Buffer, VertexArray};

pub struct Terrain {
    pub vertices: Vec<Vertex>,

    // tmp
    pub vao: VertexArray,
    vbo: Buffer,
}

impl Terrain {
    pub fn new(size: f32, cells: usize) -> Self {
        let mut vertices = vec![];
        let cell_size = size / cells as f32;
        let start_x = -size / 2.0;
        let start_z = -size / 2.0;

        for x in 0..cells {
            for z in 0..cells {
                let pos = Vec3::new(
                    start_x + x as f32 * cell_size,
                    0.0,
                    start_z + z as f32 * cell_size,
                );
                vertices.push(Vertex { pos });
            }
        }

        let vao = VertexArray::new();
        vao.bind();

        let vbo = Buffer::new();
        vbo.bind_as(gl::ARRAY_BUFFER);
        vbo.send_data(&vertices);

        unsafe {
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
            gl::EnableVertexAttribArray(0);
        }

        Terrain { vertices, vao, vbo }
    }
}

#[repr(C)]
pub struct Vertex {
    pos: Vec3,
}
