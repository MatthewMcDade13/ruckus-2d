extern crate glutin;
extern crate image;

mod opengl;
mod buffers;
mod graphics;
mod math;

use buffers::VertexArray;

pub struct Renderer {
    clip_near: f32, clip_far: f32, 
    draw_vao: VertexArray
}