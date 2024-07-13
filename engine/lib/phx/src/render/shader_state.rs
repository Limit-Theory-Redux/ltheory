use super::*;
use crate::math::*;

pub struct ShaderState {
    shader: *mut Shader,
    elems: Vec<(i32, ShaderVarData)>,
}

#[no_mangle]
pub extern "C" fn ShaderState_Create(shader: &mut Shader) -> Box<ShaderState> {
    Box::new(ShaderState {
        shader: {
            Shader_Acquire(shader);
            shader as *mut _
        },
        elems: Vec::new(),
    })
}

#[no_mangle]
pub extern "C" fn ShaderState_Free(this: Box<ShaderState>) {
    for (_, data) in this.elems.iter() {
        match data {
            ShaderVarData::Tex1D(t) => unsafe {
                Tex1D_Free(*t);
            },
            ShaderVarData::Tex2D(t) => unsafe {
                Tex2D_Free(*t);
            },
            ShaderVarData::Tex3D(t) => unsafe {
                Tex3D_Free(*t);
            },
            ShaderVarData::TexCube(t) => unsafe {
                TexCube_Free(*t);
            },
            _ => {}
        }
    }
    unsafe {
        Shader_Free(this.shader);
    }
}

#[no_mangle]
pub extern "C" fn ShaderState_FromShaderLoad(
    vertName: *const libc::c_char,
    fragName: *const libc::c_char,
) -> Box<ShaderState> {
    let mut shader = Shader_Load(vertName, fragName);
    ShaderState_Create(shader.as_mut())
}

#[no_mangle]
pub extern "C" fn ShaderState_SetFloat(this: &mut ShaderState, name: *const libc::c_char, x: f32) {
    this.elems.push((
        unsafe { Shader_GetVariable(&mut *this.shader, name) },
        ShaderVarData::Float(x),
    ));
}

#[no_mangle]
pub extern "C" fn ShaderState_SetFloat2(
    this: &mut ShaderState,
    name: *const libc::c_char,
    x: f32,
    y: f32,
) {
    this.elems.push((
        unsafe { Shader_GetVariable(&mut *this.shader, name) },
        ShaderVarData::Float2(vec2(x, y)),
    ));
}

#[no_mangle]
pub extern "C" fn ShaderState_SetFloat3(
    this: &mut ShaderState,
    name: *const libc::c_char,
    x: f32,
    y: f32,
    z: f32,
) {
    this.elems.push((
        unsafe { Shader_GetVariable(&mut *this.shader, name) },
        ShaderVarData::Float3(vec3(x, y, z)),
    ));
}

#[no_mangle]
pub extern "C" fn ShaderState_SetFloat4(
    this: &mut ShaderState,
    name: *const libc::c_char,
    x: f32,
    y: f32,
    z: f32,
    w: f32,
) {
    this.elems.push((
        unsafe { Shader_GetVariable(&mut *this.shader, name) },
        ShaderVarData::Float4(vec4(x, y, z, w)),
    ));
}

#[no_mangle]
pub extern "C" fn ShaderState_SetInt(this: &mut ShaderState, name: *const libc::c_char, x: i32) {
    this.elems.push((
        unsafe { Shader_GetVariable(&mut *this.shader, name) },
        ShaderVarData::Int(x),
    ));
}

#[no_mangle]
pub extern "C" fn ShaderState_SetMatrix(
    this: &mut ShaderState,
    name: *const libc::c_char,
    x: &Matrix,
) {
    this.elems.push((
        unsafe { Shader_GetVariable(&mut *this.shader, name) },
        ShaderVarData::Matrix(*x),
    ));
}

#[no_mangle]
pub extern "C" fn ShaderState_SetTex1D(
    this: &mut ShaderState,
    name: *const libc::c_char,
    x: &mut Tex1D,
) {
    Tex1D_Acquire(x);
    this.elems.push((
        unsafe { Shader_GetVariable(&mut *this.shader, name) },
        ShaderVarData::Tex1D(x),
    ));
}

#[no_mangle]
pub extern "C" fn ShaderState_SetTex2D(
    this: &mut ShaderState,
    name: *const libc::c_char,
    x: &mut Tex2D,
) {
    Tex2D_Acquire(x);
    this.elems.push((
        unsafe { Shader_GetVariable(&mut *this.shader, name) },
        ShaderVarData::Tex2D(x),
    ));
}

#[no_mangle]
pub extern "C" fn ShaderState_SetTex3D(
    this: &mut ShaderState,
    name: *const libc::c_char,
    x: &mut Tex3D,
) {
    Tex3D_Acquire(x);
    this.elems.push((
        unsafe { Shader_GetVariable(&mut *this.shader, name) },
        ShaderVarData::Tex3D(x),
    ));
}

#[no_mangle]
pub extern "C" fn ShaderState_SetTexCube(
    this: &mut ShaderState,
    name: *const libc::c_char,
    x: &mut TexCube,
) {
    TexCube_Acquire(x);
    this.elems.push((
        unsafe { Shader_GetVariable(&mut *this.shader, name) },
        ShaderVarData::TexCube(x),
    ));
}

#[no_mangle]
pub extern "C" fn ShaderState_Start(this: &mut ShaderState) {
    unsafe {
        Shader_Start(&mut *this.shader);
    }

    for (index, data) in this.elems.iter() {
        match data {
            ShaderVarData::Float(v) => glcheck!(gl::Uniform1f(*index, *v)),
            ShaderVarData::Float2(v) => glcheck!(gl::Uniform2f(*index, v.x, v.y)),
            ShaderVarData::Float3(v) => glcheck!(gl::Uniform3f(*index, v.x, v.y, v.z)),
            ShaderVarData::Float4(v) => glcheck!(gl::Uniform4f(*index, v.x, v.y, v.z, v.w)),
            ShaderVarData::Int(v) => glcheck!(gl::Uniform1i(*index, *v)),
            ShaderVarData::Int2(v) => glcheck!(gl::Uniform2i(*index, v.x, v.y)),
            ShaderVarData::Int3(v) => glcheck!(gl::Uniform3i(*index, v.x, v.y, v.z)),
            ShaderVarData::Int4(v) => glcheck!(gl::Uniform4i(*index, v.x, v.y, v.z, v.w)),
            ShaderVarData::Matrix(m) => Shader_ISetMatrix(*index, &m),
            ShaderVarData::Tex1D(t) => unsafe { Shader_ISetTex1D(*index, &mut **t) },
            ShaderVarData::Tex2D(t) => unsafe { Shader_ISetTex2D(*index, &mut **t) },
            ShaderVarData::Tex3D(t) => unsafe { Shader_ISetTex3D(*index, &mut **t) },
            ShaderVarData::TexCube(t) => unsafe { Shader_ISetTexCube(*index, &mut **t) },
        }
    }
}

#[no_mangle]
pub extern "C" fn ShaderState_Stop(this: &mut ShaderState) {
    Shader_Stop(unsafe { &*this.shader });
}
