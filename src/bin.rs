
use glutin::dpi::PhysicalSize;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use nalgebra_glm as glm;

use ruckus::vertex::*;
use ruckus::sys::*;
use ruckus::{Renderer};



// TODO :: Bad example, we need an easy way to draw quads as static to the screen, otherwise pick an easier example
fn main() {
    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title("Ruckus Example");

    let windowed_context = ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)))
        .build_windowed(wb, &el).unwrap();

    let context = unsafe { windowed_context.make_current().unwrap() };
    let window = context.window();
    let PhysicalSize { width, height } = window.inner_size();

    ruckus::opengl::load_opengl(|ptr| context.get_proc_address(ptr) as *const _);
    let renderer = Renderer::new(width, height);

    let mut qxform = Transform::from_position(glm::vec2(width as f32 * 0.5, height as f32 * 0.5));
    let mut q = Quad::default();

    q.verts[0].color = Vert2DColor { r: 1., g: 0., b: 0., a: 1. };
    q.verts[1].color = Vert2DColor { r: 0., g: 1., b: 0., a: 1. };
    q.verts[2].color = Vert2DColor { r: 0., g: 0., b: 1., a: 1. };
    q.verts[3].color = Vert2DColor { r: 1., g: 1., b: 1., a: 1. };

    qxform.scale(glm::vec2(500., 500.));

    el.run(move |event, _, control_flow| {
        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                },
                _ => {}
            },
            _ => {}
        }

        renderer.clear_black();
        renderer.draw_quad(&q, &qxform, None);
        let _ = context.swap_buffers();

    });

}

// trait Foo {}

// struct MyFoo;

// impl Foo for MyFoo {}

// struct Bar<'a> {
//     foo: &'a (dyn Foo + 'a),
// }

// impl<'a> Bar<'a> {
//     fn new(the_foo: &'a Foo) -> Bar<'a> {
//         Bar { foo: the_foo }
//     }

//     fn get_foo(&'a self) -> &'a Foo {
//         self.foo
//     }
// }

// fn main() {
//     let myfoo = MyFoo;
//     let mybar = Bar::new(&myfoo as &Foo);


// }

