
use glutin::PossiblyCurrent;
use std::ffi::CStr;
use crate::buffers::VertexArray;

pub(crate) mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

#[allow(non_upper_case_globals)]
static mut gl_context: Option<gl::Gl> = None;

pub(crate) fn load_opengl(context: &glutin::Context<PossiblyCurrent>) {

    let gl = gl::Gl::load_with(|ptr| context.get_proc_address(ptr) as *const _);

    let version = unsafe {
        let data = CStr::from_ptr(gl.GetString(gl::VERSION) as *const _)
            .to_bytes()
            .to_vec();
        String::from_utf8(data).unwrap()
    };

    println!("OpenGL version {}", version);

    unsafe {
        gl_context = Some(gl);
    }
}

pub(crate) fn opengl() -> &'static gl::Gl {
    unsafe {
        assert_eq!(gl_context.is_some(), true, "Initialize OpenGL with load_opengl() before calling opengl()");
        return gl_context.as_ref().expect("OpenGL not yet loaded");
    }
}

pub(crate) fn gl_bind_vertex_array(id: u32) {
    unsafe { opengl().BindVertexArray(id) };
}

pub(crate) fn gl_bind_array_buffer(id: u32) {
    unsafe { opengl().BindBuffer(gl::ARRAY_BUFFER, id) }
}

pub(crate) fn gl_gen_buffer() -> u32 {
    let mut id = unsafe { std::mem::zeroed() };
    unsafe { opengl().GenBuffers(1, &mut id) };
    id
}

pub(crate) fn gl_delete_buffer(id: u32) {
    unsafe {
        opengl().DeleteBuffers(1, &id);
    }
}

pub(crate) fn gl_gen_texture() -> u32 {
    let mut id = unsafe { std::mem::zeroed() };
    unsafe { opengl().GenTextures(1, &mut id) };
    id
}