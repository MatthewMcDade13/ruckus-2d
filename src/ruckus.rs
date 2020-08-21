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

const CLIP_NEAR_DEFAULT: f32 = -50.;
const CLIP_FAR_DEFAULT: f32 = 50.;

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
    viewport: sys::Rectf,

    attrib2d: [VertexAttribute; 3],
    attrib2d_instanced: [VertexAttribute; 8]
}

// TODO :: Implement some type of builder pattern for renderer to pass flags on create,
//         until then we have no way of modifying renderer defaults outside of cumbersome set_* calls
impl Renderer {

    pub fn new(width: u32, height: u32) -> Self {     
        unsafe {
            let gl = opengl();
            gl.Viewport(0, 0, width as i32, height as i32);
            gl.Enable(gl::DEPTH_TEST);
            gl.Enable(gl::BLEND);
            gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        };
        let viewport = sys::Rect::new(0., 0., width as f32, height as f32);
        let projection = glm::ortho(viewport.x, viewport.w, viewport.h, viewport.y, CLIP_NEAR_DEFAULT, CLIP_FAR_DEFAULT);

        let camera = { 
            let mut c = FlyCamera::new();
            c.position.z = 0.;
            c.look_direction.z = -1.;
            c
        };

        let attrib2d = { 
            let vert_size = std::mem::size_of::<Vertex2D>();
            // let uv_offset = 
            let mut attr = [VertexAttribute::default(); 3];
            attr[0] = VertexAttribute { buffer_index: 0, elem_count: 3, dtype: DataType::Float, stride: vert_size, offset: 0, is_instanced: false };
            attr[1] = VertexAttribute { buffer_index: 1, elem_count: 2, dtype: DataType::Float, stride: vert_size, offset: memory_offset!(Vertex2D, text_coord), is_instanced: false };
            attr[2] = VertexAttribute { buffer_index: 0, elem_count: 3, dtype: DataType::Float, stride: vert_size, offset: memory_offset!(Vertex2D, color), is_instanced: false };
            attr
        };

        let attrib2d_instanced = {
            let mat_size = std::mem::size_of::<glm::Mat4>();
            let v4_size = std::mem::size_of::<glm::Vec4>();
            let mut attr = [VertexAttribute::default(); 8];
            
            for (i, a) in attr.iter_mut().enumerate() {
                // There are 2 matricies in our instanced vertex shader, so stride is sizeof 2 matricies
                *a = VertexAttribute { buffer_index: i as u32 + 3, elem_count: 4, dtype: DataType::Float, stride: mat_size * 2, offset: i * v4_size, is_instanced: true }
            }
            attr
        };

        let shader = Shader::default();
        let instanced_shader = Shader::default_instanced();
        let default_texture = Texture::new_blank();

        let quad_buffer = VertexBuffer::new(&sys::Quad::default_verts(), DrawUsage::Dynamic);

        let draw_vao = VAO::new();
        draw_vao.apply();

        let instanced_mat_buffer = VertexBuffer::zeroed::<glm::Mat4>(2, DrawUsage::Dynamic, DrawPrimitive::Triangles);

        Renderer { 
            camera, draw_vao, quad_buffer,
            instanced_mat_buffer, shader, 
            instanced_shader,
            default_texture, projection, viewport,
            attrib2d, attrib2d_instanced
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
        let shader = mesh.shader.as_ref().unwrap_or(&self.shader);

        shader.apply();

        set_vertex_layout(&mesh.buffer, &self.attrib2d);

        // let mvp = self.projection * self.camera.view() * mesh.transform.model();        
        self.shader.set_uniform_matrix("u_projection", &self.projection);
        self.shader.set_uniform_matrix("u_view", &self.camera.view());
        self.shader.set_uniform_matrix("u_model", mesh.transform.model());

        let texture = mesh.texture.as_ref().unwrap_or(&self.default_texture);

        if let Some(e) = mesh.indicies.as_ref() {
            self.draw_indexed_buffer_static(&mesh.buffer, &e, texture);
        } else {
            self.draw_buffer_static(&mesh.buffer, 0, texture);
        }

    }

    pub fn draw_quad<'b, T>(&self, q: &Quad, xform: &Transform, texture: T) where T: Into<Option<&'b Texture>> {
        self.draw_quad_shader(q, xform, &self.shader, texture);
    }
    
    pub fn draw_quad_shader<'b, T>(&self, q: &Quad, xform: &Transform, shader: &Shader, texture: T) where T: Into<Option<&'b Texture>> {
        let texture = texture.into().unwrap_or(&self.default_texture);
        texture.apply();
        shader.apply();
        set_vertex_layout(&self.quad_buffer, &self.attrib2d);

        self.shader.set_uniform_matrix("u_projection", &self.projection);
        self.shader.set_uniform_matrix("u_view", &self.camera.view());
        self.shader.set_uniform_matrix("u_model", xform.model());

        self.quad_buffer.write(&q.verts, 0);

        gl_draw_arrays(0, self.quad_buffer.vert_count(), DrawPrimitive::Triangles);

    }

    pub fn draw_buffer<'b, T>(&self, buffer: &VertexBuffer, first_vertex: u32, xform: &Transform, texture: T) where T: Into<Option<&'b Texture>> {
        let texture = texture.into().unwrap_or(&self.default_texture);
        texture.apply();

        self.shader.set_uniform_matrix("u_projection", &self.projection);
        self.shader.set_uniform_matrix("u_view", &self.camera.view());
        self.shader.set_uniform_matrix("u_model", xform.model());

        set_vertex_layout(buffer, &self.attrib2d);

        gl_draw_arrays(first_vertex, buffer.vert_count(), buffer.draw_prim);
        gl_unbind_array_buffer();
    }

    /** 
     * Draws given buffer without any kind of vertex transformation
    */
    pub fn draw_buffer_static<'b, T>(&self, buffer: &VertexBuffer, first_vertex: u32, texture: T) where T: Into<Option<&'b Texture>> {
        let texture = texture.into().unwrap_or(&self.default_texture);
        texture.apply();

        self.shader.set_uniform_matrix("u_projection", &glm::Mat4::identity());
        self.shader.set_uniform_matrix("u_view", &glm::Mat4::identity());
        self.shader.set_uniform_matrix("u_model", &glm::Mat4::identity());

        set_vertex_layout(buffer, &self.attrib2d);

        gl_draw_arrays(first_vertex, buffer.vert_count(), buffer.draw_prim);
        gl_unbind_array_buffer();
    }
    /** 
     * Draws given buffer without any kind of vertex transformation
    */
    pub fn draw_indexed_buffer_static<'b, T>(&self, buffer: &VertexBuffer, ebo: &ElementBuffer, texture: T) where T: Into<Option<&'b Texture>> {
        let texture = texture.into().unwrap_or(&self.default_texture);
        texture.apply();
        
        self.shader.set_uniform_matrix("u_projection", &glm::Mat4::identity());
        self.shader.set_uniform_matrix("u_view", &glm::Mat4::identity());
        self.shader.set_uniform_matrix("u_model", &glm::Mat4::identity());

        set_vertex_layout(buffer, &self.attrib2d);

        ebo.apply();
        gl_draw_elements(ebo.count, buffer.draw_prim);
        gl_unbind_element_buffer();
    }
    
    pub fn clear_black(&self) {
        unsafe {
            opengl().ClearColor(0., 0., 0., 1.);
            opengl().Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    pub fn clear(&self, color: &glm::Vec4) {
        unsafe {
            opengl().ClearColor(color.x, color.y, color.z, color.w);
            opengl().Clear(gl::COLOR_BUFFER_BIT);
        }
    }
    
    /**
     * Binds renderer's internal VAO
    */
    pub fn set_current(&self) {
        self.draw_vao.apply();
    } 
}


pub struct FlyCamera {
    position: glm::Vec3,
    look_direction: glm::Vec3,
    rotation: glm::Vec3,

    speed: f32,
    sensitivity: f32,
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
