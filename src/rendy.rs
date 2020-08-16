mod opengl;
mod buffers;
use buffers::VertexArray;

pub struct Rendy {
    clip_near: f32, clip_far: f32, 
    draw_vao: VertexArray
}