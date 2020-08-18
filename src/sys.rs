use crate::graphics::*;
use nalgebra_glm as glm;

pub const PI: f32 = 3.1415926535897932384626433832795;
pub const HALF_PI: f32 = 1.5707963267948966192313216916398;
pub const TWO_PI: f32 = 6.283185307179586476925286766559;
pub const DEG_TO_RAD: f32 = 0.017453292519943295769236907684886;
pub const RAD_TO_DEG: f32 = 57.295779513082320876798154814105;
pub const EULER: f32 = 2.718281828459045235360287471352;

pub fn radians(degrees: f32) -> f32 {
    degrees * DEG_TO_RAD
}

pub fn degrees(radians: f32) -> f32 {
    radians * RAD_TO_DEG
}

pub fn min_f(a: f32, b: f32) -> f32 {
    if a < b { a } else { b }
}

pub fn max_f(a: f32, b: f32) -> f32 {
    if a > b { a } else { b }
}

pub fn clamp_s(s: f32, smin: f32, smax: f32) -> f32 {
    min_f(max_f(s, smin), smax)
}

pub struct Rect<T: NumDefault> {
    x: T, y: T, w: T, h: T
}

impl<T> Rect<T> where T: NumDefault {
    pub fn new(x: T, y: T, w: T, h: T) -> Self {
        Rect { x, y, w, h }
    }

    pub fn bottom(&self) -> T {
        self.y + self.h
    }

    pub fn right(&self) -> T {
        self.x + self.w
    }
}

impl<T> Default for Rect<T> where T: NumDefault {
    
    fn default() -> Self { 
        let default = T::default();
        Rect {
            x: default,
            y: default,
            w: default,
            h: default
        }
    }
}

pub type Rectf = Rect<f32>;
pub type Recti = Rect<i32>;
pub type Rectui = Rect<u32>;

// TODO :: Finish Quad -- Depends on: Vertex2D, Mesh and Transform
struct Quad {
    verts: [Vertex2D; 4]
}

impl Quad {
    pub const fn verts() -> [Vertex2D; 4] {
        let mut result = [Vertex2D::new(); 4];
        result[0] = Vertex2D{ position: Vert2DPosition { x: 0., y: 0., z: 0. }, text_coord: Vert2DTextureCoord { u: 0., v: 1. }, color: Vert2DColor::white() };
        result[1] = Vertex2D{ position: Vert2DPosition { x: 0., y: 1., z: 0. }, text_coord: Vert2DTextureCoord { u: 0., v: 0. }, color: Vert2DColor::white() };
        result[2] = Vertex2D{ position: Vert2DPosition { x: 1., y: 0., z: 0. }, text_coord: Vert2DTextureCoord { u: 1., v: 1. }, color: Vert2DColor::white() };
        result[3] = Vertex2D{ position: Vert2DPosition { x: 1., y: 1., z: 0. }, text_coord: Vert2DTextureCoord { u: 1., v: 0. }, color: Vert2DColor::white() };
        result
    }

    // pub fn new(mat: &glm::Mat4, texture_rect: Option<&Rectui>, texture_size: glm::TVec2<i32>) -> Self {
    // }
}


pub fn read_file(path: &str) -> std::io::Result<String> {


    use std::fs::File;
    use std::io::BufReader;
    use std::io::prelude::*;

    let f = File::open(path)?;
    let mut reader = BufReader::new(f);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;
    Ok(contents)
}
