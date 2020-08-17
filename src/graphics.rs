use crate::buffers::DataType;
use std::ops::*;
use std::collections::HashMap;
use image::{open, DynamicImage};
use image::imageops::flip_vertical;
use crate::opengl::*;
use crate::math::{Vec2ui, Vec2i};

// TODO :: Implement Shader and Mesh here

#[derive(Debug, Copy, Clone)]
pub enum TextureFormat {
    Alpha = gl::ALPHA as isize,
    Rgb = gl::RGB as isize,
    Rgba = gl::RGBA as isize,
}

pub struct Texture {
    id: u32,
    unit: u32,
    size: Vec2ui
}

impl Texture {

    pub fn from_file(filename: &str) -> Result<Texture, String> {

        let im = match image::open(filename) {
            Ok(d) => d,
            Err(e) => return Err(format!("Error loading file: {} :: ImageError: {}", filename, e))
        };
        let format = match im {
            DynamicImage::ImageRgb8(_) => TextureFormat::Rgb,
            DynamicImage::ImageRgba8(_) => TextureFormat::Rgba,
            _ => TextureFormat::Alpha
        };
        let im = flip_vertical(&im);
        let (w, h) = (im.width(), im.height());

        let t = Texture::from_memory(im.into_raw(), w, h, format);
        Ok(t)
    }

    pub fn from_memory(data: Vec<u8>, w: u32, h: u32, format: TextureFormat) -> Texture {
        let gl = opengl();
        let mut t = Texture { 
            id: gl_gen_texture(), 
            unit: 0, size: Vec2ui::default() 
        };
        t.bind();

        unsafe {
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }
        
        t.unit = gl::TEXTURE0;
        t.size = Vec2ui::new(w, h);
        
        unsafe {
            gl.TexImage2D(gl::TEXTURE_2D, 0, 
                format as i32, w as i32, h as i32,
                0, format as u32, gl::UNSIGNED_BYTE, data.as_ptr() as *const _
            );
            gl.GenerateMipmap(gl::TEXTURE_2D);
        };

        t
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

    pub fn write(&self, offset: Vec2i, w: i32, h: i32, format: TextureFormat, dtype: DataType, data: Vec<u8>) {
        self.bind();
        unsafe {
            opengl().TexSubImage2D(gl::TEXTURE_2D, 0, offset.x, offset.y, w, h, format as u32, dtype as u32, data.as_ptr() as *const _);
        }
    }

    pub fn bind(&self) {
        unsafe { opengl().BindTexture(gl::TEXTURE_2D, self.id) }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ShaderType {
    Vertex = gl::VERTEX_SHADER as isize,
    Fragment = gl::FRAGMENT_SHADER as isize
}

pub struct Shader {
    id: u32,
    uniform_locations: std::collections::HashMap<String, i32>
}

const FRAG_TEMPLATE_VARS: &str = r"#version 330
    out vec4 FragColor;
    in vec2 TexCoord;
    in vec4 Color;
    in vec3 FragPos;

    uniform sampler2D u_texture;
";

const FRAG_TEMPLATE_MAIN: &str = r"
void main()
{
    FragColor = effect(Color, u_texture, TexCoord, FragPos);
}
";

const VERT_TEMPLATE_DECLS: &str = r"#version 330
    layout(location = 0) in vec3 l_pos;
    layout(location = 1) in vec2 l_texCoords;
    layout(location = 2) in vec4 l_color;

    out vec2 TexCoord;
    out vec4 Color;
    out vec3 FragPos;

    uniform mat4 u_matrixMVP;
    uniform mat4 u_modelMatrix;
    mat4 matrixMVP = u_matrixMVP;
";

const VERT_TEMPLATE_DECLS_INSTANCED: &str = r"#version 330
    layout(location = 0) in vec3 l_pos;
    layout(location = 1) in vec2 l_texCoords;
    layout(location = 2) in vec4 l_color;
    layout(location = 3) in mat4 l_matrixMVP;
    layout(location = 7) in mat4 u_modelMatrix;

    out vec2 TexCoord;
    out vec4 Color;
    out vec3 FragPos;
    mat4 matrixMVP = l_matrixMVP;
";

const VERT_TEMPLATE_MAIN: &str = r"
void main()
{
    TexCoord = l_texCoords;
    Color = l_color;

    FragPos = vec3(u_modelMatrix * vec4(l_pos, 1.0));

    gl_Position = position(matrixMVP, l_pos);
}
";

const DEFAULT_VERT: &str = r"#version 330
layout(location = 0) in vec3 l_pos;
layout(location = 1) in vec2 l_texCoords;
layout(location = 2) in vec4 l_color;

out vec2 TexCoord;
out vec4 Color;
out vec3 FragPos;

uniform mat4 u_matrixMVP;
uniform mat4 u_modelMatrix;

void main()
{
    TexCoord = l_texCoords;
    Color = l_color;
    FragPos = vec3(u_modelMatrix * vec4(l_pos, 1.0));

    gl_Position = u_matrixMVP * vec4(l_pos, 1.0);
}
";

const DEFAULT_INSTANCED_VERT: &str = r"#version 330
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
";

const DEFAULT_FRAG: &str = r"#version 330
out vec4 FragColor;

in vec4 Color;
in vec2 TexCoord;
uniform sampler2D u_texture;

void main()
{
	FragColor = texture(u_texture, TexCoord) * Color
}";

impl Shader {

    pub fn from_file(vert_filename: &str, frag_filename: &str) -> Result<Self, String> {
        let vshader = gl_compile_shader_from_file(vert_filename, ShaderType::Vertex)?;
        let fshader = gl_compile_shader_from_file(frag_filename, ShaderType::Fragment)?;

        let shaderid = gl_create_shader_program(vshader, fshader)?;
        Ok(Shader::new(shaderid))
    }

    pub fn from_memory(vert: &str, frag: &str) -> Result<Self, String> {
        let vshader = gl_compile_shader(vert, ShaderType::Vertex)?;
        let fshader = gl_compile_shader(frag, ShaderType::Fragment)?;
        let id = gl_create_shader_program(vshader, fshader)?;

        Ok(Shader::new(id))
    }

    pub fn from_template(position: &str, effect: &str) -> Result<Self, String> {
        let vert_full = Shader::concat_shader_sources(VERT_TEMPLATE_DECLS, position, VERT_TEMPLATE_MAIN);
        let frag_full = Shader::concat_shader_sources(FRAG_TEMPLATE_VARS, effect, FRAG_TEMPLATE_MAIN);
        let mut s = Shader::from_memory(&vert_full, &frag_full)?;
        Shader::fill_template_uniforms(&mut s);
        Ok(s)
    }

    pub fn from_vert_template(position: &str) -> Result<Self, String> {
        let vert_full = Shader::concat_shader_sources(VERT_TEMPLATE_DECLS, position, VERT_TEMPLATE_MAIN);
        let frag_full = DEFAULT_FRAG;
        let mut s = Shader::from_memory(&vert_full, &frag_full)?;
        Shader::fill_template_uniforms(&mut s);
        Ok(s)
    }
    pub fn from_frag_template(effect: &str) -> Result<Self, String> {
        let vert_full = DEFAULT_VERT;
        let frag_full = Shader::concat_shader_sources(FRAG_TEMPLATE_VARS, effect, FRAG_TEMPLATE_MAIN);
        let mut s = Shader::from_memory(&vert_full, &frag_full)?;
        Shader::fill_template_uniforms(&mut s);
        Ok(s)
    }
    
    pub fn from_template_instanced(vert: &str, frag: &str) -> Result<Self, String> {
        unimplemented!("Instancing in shaders not yet supported");
    }

    pub fn set_uniform_4f(&self, name: &str, floats: (f32, f32, f32, f32)) {
        self.bind();
        gl_set_uniform_4f(self.uniform_locations[name], floats);
    }

    pub fn set_uniform_3f(&self, name: &str, floats: (f32, f32, f32)) {
        self.bind();
        gl_set_uniform_3f(self.uniform_locations[name], floats);
    }

    pub fn set_uniform_2f(&self, name: &str, floats: (f32, f32)) {
        self.bind();
        gl_set_uniform_2f(self.uniform_locations[name], floats);
    }

    pub fn set_uniform_f(&self, name: &str, n: f32) {
        self.bind();
        gl_set_uniform_f(self.uniform_locations[name], n);
    }

    pub fn set_uniform_i(&self, name: &str, n: i32) {
        self.bind();
        gl_set_uniform_i(self.uniform_locations[name], n);
    }

    pub fn set_uniform_matrix(&self) {
        self.bind();
        unimplemented!("Matrix Operations not yet implemented");
    }

    pub fn register_uniform(&mut self, name: &str) {
        let k = String::from(name);
        let v = gl_get_uniform_location(self.id, name);
        let _  = self.uniform_locations.insert(k, v);
    }

    pub fn bind(&self) {
        unsafe { opengl().UseProgram(self.id) }
    }

    fn new(id: u32) -> Self {
        Shader { id, uniform_locations: HashMap::new() }
    }

    fn fill_template_uniforms(s: &mut Shader) {
        s.register_uniform("u_matrixMVP");
        s.register_uniform("u_modelMatrix");
    }

    fn concat_shader_sources(a: &str, b: &str, c: &str) -> String {
        let mut result = String::from(a);
        result.push_str(b);
        result.push_str(c); 
        result
    }
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

