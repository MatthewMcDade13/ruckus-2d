use std::ops::Mul;
use crate::graphics::*;
use crate::vertex::*;
use nalgebra_glm as glm;

pub const PI: f32 = 3.1415926535897932384626433832795;
pub const HALF_PI: f32 = 1.5707963267948966192313216916398;
pub const TWO_PI: f32 = 6.283185307179586476925286766559;
pub const DEG_TO_RAD: f32 = 0.017453292519943295769236907684886;
pub const RAD_TO_DEG: f32 = 57.295779513082320876798154814105;
pub const EULER: f32 = 2.718281828459045235360287471352;

#[macro_export]
macro_rules! memory_offset {
    ($ty:ty, $field:ident) => {
        unsafe { &((*(0 as *const $ty)).$field) as *const _ as usize }
    };
}

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
    pub x: T, pub y: T, pub w: T, pub h: T
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
pub struct Quad {
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

    pub fn new<T>(pos: glm::Vec2, size: glm::Vec2, rotation_degrees: f32, texture_rect: T) -> Self where T: Into<Option<Rectui>> {
        let mut t = Transform::default();
        t.translate(pos)
            .rotate(rotation_degrees)
            .scale(size);

        Self::with_mat(t.model(), texture_rect)
    }

    pub fn with_mat<T>(mat: &glm::Mat4, texture_rect: T) -> Self where T: Into<Option<Rectui>> {
        let mut verts = Self::verts();
        verts.translate(&mat);

        if let Some(r) = texture_rect.into() {
            verts.calc_texture_coords(&r);
        }

        Self::with_verts(&verts)
    }

    pub fn with_verts(verts: &[Vertex2D]) -> Self {
        let mut vs = [Vertex2D::default(); 4];
        for (i, v) in vs.iter_mut().enumerate() {
            *v = verts[i];
        }
        Quad { verts: vs }
    }

    pub fn flip_vertical_text_coords(&mut self, min: f32, max: f32) {
        self.verts.flip_texture_coords_vert(min, max);
    }
}

impl Default for Quad {
    fn default() -> Self { 
        Quad { verts: Self::verts() }
    }
}

// Move to new file? Doesnt really seem like it belongs here...
pub struct Transform(glm::Mat4);

impl Transform {

    pub fn new(mat: &glm::Mat4) -> Self {
        Transform(*mat)
    }
 
    pub fn translate<T>(&mut self, offset: T) -> &mut Self  where T: Into<glm::Vec2> {
        let offset: glm::Vec2 = offset.into();
        self.0 = self.0 * Transform::from_position(offset).model();
        self
    }

    pub fn scale<T>(&mut self, scale: T) -> &mut Self where T: Into<glm::Vec2> {
        let scale = scale.into();
        self.0 = self.0 * Transform::from_scale(scale).model();
        self
    }

    pub fn rotate(&mut self, angle_degrees: f32) -> &mut Self {
        self.0 = self.0 * Transform::from_rotation(angle_degrees).model();
        self
    }

    /**
     * Rotates transform from offset of top left corner
     */
    pub fn rotate_offset(&mut self, angle_degrees: f32, offset: glm::Vec2) -> &mut Self { 
        self.translate(offset)
            .rotate(angle_degrees)
            .translate(-offset)
    }

    pub fn from_position<T>(position: T) -> Self where T: Into<glm::Vec2> {
        let position = position.into();
        let (x, y) = (position.x, position.y);
        let model = glm::mat4(
            1., 0., 0., x,
            0., 1., 0., y,
            0., 0., 1., 0.,
            0., 0., 0., 1.
        );
        Transform(model)
    }

    pub fn from_scale<T>(scale: T) -> Self where T: Into<glm::Vec2> {
        let scale = scale.into();
        let (x, y) = (scale.x, scale.y);
        let model = glm::mat4(
            x , 0., 0., 0.,
            0., y , 0., 0.,
            0., 0., 1., 0.,
            0., 0., 0., 1.
        );
        Transform(model)
    }

    pub fn from_rotation(angle_degrees: f32) -> Self {
        let rotation = radians(angle_degrees);
        let (sin, cos) = (f32::sin(rotation), f32::cos(rotation));
        let model = glm::mat4(
            cos, -sin,  0.,  0.,
            sin,  cos,  0.,  0.,
            0. ,  0. ,  1.,  0.,
            0. ,  0. ,  0.,  1.
        );
        Transform(model)
    }

    pub fn combine_mut(&mut self, other: &Self) {
        self.0 = self.0 * other.0;
    }

    pub fn combine(left: &Self, right: &Self) -> Self {
        Transform(left.0 * right.0)
    }

    pub fn model(&self) -> &glm::Mat4 { &self.0 }
}

impl Mul for Transform {
    type Output = Self;

    fn mul(self, right: Self) -> Self { 
        Self::combine(&self, &right)
    }
}

impl Default for Transform {    
    fn default() -> Self { 
        Transform(glm::Mat4::identity())         
    }
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
