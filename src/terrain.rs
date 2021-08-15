use gl::types::*;
use glam::Vec3Swizzles;
use glam::{Vec2, Vec3};
use opengl_lib::types::GLvoid;
use stb_image::image::{Image, LoadResult};

use crate::{
    camera::Camera,
    editor::Brush,
    opengl::shader::Program,
    ray::{Ray, AABB},
    texture::Texture,
    utils::{vec2_infinity, vec3_infinity},
    Result,
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

    fn sample_height(&self, point: Vec2) -> f32 {
        let x = (point.x as usize).clamp(0, self.size);
        let y = (point.y as usize).clamp(0, self.size);

        self.pixels[y * self.size + x]
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
    pub cursor: Vec2,
    pub brush: Brush,

    debug: TerrainDebug,
}

struct TerrainDebug {
    aabb_shader: Program,
    line_shader: Program,
    point_shader: Program,

    lines: Vec<Vec3>,
    lines_max: usize,

    points: Vec<Vec3>,
    points_max: usize,

    lines_vao: GLuint,
    points_vao: GLuint,
    vbo: GLuint,
    buffer_changed: bool,
}

impl Drop for TerrainDebug {
    fn drop(&mut self) {
        let vaos = [self.lines_vao, self.points_vao];
        unsafe {
            gl::DeleteVertexArrays(2, vaos.as_ptr() as *const _);
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}

impl Terrain {
    pub fn new(center: Vec2) -> Result<Self> {
        // TODO: support centers other than 0, 0
        // (currently hard-coded in terrain.vert.glsl)
        assert_eq!(center, Vec2::new(0.0, 0.0));
        const NUM_PATCHES: f32 = 64.0;
        const PATCH_SIZE: f32 = 8.0;
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

        let heightmap = Heightmap::new("textures/heightmaps/ruapehu.png");

        let cursor = vec2_infinity();

        let brush = Brush::new("src/editor/brushes/brush1.png");

        let shader = Program::new()
            .vertex_shader(include_str!("shaders/editor/terrain.vert.glsl"))?
            .tess_control_shader(include_str!("shaders/editor/terrain.tc.glsl"))?
            .tess_evaluation_shader(include_str!("shaders/editor/terrain.te.glsl"))?
            .fragment_shader(include_str!("shaders/editor/terrain.frag.glsl"))?
            .link()?;

        let debug = {
            let aabb_shader = Program::new()
                .vertex_shader(include_str!("shaders/debug/aabb.vert"))?
                .fragment_shader(include_str!("shaders/debug/aabb.frag"))?
                .link()?;
            aabb_shader.set_used();
            aabb_shader.set_vec3("aabb_min", &aabb.min)?;
            aabb_shader.set_vec3("aabb_max", &aabb.max)?;

            let line_shader = Program::new()
                .vertex_shader(include_str!("shaders/debug/terrain/line.vert"))?
                .fragment_shader(include_str!("shaders/debug/terrain/line.frag"))?
                .link()?;

            let point_shader = Program::new()
                .vertex_shader(include_str!("shaders/debug/terrain/point.vert"))?
                .fragment_shader(include_str!("shaders/debug/terrain/point.frag"))?
                .link()?;

            let lines_max = 50;
            let lines = Vec::with_capacity(lines_max * 2); // coord1, coord2

            let points_max = 1000;
            let points = Vec::with_capacity(points_max); // coord, color

            let mut lines_vao: GLuint = 0;
            let mut points_vao: GLuint = 0;
            let mut vbo: GLuint = 0;
            unsafe {
                gl::CreateVertexArrays(1, &mut lines_vao);
                gl::CreateVertexArrays(1, &mut points_vao);

                // Single buffer for lines and points
                gl::CreateBuffers(1, &mut vbo);
                let vec3_size = std::mem::size_of::<Vec3>();
                gl::VertexArrayVertexBuffer(lines_vao, 0, vbo, 0, vec3_size as i32);
                gl::VertexArrayAttribFormat(lines_vao, 0, 3, gl::FLOAT, gl::FALSE, 0);
                gl::VertexArrayAttribBinding(lines_vao, 0, 0);
                gl::EnableVertexArrayAttrib(lines_vao, 0);

                gl::VertexArrayVertexBuffer(
                    points_vao,
                    0,
                    vbo,
                    (vec3_size * lines_max * 2) as isize,
                    2 * vec3_size as i32,
                );
                gl::VertexArrayAttribFormat(points_vao, 0, 3, gl::FLOAT, gl::FALSE, 0);
                gl::VertexArrayAttribFormat(
                    points_vao,
                    1,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    vec3_size as u32,
                );
                gl::VertexArrayAttribBinding(points_vao, 0, 0);
                gl::VertexArrayAttribBinding(points_vao, 1, 0);
                gl::EnableVertexArrayAttrib(points_vao, 0);
                gl::EnableVertexArrayAttrib(points_vao, 1);

                // Allocate memory
                let buffer_size = vec3_size * lines_max * 2 * points_max * 2;
                gl::NamedBufferStorage(
                    vbo,
                    buffer_size as isize,
                    std::ptr::null() as *const _,
                    gl::DYNAMIC_STORAGE_BIT,
                );
            }

            TerrainDebug {
                aabb_shader,
                line_shader,
                point_shader,
                lines,
                lines_max,
                points,
                points_max,
                lines_vao,
                points_vao,
                vbo,
                buffer_changed: false,
            }
        };

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

            debug,
        })
    }

    // @tmp: remove camera and move to renderer
    pub fn draw(&mut self, camera: &Camera, camera_moved: bool, time: f32) -> Result<()> {
        self.shader.set_used();
        self.shader.set_vec2("cursor", &self.cursor)?;
        // self.shader.set_float("brush_size", self.brush.size)?;
        self.shader.set_f32("tess_level", self.tess_level)?;

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

        // Draw debug stuff
        {
            // Draw AABB
            let debug = &mut self.debug;
            debug.aabb_shader.set_used();
            debug.aabb_shader.set_f32("time", time)?;
            // @tmp
            if camera_moved {
                let proj = camera.get_projection_matrix();
                let view = camera.get_view_matrix();
                let mvp = proj * view;
                debug.aabb_shader.set_mat4("mvp", &mvp)?;
                debug.line_shader.set_used();
                debug.line_shader.set_mat4("mvp", &mvp)?;
                debug.point_shader.set_used();
                debug.point_shader.set_mat4("mvp", &mvp)?;
            }

            debug.aabb_shader.set_used();
            unsafe {
                gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
                gl::DrawArrays(gl::TRIANGLES, 0, 6 * 2 * 3);
                gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            }

            if debug.buffer_changed {
                eprintln!("Uploading buffer");
                dbg!(&debug.lines);
                unsafe {
                    gl::PointSize(3.0);
                    // Upload lines
                    gl::NamedBufferSubData(
                        debug.vbo,
                        0,
                        (std::mem::size_of::<Vec3>() * debug.lines.len()) as isize,
                        debug.lines.as_ptr() as *const _,
                    );
                    // Upload points
                    gl::NamedBufferSubData(
                        debug.vbo,
                        (std::mem::size_of::<Vec3>() * debug.lines_max * 2) as isize,
                        (std::mem::size_of::<Vec3>() * debug.points.len()) as isize,
                        debug.points.as_ptr() as *const _,
                    );
                }
                debug.buffer_changed = false;
            }

            debug.line_shader.set_used();
            unsafe {
                gl::BindVertexArray(debug.lines_vao);
                gl::DrawArrays(gl::LINES, 0, debug.lines.len() as i32);
            }

            debug.point_shader.set_used();
            unsafe {
                gl::BindVertexArray(debug.points_vao);
                gl::DrawArrays(gl::POINTS, 0, debug.points.len() as i32);
            }
        }

        Ok(())
    }

    pub fn raise_terrain(&mut self, delta_time: f32) {
        let size = 1000.0; // @todo: variable size or put in a proper const

        // Find where the cursor is on the texture (relative to center)
        let cursor =
            Vec2::new(0.5, 0.5) + (Vec2::new(self.cursor.x, self.cursor.y) - self.center) / size;

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

    pub fn is_point_above_surface(&self, point: &Vec3) -> bool {
        if !self.aabb.contains(point) {
            return true;
        }
        let height = self.heightmap.sample_height(point.xz()) * self.max_height;

        height < point.y
    }

    pub fn intersect_with_ray(&mut self, ray: &Ray) -> Option<Vec3> {
        if let Some(hit) = ray.hits_aabb(&self.aabb) {
            self.debug.buffer_changed = true;
            assert!(self.debug.lines.len() < self.debug.lines_max);
            assert!(self.debug.points.len() < self.debug.points_max);
            self.debug.lines.push(ray.get_point_at(hit.t_min));
            self.debug.lines.push(ray.get_point_at(hit.t_max));

            let point = ray.get_point_at(hit.t_min);
            self.debug.points.push(Vec3::new(1.0, 0.0, 0.0)); // red
            self.debug.points.push(point);
            if !self.is_point_above_surface(&point) {
                // Definitely not intersecting, at least from above
                return None;
            }
            // March the ray to get the intersection point
            let step = 0.001f32.max((hit.t_max - hit.t_min) / 100.0);
            let mut t = hit.t_min;

            while t < hit.t_max {
                t += step;
                let point = ray.get_point_at(t);
                self.debug.points.push(point);
                if !self.is_point_above_surface(&point) {
                    // TODO: binary search inside and reduce the initial step count
                    self.debug.points.push(Vec3::new(0.0, 1.0, 0.0)); // green
                    return Some(point);
                }
                self.debug.points.push(Vec3::new(1.0, 0.0, 0.0)); // red
            }
        }
        None
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
