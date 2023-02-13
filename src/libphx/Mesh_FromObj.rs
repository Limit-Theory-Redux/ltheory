use ::libc;
use super::internal::Memory::*;
extern "C" {
    pub type Mesh;
    fn Fatal(_: cstr, _: ...);
    fn strcmp(_: *const libc::c_char, _: *const libc::c_char) -> libc::c_int;
    fn memcpy(
        _: *mut libc::c_void,
        _: *const libc::c_void,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn realloc(_: *mut libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
    fn strtof(_: *const libc::c_char, _: *mut *mut libc::c_char) -> libc::c_float;
    fn strtol(
        _: *const libc::c_char,
        _: *mut *mut libc::c_char,
        _: libc::c_int,
    ) -> libc::c_long;
    fn Mesh_Create() -> *mut Mesh;
    fn Mesh_AddQuad(
        _: *mut Mesh,
        _: libc::c_int,
        _: libc::c_int,
        _: libc::c_int,
        _: libc::c_int,
    );
    fn Mesh_AddTri(_: *mut Mesh, _: libc::c_int, _: libc::c_int, _: libc::c_int);
    fn Mesh_AddVertexRaw(_: *mut Mesh, _: *const Vertex);
    fn Mesh_GetVertexCount(_: *mut Mesh) -> libc::c_int;
    fn Mesh_GetVertexData(_: *mut Mesh) -> *mut Vertex;
    fn Mesh_ReserveIndexData(_: *mut Mesh, capacity: libc::c_int);
    fn Mesh_ReserveVertexData(_: *mut Mesh, capacity: libc::c_int);
    fn __error() -> *mut libc::c_int;
}
pub type int32_t = libc::c_int;
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
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
pub struct Vertex {
    pub p: Vec3f,
    pub n: Vec3f,
    pub uv: Vec2f,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ParseState {
    pub cursor: *const libc::c_char,
    pub endOfData: *const libc::c_char,
    pub lineStart: *const libc::c_char,
    pub lineNumber: int32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VertexIndices {
    pub iP: int32,
    pub iN: int32,
    pub iUV: int32,
}



#[inline]
unsafe extern "C" fn StrEqual(mut a: cstr, mut b: cstr) -> bool {
    return strcmp(a, b) == 0 as libc::c_int;
}
#[inline]
unsafe extern "C" fn StrLen(mut s: cstr) -> size_t {
    if s.is_null() {
        return 0 as libc::c_int as size_t;
    }
    let mut begin: cstr = s;
    while *s != 0 {
        s = s.offset(1);
    }
    return s.offset_from(begin) as libc::c_long as size_t;
}
#[inline]
unsafe extern "C" fn Vec3f_Equal(mut a: Vec3f, mut b: Vec3f) -> bool {
    return a.x == b.x && a.y == b.y && a.z == b.z;
}
unsafe extern "C" fn Obj_Fatal(mut message: cstr, mut s: *mut ParseState) {
    let mut len: int32 = 0 as libc::c_int;
    let mut ch: *const libc::c_char = (*s).lineStart;
    while ch < (*s).endOfData && *ch as libc::c_int != '\r' as i32
        && *ch as libc::c_int != '\n' as i32
    {
        ch = ch.offset(1);
        len += 1 as libc::c_int;
    }
    let mut line: *mut libc::c_char = MemAlloc((len + 1 as libc::c_int) as size_t)
        as *mut libc::c_char;
    MemCpy(
        line as *mut libc::c_void,
        (*s).lineStart as *const libc::c_void,
        len as usize,
    );
    *line.offset(len as isize) = 0 as libc::c_int as libc::c_char;
    Fatal(
        b"%s Line %i\n%s\0" as *const u8 as *const libc::c_char,
        message,
        (*s).lineNumber,
        line,
    );
}
unsafe extern "C" fn ConsumeRestOfLine(mut s: *mut ParseState) -> bool {
    let mut oldPosition: *const libc::c_char = (*s).cursor;
    while (*s).cursor < (*s).endOfData && *(*s).cursor as libc::c_int != '\r' as i32
        && *(*s).cursor as libc::c_int != '\n' as i32
    {
        (*s).cursor = ((*s).cursor).offset(1);
    }
    let mut cr: int32 = 0 as libc::c_int;
    let mut nl: int32 = 0 as libc::c_int;
    while (*s).cursor < (*s).endOfData
        && (*(*s).cursor as libc::c_int == '\r' as i32
            || *(*s).cursor as libc::c_int == '\n' as i32)
    {
        if *(*s).cursor as libc::c_int == '\r' as i32 {
            if cr == 1 as libc::c_int {
                nl = 0 as libc::c_int;
                cr = nl;
                (*s).lineNumber += 1;
            }
            cr += 1;
        }
        if *(*s).cursor as libc::c_int == '\n' as i32 {
            if nl == 1 as libc::c_int {
                nl = 0 as libc::c_int;
                cr = nl;
                (*s).lineNumber += 1;
            }
            nl += 1;
        }
        (*s).cursor = ((*s).cursor).offset(1);
    }
    return (*s).cursor != oldPosition;
}
unsafe extern "C" fn ConsumeWhitespace(mut s: *mut ParseState) -> bool {
    let mut oldPosition: *const libc::c_char = (*s).cursor;
    while (*s).cursor < (*s).endOfData
        && (*(*s).cursor as libc::c_int == ' ' as i32
            || *(*s).cursor as libc::c_int == '\t' as i32)
    {
        (*s).cursor = ((*s).cursor).offset(1);
    }
    return (*s).cursor != oldPosition;
}
unsafe extern "C" fn ConsumeToken(
    mut token: *mut libc::c_char,
    mut tokenLen: int32,
    mut s: *mut ParseState,
) -> bool {
    let mut i: int32 = 0 as libc::c_int;
    while (*s).cursor < (*s).endOfData && i < tokenLen - 1 as libc::c_int
        && *(*s).cursor as libc::c_int != ' ' as i32
        && *(*s).cursor as libc::c_int != '\t' as i32
        && *(*s).cursor as libc::c_int != '\r' as i32
        && *(*s).cursor as libc::c_int != '\n' as i32
    {
        let fresh0 = i;
        i = i + 1;
        *token.offset(fresh0 as isize) = *(*s).cursor;
        (*s).cursor = ((*s).cursor).offset(1);
    }
    *token.offset(i as isize) = 0 as libc::c_int as libc::c_char;
    return i != 0 as libc::c_int;
}
unsafe extern "C" fn ConsumeFloat(
    mut value: *mut libc::c_float,
    mut s: *mut ParseState,
) -> bool {
    let mut afterFloat: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut f: libc::c_float = strtof((*s).cursor, &mut afterFloat);
    if *__error() == 34 as libc::c_int {
        Obj_Fatal(
            b"Parsed float in .obj data is out of range.\0" as *const u8
                as *const libc::c_char,
            s,
        );
    }
    if afterFloat != (*s).cursor as *mut libc::c_char {
        (*s).cursor = afterFloat;
        *value = f;
        return 1 as libc::c_int != 0;
    }
    return 0 as libc::c_int != 0;
}
unsafe extern "C" fn ConsumeInt(mut value: *mut int32, mut s: *mut ParseState) -> bool {
    let mut afterInt: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut i: int32 = strtol((*s).cursor, &mut afterInt, 10 as libc::c_int) as int32;
    if *__error() == 34 as libc::c_int {
        Obj_Fatal(
            b"Parsed int in .obj data is out of range.\0" as *const u8
                as *const libc::c_char,
            s,
        );
    }
    if afterInt != (*s).cursor as *mut libc::c_char {
        (*s).cursor = afterInt;
        *value = i;
        return 1 as libc::c_int != 0;
    }
    return 0 as libc::c_int != 0;
}
unsafe extern "C" fn ConsumeCharacter(
    mut character: libc::c_char,
    mut s: *mut ParseState,
) -> bool {
    if (*s).cursor < (*s).endOfData
        && *(*s).cursor as libc::c_int == character as libc::c_int
    {
        (*s).cursor = ((*s).cursor).offset(1);
        return 1 as libc::c_int != 0;
    }
    return 0 as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_FromObj(mut bytes: cstr) -> *mut Mesh {
    let mut bytesSize: libc::c_int = StrLen(bytes) as libc::c_int;
    let mut s: ParseState = {
        let mut init = ParseState {
            cursor: 0 as *const libc::c_char,
            endOfData: 0 as *const libc::c_char,
            lineStart: 0 as *const libc::c_char,
            lineNumber: 0,
        };
        init
    };
    s.cursor = bytes;
    s.endOfData = (s.cursor).offset(bytesSize as isize);
    let mut mesh: *mut Mesh = Mesh_Create();
    let mut vertexCount: int32 = 0 as libc::c_int;
    let mut indexCount: int32 = 0 as libc::c_int;
    let mut faceCount: int32 = 0 as libc::c_int;
    let mut positions_size: int32 = 0;
    let mut positions_capacity: int32 = 0;
    let mut positions_data: *mut Vec3f = 0 as *mut Vec3f;
    positions_capacity = 0 as libc::c_int;
    positions_size = 0 as libc::c_int;
    positions_data = 0 as *mut Vec3f;
    let mut uvs_size: int32 = 0;
    let mut uvs_capacity: int32 = 0;
    let mut uvs_data: *mut Vec2f = 0 as *mut Vec2f;
    uvs_capacity = 0 as libc::c_int;
    uvs_size = 0 as libc::c_int;
    uvs_data = 0 as *mut Vec2f;
    let mut normals_size: int32 = 0;
    let mut normals_capacity: int32 = 0;
    let mut normals_data: *mut Vec3f = 0 as *mut Vec3f;
    normals_capacity = 0 as libc::c_int;
    normals_size = 0 as libc::c_int;
    normals_data = 0 as *mut Vec3f;
    if (positions_capacity < (0.008f32 * bytesSize as libc::c_float) as int32)
        as libc::c_int as libc::c_long != 0
    {
        positions_capacity = (0.008f32 * bytesSize as libc::c_float) as int32;
        let mut elemSize: usize = ::core::mem::size_of::<Vec3f>();
        let mut pData: *mut *mut libc::c_void = &mut positions_data as *mut *mut Vec3f
            as *mut *mut libc::c_void;
        *pData = MemRealloc(
            positions_data as *mut libc::c_void,
            (positions_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
    if (uvs_capacity < (0.008f32 * bytesSize as libc::c_float) as int32) as libc::c_int
        as libc::c_long != 0
    {
        uvs_capacity = (0.008f32 * bytesSize as libc::c_float) as int32;
        let mut elemSize_0: usize = ::core::mem::size_of::<Vec2f>();
        let mut pData_0: *mut *mut libc::c_void = &mut uvs_data as *mut *mut Vec2f
            as *mut *mut libc::c_void;
        *pData_0 = MemRealloc(
            uvs_data as *mut libc::c_void,
            (uvs_capacity as usize).wrapping_mul(elemSize_0 as usize),
        );
    }
    if (normals_capacity < (0.008f32 * bytesSize as libc::c_float) as int32)
        as libc::c_int as libc::c_long != 0
    {
        normals_capacity = (0.008f32 * bytesSize as libc::c_float) as int32;
        let mut elemSize_1: usize = ::core::mem::size_of::<Vec3f>();
        let mut pData_1: *mut *mut libc::c_void = &mut normals_data as *mut *mut Vec3f
            as *mut *mut libc::c_void;
        *pData_1 = MemRealloc(
            normals_data as *mut libc::c_void,
            (normals_capacity as usize).wrapping_mul(elemSize_1 as usize),
        );
    }
    Mesh_ReserveIndexData(mesh, (0.050f32 * bytesSize as libc::c_float) as int32);
    Mesh_ReserveVertexData(mesh, (0.050f32 * bytesSize as libc::c_float) as int32);
    loop {
        s.lineStart = s.cursor;
        s.lineNumber += 1;
        let mut token: [libc::c_char; 16] = [0; 16];
        ConsumeWhitespace(&mut s);
        ConsumeToken(token.as_mut_ptr(), 16 as libc::c_int, &mut s);
        ConsumeWhitespace(&mut s);
        if StrEqual(
            token.as_mut_ptr() as cstr,
            b"\0" as *const u8 as *const libc::c_char,
        ) {
            if s.cursor >= s.endOfData {
                break;
            }
        } else if StrEqual(
            token.as_mut_ptr() as cstr,
            b"v\0" as *const u8 as *const libc::c_char,
        ) {
            if positions_size == 2147483647 as libc::c_int {
                Obj_Fatal(
                    b".obj data contains more vertex positions than will fit in an ArrayList.\0"
                        as *const u8 as *const libc::c_char,
                    &mut s,
                );
            }
            let mut p: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
            if !(ConsumeFloat(&mut p.x, &mut s) as libc::c_int != 0
                && ConsumeFloat(&mut p.y, &mut s) as libc::c_int != 0
                && ConsumeFloat(&mut p.z, &mut s) as libc::c_int != 0)
            {
                Obj_Fatal(
                    b"Failed to parse geometric vertex from .obj data.\0" as *const u8
                        as *const libc::c_char,
                    &mut s,
                );
            }
            if (positions_capacity == positions_size) as libc::c_int as libc::c_long != 0
            {
                positions_capacity = if positions_capacity != 0 {
                    positions_capacity * 2 as libc::c_int
                } else {
                    1 as libc::c_int
                };
                let mut elemSize_2: usize = ::core::mem::size_of::<Vec3f>();
                let mut pData_2: *mut *mut libc::c_void = &mut positions_data
                    as *mut *mut Vec3f as *mut *mut libc::c_void;
                *pData_2 = MemRealloc(
                    positions_data as *mut libc::c_void,
                    (positions_capacity as usize).wrapping_mul(elemSize_2 as usize),
                );
            }
            let fresh1 = positions_size;
            positions_size = positions_size + 1;
            *positions_data.offset(fresh1 as isize) = p;
        } else if StrEqual(
            token.as_mut_ptr() as cstr,
            b"vt\0" as *const u8 as *const libc::c_char,
        ) {
            if uvs_size == 2147483647 as libc::c_int {
                Obj_Fatal(
                    b".obj data contains more UVs than will fit in an ArrayList.\0"
                        as *const u8 as *const libc::c_char,
                    &mut s,
                );
            }
            let mut uv: Vec2f = Vec2f { x: 0., y: 0. };
            if !(ConsumeFloat(&mut uv.x, &mut s) as libc::c_int != 0
                && ConsumeFloat(&mut uv.y, &mut s) as libc::c_int != 0)
            {
                Obj_Fatal(
                    b"Failed to parse texture vertex from .obj data.\0" as *const u8
                        as *const libc::c_char,
                    &mut s,
                );
            }
            if (uvs_capacity == uvs_size) as libc::c_int as libc::c_long != 0 {
                uvs_capacity = if uvs_capacity != 0 {
                    uvs_capacity * 2 as libc::c_int
                } else {
                    1 as libc::c_int
                };
                let mut elemSize_3: usize = ::core::mem::size_of::<Vec2f>();
                let mut pData_3: *mut *mut libc::c_void = &mut uvs_data
                    as *mut *mut Vec2f as *mut *mut libc::c_void;
                *pData_3 = MemRealloc(
                    uvs_data as *mut libc::c_void,
                    (uvs_capacity as usize).wrapping_mul(elemSize_3 as usize),
                );
            }
            let fresh2 = uvs_size;
            uvs_size = uvs_size + 1;
            *uvs_data.offset(fresh2 as isize) = uv;
        } else if StrEqual(
            token.as_mut_ptr() as cstr,
            b"vn\0" as *const u8 as *const libc::c_char,
        ) {
            if normals_size == 2147483647 as libc::c_int {
                Obj_Fatal(
                    b".obj data contains more normals than will fit in an ArrayList.\0"
                        as *const u8 as *const libc::c_char,
                    &mut s,
                );
            }
            let mut n: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
            if !(ConsumeFloat(&mut n.x, &mut s) as libc::c_int != 0
                && ConsumeFloat(&mut n.y, &mut s) as libc::c_int != 0
                && ConsumeFloat(&mut n.z, &mut s) as libc::c_int != 0)
            {
                Obj_Fatal(
                    b"Failed to parse vertex normal from .obj data.\0" as *const u8
                        as *const libc::c_char,
                    &mut s,
                );
            }
            if (normals_capacity == normals_size) as libc::c_int as libc::c_long != 0 {
                normals_capacity = if normals_capacity != 0 {
                    normals_capacity * 2 as libc::c_int
                } else {
                    1 as libc::c_int
                };
                let mut elemSize_4: usize = ::core::mem::size_of::<Vec3f>();
                let mut pData_4: *mut *mut libc::c_void = &mut normals_data
                    as *mut *mut Vec3f as *mut *mut libc::c_void;
                *pData_4 = MemRealloc(
                    normals_data as *mut libc::c_void,
                    (normals_capacity as usize).wrapping_mul(elemSize_4 as usize),
                );
            }
            let fresh3 = normals_size;
            normals_size = normals_size + 1;
            *normals_data.offset(fresh3 as isize) = n;
        } else if StrEqual(
            token.as_mut_ptr() as cstr,
            b"f\0" as *const u8 as *const libc::c_char,
        ) {
            let mut vertexIndicesCount: int32 = 0 as libc::c_int;
            let mut vertexIndices: [VertexIndices; 4] = [
                VertexIndices {
                    iP: 0,
                    iN: 0,
                    iUV: 0,
                },
                VertexIndices {
                    iP: 0,
                    iN: 0,
                    iUV: 0,
                },
                VertexIndices {
                    iP: 0,
                    iN: 0,
                    iUV: 0,
                },
                VertexIndices {
                    iP: 0,
                    iN: 0,
                    iUV: 0,
                },
            ];
            while s.cursor < s.endOfData && *s.cursor as libc::c_int != '\r' as i32
                && *s.cursor as libc::c_int != '\n' as i32
            {
                let mut face: *mut VertexIndices = &mut *vertexIndices
                    .as_mut_ptr()
                    .offset(vertexIndicesCount as isize) as *mut VertexIndices;
                (*face).iUV = -(2147483647 as libc::c_int) - 1 as libc::c_int;
                (*face).iN = -(2147483647 as libc::c_int) - 1 as libc::c_int;
                if !ConsumeInt(&mut (*face).iP, &mut s) {
                    Obj_Fatal(
                        b"Failed to parse face vertex index from .obj data.\0"
                            as *const u8 as *const libc::c_char,
                        &mut s,
                    );
                }
                if ConsumeCharacter('/' as i32 as libc::c_char, &mut s) {
                    ConsumeInt(&mut (*face).iUV, &mut s);
                    if ConsumeCharacter('/' as i32 as libc::c_char, &mut s) {
                        ConsumeInt(&mut (*face).iN, &mut s);
                    }
                }
                vertexIndicesCount += 1;
                ConsumeWhitespace(&mut s);
            }
            let mut i: int32 = 0 as libc::c_int;
            while i < vertexIndicesCount {
                if vertexCount == 2147483647 as libc::c_int {
                    Obj_Fatal(
                        b".obj data contains more vertex indices than will fit in an ArrayList.\0"
                            as *const u8 as *const libc::c_char,
                        &mut s,
                    );
                }
                let mut face_0: *mut VertexIndices = &mut *vertexIndices
                    .as_mut_ptr()
                    .offset(i as isize) as *mut VertexIndices;
                let mut vertex: Vertex = {
                    let mut init = Vertex {
                        p: Vec3f { x: 0., y: 0., z: 0. },
                        n: Vec3f { x: 0., y: 0., z: 0. },
                        uv: Vec2f { x: 0., y: 0. },
                    };
                    init
                };
                (*face_0).iP
                    += if (*face_0).iP < 0 as libc::c_int {
                        positions_size
                    } else {
                        -(1 as libc::c_int)
                    };
                if (*face_0).iP < 0 as libc::c_int || (*face_0).iP >= positions_size {
                    Obj_Fatal(
                        b"Face vertex index is out of range in .obj data\0" as *const u8
                            as *const libc::c_char,
                        &mut s,
                    );
                }
                vertex.p = *positions_data.offset((*face_0).iP as isize);
                if (*face_0).iN != -(2147483647 as libc::c_int) - 1 as libc::c_int {
                    (*face_0).iN
                        += if (*face_0).iN < 0 as libc::c_int {
                            normals_size
                        } else {
                            -(1 as libc::c_int)
                        };
                    if (*face_0).iN < 0 as libc::c_int || (*face_0).iN >= normals_size {
                        Obj_Fatal(
                            b"Face normal index is out of range in .obj data\0"
                                as *const u8 as *const libc::c_char,
                            &mut s,
                        );
                    }
                    vertex.n = *normals_data.offset((*face_0).iN as isize);
                }
                if (*face_0).iUV != -(2147483647 as libc::c_int) - 1 as libc::c_int {
                    (*face_0).iUV
                        += if (*face_0).iUV < 0 as libc::c_int {
                            uvs_size
                        } else {
                            -(1 as libc::c_int)
                        };
                    if (*face_0).iUV < 0 as libc::c_int || (*face_0).iUV >= uvs_size {
                        Obj_Fatal(
                            b"Face UV index is out of range in .obj data\0" as *const u8
                                as *const libc::c_char,
                            &mut s,
                        );
                    }
                    vertex.uv = *uvs_data.offset((*face_0).iUV as isize);
                }
                vertexCount += 1;
                Mesh_AddVertexRaw(mesh, &mut vertex);
                i += 1;
            }
            if indexCount >= 2147483647 as libc::c_int - vertexIndicesCount {
                Obj_Fatal(
                    b".obj data contains more vertex indices than will fit in an ArrayList\0"
                        as *const u8 as *const libc::c_char,
                    &mut s,
                );
            }
            let mut vertices: *mut Vertex = Mesh_GetVertexData(mesh);
            let mut verticesLen: int32 = Mesh_GetVertexCount(mesh);
            let mut i_0: int32 = 0 as libc::c_int;
            while i_0 < vertexIndicesCount {
                let mut j: int32 = i_0 + 1 as libc::c_int;
                while j < vertexIndicesCount {
                    let mut p1: Vec3f = (*vertices
                        .offset((verticesLen - vertexIndicesCount + i_0) as isize))
                        .p;
                    let mut p2: Vec3f = (*vertices
                        .offset((verticesLen - vertexIndicesCount + j) as isize))
                        .p;
                    if Vec3f_Equal(p1, p2) {
                        Obj_Fatal(
                            b".obj data contains a degenerate polygon.\0" as *const u8
                                as *const libc::c_char,
                            &mut s,
                        );
                    }
                    j += 1;
                }
                i_0 += 1;
            }
            if vertexIndicesCount == 3 as libc::c_int {
                faceCount += 1 as libc::c_int;
                indexCount += vertexIndicesCount;
                Mesh_AddTri(
                    mesh,
                    vertexCount - 3 as libc::c_int,
                    vertexCount - 2 as libc::c_int,
                    vertexCount - 1 as libc::c_int,
                );
            } else if vertexIndicesCount == 4 as libc::c_int {
                faceCount += 2 as libc::c_int;
                indexCount += vertexIndicesCount;
                Mesh_AddQuad(
                    mesh,
                    vertexCount - 4 as libc::c_int,
                    vertexCount - 3 as libc::c_int,
                    vertexCount - 2 as libc::c_int,
                    vertexCount - 1 as libc::c_int,
                );
            } else {
                Obj_Fatal(
                    b".obj data has an unexpected number of vertices in a face\0"
                        as *const u8 as *const libc::c_char,
                    &mut s,
                );
            }
        } else if !(StrEqual(
            token.as_mut_ptr() as cstr,
            b"#\0" as *const u8 as *const libc::c_char,
        ) as libc::c_int != 0
            || StrEqual(
                token.as_mut_ptr() as cstr,
                b"f\0" as *const u8 as *const libc::c_char,
            ) as libc::c_int != 0
            || StrEqual(
                token.as_mut_ptr() as cstr,
                b"s\0" as *const u8 as *const libc::c_char,
            ) as libc::c_int != 0
            || StrEqual(
                token.as_mut_ptr() as cstr,
                b"p\0" as *const u8 as *const libc::c_char,
            ) as libc::c_int != 0
            || StrEqual(
                token.as_mut_ptr() as cstr,
                b"l\0" as *const u8 as *const libc::c_char,
            ) as libc::c_int != 0
            || StrEqual(
                token.as_mut_ptr() as cstr,
                b"g\0" as *const u8 as *const libc::c_char,
            ) as libc::c_int != 0
            || StrEqual(
                token.as_mut_ptr() as cstr,
                b"o\0" as *const u8 as *const libc::c_char,
            ) as libc::c_int != 0
            || StrEqual(
                token.as_mut_ptr() as cstr,
                b"maplib\0" as *const u8 as *const libc::c_char,
            ) as libc::c_int != 0
            || StrEqual(
                token.as_mut_ptr() as cstr,
                b"usemap\0" as *const u8 as *const libc::c_char,
            ) as libc::c_int != 0
            || StrEqual(
                token.as_mut_ptr() as cstr,
                b"usemtl\0" as *const u8 as *const libc::c_char,
            ) as libc::c_int != 0
            || StrEqual(
                token.as_mut_ptr() as cstr,
                b"mtllib\0" as *const u8 as *const libc::c_char,
            ) as libc::c_int != 0)
        {
            Obj_Fatal(
                b"Unsupported token in .obj data.\0" as *const u8 as *const libc::c_char,
                &mut s,
            );
        }
        ConsumeRestOfLine(&mut s);
    }
    MemFree(positions_data as *const libc::c_void);
    MemFree(uvs_data as *const libc::c_void);
    MemFree(normals_data as *const libc::c_void);
    return mesh;
}
