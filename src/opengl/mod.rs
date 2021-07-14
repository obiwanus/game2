#![macro_use]
#![allow(dead_code)]

pub mod buffers;
pub mod shader;

pub fn gl_check_error(file: &str, line: u32) {
    let error_code = unsafe { gl::GetError() };
    if error_code != gl::NO_ERROR {
        let error = match error_code {
            gl::INVALID_ENUM => "INVALID_ENUM",
            gl::INVALID_VALUE => "INVALID_VALUE",
            gl::INVALID_OPERATION => "INVALID_OPERATION",
            gl::OUT_OF_MEMORY => "OUT_OF_MEMORY",
            gl::INVALID_FRAMEBUFFER_OPERATION => "INVALID_FRAMEBUFFER_OPERATION",
            _ => "unknown GL error code",
        };

        panic!("{} | {} ({})", error, file, line);
    }
}

#[allow(unused_macros)]
macro_rules! gl_check_error {
    () => {
        gl_check_error(file!(), line!())
    };
}
