use crate::opengl::*;
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

pub struct ElementBuffer {
    id: u32,
    count: usize
}

impl ElementBuffer {
    pub fn new(indicies: Vec<u32>) -> Self {
        ElementBuffer::new_with_draw(indicies, DrawUsage::Static)
    }

    pub fn new_with_draw(indicies: Vec<u32>, usage: DrawUsage) -> Self {
        let eb = ElementBuffer {
            id: gl_gen_buffer(),
            count: indicies.len()
        };
        eb.bind();

        unsafe {
            opengl().BufferData(gl::ELEMENT_ARRAY_BUFFER, (eb.count * mem::size_of::<u32>()) as isize, indicies.as_ptr() as *const _, usage as u32)
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

        ElementBuffer::new(indicies)
    }

    pub fn bind(&self) {
        unsafe {
            opengl().BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id)
        }
    }
}

pub struct RenderBuffer {
    id: u32
}

impl RenderBuffer {
    pub fn new(width: i32, height: i32) -> Self {
        let id = gl_gen_buffer();
        let rb = RenderBuffer { id };
        rb.bind();

        unsafe {
            opengl().RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);
        };
        rb
    }

    pub fn bind(&self) {
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
            FrameBuffer::bind(&fb);
            return fb;
        }
    }

    fn bind(fb: &FrameBuffer) {
        unsafe {
            opengl().BindFramebuffer(gl::FRAMEBUFFER, fb.id)
        };
    }

    fn unbind() {
        unsafe {
            opengl().BindFramebuffer(gl::FRAMEBUFFER, 0)
        };
    }

    fn attach_render_buffer(&self, rb: &RenderBuffer) {
        FrameBuffer::bind(self);
        unsafe {
            opengl().FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, rb.id)
        };
    }

    // TODO: Implement these after we have finished Texture
    // fn attach_texture(texture: &Texture) {}
    // fn attach_texture_n(texture: &Texture, num_attachments: i32) {}
}

pub struct VertexBuffer {
    id: u32,
    dtype: DataType,
    vert_count: usize,
    size_bytes: usize
}

impl VertexBuffer {
    pub fn zeroed<T>(count: usize, usage: DrawUsage, dtype: DataType) -> Self {
        let type_size = mem::size_of::<T>();
        let id = gl_gen_buffer();
        let size_bytes = (type_size * count) as isize;
        
        gl_bind_array_buffer(id);

        unsafe { opengl().BufferData(gl::ARRAY_BUFFER, size_bytes, 0 as *const _, usage as u32) };

        VertexBuffer {
            id, dtype, vert_count: 0, size_bytes: size_bytes as usize
        }
    }

    /**
     * Uses default Float Datatype
    */
    pub fn new<T>(verts: &[T], count: usize, usage: DrawUsage) -> Self {
        VertexBuffer::new_t(verts, count, usage, DataType::Float)
    }

    pub fn new_t<T>(verts: &[T], count: usize, usage: DrawUsage, dtype: DataType) -> Self {
        let id = gl_gen_buffer();
        let size_bytes = mem::size_of::<T>() * count;

        let mut vb = VertexBuffer {
            id, dtype, vert_count: count, size_bytes
        };
        vb.alloc(verts, count, usage);

        vb

    }
  
    /**
     * Uses default Float Datatype
    */
    pub fn alloc<T>(&mut self, verts: &[T], count: usize, usage: DrawUsage) {
        self.bind();
        self.size_bytes = count * mem::size_of::<T>();

        unsafe { 
            opengl().BufferData(gl::ARRAY_BUFFER, self.size_bytes as isize, verts.as_ptr() as *const _, usage as u32) 
        };
    }

    pub fn write<T>(&self, verts: &[T], count: usize, offset: isize) {
        self.bind();
        unsafe {
            opengl().BufferSubData(gl::ARRAY_BUFFER, offset, (mem::size_of::<T>() * count) as isize, verts.as_ptr() as *const _)
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

    pub unsafe fn map_buffer(&self, access: BufferAccess) -> *mut std::ffi::c_void {
        self.bind();
        opengl().MapBuffer(gl::ARRAY_BUFFER, access as u32)
    }

    pub unsafe fn unmap(&self) {
        self.bind();
        opengl().UnmapBuffer(gl::ARRAY_BUFFER);
    }

    pub fn bind(&self) {
        gl_bind_array_buffer(self.id)
    }

}

pub struct VertexArray {
    id: u32
}

impl VertexArray {
    pub fn bind(&self) {
        gl_bind_vertex_array(self.id);
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

impl Drop for VertexArray {
    
    fn drop(&mut self) { 
        gl_delete_buffer(self.id)
    }
}

pub fn set_vertex_layout(buffer: &VertexBuffer, attribs: &[VertexAttribute]) {
    let gl = opengl();
    buffer.bind();

    for attrib in attribs.iter() {
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
}