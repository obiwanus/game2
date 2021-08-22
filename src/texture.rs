use gl::types::GLenum;

pub fn calculate_mip_levels(width: usize, height: usize) -> i32 {
    let dimension = width.max(height) as f32;
    dimension.log2().floor() as i32 + 1
}

pub fn get_max_anisotropy() -> f32 {
    const MAX_ANISOTROPY: f32 = 8.0;
    let mut value: f32 = 0.0;
    unsafe {
        gl::GetFloatv(gl::MAX_TEXTURE_MAX_ANISOTROPY, &mut value);
    }
    if value < MAX_ANISOTROPY {
        value
    } else {
        MAX_ANISOTROPY
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
