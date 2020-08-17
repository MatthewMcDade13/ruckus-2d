use crate::buffers::DataType;
use std::ops::*;
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

impl Drop for Texture {
    
    fn drop(&mut self) { 
        unsafe { opengl().DeleteTextures(1, &self.id) }
    }
}