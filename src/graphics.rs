use crate::opengl::*;
use crate::buffers::*;
use crate::sys::*;
use std::ops::*;
use num::Num;
use nalgebra_glm as glm;

use std::collections::HashMap;

#[allow(unused_imports)]
use image::{open, DynamicImage};

use image::imageops::{flip_vertical, flip_vertical_in};

pub trait NumDefault: Num + Default + Copy{}
impl <T: Num + Default + Copy> NumDefault for T {}

#[derive(Debug, Copy, Clone)]
pub enum TextureFormat {
    Alpha = gl::ALPHA as isize,
    Rgb = gl::RGB as isize,
    Rgba = gl::RGBA as isize,
}

pub struct Texture {
    id: u32,
    unit: u32,
    pub size: glm::TVec2<u32>
}

impl Texture {

    pub fn from_file(filename: &str) -> Result<Texture, String> {

        let im = match image::open(filename) {
            Ok(d) => d,
            Err(e) => return Err(format!("Error loading file: {} :: ImageError: {}", filename, e))
        };
        let im = flip_vertical(&im);
        let layout = im.sample_layout();

        let format = match layout.channels {
            3 => TextureFormat::Rgb,
            4 => TextureFormat::Rgba,
            _ => TextureFormat::Alpha
        };
        let (w, h) = (im.width(), im.height());

        let t = Texture::from_memory(im.into_raw(), w, h, format);
        Ok(t)
    }

    pub fn from_memory(data: Vec<u8>, w: u32, h: u32, format: TextureFormat) -> Texture {
        let gl = opengl();
    
        let tid = gl_gen_texture();
        unsafe {
            opengl().BindTexture(gl::TEXTURE_2D, tid);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }
        
        
        unsafe {
            gl.TexImage2D(gl::TEXTURE_2D, 0, 
                gl::RGBA as i32, w as i32, h as i32,
                0, gl::RGBA as u32, gl::UNSIGNED_BYTE, data.as_ptr() as *const _
            );
            gl.GenerateMipmap(gl::TEXTURE_2D);
        };

        Texture { 
            id: tid, 
            unit: gl::TEXTURE0, size: glm::vec2(w,h)
        }
    }

    pub fn new_blank() -> Self {
        let texture_data = vec![255,255,255,255];
        Texture::from_memory(texture_data.into(), 1, 1, TextureFormat::Rgba)
    }

    pub fn set_alignment(alignment: i32) {
        unsafe { opengl().PixelStorei(gl::UNPACK_ALIGNMENT, alignment) }
    } 

    pub fn unit(&self) -> u32 { self.unit }
    pub fn set_unit(&mut self, unit_num: u32) {
        self.unit = gl::TEXTURE0 + unit_num;
    }

    pub fn write(&self, offset: glm::TVec2<i32>, w: i32, h: i32, format: TextureFormat, dtype: DataType, data: Vec<u8>) {
        self.apply();
        unsafe {
            opengl().TexSubImage2D(gl::TEXTURE_2D, 0, offset.x, offset.y, w, h, format as u32, dtype as u32, data.as_ptr() as *const _);
        }
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn apply(&self) {
        unsafe { 
            opengl().ActiveTexture(self.unit());
            opengl().BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
}


// TODO :: Consolidate these with #ifdefs
const FRAG_TEMPLATE_VARS: &'static [u8] = b"
#version 330
out vec4 FragColor;
in vec2 TexCoord;
in vec4 Color;
in vec3 FragPos;

uniform sampler2D u_texture;
\0";

const FRAG_TEMPLATE_MAIN: &'static [u8] = b"
void main()
{
    FragColor = effect(Color, u_texture, TexCoord, FragPos);
}
\0";

const VERT_TEMPLATE_DECLS: &'static [u8] = b"#version 330
    layout(location = 0) in vec3 l_pos;
    layout(location = 1) in vec2 l_texCoords;
    layout(location = 2) in vec4 l_color;

    out vec2 TexCoord;
    out vec4 Color;
    out vec3 FragPos;

    uniform mat4 u_model;
    uniform mat4 u_view;
    uniform mat4 u_projection;
\0";


const VERT_TEMPLATE_MAIN: &'static [u8] = b"
void main()
{
    TexCoord = l_texCoords;
    Color = l_color;

    FragPos = vec3(u_model * vec4(l_pos, 1.0));

    gl_Position = position(u_projection * u_view * u_model, vec4(l_pos.x, l_pos.y, l_pos.z, 1.0));
}
\0";

const VERT_TEMPLATE_DECLS_INSTANCED: &'static [u8] = b"#version 330
    layout(location = 0) in vec3 l_pos;
    layout(location = 1) in vec2 l_texCoords;
    layout(location = 2) in vec4 l_color;
    layout(location = 3) in mat4 l_matrixMVP;
    layout(location = 7) in mat4 u_modelMatrix;

    out vec2 TexCoord;
    out vec4 Color;
    out vec3 FragPos;
    mat4 matrixMVP = l_matrixMVP;
\0";

const DEFAULT_VERT: &'static [u8] = b"#version 330
    layout(location = 0) in vec3 l_pos;
    layout(location = 1) in vec2 l_texCoords;
    layout(location = 2) in vec4 l_color;

    out vec2 TexCoord;
    out vec4 Color;
    out vec3 FragPos;

    uniform mat4 u_model;
    uniform mat4 u_view;
    uniform mat4 u_projection;

    void main()
    {
        TexCoord = l_texCoords;
        Color = l_color;

        mat4 mvp = u_projection * u_view * u_model;
        gl_Position = mvp * vec4(l_pos, 1.0);
    }
\0";

const DEFAULT_INSTANCED_VERT: &'static [u8] = b"#version 330
layout(location = 0) in vec3 l_pos;
layout(location = 1) in vec2 l_texCoords;
layout(location = 2) in vec4 l_color;
layout(location = 3) in mat4 l_matrixMVP;
layout(location = 7) in mat4 u_modelMatrix;

out vec2 TexCoord;
out vec4 Color;
out vec3 FragPos;

void main()
{
    TexCoord = l_texCoords;
    Color = l_color;

    FragPos = vec3(u_modelMatrix * vec4(l_pos, 1.0));

    gl_Position = l_matrixMVP * vec4(l_pos, 1.0);
}
\0";

const DEFAULT_FRAG: &'static [u8] = b"#version 330
out vec4 FragColor;

in vec4 Color;
in vec2 TexCoord;
uniform sampler2D u_texture;

void main()
{
	FragColor = texture(u_texture, TexCoord) * Color;
}
\0";

#[derive(Debug, Copy, Clone)]
pub enum ShaderType {
    Vertex = gl::VERTEX_SHADER as isize,
    Fragment = gl::FRAGMENT_SHADER as isize
}

pub struct Shader {
    id: u32,
    uniform_locations: std::collections::HashMap<String, i32>
}

impl Shader {

    pub fn from_file(vert_filename: &str, frag_filename: &str) -> Result<Self, String> {
        let vshader = gl_compile_shader_from_file(vert_filename, ShaderType::Vertex)?;
        let fshader = gl_compile_shader_from_file(frag_filename, ShaderType::Fragment)?;
        
        let shaderid = gl_create_shader_program(vshader, fshader)?;
        let uniforms = gl_get_active_uniforms(shaderid);
        Ok(Shader::new(shaderid, uniforms))
    }
    
    pub fn from_memory<T>(vert: T, frag: T) -> Result<Self, String> where T: Into<Vec<u8>> {
        let vshader = gl_compile_shader(vert.into().as_slice(), ShaderType::Vertex)?;
        let fshader = gl_compile_shader(frag.into().as_slice(), ShaderType::Fragment)?;

        let id = gl_create_shader_program(vshader, fshader)?;
        let uniforms = gl_get_active_uniforms(id);
        Ok(Shader::new(id, uniforms))
    }

    pub fn from_template(position: &[u8], effect: &[u8]) -> Result<Self, String> {
        let vert_full = Self::concat_shader_sources(VERT_TEMPLATE_DECLS, position, VERT_TEMPLATE_MAIN);
        let frag_full = Self::concat_shader_sources(FRAG_TEMPLATE_VARS, effect, FRAG_TEMPLATE_MAIN);
        let s = Self::from_memory(vert_full.as_slice(), frag_full.as_slice())?;

        Ok(s)
    }

    pub fn from_vert_template(position: &[u8]) -> Result<Self, String> {
        let vert_full = Shader::concat_shader_sources(VERT_TEMPLATE_DECLS, position, VERT_TEMPLATE_MAIN);
        let frag_full = DEFAULT_FRAG;
        let s = Shader::from_memory(vert_full.as_slice(), frag_full)?;
        Ok(s)
    }
    pub fn from_frag_template(effect: &[u8]) -> Result<Self, String> {
        let vert_full = DEFAULT_VERT;
        let frag_full = Shader::concat_shader_sources(FRAG_TEMPLATE_VARS, effect, FRAG_TEMPLATE_MAIN);
        let s = Shader::from_memory(vert_full, frag_full.as_slice())?;
        Ok(s)
    }
    
    pub fn from_template_instanced<'a, T>(position: T, effect:T) -> Result<Self, String> where T: Into<Option<Vec<u8>>> {
        let (position, effect) = (position.into(), effect.into());
        assert!(position.is_some() || effect.is_some(), " Both of the arguments for function from_template_instanced() are None. Please pass at least 1 value with Some");
        let vert_full = match position {
            Some(s) => Shader::concat_shader_sources(VERT_TEMPLATE_DECLS_INSTANCED, s.as_slice(), VERT_TEMPLATE_MAIN),
            None => DEFAULT_INSTANCED_VERT.into()
        };
        let frag_full = match effect {
            Some(s) => Shader::concat_shader_sources(FRAG_TEMPLATE_VARS, s.as_slice(), FRAG_TEMPLATE_MAIN),
            None => DEFAULT_FRAG.into()
        };
        let s = Shader::from_memory(vert_full, frag_full)?;
        Ok(s)
    }

    pub fn set_uniform_4f(&self, name: &str, floats: (f32, f32, f32, f32)) {
        self.apply();
        gl_set_uniform_4f(self.uniform_locations[name], floats);
    }

    pub fn set_uniform_3f(&self, name: &str, floats: (f32, f32, f32)) {
        self.apply();
        gl_set_uniform_3f(self.uniform_locations[name], floats);
    }

    pub fn set_uniform_2f(&self, name: &str, floats: (f32, f32)) {
        self.apply();
        gl_set_uniform_2f(self.uniform_locations[name], floats);
    }

    pub fn set_uniform_f(&self, name: &str, n: f32) {
        self.apply();
        gl_set_uniform_f(self.uniform_locations[name], n);
    }

    pub fn set_uniform_i(&self, name: &str, n: i32) {
        self.apply();
        gl_set_uniform_i(self.uniform_locations[name], n);
    }

    pub fn set_uniform_matrix(&self, name: &str, mat: &glm::Mat4) {
        self.set_uniform_matrix_xpose(name, mat, false)
        
    }

    pub fn set_uniform_matrix_xpose(&self, name: &str, mat: &glm::Mat4, transpose: bool) {
        self.apply();
        gl_set_uniform_matrix_xpose(self.uniform_locations[name], glm::value_ptr(mat), transpose)
    }

    // TODO :: Make sure getting the uniform names from linked shaders actually
    //         works before removing register_uniform()
    pub fn register_uniform(&mut self, name: &str) {
        let k = String::from(name);
        let v = gl_get_uniform_location(self.id, name);
        let _  = self.uniform_locations.insert(k, v);
    }
    
    pub fn apply(&self) {
        unsafe { opengl().UseProgram(self.id) }
    }
    
    pub fn default_instanced() -> Self {
        Self::from_memory(DEFAULT_INSTANCED_VERT, DEFAULT_FRAG).unwrap()
    }

    fn new(id: u32, uniform_locations: HashMap<String, i32>) -> Self {
        Shader { id, uniform_locations }
    }

    fn concat_shader_sources<'a, T>(a: T, b: T, c: T) -> Vec<u8> where T: Into<Vec<u8>>{
        let su = String::from_utf8;
        let (a, b, c) = (a.into(), b.into(), c.into());
        let (a, b, c) = (su(a).unwrap(), su(b).unwrap(), su(c).unwrap());
        let mut result = String::from(a);
        result.push_str(&b);
        result.push_str(&c); 
        result.as_bytes().to_vec()
    }
}

impl Default for Shader {
    fn default() -> Self {
        Self::from_memory(DEFAULT_VERT, DEFAULT_FRAG).unwrap() 
    }
}

pub struct Mesh {
    pub transform: Transform,
    pub buffer: VertexBuffer,
    pub indices: Option<ElementBuffer>,
    pub texture: Option<Texture>,
    pub shader: Option<Shader>,
}

impl Mesh {
    pub fn new(buffer: VertexBuffer) -> Self {
        Self {
            transform: Transform::default(),
            buffer: buffer,
            indices: None,
            texture: None,
            shader: None,
        }
    }

    pub fn new_quad(usage: DrawUsage) -> Self {
        Self {
            transform: Transform::default(),
            buffer: VertexBuffer::new(&Quad::default_verts(), usage),
            indices: Some(ElementBuffer::new_quad(6)),
            texture: None,
            shader: None,

        }
    }

    pub fn indices(&self) -> &ElementBuffer {
        self.indices.as_ref().expect("No Elementbuffer for Mesh")
    }

    pub fn texture(&self) -> &Texture {
        self.texture.as_ref().expect("No Texture for Mesh")
    }

    pub fn shader(&self) -> &Shader {
        self.shader.as_ref().expect("No Shader for Mesh")
    }
}

pub struct RenderTexture {
    pub frame_buffer: FrameBuffer,
    pub render_buffer: RenderBuffer,
    pub texture: Texture
}

impl RenderTexture {
    pub fn new(width: u32, height: u32) -> Result<Self, String> {
        let rt = RenderTexture {
            frame_buffer: FrameBuffer::new(),
            render_buffer: RenderBuffer::new(width as i32, height as i32),
            texture: Texture::from_memory(vec![], width, height, TextureFormat::Rgb)
        };
        rt.frame_buffer.attach_texture(&rt.texture);
        rt.frame_buffer.attach_render_buffer(&rt.render_buffer);

        unsafe {
            if opengl().CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                return Err("OpenGL :: Could not create RenderTexture. Framebuffer is not complete".into());
            }
        }
        FrameBuffer::unbind();
        Ok(rt)
    }
}

impl Drop for RenderTexture {
    fn drop(&mut self) { }
}

impl Drop for Mesh {
    fn drop(&mut self) { }
}

impl Drop for Shader {
    
    fn drop(&mut self) {
        unsafe { opengl().DeleteProgram(self.id) }
    }
}

impl Drop for Texture {
    
    fn drop(&mut self) { 
        unsafe { opengl().DeleteTextures(1, &self.id) }
    }
}

