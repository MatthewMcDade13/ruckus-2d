use crate::opengl::gl_unbind_array_buffer;
use crate::opengl::gl_draw_arrays;
use crate::opengl::gl_unbind_element_buffer;
use crate::opengl::gl_draw_elements;
pub use nalgebra_glm as glm;

pub mod sys;
pub mod opengl;
pub mod buffers;
pub mod graphics;
pub mod vertex;

use sys::*;
use buffers::*;
use graphics::*;
use vertex::*;
use opengl::{opengl, gl};

const CLIP_NEAR_DEFAULT: f32 = 0.1;
const CLIP_FAR_DEFAULT: f32 = 1000.;

// pub trait DrawSurface {

//     fn swap_buffers(&self);

//     fn clear(&self, color: &glm::Vec4) {
//         unsafe {
//             opengl().ClearColor(color.x, color.y, color.z, color.w);
//             opengl().Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
//         }
//     }

//     fn width(&self) -> f32;
//     fn height(&self) -> f32;
// }

pub trait Renderable {  
    fn draw(&self, renderer: &Renderer);
}

#[derive(Copy, Clone, Debug, Default)]
struct ProjectionInfo {
    width: f32, height: f32,
    fov_deg: f32, clip_near: f32, clip_far: f32
}

impl ProjectionInfo {
    pub fn aspect(&self) -> f32 { self.width / self.height }
    pub fn fov_rad(&self) -> f32 { sys::radians(self.fov_deg) }

    pub fn to_matrix(&self) -> glm::Mat4 {
        glm::perspective(self.aspect(), self.fov_rad(), self.clip_near, self.clip_far)
    }
}

// TODO :: Implement uses for unsued field memebers
pub struct Renderer {
    pub camera: FlyCamera,

    draw_vao: VAO,
    quad_buffer: VertexBuffer,
    instanced_mat_buffer: VertexBuffer,

    shader: Shader,
    instanced_shader: Shader,
    
    default_texture: Texture,
    projection: glm::Mat4,
    projection_info: ProjectionInfo,
}

// TODO :: Implement some type of builder pattern for renderer to pass flags on create,
//         until then we have no way of modifying renderer defaults outside of cumbersome set_* calls
impl Renderer {

    pub const DEFAULT_FOV: f32 = 45.;
    pub const U_PROJECTION: &'static str = "u_projection";
    pub const U_VIEW: &'static str = "u_view";
    pub const U_MODEL: &'static str = "u_model";

    pub fn new(width: u32, height: u32) -> Self {     
        unsafe {
            let gl = opengl();
            gl.Viewport(0, 0, width as i32, height as i32);
            gl.Enable(gl::DEPTH_TEST);
            gl.Enable(gl::BLEND);
            gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        };
        let viewport = sys::Rect::new(0., 0., width as f32, height as f32);
        let projection_info = ProjectionInfo { 
            width: width as f32, height: height as f32, 
            fov_deg: Self::DEFAULT_FOV, clip_near: 0.1, clip_far: 100. 
        };

        let projection = projection_info.to_matrix();

        let camera = { 
            let mut c = FlyCamera::new();
            c.position.z = -3.;
            c.look_direction.z = 3.;
            c
        };

        let shader = Shader::default();
        let instanced_shader = Shader::default_instanced();
        let default_texture = Texture::new_blank();

        let quad_buffer = VertexBuffer::new(&sys::Quad::default_verts(), DrawUsage::Dynamic);
        let instanced_mat_buffer = VertexBuffer::zeroed::<glm::Mat4>(2, DrawUsage::Dynamic, DrawPrimitive::Triangles);

        let draw_vao = VAO::new();

        Renderer { 
            camera, draw_vao, quad_buffer,
            instanced_mat_buffer, shader, 
            instanced_shader,
            default_texture, projection,
            projection_info
        }
    }

    
    pub fn begin_draw_texture(rt: &RenderTexture) {
        FrameBuffer::apply(&rt.frame_buffer);
    }
    pub fn end_draw_texture() {
        FrameBuffer::unbind();
    }

    pub fn draw<T>(&self, renderable: &T) where T: Renderable {
        renderable.draw(self);
    }

    pub fn draw_mesh(&self, mesh: &Mesh) {
        let shader = match mesh.shader.as_ref() {
            Some(s) => s,
            None => {
                self.shader.set_uniform_matrix("u_projection", &self.projection);
                self.shader.set_uniform_matrix("u_view", &self.camera.view());
                self.shader.set_uniform_matrix("u_model", mesh.transform.model());
                &self.shader
            }
        };

        shader.apply();

        self.draw_vao.set_buffer_layout(&mesh.buffer);

        let texture = mesh.texture.as_ref().unwrap_or(&self.default_texture);
        texture.apply();

        if let Some(e) = mesh.indices.as_ref() {
            self.draw_elements(&e, mesh.buffer.draw_prim);
        } else {
            self.draw_arrays(0, mesh.buffer.vert_count(), mesh.buffer.draw_prim);
        }

    }

    pub fn draw_quad<'b, T>(&self, q: &Quad, texture: T) where T: Into<Option<&'b Texture>> {
        let texture = texture.into().unwrap_or(&self.default_texture);
        texture.apply();

        self.draw_vao.set_buffer_layout(&self.quad_buffer);

        self.quad_buffer.write(&q.verts, 0);

        self.draw_arrays(0, self.quad_buffer.vert_count(), DrawPrimitive::TriangleStrip);
    }

    pub fn draw_buffer<'b, T>(&self, buffer: &VertexBuffer, first_vertex: u32, texture: T) where T: Into<Option<&'b Texture>> {
        let texture = texture.into().unwrap_or(&self.default_texture);
        texture.apply();

        self.draw_vao.set_buffer_layout(&buffer);

        self.draw_arrays(first_vertex, buffer.vert_count(), buffer.draw_prim);
        // gl_unbind_array_buffer();
    }

    /** 
     * Draws given buffer without any kind of vertex transformation
    // */
    // pub fn draw_buffer_static<'b, T>(&self, buffer: &VertexBuffer, first_vertex: u32, texture: T) where T: Into<Option<&'b Texture>> {
    //     let texture = texture.into().unwrap_or(&self.default_texture);
    //     texture.apply();

    //     self.shader.set_uniform_matrix("u_projection", &glm::Mat4::identity());
    //     self.shader.set_uniform_matrix("u_view", &glm::Mat4::identity());
    //     self.shader.set_uniform_matrix("u_model", &glm::Mat4::identity());

    //     self.draw_vao.set_buffer_layout(&buffer);

    //     gl_draw_arrays(first_vertex, buffer.vert_count(), buffer.draw_prim);
    //     // gl_unbind_array_buffer();
    // }
    /** 
     * Draws given buffer without any kind of vertex transformation
    */
    pub fn draw_indexed_buffer<'b, T>(&self, buffer: &VertexBuffer, ebo: &ElementBuffer, texture: T) where T: Into<Option<&'b Texture>> {
        let texture = texture.into().unwrap_or(&self.default_texture);
        texture.apply();

        self.draw_vao.set_buffer_layout(&buffer);

        self.draw_elements(&ebo, buffer.draw_prim);
    }
    
    pub fn clear_black(&self) {
       self.clear(0., 0., 0., 1.);
    }

    pub fn use_default_shader<'b, T>(&self, xform: T) where T: Into<Option<&'b Transform>> {
        if let Some(xform) = xform.into() {
            self.shader.set_uniform_matrix("u_projection", &self.projection);
            self.shader.set_uniform_matrix("u_view", &self.camera.view());
            self.shader.set_uniform_matrix("u_model", xform.model());
        } else {
            self.shader.set_uniform_matrix("u_projection", &glm::Mat4::identity());
            self.shader.set_uniform_matrix("u_view", &glm::Mat4::identity());
            self.shader.set_uniform_matrix("u_model", &glm::Mat4::identity());
        }

        self.shader.apply(); // we dont need this apply() call... but why not for good measure lol

    }

    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            opengl().ClearColor(r, g, b, a);
            opengl().Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn projection(&self) -> &glm::Mat4 { &self.projection }
    pub fn view(&self) -> glm::Mat4 { self.camera.view() }
    
    /**
     * Binds renderer's internal VAO
    */
    pub fn set_current(&self) {
        self.draw_vao.apply();
    }
    
    fn draw_arrays(&self, start: u32, vert_count: u32, prim: DrawPrimitive) {
        self.draw_vao.apply();
        gl_draw_arrays(start, vert_count, prim);        
    }

    fn draw_elements(&self, ebo: &ElementBuffer, prim: DrawPrimitive) {
        self.draw_vao.apply();
        ebo.apply();
        gl_draw_elements(ebo.count, prim);
        ebo.unbind();
    }
}


pub struct FlyCamera {
    pub position: glm::Vec3,
    pub look_direction: glm::Vec3,
    pub rotation: glm::Vec3,

    pub speed: f32,
    pub sensitivity: f32,
}

impl FlyCamera {

    pub fn new() -> Self {
        FlyCamera::default()
    }

    pub fn view(&self) -> glm::Mat4 {
        glm::look_at(&self.position, &self.target(), &UP_VECTOR.into())
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
        let v: glm::Vec3 = self.look_direction.cross(&UP_VECTOR.into());
        self.position -= glm::normalize(&v) * self.speed * delta_time;
    }

    pub fn move_right(&mut self, delta_time: f32) {
        let v = self.look_direction.cross(&UP_VECTOR.into());
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

impl Default for FlyCamera {
    
    fn default() -> Self { 
        let zvec = glm::vec3(0., 0., 0.);
        FlyCamera { 
            position: zvec,
            look_direction: zvec,
            rotation: zvec,
            speed: 0., sensitivity: 0.
        }
    }
}
