use gl::types::*;
use glam::{Mat4, Vec3Swizzles};
use glam::{Vec2, Vec3};

use crate::opengl::get_framebuffer_status_str;
use crate::texture::{calculate_mip_levels, get_max_anisotropy, unit_to_gl_const};
use crate::{
    opengl::shader::Program,
    ray::{Ray, AABB},
    utils::vec2_infinity,
    Result,
};
use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};

struct Heightmap {
    texture: GLuint,
    texture_size: usize,

    pixels: Vec<f32>, // @speed: store efficiently?

    // For drawing on heightmap
    fbo: GLuint,
    shader: Program,
}

impl Heightmap {
    fn new(path: &str) -> Result<Self> {
        let image = stb_image::load_f32(path, 1, false).unwrap();
        assert_eq!(
            image.width, image.height,
            "Only square heightmaps are supported"
        );
        let texture_size = image.width;

        let mut texture: GLuint = 0;
        unsafe {
            gl::CreateTextures(gl::TEXTURE_2D, 1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TextureParameteri(texture, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl::TextureParameteri(texture, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
            gl::TextureParameteri(texture, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TextureParameteri(texture, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl::TextureStorage2D(
                texture,
                1,
                gl::R16,
                texture_size as i32,
                texture_size as i32,
            );
            gl::TextureSubImage2D(
                texture,
                0,
                0,
                0,
                texture_size as i32,
                texture_size as i32,
                gl::RED,
                gl::FLOAT,
                image.data.as_ptr() as *const _,
            );
        }

        // Framebuffer object for rendering to heightmap
        let mut fbo: GLuint = 0;
        unsafe {
            gl::CreateFramebuffers(1, &mut fbo);
            gl::NamedFramebufferTexture(fbo, gl::COLOR_ATTACHMENT0, texture, 0);
            let draw_buffers = [gl::COLOR_ATTACHMENT0];
            gl::NamedFramebufferDrawBuffers(fbo, 1, draw_buffers.as_ptr() as *const _);
            assert_eq!(
                gl::CheckNamedFramebufferStatus(fbo, gl::FRAMEBUFFER),
                gl::FRAMEBUFFER_COMPLETE,
                "Heightmap texture framebuffer is incomplete",
            );
        }

        let shader = Program::new()
            .vertex_shader(include_str!("shaders/editor/terrain/heightmap.vert"))?
            .fragment_shader(include_str!("shaders/editor/terrain/heightmap.frag"))?
            .link()?;

        Ok(Heightmap {
            texture,
            texture_size,

            pixels: image.data,

            fbo,
            shader,
        })
    }

    fn sample_height(&self, point: Vec2) -> f32 {
        let size = self.texture_size as f32;

        // @speed/@correctness: 0.99999 is chosen arbitrarily and may not be very precise.
        // Also, I'm not sure how fast the clamping is (and it's not precise at all for heightmaps).
        // Still, it should be ok for picking (but probably not OK for anything else)
        let x = (point.x.clamp(0.0, 0.99999) * size) as usize;
        let y = (point.y.clamp(0.0, 0.99999) * size) as usize;

        self.pixels[y * self.texture_size + x]
    }

    fn draw_on_heightmap(
        &self,
        cursor: Vec2,
        brush: &Brush,
        terrain_size: f32,
        delta_time: f32,
        raise: bool,
    ) {
        self.shader.set_used();
        debug_assert!(cursor.x <= 1.0 && cursor.x >= 0.0);
        debug_assert!(cursor.y <= 1.0 && cursor.y >= 0.0);
        self.shader.set_vec2("cursor", &cursor).unwrap();
        let brush_size = brush.size as f32 / terrain_size;
        self.shader.set_f32("brush_size", brush_size).unwrap();
        self.shader.set_f32("delta_time", delta_time).unwrap();

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
            gl::Viewport(0, 0, self.texture_size as i32, self.texture_size as i32);

            gl::ActiveTexture(unit_to_gl_const(0));
            gl::BindTexture(gl::TEXTURE_2D, brush.texture);

            gl::Enable(gl::BLEND);
            gl::Disable(gl::DEPTH_TEST);

            gl::BlendFunc(gl::ONE, gl::ONE);
            gl::BlendEquation(if raise {
                gl::FUNC_ADD
            } else {
                gl::FUNC_REVERSE_SUBTRACT
            });

            gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
            gl::MemoryBarrier(gl::FRAMEBUFFER_BARRIER_BIT); // not critical

            // Reset everything back
            gl::Disable(gl::BLEND);
            gl::Enable(gl::DEPTH_TEST);
            gl::BlendEquation(gl::FUNC_ADD);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Viewport(0, 0, WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32);
        }
    }
}

pub struct Brush {
    texture: GLuint,
    texture_size: usize,
    pixels: Vec<f32>, // @speed: swizzle?
    pub size: f32,
}

impl Brush {
    pub fn new(path: &str, size: f32) -> Self {
        let image = stb_image::load_f32(path, 1, false).unwrap();
        assert_eq!(
            image.width, image.height,
            "Only square brushes are supported"
        );
        let texture_size = image.width;

        let mut texture: GLuint = 0;
        unsafe {
            gl::CreateTextures(gl::TEXTURE_2D, 1, &mut texture);
            gl::TextureParameteri(texture, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as GLint);
            gl::TextureParameteri(texture, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as GLint);
            gl::TextureParameteri(
                texture,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as GLint,
            );
            gl::TextureParameteri(texture, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl::TextureStorage2D(
                texture,
                calculate_mip_levels(texture_size, texture_size),
                gl::R16,
                texture_size as i32,
                texture_size as i32,
            );
            gl::TextureSubImage2D(
                texture,
                0,
                0,
                0,
                texture_size as i32,
                texture_size as i32,
                gl::RED,
                gl::FLOAT,
                image.data.as_ptr() as *const _,
            );
            gl::GenerateTextureMipmap(texture);
        }

        Brush {
            texture,
            size,
            pixels: image.data,
            texture_size,
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

    texture: GLuint,
    heightmap: Heightmap,

    pub cursor: Vec2,
    pub brush: Brush,

    shadow_map_fbo: GLuint,
    shadow_map: GLuint,
    shadow_map_width: i32,
    shadow_map_height: i32,
    shadow_map_shader: Program,

    debug: TerrainDebug,
}

struct TerrainDebug {
    aabb_shader: Program,
}

impl Terrain {
    pub fn new(center: Vec2) -> Result<Self> {
        // TODO: support centers other than 0, 0
        // (currently hard-coded in terrain.vert.glsl)
        assert_eq!(center, Vec2::new(0.0, 0.0));
        const NUM_PATCHES: f32 = 64.0;
        const PATCH_SIZE: f32 = 16.0;
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
        }

        let texture = {
            let image = stb_image::load_u8("textures/checkerboard.png", 3, true).unwrap();
            assert_eq!(image.width, image.height);

            let mut texture: GLuint = 0;
            unsafe {
                gl::CreateTextures(gl::TEXTURE_2D, 1, &mut texture);
                gl::TextureParameteri(texture, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
                gl::TextureParameteri(texture, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
                gl::TextureParameteri(
                    texture,
                    gl::TEXTURE_MIN_FILTER,
                    gl::LINEAR_MIPMAP_LINEAR as GLint,
                );
                gl::TextureParameteri(texture, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
                gl::TextureParameterf(texture, gl::TEXTURE_MAX_ANISOTROPY, get_max_anisotropy());
                gl::TextureStorage2D(
                    texture,
                    calculate_mip_levels(image.width, image.height),
                    gl::SRGB8,
                    image.width as i32,
                    image.height as i32,
                );
                gl::TextureSubImage2D(
                    texture,
                    0,
                    0,
                    0,
                    image.width as i32,
                    image.height as i32,
                    gl::RGB,
                    gl::UNSIGNED_BYTE,
                    image.data.as_ptr() as *const _,
                );
                gl::GenerateTextureMipmap(texture);
            }

            texture
        };

        let cursor = vec2_infinity();
        let heightmap = Heightmap::new("textures/heightmaps/valley.png")?;
        let brush = Brush::new("textures/brushes/mountain04.png", 100.0);

        let shader = Program::new()
            .vertex_shader(include_str!("shaders/editor/terrain/terrain.vert.glsl"))?
            .tess_control_shader(include_str!("shaders/editor/terrain/terrain.tc.glsl"))?
            .tess_evaluation_shader(include_str!("shaders/editor/terrain/terrain.te.glsl"))?
            .fragment_shader(include_str!("shaders/editor/terrain/terrain.frag.glsl"))?
            .link()?;

        // Shadow map
        let mut shadow_map_fbo: GLuint = 0;
        let mut shadow_map: GLuint = 0;
        let (shadow_map_width, shadow_map_height) = (2048, 2048);
        unsafe {
            gl::CreateFramebuffers(1, &mut shadow_map_fbo);
            gl::CreateTextures(gl::TEXTURE_2D, 1, &mut shadow_map);
            gl::TextureParameteri(shadow_map, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TextureParameteri(shadow_map, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TextureParameteri(shadow_map, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TextureParameteri(shadow_map, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TextureStorage2D(
                shadow_map,
                1,
                gl::DEPTH_COMPONENT16,
                shadow_map_width,
                shadow_map_height,
            );
            gl::NamedFramebufferTexture(shadow_map_fbo, gl::DEPTH_ATTACHMENT, shadow_map, 0);
            gl::NamedFramebufferDrawBuffer(shadow_map_fbo, gl::NONE);
            gl::NamedFramebufferReadBuffer(shadow_map_fbo, gl::NONE);

            assert_eq!(
                gl::CheckNamedFramebufferStatus(shadow_map_fbo, gl::FRAMEBUFFER),
                gl::FRAMEBUFFER_COMPLETE,
                "Shadow map framebuffer is incomplete",
            );
        }
        let shadow_map_shader = Program::new()
            .vertex_shader(include_str!("shaders/editor/terrain/terrain.vert.glsl"))?
            .tess_control_shader(include_str!("shaders/editor/terrain/terrain.tc.glsl"))?
            .tess_evaluation_shader(include_str!("shaders/editor/terrain/shadow.te.glsl"))?
            .fragment_shader(include_str!("shaders/editor/terrain/shadow.frag.glsl"))?
            .link()?;

        let debug = {
            let aabb_shader = Program::new()
                .vertex_shader(include_str!("shaders/debug/aabb.vert"))?
                .fragment_shader(include_str!("shaders/debug/aabb.frag"))?
                .link()?;
            aabb_shader.set_used();
            aabb_shader.set_vec3("aabb_min", &aabb.min)?;
            aabb_shader.set_vec3("aabb_max", &aabb.max)?;

            TerrainDebug { aabb_shader }
        };

        Ok(Terrain {
            center,
            max_height,
            aabb,

            vao,
            shader,
            tess_level: 10.0,

            texture,
            heightmap,

            cursor,
            brush,

            shadow_map_fbo,
            shadow_map,
            shadow_map_width,
            shadow_map_height,
            shadow_map_shader,

            debug,
        })
    }

    // TODO: use a renderer
    pub fn draw(&mut self, time: f32) -> Result<()> {
        // Set common stuff for shadow pass / render pass
        unsafe {
            gl::PatchParameteri(gl::PATCH_VERTICES, 4);
            gl::BindVertexArray(self.vao);

            // Default texture
            gl::ActiveTexture(unit_to_gl_const(0));
            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            // Heightmap
            gl::ActiveTexture(unit_to_gl_const(1));
            gl::BindTexture(gl::TEXTURE_2D, self.heightmap.texture);

            // Brush
            gl::ActiveTexture(unit_to_gl_const(2));
            gl::BindTexture(gl::TEXTURE_2D, self.brush.texture);

            // Shadow map
            gl::ActiveTexture(unit_to_gl_const(3));
            gl::BindTexture(gl::TEXTURE_2D, self.shadow_map);
        }

        // Draw into shadow map
        self.shadow_map_shader.set_used();
        self.shadow_map_shader
            .set_f32("tess_level", self.tess_level)?;
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.shadow_map_fbo);
            gl::Viewport(0, 0, self.shadow_map_width, self.shadow_map_height);
            gl::Clear(gl::DEPTH_BUFFER_BIT);

            gl::DrawArraysInstanced(gl::PATCHES, 0, 4, 64 * 64);

            gl::Viewport(0, 0, WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        // Draw the scene
        self.shader.set_used();
        self.shader.set_vec2("cursor", &self.cursor)?;
        self.shader.set_f32("brush_size", self.brush.size)?;
        self.shader.set_f32("tess_level", self.tess_level)?;

        unsafe {
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

            debug.aabb_shader.set_used();
            unsafe {
                gl::DrawArrays(gl::LINE_STRIP, 0, 16);
            }
        }

        Ok(())
    }

    pub fn size(&self) -> f32 {
        self.aabb.max.x - self.aabb.min.x
    }

    pub fn shape_terrain(&mut self, delta_time: f32, raise: bool) {
        let terrain_size = self.size();
        let cursor = (self.cursor - self.aabb.min.xz()) / terrain_size;
        self.heightmap
            .draw_on_heightmap(cursor, &self.brush, terrain_size, delta_time, raise);
    }

    pub fn is_point_above_surface(&self, point: &Vec3) -> bool {
        if !self.aabb.contains(point) {
            return true;
        }
        let point_height = point.y;
        let terrain_size = self.aabb.max.x - self.aabb.min.x;
        let sample_point = (point.xz() - self.aabb.min.xz()) / terrain_size;
        let height = self.heightmap.sample_height(sample_point) * self.max_height;

        height < point_height
    }

    pub fn intersect_with_ray(&self, ray: &Ray) -> Option<Vec3> {
        if let Some(hit) = ray.hits_aabb(&self.aabb) {
            let point = ray.get_point_at(hit.t_min);
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
                if !self.is_point_above_surface(&point) {
                    // Use binary search to get a more precise intersection point
                    let point = {
                        const EPSILON: f32 = 0.001;
                        let mut new_point = point;
                        let mut low = t - step;
                        let mut high = t;
                        while (high - low) > EPSILON {
                            let new_t = low + (high - low) / 2.0;
                            new_point = ray.get_point_at(new_t);
                            if self.is_point_above_surface(&new_point) {
                                low = new_t;
                            } else {
                                high = new_t;
                            }
                        }
                        new_point
                    };
                    return Some(point);
                }
            }
        }
        None
    }
}

impl Drop for Terrain {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteTextures(1, &self.texture);
        }
    }
}
