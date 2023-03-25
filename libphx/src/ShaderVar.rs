use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec2;
use crate::Math::Vec3;
use crate::Math::Vec4;
use crate::Matrix::*;
use crate::ShaderVarType::*;
use crate::StrMap::*;
use crate::Tex1D::*;
use crate::Tex2D::*;
use crate::Tex3D::*;
use crate::TexCube::*;
use libc;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct VarStack {
    pub type_0: ShaderVarType,
    pub size: i32,
    pub capacity: i32,
    pub elemSize: i32,
    pub data: *mut libc::c_void,
}

static mut varMap: *mut StrMap = std::ptr::null_mut();

#[inline]
unsafe extern "C" fn ShaderVar_GetStack(
    var: *const libc::c_char,
    type_0: ShaderVarType,
) -> *mut VarStack {
    let mut this: *mut VarStack = StrMap_Get(varMap, var) as *mut VarStack;
    if this.is_null() {
        if type_0 == 0 {
            return std::ptr::null_mut();
        }
        this = MemNew!(VarStack);
        (*this).type_0 = type_0;
        (*this).size = 0;
        (*this).capacity = 4;
        (*this).elemSize = ShaderVarType_GetSize(type_0);
        (*this).data = MemAlloc(((*this).capacity * (*this).elemSize) as usize);
        StrMap_Set(varMap, var, this as *mut _);
    }
    if type_0 != 0 && (*this).type_0 != type_0 {
        Fatal(
            b"ShaderVar_GetStack: Attempting to get stack of type <%s> for shader variable <%s> when existing stack has type <%s>\0"
                as *const u8 as *const libc::c_char,
            ShaderVarType_GetName(type_0),
            var,
            ShaderVarType_GetName((*this).type_0),
        );
    }
    this
}

#[inline]
unsafe extern "C" fn ShaderVar_Push(
    var: *const libc::c_char,
    type_0: ShaderVarType,
    value: *const libc::c_void,
) {
    let this: *mut VarStack = ShaderVar_GetStack(var, type_0);
    if (*this).size == (*this).capacity {
        (*this).capacity *= 2;
        (*this).data = MemRealloc((*this).data, ((*this).capacity * (*this).elemSize) as usize);
    }
    MemCpy(
        ((*this).data as *mut libc::c_char).offset(((*this).size * (*this).elemSize) as isize)
            as *mut _,
        value,
        (*this).elemSize as usize,
    );
    (*this).size += 1;
}

#[no_mangle]
pub unsafe extern "C" fn ShaderVar_Init() {
    varMap = StrMap_Create(16);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderVar_Free() {
    StrMap_Free(varMap);
    varMap = std::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn ShaderVar_Get(
    name: *const libc::c_char,
    type_0: ShaderVarType,
) -> *mut libc::c_void {
    let this: *mut VarStack = ShaderVar_GetStack(name, 0);
    if this.is_null() || (*this).size == 0 {
        return std::ptr::null_mut();
    }
    if type_0 != 0 && (*this).type_0 != type_0 {
        Fatal(
            b"ShaderVar_Get: Attempting to get variable <%s> with type <%s> when existing stack has type <%s>\0"
                as *const u8 as *const libc::c_char,
            name,
            ShaderVarType_GetName(type_0),
            ShaderVarType_GetName((*this).type_0),
        );
    }
    ((*this).data as *mut libc::c_char).offset(((*this).elemSize * ((*this).size - 1)) as isize)
        as *mut _
}

#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushFloat(name: *const libc::c_char, x: f32) {
    ShaderVar_Push(name, 0x1, &x as *const f32 as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushFloat2(name: *const libc::c_char, x: f32, y: f32) {
    let mut value = Vec2::new(x, y);
    ShaderVar_Push(name, 0x2, &mut value as *mut Vec2 as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushFloat3(name: *const libc::c_char, x: f32, y: f32, z: f32) {
    let mut value: Vec3 = Vec3 { x: x, y: y, z: z };
    ShaderVar_Push(name, 0x3, &mut value as *mut Vec3 as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushFloat4(
    name: *const libc::c_char,
    x: f32,
    y: f32,
    z: f32,
    w: f32,
) {
    let mut value: Vec4 = Vec4::new(x, y, z, w);
    ShaderVar_Push(name, 0x4, &mut value as *mut Vec4 as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushInt(name: *const libc::c_char, x: i32) {
    let mut value: i32 = x;
    ShaderVar_Push(name, 0x5, &mut value as *mut i32 as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushMatrix(name: *const libc::c_char, x: *mut Matrix) {
    ShaderVar_Push(name, 0x9, &x as *const *mut Matrix as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushTex1D(name: *const libc::c_char, x: *mut Tex1D) {
    ShaderVar_Push(name, 0xa, &x as *const *mut Tex1D as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushTex2D(name: *const libc::c_char, x: *mut Tex2D) {
    ShaderVar_Push(name, 0xb, &x as *const *mut Tex2D as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushTex3D(name: *const libc::c_char, x: *mut Tex3D) {
    ShaderVar_Push(name, 0xc, &x as *const *mut Tex3D as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushTexCube(name: *const libc::c_char, x: *mut TexCube) {
    ShaderVar_Push(name, 0xd, &x as *const *mut TexCube as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn ShaderVar_Pop(name: *const libc::c_char) {
    let this: *mut VarStack = ShaderVar_GetStack(name, 0);
    if this.is_null() {
        Fatal(
            b"ShaderVar_Pop: Attempting to pop nonexistent stack <%s>\0" as *const u8
                as *const libc::c_char,
            name,
        );
    }
    if (*this).size == 0 {
        Fatal(
            b"ShaderVar_Pop: Attempting to pop empty stack <%s>\0" as *const u8
                as *const libc::c_char,
            name,
        );
    }
    (*this).size -= 1;
}
