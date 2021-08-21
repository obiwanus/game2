use gl::types::*;

const MAX_ANISOTROPY: f32 = 8.0;

pub struct Texture {
    pub id: GLuint, // @tmp_public
}

impl Texture {
    pub fn new() -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
        }
        Texture { id }
    }

    pub fn bind_2d(&self, unit: i32) {
        unsafe {
            gl::ActiveTexture(unit_to_gl_const(unit));
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    pub fn set_default_parameters(self) -> Self {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as GLint,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);

            let anisotropy = {
                let mut value: f32 = 0.0;
                gl::GetFloatv(gl::MAX_TEXTURE_MAX_ANISOTROPY, &mut value);
                if value < MAX_ANISOTROPY {
                    value
                } else {
                    MAX_ANISOTROPY
                }
            };
            // println!("Using anisotropic filtering: {:?}", anisotropy);
            gl::TexParameterf(gl::TEXTURE_2D, gl::TEXTURE_MAX_ANISOTROPY, anisotropy);
        }
        self
    }

    pub fn set_image_2d(self, path: &str) -> Self {
        // TODO: proper error handling?
        let image = stb_image::load_u8(path, 3, true).unwrap();

        // Send pixels to GPU
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::SRGB8 as GLint,
                image.width as GLint,
                image.height as GLint,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                image.data.as_ptr() as *const std::ffi::c_void,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        self
    }
}

pub fn unit_to_gl_const(unit: i32) -> GLenum {
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
