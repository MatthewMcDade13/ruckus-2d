
use std::ops::Deref;

struct Tag(f32);

impl Tag {
    pub fn new(n: Option<f32>) -> Self {
        match n {
            Some(v) => Tag(v),
            None => Tag(0.)
        }
    }
}

fn new_tag<T> (t: T) -> Tag where T: Into<Option<f32>> {
    let o = t.into();
    match o {
        Some((v)) => Tag(v),
        None => Tag(0.)
    }
}

fn main() {
    let x = 25.;
    let y = Some(69.);

    let t1 = new_tag(x);
    let t2 = new_tag(y);
}
 