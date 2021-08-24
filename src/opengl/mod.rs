#![macro_use]
#![allow(dead_code)]

use std::ffi::CStr;

use gl::types::*;

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

pub extern "system" fn debug_callback(
    source: GLenum,
    gltype: GLenum,
    id: GLuint,
    severity: GLenum,
    length: GLsizei,
    message: *const GLchar,
    user_param: *mut std::os::raw::c_void,
) {
    let msg_type = if gltype == gl::DEBUG_TYPE_ERROR {
        "** GL ERROR ** "
    } else {
        "** GL DEBUG **"
    };
    let msg = unsafe { CStr::from_ptr(message) };
    eprintln!("{} {}", msg_type, msg.to_str().unwrap().to_owned());
}

pub fn get_framebuffer_status_str(fbo: GLuint, target: GLenum) -> &'static str {
    let status = unsafe { gl::CheckNamedFramebufferStatus(fbo, target) };
    match status {
        gl::FRAMEBUFFER_COMPLETE => "FRAMEBUFFER_COMPLETE",
        gl::FRAMEBUFFER_UNDEFINED => "FRAMEBUFFER_UNDEFINED",
        gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => "FRAMEBUFFER_INCOMPLETE_ATTACHMENT",
        gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => {
            "FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT"
        }
        gl::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => "FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER",
        gl::FRAMEBUFFER_INCOMPLETE_READ_BUFFER => "FRAMEBUFFER_INCOMPLETE_READ_BUFFER",
        gl::FRAMEBUFFER_UNSUPPORTED => "FRAMEBUFFER_UNSUPPORTED",
        gl::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => "FRAMEBUFFER_INCOMPLETE_MULTISAMPLE",
        gl::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS => "FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS",
        _ => "Unknown",
    }
}
