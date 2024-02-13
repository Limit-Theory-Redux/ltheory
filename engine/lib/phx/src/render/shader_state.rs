use super::*;
use crate::math::*;

#[derive(Clone)]
#[repr(C)]
pub struct ShaderState {
    pub _refCount: u32,
    pub shader: *mut Shader,
    pub elems: Vec<Elem>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Elem {
    pub type_0: u32,
    pub index: i32,
    pub data: C2RustUnnamed,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub asFloat: f32,
    pub asFloat2: Vec2,
    pub asFloat3: Vec3,
    pub asFloat4: Vec4,
    pub asInt: i32,
    pub asMatrix: *mut Matrix,
    pub asTex1D: *mut Tex1D,
    pub asTex2D: *mut Tex2D,
    pub asTex3D: *mut Tex3D,
    pub asTexCube: *mut TexCube,
}

#[no_mangle]
pub static ElemType_Float: u32 = 1;

#[no_mangle]
pub static ElemType_Float2: u32 = 2;

#[no_mangle]
pub static ElemType_Float3: u32 = 3;

#[no_mangle]
pub static ElemType_Float4: u32 = 4;

#[no_mangle]
pub static ElemType_Int: u32 = 5;

#[no_mangle]
pub static ElemType_Matrix: u32 = 6;

#[no_mangle]
pub static ElemType_Tex1D: u32 = 7;

#[no_mangle]
pub static ElemType_Tex2D: u32 = 8;

#[no_mangle]
pub static ElemType_Tex3D: u32 = 9;

#[no_mangle]
pub static ElemType_TexCube: u32 = 10;

#[no_mangle]
pub extern "C" fn ShaderState_Create(shader: &mut Shader) -> Box<ShaderState> {
    Box::new(ShaderState {
        _refCount: 1,
        shader: {
            Shader_Acquire(shader);
            shader as *mut _
        },
        elems: Vec::new(),
    })
}

#[no_mangle]
pub extern "C" fn ShaderState_Acquire(this: &mut ShaderState) {
    this._refCount = (this._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderState_Free(this: *mut ShaderState) {
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount <= 0
    } {
        for e in (*this).elems.iter() {
            match (*e).type_0 {
                7 => {
                    Tex1D_Free(&mut *(*e).data.asTex1D);
                }
                8 => {
                    Tex2D_Free(&mut *(*e).data.asTex2D);
                }
                9 => {
                    Tex3D_Free(&mut *(*e).data.asTex3D);
                }
                10 => {
                    TexCube_Free(&mut *(*e).data.asTexCube);
                }
                _ => {}
            }
        }
        Shader_Free((*this).shader);
        drop(Box::from_raw(this));
    }
}

#[no_mangle]
pub unsafe extern "C" fn ShaderState_FromShaderLoad(
    vertName: *const libc::c_char,
    fragName: *const libc::c_char,
) -> Box<ShaderState> {
    let mut shader = Shader_Load(vertName, fragName);
    ShaderState_Create(shader.as_mut())
}

#[no_mangle]
pub unsafe extern "C" fn ShaderState_SetFloat(
    this: &mut ShaderState,
    name: *const libc::c_char,
    x: f32,
) {
    let mut elem: Elem = Elem {
        type_0: ElemType_Float,
        index: Shader_GetVariable(&mut *this.shader, name),
        data: C2RustUnnamed { asFloat: 0. },
    };
    elem.data.asFloat = x;
    this.elems.push(elem);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderState_SetFloat2(
    this: &mut ShaderState,
    name: *const libc::c_char,
    x: f32,
    y: f32,
) {
    let mut elem: Elem = Elem {
        type_0: ElemType_Float2,
        index: Shader_GetVariable(&mut *this.shader, name),
        data: C2RustUnnamed { asFloat: 0. },
    };
    elem.data.asFloat2 = Vec2::new(x, y);
    this.elems.push(elem);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderState_SetFloat3(
    this: &mut ShaderState,
    name: *const libc::c_char,
    x: f32,
    y: f32,
    z: f32,
) {
    let mut elem: Elem = Elem {
        type_0: ElemType_Float3,
        index: Shader_GetVariable(&mut *this.shader, name),
        data: C2RustUnnamed { asFloat: 0. },
    };
    elem.data.asFloat3 = Vec3::new(x, y, z);
    this.elems.push(elem);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderState_SetFloat4(
    this: &mut ShaderState,
    name: *const libc::c_char,
    x: f32,
    y: f32,
    z: f32,
    w: f32,
) {
    let mut elem: Elem = Elem {
        type_0: ElemType_Float4,
        index: Shader_GetVariable(&mut *this.shader, name),
        data: C2RustUnnamed { asFloat: 0. },
    };
    elem.data.asFloat4 = Vec4::new(x, y, z, w);
    this.elems.push(elem);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderState_SetInt(
    this: &mut ShaderState,
    name: *const libc::c_char,
    x: i32,
) {
    let mut elem: Elem = Elem {
        type_0: ElemType_Int,
        index: Shader_GetVariable(&mut *this.shader, name),
        data: C2RustUnnamed { asFloat: 0. },
    };
    elem.data.asInt = x;
    this.elems.push(elem);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderState_SetMatrix(
    this: &mut ShaderState,
    name: *const libc::c_char,
    x: *mut Matrix,
) {
    let mut elem: Elem = Elem {
        type_0: ElemType_Matrix,
        index: Shader_GetVariable(&mut *this.shader, name),
        data: C2RustUnnamed { asFloat: 0. },
    };
    elem.data.asMatrix = x;
    this.elems.push(elem);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderState_SetTex1D(
    this: &mut ShaderState,
    name: *const libc::c_char,
    x: &mut Tex1D,
) {
    Tex1D_Acquire(x);
    let mut elem: Elem = Elem {
        type_0: ElemType_Tex1D,
        index: Shader_GetVariable(&mut *this.shader, name),
        data: C2RustUnnamed { asFloat: 0. },
    };
    elem.data.asTex1D = x;
    this.elems.push(elem);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderState_SetTex2D(
    this: &mut ShaderState,
    name: *const libc::c_char,
    x: &mut Tex2D,
) {
    Tex2D_Acquire(x);
    let mut elem: Elem = Elem {
        type_0: ElemType_Tex2D,
        index: Shader_GetVariable(&mut *this.shader, name),
        data: C2RustUnnamed { asFloat: 0. },
    };
    elem.data.asTex2D = x;
    this.elems.push(elem);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderState_SetTex3D(
    this: &mut ShaderState,
    name: *const libc::c_char,
    x: &mut Tex3D,
) {
    Tex3D_Acquire(x);
    let mut elem: Elem = Elem {
        type_0: ElemType_Tex3D,
        index: Shader_GetVariable(&mut *this.shader, name),
        data: C2RustUnnamed { asFloat: 0. },
    };
    elem.data.asTex3D = x;
    this.elems.push(elem);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderState_SetTexCube(
    this: &mut ShaderState,
    name: *const libc::c_char,
    x: &mut TexCube,
) {
    TexCube_Acquire(x);
    let mut elem: Elem = Elem {
        type_0: ElemType_TexCube,
        index: Shader_GetVariable(&mut *this.shader, name),
        data: C2RustUnnamed { asFloat: 0. },
    };
    elem.data.asTexCube = x;
    this.elems.push(elem);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderState_Start(this: &mut ShaderState) {
    Shader_Start(&mut *this.shader);

    for e in this.elems.iter() {
        match (*e).type_0 {
            1 => {
                gl_uniform1f((*e).index, (*e).data.asFloat);
            }
            2 => {
                gl_uniform2f((*e).index, (*e).data.asFloat2.x, (*e).data.asFloat2.y);
            }
            3 => {
                gl_uniform3f(
                    (*e).index,
                    (*e).data.asFloat3.x,
                    (*e).data.asFloat3.y,
                    (*e).data.asFloat3.z,
                );
            }
            4 => {
                gl_uniform4f(
                    (*e).index,
                    (*e).data.asFloat4.x,
                    (*e).data.asFloat4.y,
                    (*e).data.asFloat4.z,
                    (*e).data.asFloat4.w,
                );
            }
            5 => {
                gl_uniform1i((*e).index, (*e).data.asInt);
            }
            6 => {
                Shader_ISetMatrix((*e).index, &mut *(*e).data.asMatrix);
            }
            7 => {
                Shader_ISetTex1D((*e).index, &mut *(*e).data.asTex1D);
            }
            8 => {
                Shader_ISetTex2D((*e).index, &mut *(*e).data.asTex2D);
            }
            9 => {
                Shader_ISetTex3D((*e).index, &mut *(*e).data.asTex3D);
            }
            10 => {
                Shader_ISetTexCube((*e).index, &mut *(*e).data.asTexCube);
            }
            _ => {
                panic!("ShaderState_Start: Encountered invalid opcode");
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ShaderState_Stop(this: &mut ShaderState) {
    Shader_Stop(this.shader);
}
