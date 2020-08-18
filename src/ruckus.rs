pub use nalgebra_glm as glm;

mod sys;
mod opengl;
mod buffers;
mod graphics;

use sys::*;
use buffers::*;
use graphics::*;
use opengl::{opengl, gl};


trait DrawSurface {
    fn swap_buffers(&self);
    fn clear(&self);

    fn width(&self) -> f32;
    fn height(&self) -> f32;
}

pub struct Renderer<'a> {
    pub camera: Camera,

    draw_vao: VertexArray,
    quad_buffer: VertexBuffer,
    instanced_mat_buffer: VertexBuffer,

    shader: Shader,
    instanced_shader: Shader,
    
    surface: &'a (dyn DrawSurface + 'a),
    default_texture: Texture,
    projection: glm::Mat4,
    viewport: sys::Rectf,
}

// TODO :: Implement some type of builder pattern for renderer to pass flags on create,
//         until then we have no way of modifying renderer defaults outside of cumbersome set_* calls
impl<'a> Renderer<'a> {

    // pub fn new(surface: &'a (dyn DrawSurface + 'a)) -> Self {
    //     let gl = opengl();
    //     unsafe {
    //         gl.Viewport(0, 0, surface.width() as i32, surface.height() as i32);
    //         gl.Enable(gl::DEPTH_TEST);
    //         gl.Enable(gl::BLEND);
    //         gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    //     };

    //     let r = Renderer {
    //         camera: Camera::new(),
    //     }
    // }

    // fn new() -> Self {
    //     let zvec = glm::vec3(0., 0., 0.);
    //     Renderer {
    //         camera: Camera::new(),
    //         draw_vao: VertexArray::new(),
    //         quad_buffer: 
    //     }
    // }
}


pub struct Camera {
    position: glm::Vec3,
    look_direction: glm::Vec3,
    rotation: glm::Vec3,

    speed: f32,
    sensitivity: f32,
}

impl Camera {

    pub fn new() -> Self {
        Camera::default()
    }

    pub fn view(&self) -> glm::Mat4 {
        glm::look_at(&self.position, &self.target(), &UP_VECTOR.into_vec3())
    }

    pub fn target(&self) -> glm::Vec3 {
        self.position + self.look_direction
    }

    pub fn move_forward(&mut self, delta_time: f32) {
        self.position += self.speed * self.look_direction * delta_time;
    }

    pub fn move_backward(&mut self, delta_time: f32) {
        self.position -= self.speed * self.look_direction * delta_time;
    }

    pub fn move_left(&mut self, delta_time: f32) {
        let v: glm::Vec3 = self.look_direction.cross(&UP_VECTOR.into_vec3());
        self.position -= glm::normalize(&v) * self.speed * delta_time;
    }

    pub fn move_right(&mut self, delta_time: f32) {
        let v = self.look_direction.cross(&UP_VECTOR.into_vec3());
        self.position += glm::normalize(&v) * self.speed * delta_time;
    }

    pub fn look_towards(&mut self, offset: &glm::Vec3) {
        let delta = offset * self.sensitivity;

        self.rotation += glm::vec3(delta.x, delta.y, 0.);
        self.rotation.y = sys::clamp_s(self.rotation.y, -89., 89.);
        let frontx = f32::cos(radians(self.rotation.x)) * f32::cos(radians(self.rotation.y));
        let fronty = f32::sin(radians(self.rotation.y));
        let frontz = f32::sin(radians(self.rotation.x)) * f32::cos(radians(self.rotation.y));
        
        self.look_direction = glm::normalize(&glm::vec3(frontx, fronty, frontz));
    }
}

impl Default for Camera {
    
    fn default() -> Self { 
        let zvec = glm::vec3(0., 0., 0.);
        Camera { 
            position: zvec,
            look_direction: zvec,
            rotation: zvec,
            speed: 0., sensitivity: 0.
        }
    }
}
