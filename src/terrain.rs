use glam::Vec3;
use opengl_lib::types::GLvoid;

use crate::opengl::buffers::{Buffer, VertexArray};

pub struct Terrain {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    num_indices: i32,

    vao: VertexArray,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl Terrain {
    pub fn new(size: f32, cells: i32) -> Self {
        let mut vertices = vec![];
        let cell_size = size / cells as f32;
        let start_x = -size / 2.0;
        let start_z = -size / 2.0;
        for x in 0..cells + 1 {
            for z in 0..cells + 1 {
                let pos = Vec3::new(
                    start_x + x as f32 * cell_size,
                    0.0,
                    start_z + z as f32 * cell_size,
                );
                vertices.push(Vertex { pos });
            }
        }

        // TODO: triangle strip?
        // TODO: optimise mesh?
        let num_indices = cells * cells * 6;
        let mut indices = Vec::with_capacity(num_indices as usize);
        let stride = (cells + 1) as u16;
        for x in 0..cells as u16 {
            for y in 0..cells as u16 {
                let i = x * stride + y;
                // Quad vertices
                let (v1, v2, v3, v4) = (i, i + 1, i + 1 + stride, i + stride);

                // First triangle
                indices.push(v1);
                indices.push(v2);
                indices.push(v3);

                // Second triangle
                indices.push(v1);
                indices.push(v3);
                indices.push(v4);
            }
        }

        let vao = VertexArray::new();
        vao.bind();

        let vertex_buffer = Buffer::new();
        vertex_buffer.bind_as(gl::ARRAY_BUFFER);
        Buffer::send_data(gl::ARRAY_BUFFER, &vertices, gl::STATIC_DRAW);
        unsafe {
            // Position
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
            gl::EnableVertexAttribArray(0);
        }

        let index_buffer = Buffer::new();
        index_buffer.bind_as(gl::ELEMENT_ARRAY_BUFFER);
        Buffer::send_data(gl::ELEMENT_ARRAY_BUFFER, &indices, gl::STATIC_DRAW);

        println!("Indices: {}", indices.len());
        println!("TRiangles: {}", indices.len() / 3);

        Terrain {
            vertices,
            indices,
            num_indices,

            vao,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn draw(&self) {
        self.vao.bind();

        unsafe {
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            // use crate::opengl::gl_check_error;
            // gl_check_error!();

            gl::DrawElements(
                gl::TRIANGLES,
                self.num_indices,
                gl::UNSIGNED_SHORT,
                std::ptr::null(),
            );
        }
    }

    pub fn draw_highlighted_triangle(&self, index: usize) {
        assert!(index <= (self.indices.len() / 3));
        self.vao.bind();
        unsafe {
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            gl::DrawElements(
                gl::TRIANGLES,
                3,
                gl::UNSIGNED_SHORT,
                (std::ptr::null() as *const u16).add((index * 3) as usize) as *const GLvoid,
            );
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Vertex {
    pos: Vec3,
}
