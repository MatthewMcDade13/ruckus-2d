
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

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;
static mut GL_CONTEXT: Option<gl::Gl> = None;

const VS_SRC: &'static [u8] = b"
#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTex;
layout (location = 2) in vec4 aColor;

out vec4 PxColor;
out vec2 TexCoord;

uniform mat4 u_mvp;

void main() {
    gl_Position = u_mvp * vec4(aPos, 1.0);
    PxColor = aColor;
    TexCoord = aTex;
}
\0";

const FS_SRC: &'static [u8] = b"
#version 330 core

out vec4 FragColor;
in vec4 PxColor;
in vec2 TexCoord;

uniform sampler2D s_texture;

void main() {
    FragColor = texture(s_texture, TexCoord) * PxColor;
}
\0";

#[macro_export]
macro_rules! memory_offset {
    ($ty:ty, $field:ident) => {
        unsafe { &((*(0 as *const $ty)).$field) as *const _ as usize }
    };
}


fn load_audio_manager() -> Result<AudioManager, kira::manager::error::SetupError>{
    let audio_manager = AudioManager::new(AudioManagerSettings::default())?;

    Ok(audio_manager)
}

fn main() -> Result<(), ()> {
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

    let mut audio = load_audio_manager().unwrap();
    let mut sound_handle = audio.load_sound("./rainy_village_8_bit_lofi.mp3", SoundSettings::default()).unwrap();
    let lofi_handle = sound_handle.play(InstanceSettings::default()).unwrap();


    load_opengl(|p| windowed_context.get_proc_address(p));
    
    // let r = ruckus::Renderer::new(800, 600);

    let tex = ruckus::graphics::Texture::from_file("./pepe-the-frog.jpg").unwrap();
    let mut q = ruckus::sys::Quad::default();

    let mut renderer = ruckus::Renderer::new(800,600);

    let shader =  Shader::from_memory(VS_SRC, FS_SRC).unwrap();


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

        let mut model = glm::Mat4::identity();
        model = glm::rotate(&model, ruckus::sys::radians(90.0), &glm::vec3(0., 0., 1.));

        renderer.camera.position = glm::vec3(4., 3., 3.);            
        renderer.camera.look_direction = glm::vec3(-4., -3., -3.);            
        let x = renderer.projection() * renderer.view() * model;
        shader.set_uniform_matrix("u_mvp", &x);

        renderer.clear(0.2, 0.3, 0.3, 1.0);
        shader.apply();
        renderer.draw_quad(&q, &tex);


        windowed_context.swap_buffers().unwrap();
    });
}

