
struct A {
    x: Point
}

struct Point { pub x: f32, pub y: f32 }

impl Drop for Point {
    fn drop(&mut self) {
        println!("DROP CALLED");
    }
}

// impl Clone for Point {
    
//     fn clone(&self) -> Self { 
//         Point { x: self.x, y: self.y }
//     }
// }

impl A {
    pub fn new(a: Point) -> A {
        A { x: a }
    }
}


fn main() {
    let p = Point {x: 1., y: 2.};
    let a = A::new(p);
    println!("AFTER NEW");
    println!("{}", a.x.x);
    // p.x = 5.;
}
 