
use crate::buffers::VertexAttribute;
use crate::buffers::VertexBuffer;
use crate::buffers::DrawPrimitive;
use std::ffi::CStr;
use std::collections::HashMap;
use crate::sys::read_file;
use crate::graphics::{ShaderType};

pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

static mut GL_CONTEXT: Option<gl::Gl> = None;

pub fn load_opengl<F>(loader: F) where F: FnMut(&'static str)  -> *const std::ffi::c_void {

    let gl = gl::Gl::load_with(loader);

    let version = unsafe {
        let data = CStr::from_ptr(gl.GetString(gl::VERSION) as *const _)
            .to_bytes()
            .to_vec();
        String::from_utf8(data).unwrap()
    };

    println!("OpenGL version {}", version);

    unsafe {
        GL_CONTEXT = Some(gl);
    }
}

pub fn opengl() -> &'static gl::Gl {
    unsafe {
        GL_CONTEXT.as_ref().expect("Initialize OpenGL with load_opengl() before using any OpenGL draw calls)")
    }
}

#[allow(dead_code)]
pub(crate) fn gl_bind_vertex_array(id: u32) {
    unsafe { opengl().BindVertexArray(id) };
}

#[allow(dead_code)]
pub(crate) fn gl_bind_array_buffer(id: u32) {
    unsafe { opengl().BindBuffer(gl::ARRAY_BUFFER, id) }
}

#[allow(dead_code)]
pub(crate) fn gl_gen_vertex_array() -> u32 {
    unsafe {
        let mut id = std::mem::zeroed();
        opengl().GenVertexArrays(1, &mut id);
        id
    }
}

#[allow(dead_code)]
pub(crate) fn gl_gen_buffer() -> u32 {
    let mut id = unsafe { std::mem::zeroed() };
    unsafe { opengl().GenBuffers(1, &mut id) };
    id
}

#[allow(dead_code)]
pub(crate) fn gl_delete_buffer(id: u32) {
    unsafe {
        opengl().DeleteBuffers(1, &id);
    }
}

#[allow(dead_code)]
pub(crate) fn gl_gen_texture() -> u32 {
    let mut id = unsafe { std::mem::zeroed() };
    unsafe { opengl().GenTextures(1, &mut id) };
    id
}

#[allow(dead_code)]
pub(crate) fn gl_get_uniform_location(shader_id: u32, name: &str) -> i32 {
    unsafe { opengl().GetUniformLocation(shader_id, name.as_ptr() as *const _) }
}

#[allow(dead_code)]
pub(crate) fn gl_set_uniform_4f(location: i32, floats: (f32, f32, f32, f32)) {
    unsafe { opengl().Uniform4f(location, floats.0, floats.1, floats.2, floats.3) }
}

#[allow(dead_code)]
pub(crate) fn gl_set_uniform_3f(location: i32, floats: (f32, f32, f32)) {
    unsafe { opengl().Uniform3f(location, floats.0, floats.1, floats.2) }
}

#[allow(dead_code)]
pub(crate) fn gl_set_uniform_2f(location: i32, floats: (f32, f32)) {
    unsafe { opengl().Uniform2f(location, floats.0, floats.1) }
}

#[allow(dead_code)]
pub(crate) fn gl_set_uniform_f(location: i32, n: f32) {
    unsafe { opengl().Uniform1f(location, n) }
}
#[allow(dead_code)]
pub(crate) fn gl_set_uniform_matrix(location: i32, mat: &[f32]) {
    gl_set_uniform_matrix_xpose(location, mat, false)
}

#[allow(dead_code)]
pub(crate) fn gl_set_uniform_matrix_xpose(location: i32, mat: &[f32], transpose: bool) {
    unsafe { opengl().UniformMatrix4fv(location, 1, transpose as u8, mat.as_ptr()) }
}

#[allow(dead_code)]
pub(crate) fn gl_set_uniform_i(location: i32, n: i32) {
    unsafe { opengl().Uniform1i(location, n) }
}

#[allow(dead_code)]
pub(crate) fn gl_compile_shader_from_file(path: &str, shader_type: ShaderType) -> Result<u32, String> {
    let shader_source = match read_file(path) {
        Ok(s) => s,
        Err(e) => return Err(format!("FILE READ ERROR :: {}", e))
    };
    gl_compile_shader(shader_source.as_bytes(), shader_type)
}

#[allow(dead_code)]
pub fn gl_draw_arrays(start: u32, vert_count: u32, prim: DrawPrimitive) {
    unsafe { opengl().DrawArrays(prim as u32, start as i32, vert_count as i32) }
}

#[allow(dead_code)]
pub fn gl_draw_elements(count: u32, prim: DrawPrimitive) {
    unsafe { opengl().DrawElements(prim as u32, count as i32, gl::UNSIGNED_INT, 0 as *const _) }
}

#[allow(dead_code)]
const GL_MAX_LOG_BUFFER_LENGTH: usize = 2564;

#[allow(dead_code)]
// TODO :: Consolidate all default shaders (except instanced, do those too but separate) into a single shader with #ifdef to make
//         everything more clean and DRY
pub(crate) fn gl_compile_shader(source: &[u8], stype: ShaderType) -> Result<u32, String> {
    let gl = opengl();
    let sid = unsafe { gl.CreateShader(stype as u32) };
    unsafe {
        let source = vec![source.as_ptr() as *const _];
        gl.ShaderSource(sid, 1, source.as_ptr() as *const _, 0 as *const _);
        gl.CompileShader(sid);

        let mut success = std::mem::zeroed();
        gl.GetShaderiv(sid, gl::COMPILE_STATUS, &mut success);

        if success == 0 {

            let mut info_log = [0; GL_MAX_LOG_BUFFER_LENGTH];
            gl.GetShaderInfoLog(sid, GL_MAX_LOG_BUFFER_LENGTH as i32, 0 as *mut _, info_log.as_mut_ptr() as *mut _);
        
            let cerror = std::ffi::CStr::from_ptr(info_log.as_ptr()).to_str().unwrap();
            let shader_error = String::from(cerror);
            gl.DeleteShader(sid);
            return Err(format!("SHADER COMPILE ERROR\n\r--------------------\n\r{}", shader_error))
        }
    };        
    Ok(sid)
}

#[allow(dead_code)]
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
            
            let mut info_log = [0; GL_MAX_LOG_BUFFER_LENGTH];
            
            gl.GetProgramInfoLog(id, info_log.len() as i32, 0 as *mut _, info_log.as_mut_ptr() as *mut _);

            let cerror = std::ffi::CStr::from_ptr(info_log.as_ptr()).to_str().unwrap();
            let shader_error = String::from(cerror);   

            gl.DeleteProgram(id);
            return Err(format!("SHADER PROGRAM LINK ERROR\n\r-------------------\n\r{}", shader_error));
        }

        gl.DeleteShader(vert_id);
        gl.DeleteShader(frag_id);

        Ok(id)
    }
}


#[allow(dead_code)]
pub(crate) fn gl_get_active_uniforms(shader_id: u32) -> HashMap<String, i32> {
    let gl = opengl();
    let count = unsafe {
        let mut count = std::mem::zeroed();
        gl.GetProgramiv(shader_id, gl::ACTIVE_UNIFORMS, &mut count);
        count
    };

    let mut result = HashMap::new();

    for i in 0..count {
        const NAME_SIZE: usize = 32;
        unsafe {
            let mut length = std::mem::zeroed();
            let mut size = std::mem::zeroed();
            let mut dtype = std::mem::zeroed();
            let mut name = vec![0; NAME_SIZE];
            gl.GetActiveUniform(shader_id, i as u32, NAME_SIZE as i32, &mut length, &mut size, &mut dtype, name.as_mut_ptr() as *mut _);   

            let name = unsafe { std::ffi::CStr::from_ptr(name.as_ptr()).to_str().unwrap() };
            let k = String::from(name);

            let v = gl_get_uniform_location(shader_id, &k);
            let _ = result.insert(k, v);
        }
    }
    result
}

#[allow(dead_code)]
pub(crate) fn set_vertex_layout(buffer: &VertexBuffer, attribs: &[VertexAttribute]) {
    buffer.apply();

    for attr in attribs.iter() {
        unsafe {
            let gl = opengl();
            gl.VertexAttribPointer(attr.buffer_index, attr.elem_count as i32, attr.dtype as u32, gl::FALSE, attr.stride as i32, attr.offset as *const _);
            gl.EnableVertexAttribArray(attr.buffer_index);
            gl.VertexAttribDivisor(attr.buffer_index, if attr.is_instanced { 1 } else { 0 })
        }
    }
}


#[allow(dead_code)]
pub(crate) fn gl_unbind_element_buffer() {
    unsafe { opengl().BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0) }
}

#[allow(dead_code)]
pub(crate) fn gl_unbind_array_buffer() {
    unsafe { opengl().BindBuffer(gl::ARRAY_BUFFER, 0) }
}
