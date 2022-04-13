use std::cmp::max;
use std::os::raw::c_uint;
use std::ptr::null;

use gl33::*;
use gl33::global_loader::*;
use glutin::dpi::Pixel;

use crate::types::{GLboolean, GLfloat, GLint, GLsizei, GLuint, GLvoid};

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
    mode: CullFaceMode,
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
    mask: GLuint,
}

struct StencilState {
    sub_state: StencilSubState,
    mask: GLuint,
    sfail: StencilOp,
    dpfail: StencilOp,
    dppass: StencilOp,
}

struct Viewport {
    x: GLint,
    y: GLint,
    width: GLsizei,
    height: GLsizei,
}

// red, green, blue, alpha
pub struct ColorMask(GLboolean, GLboolean, GLboolean, GLboolean);

static mut ACTIVE_TEXTURE: usize = 0;
static mut TEXTURES: [Texture2DState; 12] = [Texture2DState { cap_state: false, bound: Some(GL_ZERO) }; 12];
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
    dst_factor_alpha: GL_ZERO,
};
static mut CULL_FACE_STATE: CullFaceState = CullFaceState {
    cap: CapTracker(GL_CULL_FACE, false),
    mode: GL_BACK,
};
static mut POLYGON_OFFSET_STATE: PolygonOffsetState = PolygonOffsetState {
    cap_fill: CapTracker(GL_POLYGON_OFFSET_FILL, false),
    cap_line: CapTracker(GL_POLYGON_OFFSET_LINE, false),
    factor: 0.0,
    units: 0.0,
};
static mut LOGIC_OP_STATE: LogicOpState = LogicOpState {
    cap: CapTracker(GL_LOGIC_OP_MODE, false),
    op: GL_COPY,
};

static mut STENCIL_STATE: StencilState = StencilState {
    sub_state: StencilSubState {
        func: GL_ALWAYS,
        ref_: 0,
        mask: 0,
    },
    mask: 0,
    sfail: GL_KEEP,
    dpfail: GL_KEEP,
    dppass: GL_KEEP,
};
static mut VIEWPORT: Viewport = Viewport {
    x: 0,
    y: 0,
    width: 0,
    height: 0,
};
static mut COLOR_MASK: ColorMask = ColorMask(1, 1, 1, 1);

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

pub unsafe fn delete_textures(textures: Vec<TextureUnit>) {
    TEXTURES.iter_mut().for_each(|v| {
        if let Some(value) = v.bound {
            textures.iter().for_each(|t| {
                if value.0 == t.0 {
                    v.bound = None
                }
            });
        }
    });
    let va: Vec<GLuint> = textures.iter().map(|v| v.0).collect();
    glDeleteTextures(va.len() as GLsizei, va.as_ptr());
}

pub unsafe fn set_active_texture(texture: TextureUnit) {
    let ts = (texture.0 - 0x84C0 /* GL_TEXTURE0 */) as usize;
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

#[inline]
pub unsafe fn disable_scissor_test() { SCISSOR_TEST_STATE.set_state(false) }

#[inline]
pub unsafe fn enable_scissor_test() { SCISSOR_TEST_STATE.set_state(false) }

#[inline]
pub unsafe fn disable_depth_test() { DEPTH_TEST_STATE.cap.set_state(false) }

#[inline]
pub unsafe fn enable_depth_test() { DEPTH_TEST_STATE.cap.set_state(false) }

pub unsafe fn depth_func(func: DepthFunction) {
    if func != DEPTH_TEST_STATE.func {
        DEPTH_TEST_STATE.func = func;
        glDepthFunc(func);
    }
}

pub unsafe fn depth_mask(mask: bool) {
    if mask != DEPTH_TEST_STATE.mask {
        DEPTH_TEST_STATE.mask = mask;
        glDepthMask(mask as GLboolean)
    }
}

#[inline]
pub unsafe fn disable_blend() { BLEND_FUNC_STATE.cap.set_state(false) }

#[inline]
pub unsafe fn enable_blend() { BLEND_FUNC_STATE.cap.set_state(true) }

pub unsafe fn blend_func(src_factor: BlendingFactor, dst_factor: BlendingFactor) {
    if src_factor != BLEND_FUNC_STATE.src_factor_rgb || dst_factor != BLEND_FUNC_STATE.dst_factor_rgb {
        BLEND_FUNC_STATE.src_factor_rgb = src_factor;
        BLEND_FUNC_STATE.dst_factor_rgb = dst_factor;
        glBlendFunc(src_factor, dst_factor)
    }
}

pub unsafe fn blend_func_separate(
    src_factor_rgb: BlendingFactor,
    dst_factor_rgb: BlendingFactor,
    src_factor_alpha: BlendingFactor,
    dst_factor_alpha: BlendingFactor,
) {
    if src_factor_rgb != BLEND_FUNC_STATE.src_factor_rgb
        || dst_factor_rgb != BLEND_FUNC_STATE.dst_factor_rgb
        || src_factor_alpha != BLEND_FUNC_STATE.src_factor_alpha
        || dst_factor_alpha != BLEND_FUNC_STATE.dst_factor_alpha {
        BLEND_FUNC_STATE.src_factor_rgb = src_factor_rgb;
        BLEND_FUNC_STATE.dst_factor_rgb = dst_factor_rgb;
        BLEND_FUNC_STATE.src_factor_alpha = src_factor_alpha;
        BLEND_FUNC_STATE.dst_factor_alpha = dst_factor_alpha;
        glBlendFuncSeparate(src_factor_rgb, dst_factor_rgb, src_factor_alpha, dst_factor_alpha);
    }
}

pub unsafe fn delete_buffers(buffer: GLuint) {
    if cfg!(target_os = "linux") {
        glBindBuffer(GL_ARRAY_BUFFER, buffer);
        let v: [u8; 0] = [];
        glBufferData(GL_ARRAY_BUFFER, 0, v.as_ptr().cast(), GL_DYNAMIC_DRAW);
        glBindBuffer(GL_ARRAY_BUFFER, 0);
    }
    glDeleteBuffers(1, &buffer);
}

#[inline]
pub unsafe fn enable_cull() { CULL_FACE_STATE.cap.set_state(true); }

#[inline]
pub unsafe fn disable_cull() { CULL_FACE_STATE.cap.set_state(false); }

pub unsafe fn polygon_offset(factor: GLfloat, units: GLfloat) {
    if factor != POLYGON_OFFSET_STATE.factor || units != POLYGON_OFFSET_STATE.units {
        POLYGON_OFFSET_STATE.factor = factor;
        POLYGON_OFFSET_STATE.units = units;
        glPolygonOffset(factor, units)
    }
}

#[inline]
pub unsafe fn enable_color_logic_op() { LOGIC_OP_STATE.cap.set_state(true) }

#[inline]
pub unsafe fn disable_color_logic_op() { LOGIC_OP_STATE.cap.set_state(false) }

pub unsafe fn logic_op(op: LogicOp) {
    if op != LOGIC_OP_STATE.op {
        LOGIC_OP_STATE.op = op;
        glLogicOp(op);
    }
}

pub unsafe fn viewport(x: GLint, y: GLint, width: GLsizei, height: GLsizei) {
    VIEWPORT.x = x;
    VIEWPORT.y = y;
    VIEWPORT.width = width;
    VIEWPORT.height = height;
    glViewport(x, y, width, height)
}

pub unsafe fn color_mask(red: GLboolean, green: GLboolean, blue: GLboolean, alpha: GLboolean) {
    if red != COLOR_MASK.0 ||
        green != COLOR_MASK.1 ||
        blue != COLOR_MASK.2 ||
        alpha != COLOR_MASK.3 {
        COLOR_MASK.0 = red;
        COLOR_MASK.1 = green;
        COLOR_MASK.2 = blue;
        COLOR_MASK.3 = alpha;
        glColorMask(red, green, blue, alpha)
    }
}

pub unsafe fn stencil_func(func: StencilFunction, ref_: GLint, mask: GLuint) {
    let s = &mut STENCIL_STATE.sub_state;
    if s.func != func || s.ref_ != ref_ || s.mask != mask {
        s.func = func;
        s.ref_ = ref_;
        s.mask = mask;
        glStencilFunc(func, ref_, mask)
    }
}

pub unsafe fn stencil_mask(mask: GLuint) {
    if mask != STENCIL_STATE.mask {
        STENCIL_STATE.mask = mask;
        glStencilMask(mask);
    }
}

pub unsafe fn stencil_op(sfail: StencilOp, dpfail: StencilOp, dppass: StencilOp) {
    if sfail != STENCIL_STATE.sfail || dpfail != STENCIL_STATE.dpfail || dppass != STENCIL_STATE.dppass {
        STENCIL_STATE.sfail = sfail;
        STENCIL_STATE.dpfail = dpfail;
        STENCIL_STATE.dppass = dppass;
        glStencilOp(sfail, dpfail, dppass);
    }
}