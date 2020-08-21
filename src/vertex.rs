use nalgebra_glm as glm;

use crate::sys::{Rect, Rectui, Transform};

pub struct TupleVector(f32, f32, f32);
pub const UP_VECTOR: TupleVector = TupleVector(0., 1., 0.);
pub const FORWARD_VECTOR: TupleVector = TupleVector(0., 0., 1.);

impl From<TupleVector> for glm::Vec3 {
    fn from(t: TupleVector) -> Self { glm::vec3(t.0, t.1, t.2) }
}


#[repr(C)] #[derive(Copy, Clone, Default)] pub struct Vert2DPosition { pub x: f32, pub y: f32, pub z: f32 }
#[repr(C)] #[derive(Copy, Clone, Default)] pub struct Vert2DTextureCoord { pub u: f32, pub v: f32 }
#[repr(C)] #[derive(Copy, Clone, Default)] pub struct Vert2DColor { pub r: f32, pub g: f32, pub b: f32, pub a: f32 }
#[repr(C)] #[derive(Copy, Clone, Default)] pub struct Vertex2D {
    pub position: Vert2DPosition,
    pub text_coord: Vert2DTextureCoord,
    pub color: Vert2DColor
}

impl From<&glm::Vec4> for Vert2DColor {
    fn from(v: &glm::Vec4) -> Self { 
        Vert2DColor {
            r: v.x, g: v.y, b: v.z, a: v.w
        }
    }
}

impl From<glm::Vec4> for Vert2DPosition {
    fn from(v: glm::Vec4) -> Self { 
        Vert2DPosition {
            x: v.x, y: v.y, z: v.z
        }
    }
}

impl From<glm::Vec2> for Vert2DTextureCoord {
    fn from(v: glm::Vec2) -> Self { 
        Vert2DTextureCoord {
            u: v.x, v: v.y
        }
    }
}

impl From<Vert2DPosition> for glm::Vec4 {
    fn from(p: Vert2DPosition) -> Self { 
        glm::vec4(p.x, p.y, p.z, 1.)
    }
}

pub trait Vertex2DSliceOps<'a> {
    fn set_color(&mut self, color: &glm::Vec4);
    fn translate(&mut self, xform: &Transform);
    fn calc_texture_coords(&mut self, texture_rect: &Rectui);
    /** 
     * Flips given verts tex coordinates along vertical (y) axis. Assumes
     * coordinates are NDC
    */
    fn flip_texture_coords_vert(&mut self, min: f32, max: f32);

    fn normalize_texture_coords(&mut self, bounds: glm::Vec2);
}

trait Vertex2DAsRaw<'a> { fn as_raw(&self) -> &'a [f32]; }

impl<'a> Vertex2DAsRaw<'a> for std::slice::Iter<'_, Vertex2D> {
    fn as_raw(&self) -> &'a [f32] {
        unsafe { std::mem::transmute(self.as_slice()) }
    }
}


impl<'a> Vertex2DSliceOps<'a> for std::slice::IterMut<'_, Vertex2D> {
    
    fn set_color(&mut self, color: &glm::Vec4) {
        for v in self { v.color = color.into() }
    }

    fn translate(&mut self, xform: &Transform) {
        for v in self {
            let position: glm::Vec4 = v.position.into();
            v.position = (xform.model() * position).into();
        }
    }

    fn flip_texture_coords_vert(&mut self, min: f32, max: f32) {
        for v in self { v.text_coord.v = min + (max - v.text_coord.v) }
    }

    fn normalize_texture_coords(&mut self, bounds: glm::Vec2) {
        for v in self { 
            v.text_coord.u /= bounds.x;
            v.text_coord.v = 1.0 - (v.text_coord.v / bounds.y);
        }
    }

    fn calc_texture_coords(&mut self, texture_rect: &Rect<u32>) {
        let rect_size = glm::vec2(texture_rect.w as f32, texture_rect.h as f32);
        let uv_offset = glm::vec2(texture_rect.x as f32, texture_rect.y as f32);

        for v in self {
            let flipped = glm::vec2(v.text_coord.u, 1. - v.text_coord.v);
            // Flip texture coordinates
            let text_dim = glm::vec2(rect_size.x * flipped.x, rect_size.y * (1. - v.text_coord.v));
            // get texture coordinate relative to top-left
            v.text_coord = (text_dim + uv_offset).into();
            // NOTE :: Only need to normalize if we are using a sprite sheet, so we will
            // have user normalize coordinates manually with normalize_coords
        }
    }
}

impl Vertex2D {
    pub const fn new() -> Self {

        Vertex2D { position: Vert2DPosition { x: 0., y: 0., z: 0. }, text_coord: Vert2DTextureCoord { u: 0., v: 0. }, color: Vert2DColor { r: 0., g: 0., b: 0., a: 0. } }      
    }
 
    pub fn as_slice(&self) -> &[f32; 9] {
        unsafe { std::mem::transmute(self) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [f32; 9] {
        unsafe { std::mem::transmute(self) }
    }
}

impl Vert2DColor {
    pub const fn white() -> Self {
        Vert2DColor { r: 1., g: 1., b: 1., a: 1. }
    }
}