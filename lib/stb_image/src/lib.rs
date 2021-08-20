#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
pub mod bindings {
    include!("bindings.rs");
}

use std::error::Error;
use std::ffi::CString;
use std::slice;

use libc::{c_int, c_void};

use bindings::*;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub struct Image<T> {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub data: Vec<T>,
}

impl<T> Image<T> {
    fn new(width: c_int, height: c_int, depth: c_int, data: Vec<T>) -> Image<T> {
        Image {
            width: width as usize,
            height: height as usize,
            depth: depth as usize,
            data: data,
        }
    }
}

pub fn load_u8(path: &str, force_depth: usize, flip: bool) -> Result<Image<u8>> {
    let mut width: c_int = 0;
    let mut height: c_int = 0;
    let mut depth: c_int = 0;
    let force_depth = force_depth as c_int;

    let path_cstring = CString::new(path)?;
    let flip = if flip { 1 } else { 0 };

    unsafe {
        stbi_set_flip_vertically_on_load(flip);

        let buffer = stbi_load(
            path_cstring.as_ptr(),
            &mut width,
            &mut height,
            &mut depth,
            force_depth,
        );
        if buffer.is_null() {
            return Err(format!("Couldn't load image {}", path).into());
        }
        let actual_depth = if force_depth != 0 { force_depth } else { depth };
        let len = (width * height * actual_depth) as usize;
        let data = slice::from_raw_parts(buffer, len).to_vec();
        stbi_image_free(buffer as *mut c_void);

        Ok(Image::new(width, height, depth, data))
    }
}

pub fn load_f32(path: &str, force_depth: usize, flip: bool) -> Result<Image<f32>> {
    let mut width: c_int = 0;
    let mut height: c_int = 0;
    let mut depth: c_int = 0;
    let force_depth = force_depth as c_int;

    let path_cstring = CString::new(path)?;
    let flip = if flip { 1 } else { 0 };

    unsafe {
        stbi_set_flip_vertically_on_load(flip);

        let buffer = stbi_loadf(
            path_cstring.as_ptr(),
            &mut width,
            &mut height,
            &mut depth,
            force_depth,
        );
        if buffer.is_null() {
            return Err(format!("Couldn't load image {}", path).into());
        }
        let actual_depth = if force_depth != 0 { force_depth } else { depth };
        let len = (width * height * actual_depth) as usize;
        let data = slice::from_raw_parts(buffer, len).to_vec();
        stbi_image_free(buffer as *mut c_void);

        Ok(Image::new(width, height, depth, data))
    }
}
