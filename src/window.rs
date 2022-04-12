use std::cmp::max;
use std::ptr::null;

use gl33::*;
use gl33::global_loader::*;

use crate::texture::{bind_texture, gen_texture_id, max_supported_texture_size, unbind_texture};
use crate::types::{GLsizei, GLuint};

#[derive(Debug, Clone, Copy)]
struct Size(GLsizei, GLsizei);

pub struct Framebuffer {
    size: Size,
    texture_width: GLsizei,
    texture_height: GLsizei,
    viewport_width: GLsizei,
    viewport_height: GLsizei,
    color_attachment: Option<TextureUnit>,
    depth_attachment: Option<TextureUnit>,
    fbo: Option<GLuint>,
    clear_color: [f32; 4],
}

impl Framebuffer {
    const DEFAULT_WIDTH: GLsizei = 845;
    const DEFAULT_HEIGHT: GLsizei = 480;
    const DEFAULT_SIZE: Size = Size(Framebuffer::DEFAULT_WIDTH, Framebuffer::DEFAULT_HEIGHT);

    pub fn new(width: GLsizei, height: GLsizei) -> Framebuffer {
        let mut framebuffer = Framebuffer {
            size: Framebuffer::DEFAULT_SIZE,
            texture_width: 0,
            texture_height: 0,
            viewport_width: 0,
            viewport_height: 0,
            color_attachment: None,
            depth_attachment: None,
            fbo: None,
            clear_color: [1.0, 1.0, 1.0, 1.0],
        };
        unsafe {
            framebuffer.set_suitable_size(width, height);
            let mut fbo = 0;
            glGenFramebuffers(1, &mut fbo);
            framebuffer.fbo = Some(fbo);
            glBindFramebuffer(GL_FRAMEBUFFER, fbo);

            bind_texture(framebuffer.color_attachment.unwrap());
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, 0x2600);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, 0x2600);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, 0x812f);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, 0x812f);
            glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, framebuffer.color_attachment.unwrap().0, 0);
            bind_texture(framebuffer.depth_attachment.unwrap());
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_COMPARE_MODE, 0);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, 0x2600);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, 0x2600);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, 0x812f);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, 0x812f);
            glFramebufferTexture2D(GL_FRAMEBUFFER, GL_DEPTH_ATTACHMENT, GL_TEXTURE_2D, framebuffer.depth_attachment.unwrap().0, 0);
            unbind_texture();
            framebuffer.viewport_width = framebuffer.size.0;
            framebuffer.viewport_width = framebuffer.size.1;
            framebuffer.texture_width = framebuffer.size.1;
            framebuffer.texture_height = framebuffer.size.1;
            let status = glCheckFramebufferStatus(GL_FRAMEBUFFER);
            if status != GL_FRAMEBUFFER_COMPLETE {
                eprintln!("Framebuffer error status {:?}", status);
                panic!();;
            }
            glBindFramebuffer(GL_FRAMEBUFFER, 0)
        }
        framebuffer
    }


    unsafe fn is_compatible(&self, size: Size) -> bool {
        self.supports_color(&size) && self.supports_depth(&size)
    }

    unsafe fn set_suitable_size(&mut self, width: GLsizei, height: GLsizei) -> Size {
        let mut size = self.size.clone();
        self.color_attachment = Some(gen_texture_id());
        self.depth_attachment = Some(gen_texture_id());
        let max_size = max_supported_texture_size();
        if width > 0 && width <= max_size && height > 0 && height <= max_size {
            let fsize = Size(width, height);
            if self.is_compatible(fsize) {
                return fsize;
            }
        }
        if !self.is_compatible(size) {
            panic!("No compatible framebuffer size")
        }
        size
    }

    unsafe fn supports_color(&self, size: &Size) -> bool {
        glGetError();
        bind_texture(self.color_attachment.unwrap());
        glTexImage2D(GL_TEXTURE_2D, 0, 0x8058, size.0, size.1, 0, GL_RGBA, GL_UNSIGNED_BYTE, null());
        return glGetError() != GL_OUT_OF_MEMORY;
    }
    unsafe fn supports_depth(&self, size: &Size) -> bool {
        glGetError();
        bind_texture(self.depth_attachment.unwrap());
        glTexImage2D(GL_TEXTURE_2D, 0, 0x1902, size.0, size.1, 0, GL_DEPTH_COMPONENT, GL_FLOAT, null());
        return glGetError() != GL_OUT_OF_MEMORY;
    }
}