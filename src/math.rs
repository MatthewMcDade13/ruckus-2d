use std::ops::Sub;
use std::ops::Add;
use std::ops::Mul;
use std::ops::Div;
use num::Num;

// TODO :: Remove this file and just use a math library (glm hopefully if we can find bindings/port)

pub trait NumDefault: Num + Default {}
impl <T: Num + Default> NumDefault for T {}

#[repr(C)]
pub struct Vec2<T: NumDefault> {
    pub x: T, 
    pub y: T
}

impl<T> Vec2<T> where T: NumDefault {
    pub fn new(x: T, y: T) -> Self {
        Vec2 { x, y }
    }
}

impl<T> Default for Vec2<T> where T: NumDefault {
    fn default() -> Self {
        Vec2 { x: T::default(), y: T::default() }
    }
}

impl<T> Sub for Vec2<T> where T: NumDefault {
    
    type Output = Vec2<T>;
    fn sub(self, other: Vec2<T>) -> <Self as Sub<Vec2<T>>>::Output { 
        Vec2 { 
            x: self.x - other.x,
            y: self.y - other.y
        }
    }
}

impl<T> Add for Vec2<T> where T: NumDefault {

    type Output =  Vec2<T>;
    fn add(self, other:  Vec2<T>) -> <Self as Add< Vec2<T>>>::Output { 
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y
        }
    }
}

impl<T> Mul for Vec2<T> where T: NumDefault {

    type Output =  Vec2<T>;
    fn mul(self, other:  Vec2<T>) -> <Self as Mul< Vec2<T>>>::Output { 
        Vec2 {
            x: self.x * other.x,
            y: self.y * other.y
        }
    }
}

impl<T> Div for Vec2<T> where T: NumDefault {

    type Output =  Vec2<T>;
    fn div(self, other:  Vec2<T>) -> <Self as Div< Vec2<T>>>::Output { 
        Vec2 {
            x: self.x / other.x,
            y: self.y / other.y
        }
    }
}

pub type Vec2i = Vec2<i32>;
pub type Vec2f = Vec2<f32>;
pub type Vec2ui = Vec2<u32>;
