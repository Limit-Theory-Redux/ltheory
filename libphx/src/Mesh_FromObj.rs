use crate::internal::Memory::*;
use crate::Mesh::*;
use glam::Vec2;
use glam::Vec3;
use libc;

extern "C" {
    fn Fatal(_: *const libc::c_char, _: ...);
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ParseState {
    pub cursor: *const libc::c_char,
    pub endOfData: *const libc::c_char,
    pub lineStart: *const libc::c_char,
    pub lineNumber: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct VertexIndices {
    pub iP: i32,
    pub iN: i32,
    pub iUV: i32,
}

unsafe extern "C" fn Obj_Fatal(mut message: *const libc::c_char, mut s: *mut ParseState) {
    let mut len: i32 = 0 as i32;
    let mut ch: *const libc::c_char = (*s).lineStart;
    while ch < (*s).endOfData && *ch as i32 != '\r' as i32 && *ch as i32 != '\n' as i32 {
        ch = ch.offset(1);
        len += 1 as i32;
    }
    let mut line: *mut libc::c_char = MemAlloc((len + 1 as i32) as usize) as *mut libc::c_char;
    MemCpy(
        line as *mut libc::c_void,
        (*s).lineStart as *const libc::c_void,
        len as usize,
    );
    *line.offset(len as isize) = 0 as i32 as libc::c_char;
    Fatal(
        b"%s Line %i\n%s\0" as *const u8 as *const libc::c_char,
        message,
        (*s).lineNumber,
        line,
    );
}

unsafe extern "C" fn ConsumeRestOfLine(mut s: *mut ParseState) -> bool {
    let mut oldPosition: *const libc::c_char = (*s).cursor;
    while (*s).cursor < (*s).endOfData
        && *(*s).cursor as i32 != '\r' as i32
        && *(*s).cursor as i32 != '\n' as i32
    {
        (*s).cursor = ((*s).cursor).offset(1);
    }
    let mut cr: i32 = 0 as i32;
    let mut nl: i32 = 0 as i32;
    while (*s).cursor < (*s).endOfData
        && (*(*s).cursor as i32 == '\r' as i32 || *(*s).cursor as i32 == '\n' as i32)
    {
        if *(*s).cursor as i32 == '\r' as i32 {
            if cr == 1 as i32 {
                nl = 0 as i32;
                cr = nl;
                (*s).lineNumber += 1;
            }
            cr += 1;
        }
        if *(*s).cursor as i32 == '\n' as i32 {
            if nl == 1 as i32 {
                nl = 0 as i32;
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
        && (*(*s).cursor as i32 == ' ' as i32 || *(*s).cursor as i32 == '\t' as i32)
    {
        (*s).cursor = ((*s).cursor).offset(1);
    }
    return (*s).cursor != oldPosition;
}

unsafe extern "C" fn ConsumeToken(
    mut token: *mut libc::c_char,
    mut tokenLen: i32,
    mut s: *mut ParseState,
) -> bool {
    let mut i: i32 = 0 as i32;
    while (*s).cursor < (*s).endOfData
        && i < tokenLen - 1 as i32
        && *(*s).cursor as i32 != ' ' as i32
        && *(*s).cursor as i32 != '\t' as i32
        && *(*s).cursor as i32 != '\r' as i32
        && *(*s).cursor as i32 != '\n' as i32
    {
        let fresh0 = i;
        i = i + 1;
        *token.offset(fresh0 as isize) = *(*s).cursor;
        (*s).cursor = ((*s).cursor).offset(1);
    }
    *token.offset(i as isize) = 0 as i32 as libc::c_char;
    return i != 0 as i32;
}

unsafe extern "C" fn ConsumeFloat(mut value: *mut f32, mut s: *mut ParseState) -> bool {
    let mut afterFloat: *mut libc::c_char = std::ptr::null_mut();
    let mut f: f32 = libc::strtof((*s).cursor, &mut afterFloat);
    if std::io::Error::last_os_error().raw_os_error().unwrap_or(0) == 34 as i32 {
        Obj_Fatal(
            b"Parsed float in .obj data is out of range.\0" as *const u8 as *const libc::c_char,
            s,
        );
    }
    if afterFloat != (*s).cursor as *mut libc::c_char {
        (*s).cursor = afterFloat;
        *value = f;
        return 1 as i32 != 0;
    }
    return 0 as i32 != 0;
}

unsafe extern "C" fn ConsumeInt(mut value: *mut i32, mut s: *mut ParseState) -> bool {
    let mut afterInt: *mut libc::c_char = std::ptr::null_mut();
    let mut i: i32 = libc::strtol((*s).cursor, &mut afterInt, 10 as i32) as i32;
    if std::io::Error::last_os_error().raw_os_error().unwrap_or(0) == 34 as i32 {
        Obj_Fatal(
            b"Parsed int in .obj data is out of range.\0" as *const u8 as *const libc::c_char,
            s,
        );
    }
    if afterInt != (*s).cursor as *mut libc::c_char {
        (*s).cursor = afterInt;
        *value = i;
        return 1 as i32 != 0;
    }
    return 0 as i32 != 0;
}

unsafe extern "C" fn ConsumeCharacter(mut character: libc::c_char, mut s: *mut ParseState) -> bool {
    if (*s).cursor < (*s).endOfData && *(*s).cursor as i32 == character as i32 {
        (*s).cursor = ((*s).cursor).offset(1);
        return 1 as i32 != 0;
    }
    return 0 as i32 != 0;
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_FromObj(mut bytes: *const libc::c_char) -> *mut Mesh {
    let mut bytesSize: i32 = StrLen(bytes) as i32;
    let mut s: ParseState = ParseState {
        cursor: std::ptr::null(),
        endOfData: std::ptr::null(),
        lineStart: std::ptr::null(),
        lineNumber: 0,
    };
    s.cursor = bytes;
    s.endOfData = (s.cursor).offset(bytesSize as isize);
    let mut mesh: *mut Mesh = Mesh_Create();
    let mut vertexCount: i32 = 0 as i32;
    let mut indexCount: i32 = 0 as i32;
    let mut faceCount: i32 = 0 as i32;
    let mut positions_size: i32 = 0;
    let mut positions_capacity: i32 = 0;
    let mut positions_data: *mut Vec3 = std::ptr::null_mut();
    positions_capacity = 0 as i32;
    positions_size = 0 as i32;
    positions_data = std::ptr::null_mut();
    let mut uvs_size: i32 = 0;
    let mut uvs_capacity: i32 = 0;
    let mut uvs_data: *mut Vec2 = std::ptr::null_mut();
    uvs_capacity = 0 as i32;
    uvs_size = 0 as i32;
    uvs_data = std::ptr::null_mut();
    let mut normals_size: i32 = 0;
    let mut normals_capacity: i32 = 0;
    let mut normals_data: *mut Vec3 = std::ptr::null_mut();
    normals_capacity = 0 as i32;
    normals_size = 0 as i32;
    normals_data = std::ptr::null_mut();
    if (positions_capacity < (0.008f32 * bytesSize as f32) as i32) as libc::c_long != 0 {
        positions_capacity = (0.008f32 * bytesSize as f32) as i32;
        let mut elemSize: usize = ::core::mem::size_of::<Vec3>();
        let mut pData: *mut *mut libc::c_void =
            &mut positions_data as *mut *mut Vec3 as *mut *mut libc::c_void;
        *pData = MemRealloc(
            positions_data as *mut libc::c_void,
            (positions_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
    if (uvs_capacity < (0.008f32 * bytesSize as f32) as i32) as i32 as libc::c_long != 0 {
        uvs_capacity = (0.008f32 * bytesSize as f32) as i32;
        let mut elemSize_0: usize = ::core::mem::size_of::<Vec2>();
        let mut pData_0: *mut *mut libc::c_void =
            &mut uvs_data as *mut *mut Vec2 as *mut *mut libc::c_void;
        *pData_0 = MemRealloc(
            uvs_data as *mut libc::c_void,
            (uvs_capacity as usize).wrapping_mul(elemSize_0 as usize),
        );
    }
    if (normals_capacity < (0.008f32 * bytesSize as f32) as i32) as libc::c_long != 0 {
        normals_capacity = (0.008f32 * bytesSize as f32) as i32;
        let mut elemSize_1: usize = ::core::mem::size_of::<Vec3>();
        let mut pData_1: *mut *mut libc::c_void =
            &mut normals_data as *mut *mut Vec3 as *mut *mut libc::c_void;
        *pData_1 = MemRealloc(
            normals_data as *mut libc::c_void,
            (normals_capacity as usize).wrapping_mul(elemSize_1 as usize),
        );
    }
    Mesh_ReserveIndexData(mesh, (0.050f32 * bytesSize as f32) as i32);
    Mesh_ReserveVertexData(mesh, (0.050f32 * bytesSize as f32) as i32);
    loop {
        s.lineStart = s.cursor;
        s.lineNumber += 1;
        let mut token: [libc::c_char; 16] = [0; 16];
        ConsumeWhitespace(&mut s);
        ConsumeToken(token.as_mut_ptr(), 16 as i32, &mut s);
        ConsumeWhitespace(&mut s);
        if StrEqual(
            token.as_mut_ptr() as *const libc::c_char,
            b"\0" as *const u8 as *const libc::c_char,
        ) {
            if s.cursor >= s.endOfData {
                break;
            }
        } else if StrEqual(
            token.as_mut_ptr() as *const libc::c_char,
            b"v\0" as *const u8 as *const libc::c_char,
        ) {
            if positions_size == 2147483647 as i32 {
                Obj_Fatal(
                    b".obj data contains more vertex positions than will fit in an ArrayList.\0"
                        as *const u8 as *const libc::c_char,
                    &mut s,
                );
            }
            let mut p = Vec3::ZERO;
            if !(ConsumeFloat(&mut p.x, &mut s) as i32 != 0
                && ConsumeFloat(&mut p.y, &mut s) as i32 != 0
                && ConsumeFloat(&mut p.z, &mut s) as i32 != 0)
            {
                Obj_Fatal(
                    b"Failed to parse geometric vertex from .obj data.\0" as *const u8
                        as *const libc::c_char,
                    &mut s,
                );
            }
            if (positions_capacity == positions_size) as libc::c_long != 0 {
                positions_capacity = if positions_capacity != 0 {
                    positions_capacity * 2 as i32
                } else {
                    1 as i32
                };
                let mut elemSize_2: usize = ::core::mem::size_of::<Vec3>();
                let mut pData_2: *mut *mut libc::c_void =
                    &mut positions_data as *mut *mut Vec3 as *mut *mut libc::c_void;
                *pData_2 = MemRealloc(
                    positions_data as *mut libc::c_void,
                    (positions_capacity as usize).wrapping_mul(elemSize_2 as usize),
                );
            }
            let fresh1 = positions_size;
            positions_size = positions_size + 1;
            *positions_data.offset(fresh1 as isize) = p;
        } else if StrEqual(
            token.as_mut_ptr() as *const libc::c_char,
            b"vt\0" as *const u8 as *const libc::c_char,
        ) {
            if uvs_size == 2147483647 as i32 {
                Obj_Fatal(
                    b".obj data contains more UVs than will fit in an ArrayList.\0" as *const u8
                        as *const libc::c_char,
                    &mut s,
                );
            }
            let mut uv = Vec2::ZERO;
            if !(ConsumeFloat(&mut uv.x, &mut s) as i32 != 0
                && ConsumeFloat(&mut uv.y, &mut s) as i32 != 0)
            {
                Obj_Fatal(
                    b"Failed to parse texture vertex from .obj data.\0" as *const u8
                        as *const libc::c_char,
                    &mut s,
                );
            }
            if (uvs_capacity == uvs_size) as libc::c_long != 0 {
                uvs_capacity = if uvs_capacity != 0 {
                    uvs_capacity * 2 as i32
                } else {
                    1 as i32
                };
                let mut elemSize_3: usize = ::core::mem::size_of::<Vec2>();
                let mut pData_3: *mut *mut libc::c_void =
                    &mut uvs_data as *mut *mut Vec2 as *mut *mut libc::c_void;
                *pData_3 = MemRealloc(
                    uvs_data as *mut libc::c_void,
                    (uvs_capacity as usize).wrapping_mul(elemSize_3 as usize),
                );
            }
            let fresh2 = uvs_size;
            uvs_size = uvs_size + 1;
            *uvs_data.offset(fresh2 as isize) = uv;
        } else if StrEqual(
            token.as_mut_ptr() as *const libc::c_char,
            b"vn\0" as *const u8 as *const libc::c_char,
        ) {
            if normals_size == 2147483647 as i32 {
                Obj_Fatal(
                    b".obj data contains more normals than will fit in an ArrayList.\0" as *const u8
                        as *const libc::c_char,
                    &mut s,
                );
            }
            let mut n = Vec3::ZERO;
            if !(ConsumeFloat(&mut n.x, &mut s) as i32 != 0
                && ConsumeFloat(&mut n.y, &mut s) as i32 != 0
                && ConsumeFloat(&mut n.z, &mut s) as i32 != 0)
            {
                Obj_Fatal(
                    b"Failed to parse vertex normal from .obj data.\0" as *const u8
                        as *const libc::c_char,
                    &mut s,
                );
            }
            if (normals_capacity == normals_size) as libc::c_long != 0 {
                normals_capacity = if normals_capacity != 0 {
                    normals_capacity * 2 as i32
                } else {
                    1 as i32
                };
                let mut elemSize_4: usize = ::core::mem::size_of::<Vec3>();
                let mut pData_4: *mut *mut libc::c_void =
                    &mut normals_data as *mut *mut Vec3 as *mut *mut libc::c_void;
                *pData_4 = MemRealloc(
                    normals_data as *mut libc::c_void,
                    (normals_capacity as usize).wrapping_mul(elemSize_4 as usize),
                );
            }
            let fresh3 = normals_size;
            normals_size = normals_size + 1;
            *normals_data.offset(fresh3 as isize) = n;
        } else if StrEqual(
            token.as_mut_ptr() as *const libc::c_char,
            b"f\0" as *const u8 as *const libc::c_char,
        ) {
            let mut vertexIndicesCount: i32 = 0 as i32;
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
            while s.cursor < s.endOfData
                && *s.cursor as i32 != '\r' as i32
                && *s.cursor as i32 != '\n' as i32
            {
                let mut face: *mut VertexIndices = &mut *vertexIndices
                    .as_mut_ptr()
                    .offset(vertexIndicesCount as isize)
                    as *mut VertexIndices;
                (*face).iUV = -(2147483647 as i32) - 1 as i32;
                (*face).iN = -(2147483647 as i32) - 1 as i32;
                if !ConsumeInt(&mut (*face).iP, &mut s) {
                    Obj_Fatal(
                        b"Failed to parse face vertex index from .obj data.\0" as *const u8
                            as *const libc::c_char,
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
            let mut i: i32 = 0 as i32;
            while i < vertexIndicesCount {
                if vertexCount == 2147483647 as i32 {
                    Obj_Fatal(
                        b".obj data contains more vertex indices than will fit in an ArrayList.\0"
                            as *const u8 as *const libc::c_char,
                        &mut s,
                    );
                }
                let mut face_0: *mut VertexIndices =
                    &mut *vertexIndices.as_mut_ptr().offset(i as isize) as *mut VertexIndices;
                let mut vertex: Vertex = Vertex {
                    p: Vec3 {
                        x: 0.,
                        y: 0.,
                        z: 0.,
                    },
                    n: Vec3 {
                        x: 0.,
                        y: 0.,
                        z: 0.,
                    },
                    uv: Vec2::ZERO,
                };
                (*face_0).iP += if (*face_0).iP < 0 as i32 {
                    positions_size
                } else {
                    -(1 as i32)
                };
                if (*face_0).iP < 0 as i32 || (*face_0).iP >= positions_size {
                    Obj_Fatal(
                        b"Face vertex index is out of range in .obj data\0" as *const u8
                            as *const libc::c_char,
                        &mut s,
                    );
                }
                vertex.p = *positions_data.offset((*face_0).iP as isize);
                if (*face_0).iN != -(2147483647 as i32) - 1 as i32 {
                    (*face_0).iN += if (*face_0).iN < 0 as i32 {
                        normals_size
                    } else {
                        -(1 as i32)
                    };
                    if (*face_0).iN < 0 as i32 || (*face_0).iN >= normals_size {
                        Obj_Fatal(
                            b"Face normal index is out of range in .obj data\0" as *const u8
                                as *const libc::c_char,
                            &mut s,
                        );
                    }
                    vertex.n = *normals_data.offset((*face_0).iN as isize);
                }
                if (*face_0).iUV != -(2147483647 as i32) - 1 as i32 {
                    (*face_0).iUV += if (*face_0).iUV < 0 as i32 {
                        uvs_size
                    } else {
                        -(1 as i32)
                    };
                    if (*face_0).iUV < 0 as i32 || (*face_0).iUV >= uvs_size {
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
            if indexCount >= 2147483647 as i32 - vertexIndicesCount {
                Obj_Fatal(
                    b".obj data contains more vertex indices than will fit in an ArrayList\0"
                        as *const u8 as *const libc::c_char,
                    &mut s,
                );
            }
            let mut vertices: *mut Vertex = Mesh_GetVertexData(mesh);
            let mut verticesLen: i32 = Mesh_GetVertexCount(mesh);
            let mut i_0: i32 = 0 as i32;
            while i_0 < vertexIndicesCount {
                let mut j: i32 = i_0 + 1 as i32;
                while j < vertexIndicesCount {
                    let mut p1: Vec3 =
                        (*vertices.offset((verticesLen - vertexIndicesCount + i_0) as isize)).p;
                    let mut p2: Vec3 =
                        (*vertices.offset((verticesLen - vertexIndicesCount + j) as isize)).p;
                    if p1 == p2 {
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
            if vertexIndicesCount == 3 as i32 {
                faceCount += 1 as i32;
                indexCount += vertexIndicesCount;
                Mesh_AddTri(
                    mesh,
                    vertexCount - 3 as i32,
                    vertexCount - 2 as i32,
                    vertexCount - 1 as i32,
                );
            } else if vertexIndicesCount == 4 as i32 {
                faceCount += 2 as i32;
                indexCount += vertexIndicesCount;
                Mesh_AddQuad(
                    mesh,
                    vertexCount - 4 as i32,
                    vertexCount - 3 as i32,
                    vertexCount - 2 as i32,
                    vertexCount - 1 as i32,
                );
            } else {
                Obj_Fatal(
                    b".obj data has an unexpected number of vertices in a face\0" as *const u8
                        as *const libc::c_char,
                    &mut s,
                );
            }
        } else if !(StrEqual(
            token.as_mut_ptr() as *const libc::c_char,
            b"#\0" as *const u8 as *const libc::c_char,
        ) as i32
            != 0
            || StrEqual(
                token.as_mut_ptr() as *const libc::c_char,
                b"f\0" as *const u8 as *const libc::c_char,
            ) as i32
                != 0
            || StrEqual(
                token.as_mut_ptr() as *const libc::c_char,
                b"s\0" as *const u8 as *const libc::c_char,
            ) as i32
                != 0
            || StrEqual(
                token.as_mut_ptr() as *const libc::c_char,
                b"p\0" as *const u8 as *const libc::c_char,
            ) as i32
                != 0
            || StrEqual(
                token.as_mut_ptr() as *const libc::c_char,
                b"l\0" as *const u8 as *const libc::c_char,
            ) as i32
                != 0
            || StrEqual(
                token.as_mut_ptr() as *const libc::c_char,
                b"g\0" as *const u8 as *const libc::c_char,
            ) as i32
                != 0
            || StrEqual(
                token.as_mut_ptr() as *const libc::c_char,
                b"o\0" as *const u8 as *const libc::c_char,
            ) as i32
                != 0
            || StrEqual(
                token.as_mut_ptr() as *const libc::c_char,
                b"maplib\0" as *const u8 as *const libc::c_char,
            ) as i32
                != 0
            || StrEqual(
                token.as_mut_ptr() as *const libc::c_char,
                b"usemap\0" as *const u8 as *const libc::c_char,
            ) as i32
                != 0
            || StrEqual(
                token.as_mut_ptr() as *const libc::c_char,
                b"usemtl\0" as *const u8 as *const libc::c_char,
            ) as i32
                != 0
            || StrEqual(
                token.as_mut_ptr() as *const libc::c_char,
                b"mtllib\0" as *const u8 as *const libc::c_char,
            ) as i32
                != 0)
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
