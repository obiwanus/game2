use std::error::Error;

use glam::{Vec2, Vec3};
use memoffset::offset_of;
use opengl_lib::types::GLvoid;

use crate::{
    opengl::{
        buffers::{Buffer, VertexArray},
        shader::Program,
    },
    texture::Texture,
};

pub struct Terrain {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    num_indices: i32,

    vao: VertexArray,
    vertex_buffer: Buffer,
    index_buffer: Buffer,

    texture: Texture,
    pub cursor: Vec3,
}

impl Terrain {
    pub fn new(size: f32, cells: i32) -> Result<Self, Box<dyn Error>> {
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
                let normal = Vec3::new(0.0, 1.0, 0.0);
                let uv = Vec2::new(x as f32, z as f32);
                vertices.push(Vertex { pos, normal, uv });
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

        let mut vertex_buffer = Buffer::new();
        vertex_buffer.bind_as(gl::ARRAY_BUFFER);
        vertex_buffer.allocate_dynamic_data(gl::ARRAY_BUFFER, &vertices);
        vertex_buffer.send_dynamic_data(gl::ARRAY_BUFFER, 0, &vertices);
        unsafe {
            // Position
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<Vertex>() as i32,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(0);

            // Normal
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<Vertex>() as i32,
                offset_of!(Vertex, normal) as *const GLvoid,
            );
            gl::EnableVertexAttribArray(1);

            // Normal
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<Vertex>() as i32,
                offset_of!(Vertex, uv) as *const GLvoid,
            );
            gl::EnableVertexAttribArray(2);
        }

        let index_buffer = Buffer::new();
        index_buffer.bind_as(gl::ELEMENT_ARRAY_BUFFER);
        Buffer::send_static_data(gl::ELEMENT_ARRAY_BUFFER, &indices);

        let texture = Texture::new()
            .set_default_parameters()
            .set_image_2d("textures/checkerboard.png")
            .expect("Coudn't load texture");

        let cursor = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);

        let shader = Program::new()
            .vertex_shader("shaders/editor/terrain.vert")?
            .fragment_shader("shaders/editor/terrain.frag")?
            .link()?;

        Ok(Terrain {
            vertices,
            indices,
            num_indices,

            vao,
            vertex_buffer,
            index_buffer,

            texture,
            cursor,
        })
    }

    pub fn triangles(&self) -> TriangleIter {
        TriangleIter::new(self)
    }

    pub fn draw(&self) {
        self.vao.bind();
        self.texture.bind_2d(0);

        unsafe {
            // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
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

    // @speed: slowwwww
    pub fn send_vertex_buffer(&self) {
        self.vertex_buffer.bind_as(gl::ARRAY_BUFFER);
        self.vertex_buffer
            .send_dynamic_data(gl::ARRAY_BUFFER, 0, &self.vertices);
    }
}

pub struct TriangleIter<'a> {
    terrain: &'a Terrain,
    num_triangles: usize,
    index: usize,
}

impl<'a> TriangleIter<'a> {
    fn new(terrain: &'a Terrain) -> Self {
        TriangleIter {
            terrain,
            num_triangles: terrain.indices.len() / 3,
            index: 0,
        }
    }
}

impl<'a> Iterator for TriangleIter<'a> {
    type Item = (&'a Vec3, &'a Vec3, &'a Vec3);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.num_triangles {
            return None;
        }
        let vertices = &self.terrain.vertices;

        // Get triangle indices
        let next_index = self.index * 3;
        let a = self.terrain.indices[next_index] as usize;
        let b = self.terrain.indices[next_index + 1] as usize;
        let c = self.terrain.indices[next_index + 2] as usize;

        self.index += 1;

        let triangle = (&vertices[a].pos, &vertices[b].pos, &vertices[c].pos);
        Some(triangle)
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Vertex {
    pub pos: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}
