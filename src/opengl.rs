
use glutin::PossiblyCurrent;
use std::ffi::CStr;
use crate::buffers::VertexArray;
use crate::graphics::{ShaderType, Shader};

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

pub(crate) fn gl_get_uniform_location(shader_id: u32, name: &str) -> i32 {
    unsafe { opengl().GetUniformLocation(shader_id, name.as_ptr() as *const _) }
}

pub(crate) fn gl_set_uniform_4f(location: i32, floats: (f32, f32, f32, f32)) {
    unsafe { opengl().Uniform4f(location, floats.0, floats.1, floats.2, floats.3) }
}
pub(crate) fn gl_set_uniform_3f(location: i32, floats: (f32, f32, f32)) {
    unsafe { opengl().Uniform3f(location, floats.0, floats.1, floats.2) }
}
pub(crate) fn gl_set_uniform_2f(location: i32, floats: (f32, f32)) {
    unsafe { opengl().Uniform2f(location, floats.0, floats.1) }
}
pub(crate) fn gl_set_uniform_f(location: i32, n: f32) {
    unsafe { opengl().Uniform1f(location, n) }
}
// TODO :: Implement after we have figured out matrix situation...
// pub(crate) fn gl_set_uniform_matrix(location: i32, floats: (f32, f32, f32, f32)) {

// }
pub(crate) fn gl_set_uniform_i(location: i32, n: i32) {
    unsafe { opengl().Uniform1i(location, n) }
}

pub(crate) fn gl_compile_shader_from_file(path: &str, shader_type: ShaderType) -> Result<u32, String> {
    let shader_source = match read_file(path) {
        Ok(s) => s,
        Err(e) => return Err(format!("FILE READ ERROR :: {}", e))
    };
    gl_compile_shader(&shader_source, shader_type)
}


pub(crate) fn read_file(path: &str) -> std::io::Result<String> {
    use std::fs::File;
    use std::io::BufReader;
    use std::io::prelude::*;

    let f = File::open(path)?;
    let mut reader = BufReader::new(f);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;
    Ok(contents)
}

const GL_MAX_LOG_BUFFER_LENGTH: usize = 512;

// TODO :: Implement a way to parse given shader source files so we can find and fill out all uniform locations on shader
pub(crate) fn gl_compile_shader(source: &str, stype: ShaderType) -> Result<u32, String> {
    let gl = opengl();
    let sid = unsafe { gl.CreateShader(stype as u32) };
    unsafe {
        gl.ShaderSource(sid, 1, source.as_ptr() as *const *const i8, 0 as *const _);
        gl.CompileShader(sid);

        let mut success = std::mem::zeroed();
        gl.GetShaderiv(sid, gl::COMPILE_STATUS, &mut success);

        if success == 0 {

            gl.DeleteShader(sid);
            let mut info_log: Vec<u8> = vec![0, GL_MAX_LOG_BUFFER_LENGTH as u8];

            gl.GetShaderInfoLog(sid, info_log.len() as i32, 0 as *mut _, info_log.as_mut_ptr() as *mut _);
            let shader_error = String::from_utf8(info_log).unwrap();
            return Err(format!("SHADER COMPILE ERROR\n\r--------------------\n\r{}", shader_error))
        }
    };        
    Ok(sid)
}

pub(crate) fn gl_create_shader_program(vert_id: u32, frag_id: u32) -> Result<u32, String> {
    unsafe {
        let gl = opengl();
        let id = gl.CreateProgram();

        gl.AttachShader(id, vert_id);
        gl.AttachShader(id, frag_id);
        gl.LinkProgram(id);

        let mut success = std::mem::zeroed();
        gl.GetProgramiv(id, gl::LINK_STATUS, &mut success);

        if success == 0 {
            gl.DeleteProgram(id);

            let mut info_log: Vec<u8> = vec![0, GL_MAX_LOG_BUFFER_LENGTH as u8];

            gl.GetProgramInfoLog(id, info_log.len() as i32, 0 as *mut _, info_log.as_mut_ptr() as *mut _);
            let shader_error = String::from_utf8(info_log).unwrap();
            return Err(format!("SHADER PROGRAM LINK ERROR\n\r-------------------\n\r{}", shader_error))
        }

        gl.DeleteShader(vert_id);
        gl.DeleteShader(frag_id);

        Ok(id)
    }
}
