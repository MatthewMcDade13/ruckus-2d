
use kira::instance::InstanceSettings;
use kira::sound::SoundSettings;
use kira::manager::AudioManagerSettings;
use glutin::PossiblyCurrent;
use ruckus::opengl::*;
use ruckus::vertex::*;
use ruckus::graphics::*;
use ruckus::buffers::*;
use std::ffi::CStr;

use nalgebra_glm as glm;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use kira::manager::{AudioManager};


fn main() {

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
    
    let mut renderer = ruckus::Renderer::new(800,600);

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

        renderer.clear(0.2, 0.3, 0.3, 1.0);


        windowed_context.swap_buffers().unwrap();
    });
}