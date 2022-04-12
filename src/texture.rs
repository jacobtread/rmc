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

#[derive(Debug)]
struct Textures {
    active_texture: TextureUnit,
    textures: [TextureState; 12],
    max_supported_texture_size: Option<GLsizei>,
}

const TEXTURES: Textures = Textures {
    active_texture: GL_ZERO,
    textures: [TextureState { cap_state: false, bound: None }; 12],
    max_supported_texture_size: None,
};

pub fn gen_texture_id() -> TextureUnit {
    let mut value = 0;
    unsafe { glGenTextures(1, &mut value); }
    GLenum(value)
}

pub fn bind_texture(texture: TextureUnit) {
    let at = &mut TEXTURES.textures[TEXTURES.active_texture.0 as usize];
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
    let at = &mut TEXTURES.textures[TEXTURES.active_texture.0 as usize];
    if let Some(value) = at.bound {
        at.bound = None;
        unsafe { glBindTexture(GL_TEXTURE_2D, 0); }
    }
}

pub fn delete_texture(texture: TextureUnit) {
    unsafe { glDeleteTextures(1, &texture.0) }
    TEXTURES.textures.iter_mut().for_each(|v| {
        if let Some(value) = v.bound {
            if value == texture {
                v.bound = None
            }
        }
    });
}

pub fn set_active_texture(texture: TextureUnit) {
    if TEXTURES.active_texture != texture {
        TEXTURES.active_texture = texture;
        unsafe { glActiveTexture(texture) }
    }
}

pub fn enable_texture() {
   TEXTURES.textures[TEXTURES.active_texture.0 as usize].cap_state = true;
}

pub fn disable_texture() {
    TEXTURES.textures[TEXTURES.active_texture.0 as usize].cap_state = false;
}

pub fn max_supported_texture_size() -> GLsizei {
    match TEXTURES.max_supported_texture_size {
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
            TEXTURES.max_supported_texture_size = Some(i);
            i
        }
        Some(value) => value
    }
}