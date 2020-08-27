use crate::vertex::Vertex2D;
use crate::opengl::*;
use crate::graphics;
use std::mem;

#[derive(Debug, Copy, Clone)]
pub enum DataType {
    Byte = gl::BYTE as isize,
    // UByte = gl::UBYTE as isize,
    Short = gl::SHORT as isize,
    // UShort = gl::USHORT as isize,
    Int = gl::INT as isize,
    // UInt = gl::UINT as isize,
    Float = gl::FLOAT as isize
}

#[derive(Debug, Copy, Clone)]
pub enum DrawPrimitive {
    Points = gl::POINTS as isize,
    Lines = gl::LINES as isize,
    LineLoop = gl::LINE_LOOP as isize,
    LineStrip = gl::LINE_STRIP as isize,
    Triangles = gl::TRIANGLES as isize,
    TriangleStrip = gl::TRIANGLE_STRIP as isize,
    Quads = gl::QUADS as isize,
}

#[derive(Debug, Copy, Clone)]
pub enum DrawUsage {
    Static = gl::STATIC_DRAW as isize, 
    Dynamic = gl::DYNAMIC_DRAW as isize, 
    Stream = gl::STREAM_DRAW as isize
}

#[derive(Debug, Copy, Clone)]
pub enum BufferAccess {
    ReadOnly = gl::READ_ONLY as isize,
    WriteOnly = gl::WRITE_ONLY as isize,
    ReadWrite = gl::READ_WRITE as isize
}

#[derive(Debug, Copy, Clone)]
pub struct VertexAttribute {
    pub buffer_index: u32,
    pub elem_count: u32,
    pub is_instanced: bool,
    pub dtype: DataType,
    pub offset: usize,
    pub stride: usize
}

impl Default for VertexAttribute {
    fn default() -> Self {
        VertexAttribute {
            buffer_index: 0, elem_count: 0, is_instanced: false,
            dtype: DataType::Float, offset: 0, stride: 0
        }
    }
}

pub struct ElementBuffer {
    id: u32,
    pub count: u32
}

impl ElementBuffer {
    pub fn new(indicies: &[u32]) -> Self {
        ElementBuffer::new_with_draw(indicies, DrawUsage::Static)
    }

    pub fn new_with_draw(indicies: &[u32], usage: DrawUsage) -> Self {
        let eb = ElementBuffer {
            id: gl_gen_buffer(),
            count: indicies.len() as u32
        };
        eb.apply();

        unsafe {
            opengl().BufferData(gl::ELEMENT_ARRAY_BUFFER, (eb.count * mem::size_of::<u32>() as u32) as isize, indicies.as_ptr() as *const _, usage as u32)
        }
        eb
    }

    pub fn new_quad(count: u32) -> Self {
        let mut indicies: Vec<u32> = vec![0, count];
        let itr_range = 0..(indicies.len() / 6);

        for i in itr_range {
            let quad_index = i * 6;
            let vert_index = (i * 4) as u32;

            indicies[quad_index + 0] = vert_index + 0;
            indicies[quad_index + 1] = vert_index + 1;
            indicies[quad_index + 2] = vert_index + 2;

            indicies[quad_index + 3] = vert_index + 3;
            indicies[quad_index + 4] = vert_index + 4;
            indicies[quad_index + 5] = vert_index + 5;
        }

        ElementBuffer::new(indicies.as_slice())
    }

    pub fn apply(&self) {
        unsafe {
            opengl().BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id)
        }
    }

    pub fn unbind(&self) {
        gl_unbind_element_buffer();
    }
}

pub struct RenderBuffer {
    id: u32
}

impl RenderBuffer {
    pub fn new(width: i32, height: i32) -> Self {
        let id = gl_gen_buffer();
        let rb = RenderBuffer { id };
        rb.apply();

        unsafe {
            opengl().RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);
        };
        rb
    }

    pub fn apply(&self) {
        unsafe { opengl().BindBuffer(gl::RENDERBUFFER, self.id) };
    }
}

pub struct FrameBuffer {
    id: u32
}

impl FrameBuffer {
    pub fn new() -> Self {
        unsafe {
            let mut id = mem::zeroed();
            opengl().GenFramebuffers(1, &mut id);
            let fb = FrameBuffer { id };
            FrameBuffer::apply(&fb);
            return fb;
        }
    }

    pub fn apply(fb: &FrameBuffer) {
        unsafe {
            opengl().BindFramebuffer(gl::FRAMEBUFFER, fb.id)
        };
    }

    pub fn unbind() {
        unsafe {
            opengl().BindFramebuffer(gl::FRAMEBUFFER, 0)
        };
    }

    pub fn attach_render_buffer(&self, rb: &RenderBuffer) {
        FrameBuffer::apply(self);
        unsafe {
            opengl().FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, rb.id)
        };
    }

    pub fn attach_texture(&self, texture: &graphics::Texture) {
        self.attach_texture_n(texture, 0);
    }

    pub fn attach_texture_n(&self, texture: &graphics::Texture, attachment_num: u32) {
        FrameBuffer::apply(self);
        unsafe {
            opengl().FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0 + attachment_num, gl::TEXTURE_2D, texture.id(), 0);
        }
    }
}

pub struct VertexBuffer {
    id: u32,
    size_bytes: usize,
    vert_count: u32,
    pub draw_prim: DrawPrimitive,
    pub layout: Vec<VertexAttribute>
}

// TODO :: Make this act like more like an actual buffer (have checks for overflow, ect)
impl VertexBuffer {
    pub fn zeroed<T>(count: u32, usage: DrawUsage, draw_prim: DrawPrimitive) -> Self {
        let type_size = mem::size_of::<T>() as u32;
        let id = gl_gen_buffer();
        let size_bytes = (type_size * count) as isize;
        
        gl_bind_array_buffer(id);

        unsafe { opengl().BufferData(gl::ARRAY_BUFFER, size_bytes, 0 as *const _, usage as u32) };

        VertexBuffer {
            id, size_bytes: size_bytes as usize, vert_count: count, draw_prim, layout: Self::vertex2d_attributes()
        }
    }

    pub fn vertex2d_attributes() -> Vec<VertexAttribute> {
        
        let vert_size = std::mem::size_of::<Vertex2D>();
        // let uv_offset = 
        let mut attr = vec![VertexAttribute::default(); 3];
        attr[0] = VertexAttribute { buffer_index: 0, elem_count: 3, dtype: DataType::Float, stride: vert_size, offset: 0, is_instanced: false };
        attr[1] = VertexAttribute { buffer_index: 1, elem_count: 2, dtype: DataType::Float, stride: vert_size, offset: crate::memory_offset!(Vertex2D, text_coord), is_instanced: false };
        attr[2] = VertexAttribute { buffer_index: 2, elem_count: 4, dtype: DataType::Float, stride: vert_size, offset: crate::memory_offset!(Vertex2D, color), is_instanced: false };
        attr
        
    }

    /**
     * Uses default Float Datatype
    */
    pub fn new<T>(verts: &[T], usage: DrawUsage) -> Self {
        VertexBuffer::new_with_prim(verts, usage, DrawPrimitive::Triangles)
    }

    pub fn new_with_prim<T>(verts: &[T], usage: DrawUsage, draw_prim: DrawPrimitive) -> Self {
        let id = gl_gen_buffer();
        let size_bytes = mem::size_of::<T>() * verts.len();

        let mut vb = VertexBuffer {
            id, size_bytes, draw_prim, vert_count: verts.len() as u32, layout: Self::vertex2d_attributes()
        };
        vb.alloc(verts, usage);

        vb
    }
  
    /**
     * Uses default Float Datatype
    */
    pub fn alloc<T>(&mut self, verts: &[T], usage: DrawUsage) {
        self.apply();
        self.size_bytes = verts.len() * mem::size_of::<T>();
        self.vert_count = verts.len() as u32;

        unsafe { 
            opengl().BufferData(gl::ARRAY_BUFFER, self.size_bytes as isize, verts.as_ptr() as *const _, usage as u32) 
        };
    }

    pub fn write<T>(&self, verts: &[T],  offset: isize) {
        self.apply();
        unsafe {
            opengl().BufferSubData(gl::ARRAY_BUFFER, offset, (mem::size_of::<T>() * verts.len()) as isize, verts.as_ptr() as *const _)
        }
    }

    pub fn copy_data(&self, other: &VertexBuffer, read_offset: isize, write_offset: isize, size: isize) {
        let gl = opengl();
        unsafe { 
            gl.BindBuffer(gl::COPY_READ_BUFFER, other.id);
            gl.BindBuffer(gl::COPY_WRITE_BUFFER, self.id);
            gl.CopyBufferSubData(gl::COPY_READ_BUFFER, gl::COPY_WRITE_BUFFER, read_offset, write_offset, size);
        }
    }

    pub fn vert_count(&self) -> u32 { self.vert_count }

    pub unsafe fn map_buffer(&self, access: BufferAccess) -> *mut std::ffi::c_void {
        self.apply();
        opengl().MapBuffer(gl::ARRAY_BUFFER, access as u32)
    }

    pub unsafe fn unmap(&self) {
        self.apply();
        opengl().UnmapBuffer(gl::ARRAY_BUFFER);
    }

    pub fn apply(&self) {
        gl_bind_array_buffer(self.id)
    }

    pub fn unbind(&self) {
        gl_bind_array_buffer(0);
    }

}

pub struct VAO {
    id: u32
}

impl VAO {

    pub fn new() -> Self {
        VAO {
            id: gl_gen_vertex_array()
        }
    }

    pub fn set_buffer_layout(&self, buffer: &VertexBuffer) {
        self.apply();
        buffer.apply();

        let gl = opengl();
        for attrib in buffer.layout.iter() {
            unsafe {
                gl.VertexAttribPointer(
                    attrib.buffer_index, attrib.elem_count as i32, 
                    attrib.dtype as u32, gl::FALSE, attrib.stride as i32, 
                    attrib.offset as *const _
                );
                gl.EnableVertexAttribArray(attrib.buffer_index);
                gl.VertexAttribDivisor(attrib.buffer_index, if attrib.is_instanced { 1 } else { 0 });
            }
        }
        buffer.unbind();
        self.unbind();
    }

    pub fn apply(&self) {
        gl_bind_vertex_array(self.id);
    }

    pub fn unbind(&self) {
        gl_bind_vertex_array(0);
    }
}

impl Drop for ElementBuffer {

    fn drop(&mut self) { 
        gl_delete_buffer(self.id);
    }
}

impl Drop for RenderBuffer {

    fn drop(&mut self) { 
        unsafe { opengl().DeleteRenderbuffers(1, &self.id) }
    }
}

impl Drop for FrameBuffer {

    fn drop(&mut self) { 
        unsafe { opengl().DeleteFramebuffers(1, &self.id) }
    }
}

impl Drop for VertexBuffer {

    fn drop(&mut self) { 
        gl_delete_buffer(self.id)
    }
}

impl Drop for VAO {
    
    fn drop(&mut self) { 
        gl_delete_buffer(self.id)
    }
}

pub fn set_vertex_layout(buffer: &VertexBuffer, attribs: &[VertexAttribute]) {
let gl = opengl();
    buffer.apply();

   
}