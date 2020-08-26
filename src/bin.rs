
use glutin::PossiblyCurrent;
use ruckus::opengl::*;
use ruckus::vertex::*;
use ruckus::graphics::*;
use ruckus::buffers::*;
use std::ffi::CStr;

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;
static mut GL_CONTEXT: Option<gl::Gl> = None;

const VS_SRC: &'static [u8] = b"
#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTex;
layout (location = 2) in vec4 aColor;

out vec4 TriColor;

void main() {
    gl_Position = vec4(aPos, 1.0);
    TriColor = aColor;
}
\0";

const FS_SRC: &'static [u8] = b"
#version 330 core
out vec4 FragColor;
in vec4 TriColor;
void main() {
    FragColor = TriColor;
}
\0";

#[macro_export]
macro_rules! memory_offset {
    ($ty:ty, $field:ident) => {
        unsafe { &((*(0 as *const $ty)).$field) as *const _ as usize }
    };
}

fn main() {
    unsafe {
        let el = EventLoop::new();
        let wb = WindowBuilder::new().with_title("A fantastic window!");
    
        let windowed_context =
            ContextBuilder::new()
            .with_double_buffer(Some(true))
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)))
            .with_gl_profile(glutin::GlProfile::Core)
            .build_windowed(wb, &el)           
            .unwrap();
    
        let windowed_context = unsafe { windowed_context.make_current().unwrap() };


        load_opengl(|p| windowed_context.get_proc_address(p));
        let gl = opengl();
        
        // let r = ruckus::Renderer::new(800, 600);

        let vs = {
            let mut result = [Vertex2D::default(); 3];
            result[0] = Vertex2D { position: Vert2DPosition { x: -0.5, y: -0.5, z: 0.0 }, color: Vert2DColor { r: 1.0, g: 0., b: 0., a: 1. }, ..Vertex2D::default() };
            result[1] = Vertex2D { position: Vert2DPosition { x:  0.5, y: -0.5, z: 0.0 }, color: Vert2DColor { r: 0.0, g: 1., b: 0., a: 1. }, ..Vertex2D::default() };
            result[2] = Vertex2D { position: Vert2DPosition { x:  0.0, y:  0.5, z: 0.0 }, color: Vert2DColor { r: 0.0, g: 0., b: 1., a: 1. },..Vertex2D::default() };
            result
        };
        let vb = ruckus::buffers::VertexBuffer::new(&vs, DrawUsage::Static);


        let rend = ruckus::Renderer::new(800,600);
    
        // let mut m = ruckus::graphics::Mesh::new(vb);
        // m.shader = Shader::from_memory(VS_SRC, FS_SRC).unwrap().into();
        let shader = Shader::from_memory(VS_SRC, FS_SRC).unwrap();

        el.run(move |event, _, control_flow| {
            match event {
                Event::LoopDestroyed => return,
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(physical_size) => {
                        windowed_context.resize(physical_size)
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit
                    }
                    _ => (),
                },
                Event::RedrawRequested(_) => {
                } 
                _ => (),
            }

            rend.clear(0.2, 0.3, 0.3, 1.0);
            shader.apply();
            rend.draw_buffer(&vb, 0, None);


            windowed_context.swap_buffers().unwrap();
        });
    }
}
