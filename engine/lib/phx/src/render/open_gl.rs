use super::*;
use crate::common::*;
use gl::types::*;

use std::ffi::{CStr, CString};

use glutin::{display::GetGlDisplay, prelude::GlDisplay};
use tracing::{debug, error};

pub fn check_error(file: &str, line: u32, msg: &str) {
    let msg_str = if !msg.is_empty() {
        format!(", Message: {msg}")
    } else {
        String::new()
    };

    let errorID = unsafe { gl::GetError() };
    let error = match errorID {
        gl::NO_ERROR => return,
        gl::INVALID_ENUM => "GL_INVALID_ENUM".into(),
        gl::INVALID_VALUE => "GL_INVALID_VALUE".into(),
        gl::INVALID_OPERATION => "GL_INVALID_OPERATION".into(),
        gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION".into(),
        gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY".into(),
        gl::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW".into(),
        gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW".into(),
        _ => format!("gl::GetError returned unknown error code {errorID}"),
    };

    error!("OpenGL_CheckError: {error} at {file}:{line}{msg_str}");
}

static mut gl_begin_count: u32 = 0;

macro_rules! gl_error_check {
    ($name:ident, $msg:expr) => {
        // NOTE: uncomment next 7 lines to catch OpenGl errors (for debugging purposes only - heavily impacts performance)
        // unsafe{
        //     if gl_begin_count == 0 {
        //         let caller_location = std::panic::Location::caller();
        //         let msg_str = format!("{}({})", stringify!($name), $msg);
        //         check_error(caller_location.file(), caller_location.line(), &msg_str);
        //     }
        // }
    };
}

macro_rules! gl_func {
    ($gl_name:ident, $name:ident($($param_name:ident: $param_ty:ty),*) -> $ret:ty, $msg:expr) => {
        #[inline]
        #[track_caller]
        pub fn $name($($param_name: $param_ty),*) -> $ret {
            let res = unsafe { gl::$gl_name($($param_name),*) };

            gl_error_check!($name, $msg);

            res
        }
    };
    ($gl_name:ident, $name:ident($($param_name:ident: $param_ty:ty),*) -> $ret:ty) => {
        gl_func!($gl_name, $name($($param_name: $param_ty),*) -> $ret, "");
    };
    ($gl_name:ident, $name:ident($($param_name:ident: $param_ty:ty),*), $msg:expr) => {
        gl_func!($gl_name, $name($($param_name: $param_ty),*) -> (), $msg);
    };
    ($gl_name:ident, $name:ident($($param_name:ident: $param_ty:ty),*)) => {
        gl_func!($gl_name, $name($($param_name: $param_ty),*) -> (), "");
    };
}

// Common
#[inline]
#[track_caller]
pub fn gl_get_string(name: GLenum) -> Option<String> {
    unsafe {
        let s = gl::GetString(name);

        gl_error_check!(gl_get_string, "");

        (!s.is_null()).then(|| CStr::from_ptr(s.cast()).to_string_lossy().to_string())
    }
}

// Use marker to prevent calling glGetError between glBegin/glEnd
#[inline]
#[track_caller]
pub fn gl_begin(mode: GLenum) {
    unsafe {
        gl::Begin(mode);

        gl_begin_count += 1;
    }
}

#[inline]
#[track_caller]
pub fn gl_end() {
    unsafe {
        gl_begin_count -= 1;

        gl::End();

        gl_error_check!(gl_end, "");
    }
}

gl_func!(Enable, gl_enable(cap: GLenum), &format!("cap = {cap}"));
gl_func!(Disable, gl_disable(cap: GLenum), &format!("cap = {cap}"));
gl_func!(Hint, gl_hint(target: GLenum, mode: GLenum));
gl_func!(BlendFunc, gl_blend_func(sfactor: GLenum, dfactor: GLenum));
gl_func!(CullFace, gl_cull_face(mode: GLenum));
gl_func!(DepthMask, gl_depth_mask(flag: GLboolean));
gl_func!(PolygonMode, gl_polygon_mode(face: GLenum, mode: GLenum));
gl_func!(
    Scissor,
    gl_scissor(x: GLint, y: GLint, width: GLsizei, height: GLsizei)
);
gl_func!(PixelStorei, gl_pixel_storei(pname: GLenum, param: GLint));
gl_func!(DepthFunc, gl_depth_func(func: GLenum));

// Draw
gl_func!(Clear, gl_clear(mask: GLbitfield));
gl_func!(CheckFramebufferStatus, gl_check_framebuffer_status(target: GLenum) -> GLenum);
gl_func!(Finish, gl_finish());
gl_func!(
    ClearColor,
    gl_clear_color(red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat)
);
gl_func!(PointSize, gl_point_size(size: GLfloat));
gl_func!(LineWidth, gl_line_width(width: GLfloat));
gl_func!(ClearDepth, gl_clear_depth(depth: GLdouble));
gl_func!(
    Color4f,
    gl_color4f(red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat)
);
gl_func!(Vertex2f, gl_vertex2f(x: GLfloat, y: GLfloat));
gl_func!(Vertex3f, gl_vertex3f(x: GLfloat, y: GLfloat, z: GLfloat));

// Matrix
gl_func!(MatrixMode, gl_matrix_mode(mode: GLenum));
gl_func!(PushMatrix, gl_push_matrix());
gl_func!(PopMatrix, gl_pop_matrix());
gl_func!(LoadIdentity, gl_load_identity());
gl_func!(LoadMatrixf, gl_load_matrixf(m: *const GLfloat));
gl_func!(MultMatrixf, gl_mult_matrixf(m: *const GLfloat));
gl_func!(MultMatrixd, gl_mult_matrixd(m: *const GLdouble));
gl_func!(Scalef, gl_scalef(x: GLfloat, y: GLfloat, z: GLfloat));
gl_func!(Scaled, gl_scaled(x: GLdouble, y: GLdouble, z: GLdouble));
gl_func!(
    Translatef,
    gl_translatef(x: GLfloat, y: GLfloat, z: GLfloat)
);
gl_func!(
    Translated,
    gl_translated(x: GLdouble, y: GLdouble, z: GLdouble)
);
gl_func!(
    Rotated,
    gl_rotated(angle: GLdouble, x: GLdouble, y: GLdouble, z: GLdouble)
);
gl_func!(
    GetIntegerv,
    gl_get_integerv(pname: GLenum, data: &mut GLint)
);
gl_func!(GetFloatv, gl_get_floatv(pname: GLenum, data: *mut GLfloat));

// Mesh
gl_func!(BindBuffer, gl_bind_buffer(target: GLenum, buffer: GLuint));
gl_func!(
    DeleteBuffers,
    gl_delete_buffers(n: GLsizei, buffers: *const GLuint)
);
gl_func!(GenBuffers, gl_gen_buffers(n: GLsizei, buffers: *mut GLuint));
gl_func!(
    BufferData,
    gl_buffer_data(
        target: GLenum,
        size: GLsizeiptr,
        data: *const libc::c_void,
        usage: GLenum
    )
);
gl_func!(
    EnableVertexAttribArray,
    gl_enable_vertex_attrib_array(index: GLuint)
);
gl_func!(
    DisableVertexAttribArray,
    gl_disable_vertex_attrib_array(index: GLuint)
);
gl_func!(
    VertexAttribPointer,
    gl_vertex_attrib_pointer(
        index: GLuint,
        size: GLint,
        type_: GLenum,
        normalized: GLboolean,
        stride: GLsizei,
        pointer: *const libc::c_void
    )
);
gl_func!(
    DrawElements,
    gl_draw_elements(
        mode: GLenum,
        count: GLsizei,
        type_: GLenum,
        indices: *const libc::c_void
    )
);

// Render target
gl_func!(
    DrawBuffers,
    gl_draw_buffers(n: GLsizei, bufs: *const GLenum)
);
gl_func!(
    GenFramebuffers,
    gl_gen_framebuffers(n: GLsizei, framebuffers: *mut GLuint)
);
gl_func!(
    DeleteFramebuffers,
    gl_delete_framebuffers(n: GLsizei, framebuffers: *mut GLuint)
);
gl_func!(
    BlendFuncSeparate,
    gl_blend_func_separate(
        sfactor_rgb: GLenum,
        dfactor_rgb: GLenum,
        sfactor_alpha: GLenum,
        dfactor_alpha: GLenum
    )
);
gl_func!(
    BindFramebuffer,
    gl_bind_framebuffer(target: GLenum, framebuffer: GLuint)
);
gl_func!(
    FramebufferTexture2D,
    gl_framebuffer_texture2d(
        target: GLenum,
        attachment: GLenum,
        textarget: GLenum,
        texture: GLuint,
        level: GLint
    )
);
gl_func!(
    FramebufferTexture3D,
    gl_framebuffer_texture3d(
        target: GLenum,
        attachment: GLenum,
        textarget: GLenum,
        texture: GLuint,
        level: GLint,
        zoffset: GLint
    )
);

// Shader
gl_func!(Uniform1f, gl_uniform1f(location: GLint, v0: GLfloat));
gl_func!(
    Uniform2f,
    gl_uniform2f(location: GLint, v0: GLfloat, v1: GLfloat)
);
gl_func!(
    Uniform3f,
    gl_uniform3f(location: GLint, v0: GLfloat, v1: GLfloat, v2: GLfloat)
);
gl_func!(
    Uniform4f,
    gl_uniform4f(
        location: GLint,
        v0: GLfloat,
        v1: GLfloat,
        v2: GLfloat,
        v3: GLfloat
    )
);
gl_func!(Uniform1i, gl_uniform1i(location: GLint, v0: GLint));
gl_func!(
    Uniform2i,
    gl_uniform2i(location: GLint, v0: GLint, v1: GLint)
);
gl_func!(
    Uniform3i,
    gl_uniform3i(location: GLint, v0: GLint, v1: GLint, v2: GLint)
);
gl_func!(
    Uniform4i,
    gl_uniform4i(location: GLint, v0: GLint, v1: GLint, v2: GLint, v3: GLint)
);
gl_func!(
    GetShaderiv,
    gl_get_shaderiv(shader: GLuint, pname: GLenum, params: &mut GLint)
);
gl_func!(
    GetProgramiv,
    gl_get_programiv(program: GLuint, pname: GLenum, params: &mut GLint)
);
gl_func!(
    GetShaderInfoLog,
    gl_get_shader_info_log(
        shader: GLuint,
        buf_size: GLsizei,
        length: *mut GLsizei,
        info_log: *mut GLchar
    )
);
gl_func!(
    GetProgramInfoLog,
    gl_get_program_info_log(
        program: GLuint,
        buf_size: GLsizei,
        length: *mut GLsizei,
        info_log: *mut GLchar
    )
);
gl_func!(
    GetUniformLocation,
    gl_get_uniform_location(program: GLuint, name: *const GLchar) -> GLint
);

gl_func!(
    CreateShader,
    gl_create_shader(type_: GLenum) -> GLuint
);
gl_func!(
    ShaderSource,
    gl_shader_source(
        shader: GLuint,
        count: GLsizei,
        string: *const *const GLchar,
        length: *const GLint
    )
);
gl_func!(CompileShader, gl_compile_shader(shader: GLuint));
gl_func!(DeleteShader, gl_delete_shader(shader: GLuint));
gl_func!(
    UniformMatrix4fv,
    gl_uniform_matrix4fv(
        location: GLint,
        count: GLsizei,
        transpose: GLboolean,
        value: *const GLfloat
    )
);

gl_func!(
    CreateProgram,
    gl_create_program() -> GLuint
);
gl_func!(
    AttachShader,
    gl_attach_shader(program: GLuint, shader: GLuint)
);
gl_func!(
    BindAttribLocation,
    gl_bind_attrib_location(program: GLuint, index: GLuint, name: *const GLchar)
);
gl_func!(LinkProgram, gl_link_program(program: GLuint));
gl_func!(DeleteProgram, gl_delete_program(program: GLuint));
gl_func!(UseProgram, gl_use_program(program: GLuint));

// Texture
gl_func!(ActiveTexture, gl_active_texture(texture: GLenum));
gl_func!(
    BindTexture,
    gl_bind_texture(target: GLenum, texture: GLuint)
);
gl_func!(
    TexParameteri,
    gl_tex_parameteri(target: GLenum, pname: GLenum, param: GLint)
);
gl_func!(
    TexParameterf,
    gl_tex_parameterf(target: GLenum, pname: GLenum, param: GLfloat)
);
gl_func!(GenerateMipmap, gl_generate_mipmap(target: GLenum));

gl_func!(
    GenTextures,
    gl_gen_textures(n: GLsizei, textures: *mut GLuint)
);
gl_func!(
    DeleteTextures,
    gl_delete_textures(n: GLsizei, textures: *mut GLuint)
);
gl_func!(
    GetTexImage,
    gl_get_tex_image(
        target: GLenum,
        level: GLint,
        format: GLenum,
        type_: GLenum,
        pixels: *mut libc::c_void
    )
);
gl_func!(
    TexImage1D,
    gl_tex_image1d(
        target: GLenum,
        level: GLint,
        internal_format: GLint,
        width: GLsizei,
        border: GLint,
        format: GLenum,
        type_: GLenum,
        pixels: *const libc::c_void
    )
);
gl_func!(
    TexImage2D,
    gl_tex_image2d(
        target: GLenum,
        level: GLint,
        internal_format: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        format: GLenum,
        type_: GLenum,
        pixels: *const libc::c_void
    )
);
gl_func!(
    TexImage3D,
    gl_tex_image3d(
        target: GLenum,
        level: GLint,
        internal_format: GLint,
        width: GLsizei,
        height: GLsizei,
        depth: GLsizei,
        border: GLint,
        format: GLenum,
        type_: GLenum,
        pixels: *const libc::c_void
    )
);
gl_func!(
    TexSubImage1D,
    gl_tex_sub_image1d(
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        width: GLsizei,
        format: GLenum,
        type_: GLenum,
        pixels: *const libc::c_void
    )
);
gl_func!(
    TexSubImage2D,
    gl_tex_sub_image2d(
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_: GLenum,
        pixels: *const libc::c_void
    )
);
gl_func!(
    CopyTexImage2D,
    gl_copy_tex_image2d(
        target: GLenum,
        level: GLint,
        internal_format: GLenum,
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint
    )
);
gl_func!(
    ReadPixels,
    gl_read_pixels(
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_: GLenum,
        pixels: *mut libc::c_void
    )
);
gl_func!(TexCoord1f, gl_tex_coord1f(s: GLfloat));
gl_func!(TexCoord2f, gl_tex_coord2f(s: GLfloat, t: GLfloat));
gl_func!(
    TexCoord3f,
    gl_tex_coord3f(s: GLfloat, t: GLfloat, r: GLfloat)
);

// Viewport
gl_func!(
    Viewport,
    gl_viewport(x: GLint, y: GLint, width: GLsizei, height: GLsizei)
);
