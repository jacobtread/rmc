use crate::render::VertexFormat;
use gl33::global_loader::*;
use gl33::*;
use glutin::dpi::LogicalSize;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{Api, ContextBuilder, GlRequest};

use crate::resources::{Identifier, Resources};
use crate::types::GLsizei;
use crate::window::Framebuffer;

pub struct Game {}

fn load_end() {
    let end_text_ident = Identifier::new("minecraft", "texts/end.txt");
    let end_text = Resources::get_utf8(&end_text_ident).expect("end texts missing");
    println!("{}", end_text);
}

impl Game {
    pub fn new() -> Game {
        Game {}
    }

    pub fn start(&mut self) {
        load_end();
        let el = EventLoop::new();
        let wb = WindowBuilder::new()
            .with_title("Rust MC")
            .with_inner_size(LogicalSize::new(854.0, 480.0));
        let context = ContextBuilder::new()
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
            .with_vsync(true)
            .build_windowed(wb, &el)
            .unwrap();
        let context = unsafe { context.make_current().unwrap() };
        unsafe {
            load_global_gl(&|ptr| {
                let c_str = std::ffi::CStr::from_ptr(ptr as *const i8);
                let r_str = c_str.to_str().unwrap();
                context.get_proc_address(r_str) as _
            });
        }

        let mut fb_size = context.window().inner_size();

        let fb = Framebuffer::new(fb_size.width as GLsizei, fb_size.height as GLsizei);

        el.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::LoopDestroyed => return,
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(physical_size) => context.resize(physical_size),
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => {
                        println!("{}", input.scancode);
                    }
                    _ => (),
                },
                Event::RedrawRequested(_) => unsafe {
                    glClearColor(1f32, 1f32, 1f32, 1f32);
                    glClear(GL_COLOR_BUFFER_BIT);
                    context.swap_buffers().unwrap();
                },
                _ => (),
            }
        });
    }
}
