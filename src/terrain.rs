use gl::types::*;
use glam::{Vec2, Vec3};
use opengl_lib::types::GLvoid;
use stb_image::image::{Image, LoadResult};

use crate::{
    camera::Camera, editor::Brush, opengl::shader::Program, ray::AABB, texture::Texture,
    utils::vec3_infinity, Result,
};

struct Heightmap {
    id: GLuint, // @tmp_public
    size: usize,
    pixels: Vec<f32>, // @speed: store efficiently?
}

impl Heightmap {
    fn new(path: &str) -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
        }

        let image = load_image(path, false);
        let (pixels, size) = Heightmap::get_pixels_from_u8_grayscale_image(image);

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        }

        let heightmap = Heightmap { id, size, pixels };
        heightmap.upload_texture();

        heightmap
    }

    // TODO: is there a way to decompose a struct so that we don't have to do this?
    fn get_pixels_from_u8_grayscale_image(image: Image<u8>) -> (Vec<f32>, usize) {
        assert_eq!(image.width, image.height);
        let size = image.width;
        let pixels = image.data.into_iter().map(|p| (p as f32) / 256.0).collect();

        (pixels, size)
    }

    fn upload_texture(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::R16 as i32,
                self.size as i32,
                self.size as i32,
                0,
                gl::RED,
                gl::FLOAT,
                self.pixels.as_ptr() as *const GLvoid,
            );
        }
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
    center: Vec2,
    max_height: f32,
    pub aabb: AABB,

    vao: GLuint,
    shader: Program,
    pub tess_level: f32,

    texture: Texture,
    heightmap: Heightmap,
    pub cursor: Vec3,
    pub brush: Brush,
}

impl Terrain {
    pub fn new(center: Vec2) -> Result<Self> {
        // TODO: support centers other than 0, 0
        // (currently hard-coded in terrain.vert.glsl)
        assert_eq!(center, Vec2::new(0.0, 0.0));
        const NUM_PATCHES: f32 = 64.0;
        const PATCH_SIZE: f32 = 4.0;
        let max_height = 200.0;
        let aabb = {
            let size = PATCH_SIZE * NUM_PATCHES / 2.0;
            let min = Vec3::new(-size, 0.0, -size);
            let max = Vec3::new(size, max_height, size);
            AABB::new(min, max)
        };

        let mut vao: GLuint = 0;
        unsafe {
            gl::CreateVertexArrays(1, &mut vao);

            gl::PatchParameteri(gl::PATCH_VERTICES, 4);
        }

        let texture = Texture::new()
            .set_default_parameters()
            .set_image_2d("textures/checkerboard.png")
            .expect("Coudn't load texture");

        let heightmap = Heightmap::new("textures/heightmaps/valley.png");

        let cursor = vec3_infinity();

        let brush = Brush::new("src/editor/brushes/brush1.png");

        let shader = Program::new()
            .vertex_shader(include_str!("shaders/editor/terrain.vert.glsl"))?
            .tess_control_shader(include_str!("shaders/editor/terrain.tc.glsl"))?
            .tess_evaluation_shader(include_str!("shaders/editor/terrain.te.glsl"))?
            .fragment_shader(include_str!("shaders/editor/terrain.frag.glsl"))?
            .link()?;

        Ok(Terrain {
            center,
            max_height,
            aabb,

            vao,
            shader,
            tess_level: 1.0,

            texture,
            heightmap,

            cursor,
            brush,
        })
    }

    // @tmp: remove camera and move to renderer
    pub fn draw(&self, camera: &Camera, camera_moved: bool) -> Result<()> {
        self.shader.set_used();
        // self.shader.set_vec3("cursor", &self.cursor)?;
        // self.shader.set_float("brush_size", self.brush.size)?;
        self.shader.set_float("tess_level", self.tess_level)?;

        // @tmp
        if camera_moved {
            let proj = camera.get_projection_matrix();
            let view = camera.get_view_matrix();
            let mvp = proj * view;
            self.shader.set_mat4("mvp", &mvp)?;
        }

        // @try moving outsize of the draw
        self.texture.bind_2d(0);
        self.shader.set_texture_unit("terrain_texture", 0)?;
        self.heightmap.bind_2d(1);
        self.shader.set_texture_unit("heightmap", 1)?;

        unsafe {
            gl::BindVertexArray(self.vao);
            // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            gl::DrawArraysInstanced(gl::PATCHES, 0, 4, 64 * 64);
            // gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        }

        Ok(())
    }

    pub fn raise_terrain(&mut self, delta_time: f32) {
        let size = 1000.0; // @todo: variable size or put in a proper const
                           // Find where the cursor is on the texture (relative to center)
        let cursor =
            Vec2::new(0.5, 0.5) + (Vec2::new(self.cursor.x, self.cursor.z) - self.center) / size;

        // Get brush size in terms of underlying texture
        let brush_size = self.brush.size / size;
        let brush_size_sq = brush_size * brush_size;

        let sensitivity = 0.3;

        // Change the values around the cursor
        // @speed: brute force (no need to step through the whole terrain)
        for z in 0..self.heightmap.size {
            for x in 0..self.heightmap.size {
                // Position of pixel in texture coordinates
                // @speed: no need to calculate every time
                let pos = Vec2::new(x as f32, z as f32) / self.heightmap.size as f32;
                let dist_sq = (pos - cursor).length_squared();
                if dist_sq < brush_size_sq {
                    let index = z * self.heightmap.size + x;
                    self.heightmap.pixels[index] += delta_time * sensitivity;
                }
            }
        }

        // Update the texture
        self.heightmap.upload_texture();
    }
}

// @duplication: merge with texture.rs/load_image?
pub fn load_image(path: &str, flip: bool) -> Image<u8> {
    let flip = if flip { 1 } else { 0 };
    unsafe {
        stb_image::stb_image::bindgen::stbi_set_flip_vertically_on_load(flip);
    }
    match stb_image::image::load_with_depth(path, 1, false) {
        LoadResult::ImageU8(image) => image,
        LoadResult::ImageF32(_) => panic!(
            "Couldn't load brush {}. Only U8 grayscale brush images are supported.",
            path
        ),
        LoadResult::Error(msg) => panic!("Couldn't load brush {}: {}", path, msg),
    }
}
