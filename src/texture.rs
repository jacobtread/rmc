use std::cmp::max;
use std::os::raw::c_uint;
use std::ptr::null;

use gl33::*;
use gl33::global_loader::*;

use crate::types::{GLboolean, GLfloat, GLint, GLsizei, GLuint};

pub struct CapTracker(EnableCap, bool);

impl CapTracker {
    pub fn set_state(&mut self, state: bool) {
        if state != self.1 {
            self.1 = state;
            unsafe {
                if state {
                    glEnable(self.0)
                } else {
                    glDisable(self.0)
                }
            }
        }
    }
}

struct DepthTestState {
    cap: CapTracker,
    mask: bool,
    func: DepthFunction,
}

struct BlendFuncState {
    cap: CapTracker,
    src_factor_rgb: BlendingFactor,
    dst_factor_rgb: BlendingFactor,
    src_factor_alpha: BlendingFactor,
    dst_factor_alpha: BlendingFactor,
}

struct CullFaceState {
    cap: CapTracker,
    mode: CullFaceMode
}

struct PolygonOffsetState {
    cap_fill: CapTracker,
    cap_line: CapTracker,
    factor: GLfloat,
    units: GLfloat,
}

struct LogicOpState {
    cap: CapTracker,
    op: LogicOp,
}

#[derive(Debug, Clone, Copy)]
struct Texture2DState {
    cap_state: bool,
    bound: Option<TextureUnit>,
}

struct StencilSubState {
    func: StencilFunction,
    ref_: GLint,
    mask: GLint,
}

struct StencilState {
    sub_state: StencilSubState,
    mask: GLuint,
    sfail: StencilOp,
    dpfail: StencilOp,
    dppass: StencilOp,
}

fn test() {
    glStencil
}

// red, green, blue, alpha
pub struct ColorMask(GLboolean, GLboolean, GLboolean, GLboolean);

static mut ACTIVE_TEXTURE: usize = 0;
static mut TEXTURES: [Texture2DState; 12] = [Texture2DState { cap_state: false, bound: None }; 12];
static mut MAX_SUPPORTED_TEXTURE_SIZE: Option<GLsizei> = None;

static mut SCISSOR_TEST_STATE: CapTracker = CapTracker(GL_SCISSOR_TEST, false);
static mut DEPTH_TEST_STATE: DepthTestState = DepthTestState {
    cap: CapTracker(GL_DEPTH_TEST, false),
    mask: true,
    func: GL_LESS,
};
static mut BLEND_FUNC_STATE: BlendFuncState = BlendFuncState {
    cap: CapTracker(GL_BLEND, false),
    src_factor_rgb: GL_ONE,
    dst_factor_rgb: GL_ZERO,
    src_factor_alpha: GL_ONE,
    dst_factor_alpha: GL_ZERO
};
static mut CULL_FACE_STATE: CullFaceState = CullFaceState {
    cap: CapTracker(GL_CULL_FACE, false),
    mode: GL_BACK
};
static mut POLYGON_OFFSET_STATE: PolygonOffsetState = PolygonOffsetState {
    cap_fill: CapTracker(GL_POLYGON_OFFSET_FILL, false),
    cap_line: CapTracker(GL_POLYGON_OFFSET_LINE, false),
    factor: 0.0,
    units: 0.0
};
static mut LOGIC_OP_STATE: LogicOpState = LogicOpState {
    cap: CapTracker(GL_LOGIC_OP_MODE, false),
    op: GL_COPY,
};

static mut STENCIL_STATE: StencilState = StencilState {
    sub_state: StencilSubState {
        func: GL_ALWAYS,
        ref_: 0,
        mask: -1
    },
    mask: -1,
    sfail: GL_KEEP,
    dpfail: GL_KEEP,
    dppass: GL_KEEP
};


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