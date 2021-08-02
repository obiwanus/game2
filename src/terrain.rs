use std::error::Error;

use gl::types::*;
use glam::{Vec2, Vec3};
use memoffset::offset_of;
use opengl_lib::types::GLvoid;

use crate::{
    camera::Camera,
    editor::Brush,
    opengl::{
        buffers::{Buffer, VertexArray},
        shader::Program,
    },
    texture::Texture,
    utils::vec3_infinity,
    Result,
};

struct Heightmap {
    id: GLuint, // @tmp_public
    size: usize,
    pixels: Vec<u16>, // @speed: store efficiently?
}

impl Heightmap {
    fn new(size: usize) -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
        }
        let pixels = vec![0u16; size * size];
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RED as i32,
                size as i32,
                size as i32,
                0,
                gl::RED,
                gl::UNSIGNED_SHORT,
                pixels.as_ptr() as *const GLvoid,
            );
        }

        Heightmap { id, size, pixels }
    }

    // @duplicate
    fn bind_2d(&self, unit: i32) {
        unsafe {
            gl::ActiveTexture(Heightmap::unit_to_gl_const(unit));
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    // @duplicate
    fn unit_to_gl_const(unit: i32) -> GLenum {
        match unit {
            0 => gl::TEXTURE0,
            1 => gl::TEXTURE1,
            2 => gl::TEXTURE2,
            3 => gl::TEXTURE3,
            4 => gl::TEXTURE4,
            5 => gl::TEXTURE5,
            6 => gl::TEXTURE6,
            7 => gl::TEXTURE7,
            8 => gl::TEXTURE8,
            9 => gl::TEXTURE9,
            10 => gl::TEXTURE10,
            11 => gl::TEXTURE11,
            12 => gl::TEXTURE12,
            13 => gl::TEXTURE13,
            14 => gl::TEXTURE14,
            15 => gl::TEXTURE15,
            _ => panic!("Unsupported texture unit"),
        }
    }
}

pub struct Terrain {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    num_indices: i32,

    vao: VertexArray,
    vertex_buffer: Buffer,
    _index_buffer: Buffer,
    shader: Program,

    texture: Texture,
    heightmap: Heightmap,
    pub cursor: Vec3,
    pub brush: Brush,
}

impl Terrain {
    pub fn new(size: f32, cells: i32) -> Result<Self> {
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

        // @efficiency: triangle strip?
        // @speed: optimise mesh?
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
        Buffer::send_static_data(gl::ARRAY_BUFFER, &vertices);
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

            // UV
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

        let heightmap = Heightmap::new(513);

        let cursor = vec3_infinity();

        let brush = Brush::new("src/editor/brushes/brush1.png");

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
            _index_buffer: index_buffer,
            shader,

            texture,
            heightmap,

            cursor,
            brush,
        })
    }

    pub fn triangles(&self) -> TriangleIter {
        TriangleIter::new(self)
    }

    // @tmp: remove camera and move to renderer
    pub fn draw(&self, camera: &Camera, camera_moved: bool) -> Result<()> {
        self.shader.set_used();
        self.shader.set_vec3("cursor", &self.cursor)?;
        self.shader.set_float("brush_size", self.brush.size)?;

        // @tmp
        if camera_moved {
            let proj = camera.get_projection_matrix();
            let view = camera.get_view_matrix();
            self.shader.set_mat4("proj", &proj)?;
            self.shader.set_mat4("view", &view)?;
        }

        self.vao.bind();
        self.texture.bind_2d(0);

        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.num_indices,
                gl::UNSIGNED_SHORT,
                std::ptr::null(),
            );
        }

        Ok(())
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
