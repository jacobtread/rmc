use std::cmp::max;
use std::os::raw::c_uint;
use std::ptr::null;

use gl33::{GL_MAX_TEXTURE_SIZE, GL_NONE, GL_PROXY_TEXTURE_2D, GL_RGBA, GL_TEXTURE0, GL_TEXTURE12, GL_TEXTURE_2D, GL_TEXTURE_WIDTH, GL_UNSIGNED_BYTE, GL_ZERO, GLenum, TextureUnit};
use gl33::global_loader::{glActiveTexture, glBindTexture, glDeleteTextures, glGenTextures, glGetIntegerv, glGetTexLevelParameterfv, glGetTexLevelParameteriv, glTexImage2D};

use crate::types::{GLsizei, GLuint};

#[derive(Debug, Clone, Copy)]
struct TextureState {
    cap_state: bool,
    bound: Option<TextureUnit>,
}

static mut ACTIVE_TEXTURE: usize = 0;
static mut TEXTURES: [TextureState; 12] = [TextureState { cap_state: false, bound: None }; 12];
static mut MAX_SUPPORTED_TEXTURE_SIZE: Option<GLsizei> = None;

pub fn gen_texture_id() -> TextureUnit {
    let mut value = 0;
    unsafe { glGenTextures(1, &mut value); }
    GLenum(value)
}

pub fn bind_texture(texture: TextureUnit) {
    let at = unsafe { &mut TEXTURES[ACTIVE_TEXTURE] };
    if let Some(value) = at.bound {
        if texture != value {
            at.bound = Some(texture);
            unsafe { glBindTexture(GL_TEXTURE_2D, texture.0) }
        }
    } else {
        at.bound = Some(texture);
        unsafe { glBindTexture(GL_TEXTURE_2D, texture.0) }
    }
}

pub fn unbind_texture() {
    let at = unsafe { &mut TEXTURES[ACTIVE_TEXTURE] };
    if let Some(value) = at.bound {
        at.bound = None;
        unsafe { glBindTexture(GL_TEXTURE_2D, 0); }
    }
}

pub unsafe fn delete_texture(texture: TextureUnit) {
    glDeleteTextures(1, &texture.0);
    TEXTURES.iter_mut().for_each(|v| {
        if let Some(value) = v.bound {
            if value == texture {
                v.bound = None
            }
        }
    });
}

pub unsafe fn set_active_texture(texture: TextureUnit) {
    let ts = texture.0 as usize;
    if ACTIVE_TEXTURE != ts {
        ACTIVE_TEXTURE = ts;
        glActiveTexture(texture);
    }
}

pub unsafe fn enable_texture() {
    TEXTURES[ACTIVE_TEXTURE].cap_state = true;
}

pub unsafe fn disable_texture() {
    TEXTURES[ACTIVE_TEXTURE].cap_state = false;
}

pub unsafe fn max_supported_texture_size() -> GLsizei {
    match MAX_SUPPORTED_TEXTURE_SIZE {
        None => {
            let mut max_size = 0;
            unsafe { glGetIntegerv(GL_MAX_TEXTURE_SIZE, &mut max_size); }
            let mut i = max(32768, max_size);
            while i >= 1024 {
                unsafe { glTexImage2D(GL_PROXY_TEXTURE_2D, 0, 0x1908, i, i, 0, GL_RGBA, GL_UNSIGNED_BYTE, null()); }
                let mut width = 0;
                unsafe { glGetTexLevelParameteriv(GL_PROXY_TEXTURE_2D, 0, GL_TEXTURE_WIDTH, &mut width); }
                if width != 0 {
                    i = width;
                    break;
                }
                i >>= 1;
            }

            i = max(i, 1024);
            MAX_SUPPORTED_TEXTURE_SIZE = Some(i);
            i
        }
        Some(value) => value
    }
}