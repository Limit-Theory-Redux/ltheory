use ::libc;
use super::internal::Memory::*;
extern "C" {
    pub type StrMap;
    pub type Tex1D;
    pub type Tex2D;
    pub type Tex3D;
    pub type TexCube;
    pub type Matrix;
    fn memcpy(
        _: *mut libc::c_void,
        _: *const libc::c_void,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn Fatal(_: cstr, _: ...);
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn realloc(_: *mut libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
    fn ShaderVarType_GetName(_: ShaderVarType) -> cstr;
    fn ShaderVarType_GetSize(_: ShaderVarType) -> libc::c_int;
    fn StrMap_Create(initCapacity: uint32) -> *mut StrMap;
    fn StrMap_Free(_: *mut StrMap);
    fn StrMap_Get(_: *mut StrMap, key: cstr) -> *mut libc::c_void;
    fn StrMap_Set(_: *mut StrMap, key: cstr, val: *mut libc::c_void);
}
pub type int32_t = libc::c_int;
pub type uint32_t = libc::c_uint;
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint32 = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec2f {
    pub x: libc::c_float,
    pub y: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec3f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec4f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
    pub w: libc::c_float,
}
pub type ShaderVarType = int32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VarStack {
    pub type_0: ShaderVarType,
    pub size: int32,
    pub capacity: int32,
    pub elemSize: int32,
    pub data: *mut libc::c_void,
}


static mut varMap: *mut StrMap = 0 as *const StrMap as *mut StrMap;
#[inline]
unsafe extern "C" fn ShaderVar_GetStack(
    mut var: cstr,
    mut type_0: ShaderVarType,
) -> *mut VarStack {
    let mut self_0: *mut VarStack = StrMap_Get(varMap, var) as *mut VarStack;
    if self_0.is_null() {
        if type_0 == 0 {
            return 0 as *mut VarStack;
        }
        self_0 = MemAlloc(::core::mem::size_of::<VarStack>())
            as *mut VarStack;
        (*self_0).type_0 = type_0;
        (*self_0).size = 0 as libc::c_int;
        (*self_0).capacity = 4 as libc::c_int;
        (*self_0).elemSize = ShaderVarType_GetSize(type_0);
        (*self_0).data = MemAlloc(((*self_0).capacity * (*self_0).elemSize) as size_t);
        StrMap_Set(varMap, var, self_0 as *mut libc::c_void);
    }
    if type_0 != 0 && (*self_0).type_0 != type_0 {
        Fatal(
            b"ShaderVar_GetStack: Attempting to get stack of type <%s> for shader variable <%s> when existing stack has type <%s>\0"
                as *const u8 as *const libc::c_char,
            ShaderVarType_GetName(type_0),
            var,
            ShaderVarType_GetName((*self_0).type_0),
        );
    }
    return self_0;
}
#[inline]
unsafe extern "C" fn ShaderVar_Push(
    mut var: cstr,
    mut type_0: ShaderVarType,
    mut value: *const libc::c_void,
) {
    let mut self_0: *mut VarStack = ShaderVar_GetStack(var, type_0);
    if (*self_0).size == (*self_0).capacity {
        (*self_0).capacity *= 2 as libc::c_int;
        (*self_0)
            .data = MemRealloc(
            (*self_0).data,
            ((*self_0).capacity * (*self_0).elemSize) as size_t,
        );
    }
    MemCpy(
        ((*self_0).data as *mut libc::c_char)
            .offset(((*self_0).size * (*self_0).elemSize) as isize) as *mut libc::c_void,
        value,
        (*self_0).elemSize as size_t,
    );
    (*self_0).size += 1;
}
#[no_mangle]
pub unsafe extern "C" fn ShaderVar_Init() {
    varMap = StrMap_Create(16 as libc::c_int as uint32);
}
#[no_mangle]
pub unsafe extern "C" fn ShaderVar_Free() {
    StrMap_Free(varMap);
    varMap = 0 as *mut StrMap;
}
#[no_mangle]
pub unsafe extern "C" fn ShaderVar_Get(
    mut name: cstr,
    mut type_0: ShaderVarType,
) -> *mut libc::c_void {
    let mut self_0: *mut VarStack = ShaderVar_GetStack(name, 0 as libc::c_int);
    if self_0.is_null() || (*self_0).size == 0 as libc::c_int {
        return 0 as *mut libc::c_void;
    }
    if type_0 != 0 && (*self_0).type_0 != type_0 {
        Fatal(
            b"ShaderVar_Get: Attempting to get variable <%s> with type <%s> when existing stack has type <%s>\0"
                as *const u8 as *const libc::c_char,
            name,
            ShaderVarType_GetName(type_0),
            ShaderVarType_GetName((*self_0).type_0),
        );
    }
    return ((*self_0).data as *mut libc::c_char)
        .offset(((*self_0).elemSize * ((*self_0).size - 1 as libc::c_int)) as isize)
        as *mut libc::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushFloat(mut name: cstr, mut x: libc::c_float) {
    ShaderVar_Push(
        name,
        0x1 as libc::c_int,
        &mut x as *mut libc::c_float as *const libc::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushFloat2(
    mut name: cstr,
    mut x: libc::c_float,
    mut y: libc::c_float,
) {
    let mut value: Vec2f = {
        let mut init = Vec2f { x: x, y: y };
        init
    };
    ShaderVar_Push(
        name,
        0x2 as libc::c_int,
        &mut value as *mut Vec2f as *const libc::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushFloat3(
    mut name: cstr,
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut z: libc::c_float,
) {
    let mut value: Vec3f = {
        let mut init = Vec3f { x: x, y: y, z: z };
        init
    };
    ShaderVar_Push(
        name,
        0x3 as libc::c_int,
        &mut value as *mut Vec3f as *const libc::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushFloat4(
    mut name: cstr,
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut z: libc::c_float,
    mut w: libc::c_float,
) {
    let mut value: Vec4f = {
        let mut init = Vec4f { x: x, y: y, z: z, w: w };
        init
    };
    ShaderVar_Push(
        name,
        0x4 as libc::c_int,
        &mut value as *mut Vec4f as *const libc::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushInt(mut name: cstr, mut x: libc::c_int) {
    let mut value: int32 = x;
    ShaderVar_Push(
        name,
        0x5 as libc::c_int,
        &mut value as *mut int32 as *const libc::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushMatrix(mut name: cstr, mut x: *mut Matrix) {
    ShaderVar_Push(
        name,
        0x9 as libc::c_int,
        &mut x as *mut *mut Matrix as *const libc::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushTex1D(mut name: cstr, mut x: *mut Tex1D) {
    ShaderVar_Push(
        name,
        0xa as libc::c_int,
        &mut x as *mut *mut Tex1D as *const libc::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushTex2D(mut name: cstr, mut x: *mut Tex2D) {
    ShaderVar_Push(
        name,
        0xb as libc::c_int,
        &mut x as *mut *mut Tex2D as *const libc::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushTex3D(mut name: cstr, mut x: *mut Tex3D) {
    ShaderVar_Push(
        name,
        0xc as libc::c_int,
        &mut x as *mut *mut Tex3D as *const libc::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ShaderVar_PushTexCube(mut name: cstr, mut x: *mut TexCube) {
    ShaderVar_Push(
        name,
        0xd as libc::c_int,
        &mut x as *mut *mut TexCube as *const libc::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ShaderVar_Pop(mut name: cstr) {
    let mut self_0: *mut VarStack = ShaderVar_GetStack(name, 0 as libc::c_int);
    if self_0.is_null() {
        Fatal(
            b"ShaderVar_Pop: Attempting to pop nonexistent stack <%s>\0" as *const u8
                as *const libc::c_char,
            name,
        );
    }
    if (*self_0).size == 0 as libc::c_int {
        Fatal(
            b"ShaderVar_Pop: Attempting to pop empty stack <%s>\0" as *const u8
                as *const libc::c_char,
            name,
        );
    }
    (*self_0).size -= 1;
}
