use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec2;
use crate::Math::Vec3;
use crate::Mesh::*;
use libc;

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
    let mut len: i32 = 0;
    let mut ch: *const libc::c_char = (*s).lineStart;
    while ch < (*s).endOfData && *ch as i32 != '\r' as i32 && *ch as i32 != '\n' as i32 {
        ch = ch.offset(1);
        len += 1;
    }
    let mut line: *mut libc::c_char = MemAlloc((len + 1) as usize) as *mut libc::c_char;
    MemCpy(line as *mut _, (*s).lineStart as *const _, len as usize);
    *line.offset(len as isize) = 0 as libc::c_char;
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
    let mut cr: i32 = 0;
    let mut nl: i32 = 0;
    while (*s).cursor < (*s).endOfData
        && (*(*s).cursor as i32 == '\r' as i32 || *(*s).cursor as i32 == '\n' as i32)
    {
        if *(*s).cursor as i32 == '\r' as i32 {
            if cr == 1 {
                nl = 0;
                cr = nl;
                (*s).lineNumber += 1;
            }
            cr += 1;
        }
        if *(*s).cursor as i32 == '\n' as i32 {
            if nl == 1 {
                nl = 0;
                cr = nl;
                (*s).lineNumber += 1;
            }
            nl += 1;
        }
        (*s).cursor = ((*s).cursor).offset(1);
    }
    (*s).cursor != oldPosition
}

unsafe extern "C" fn ConsumeWhitespace(mut s: *mut ParseState) -> bool {
    let mut oldPosition: *const libc::c_char = (*s).cursor;
    while (*s).cursor < (*s).endOfData
        && (*(*s).cursor as i32 == ' ' as i32 || *(*s).cursor as i32 == '\t' as i32)
    {
        (*s).cursor = ((*s).cursor).offset(1);
    }
    (*s).cursor != oldPosition
}

unsafe extern "C" fn ConsumeToken(
    mut token: *mut libc::c_char,
    mut tokenLen: i32,
    mut s: *mut ParseState,
) -> bool {
    let mut i: i32 = 0;
    while (*s).cursor < (*s).endOfData
        && i < tokenLen - 1
        && *(*s).cursor as i32 != ' ' as i32
        && *(*s).cursor as i32 != '\t' as i32
        && *(*s).cursor as i32 != '\r' as i32
        && *(*s).cursor as i32 != '\n' as i32
    {
        let fresh0 = i;
        i += 1;
        *token.offset(fresh0 as isize) = *(*s).cursor;
        (*s).cursor = ((*s).cursor).offset(1);
    }
    *token.offset(i as isize) = 0 as libc::c_char;
    i != 0
}

unsafe extern "C" fn ConsumeFloat(mut value: *mut f32, mut s: *mut ParseState) -> bool {
    let mut afterFloat: *mut libc::c_char = std::ptr::null_mut();
    let mut f: f32 = libc::strtof((*s).cursor, &mut afterFloat);
    if std::io::Error::last_os_error().raw_os_error().unwrap_or(0) == 34 {
        Obj_Fatal(
            b"Parsed float in .obj data is out of range.\0" as *const u8 as *const libc::c_char,
            s,
        );
    }
    if afterFloat != (*s).cursor as *mut libc::c_char {
        (*s).cursor = afterFloat;
        *value = f;
        return true;
    }
    false
}

unsafe extern "C" fn ConsumeInt(mut value: *mut i32, mut s: *mut ParseState) -> bool {
    let mut afterInt: *mut libc::c_char = std::ptr::null_mut();
    let mut i: i32 = libc::strtol((*s).cursor, &mut afterInt, 10) as i32;
    if std::io::Error::last_os_error().raw_os_error().unwrap_or(0) == 34 {
        Obj_Fatal(
            b"Parsed int in .obj data is out of range.\0" as *const u8 as *const libc::c_char,
            s,
        );
    }
    if afterInt != (*s).cursor as *mut libc::c_char {
        (*s).cursor = afterInt;
        *value = i;
        return true;
    }
    false
}

unsafe extern "C" fn ConsumeCharacter(mut character: libc::c_char, mut s: *mut ParseState) -> bool {
    if (*s).cursor < (*s).endOfData && *(*s).cursor as i32 == character as i32 {
        (*s).cursor = ((*s).cursor).offset(1);
        return true;
    }
    false
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_FromObj(mut bytes: *const libc::c_char) -> *mut Mesh {
    let mut bytesSize: i32 = StrLen(bytes) as i32;

    let mut s: ParseState = ParseState {
        cursor: bytes,
        endOfData: bytes.offset(bytesSize as isize),
        lineStart: std::ptr::null(),
        lineNumber: 0,
    };
    
    let mut mesh: *mut Mesh = Mesh_Create();
    let mut vertexCount: i32 = 0;
    let mut indexCount: i32 = 0;
    let mut faceCount: i32 = 0;

    let mut positions: Vec<Vec3> = Vec::new();
    let mut uvs: Vec<Vec2> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();
    
    positions.reserve((0.008f32 * bytesSize as f32) as usize);
    uvs.reserve((0.008f32 * bytesSize as f32) as usize);
    normals.reserve((0.008f32 * bytesSize as f32) as usize);
    Mesh_ReserveIndexData(mesh, (0.050f32 * bytesSize as f32) as i32);
    Mesh_ReserveVertexData(mesh, (0.050f32 * bytesSize as f32) as i32);

    loop {
        s.lineStart = s.cursor;
        s.lineNumber += 1;

        let mut token: [libc::c_char; 16] = [0; 16];

        ConsumeWhitespace(&mut s);
        ConsumeToken(token.as_mut_ptr(), 16, &mut s);
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
            if positions.len() == i32::MAX as usize {
                Obj_Fatal(
                    b".obj data contains more vertex positions than will fit in a vector.\0"
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
            
            positions.push(p);
        } else if StrEqual(
            token.as_mut_ptr() as *const libc::c_char,
            b"vt\0" as *const u8 as *const libc::c_char,
        ) {
            if uvs.len() == i32::MAX as usize {
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

            uvs.push(uv);
        } else if StrEqual(
            token.as_mut_ptr() as *const libc::c_char,
            b"vn\0" as *const u8 as *const libc::c_char,
        ) {
            if normals.len() == i32::MAX as usize {
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

            normals.push(n);
        } else if StrEqual(
            token.as_mut_ptr() as *const libc::c_char,
            b"f\0" as *const u8 as *const libc::c_char,
        ) {
            let mut vertexIndicesCount: i32 = 0;
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
                && *s.cursor != '\r' as libc::c_char
                && *s.cursor != '\n' as libc::c_char
            {
                let mut face: *mut VertexIndices = &mut *vertexIndices
                    .as_mut_ptr()
                    .offset(vertexIndicesCount as isize)
                    as *mut VertexIndices;

                (*face).iUV = i32::MIN;
                (*face).iN = i32::MIN;

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

            for i in 0..vertexIndicesCount {
                if vertexCount == i32::MAX {
                    Obj_Fatal(
                        b".obj data contains more vertex indices than will fit in an ArrayList.\0"
                            as *const u8 as *const libc::c_char,
                        &mut s,
                    );
                }

                let mut face: *mut VertexIndices = &mut *vertexIndices.as_mut_ptr().offset(i as isize);
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

                (*face).iP += if (*face).iP < 0 { positions.len() as i32 } else { i32::MAX };
                if (*face).iP < 0 || (*face).iP >= positions.len() as i32 {
                    Obj_Fatal(
                        b"Face vertex index is out of range in .obj data\0" as *const u8
                            as *const libc::c_char,
                        &mut s,
                    );
                }

                vertex.p = positions[(*face).iP as usize];
                if (*face).iN != i32::MIN {
                    (*face).iN += if (*face).iN < 0 { normals.len() as i32 } else { i32::MAX };
                    if (*face).iN < 0 || (*face).iN >= normals.len() as i32 {
                        Obj_Fatal(
                            b"Face normal index is out of range in .obj data\0" as *const u8
                                as *const libc::c_char,
                            &mut s,
                        );
                    }
                    vertex.n = normals[(*face).iN as usize];
                }
                
                if (*face).iUV != i32::MIN {
                    (*face).iUV += if (*face).iUV < 0 { uvs.len() as i32 } else { i32::MAX };
                    if (*face).iUV < 0 || (*face).iUV >= uvs.len() as i32 {
                        Obj_Fatal(
                            b"Face UV index is out of range in .obj data\0" as *const u8
                                as *const libc::c_char,
                            &mut s,
                        );
                    }
                    vertex.uv = uvs[(*face).iUV as usize];
                }

                vertexCount += 1;
                Mesh_AddVertexRaw(mesh, &mut vertex);
            }

            if indexCount >= i32::MAX - vertexIndicesCount {
                Obj_Fatal(
                    b".obj data contains more vertex indices than will fit in an ArrayList\0"
                        as *const u8 as *const libc::c_char,
                    &mut s,
                );
            }

            let mut vertices: *mut Vertex = Mesh_GetVertexData(mesh);
            let mut verticesLen: i32 = Mesh_GetVertexCount(mesh);
            for i in 0..vertexIndicesCount {
                for j in (i + 1)..vertexIndicesCount {
                    let p1: Vec3 =
                        (*vertices.offset((verticesLen - vertexIndicesCount + i) as isize)).p;
                    let p2: Vec3 =
                        (*vertices.offset((verticesLen - vertexIndicesCount + j) as isize)).p;
                    if p1 == p2 {
                        Obj_Fatal(
                            b".obj data contains a degenerate polygon.\0" as *const u8
                                as *const libc::c_char,
                            &mut s,
                        );
                    }
                }
            }

            if vertexIndicesCount == 3 {
                faceCount += 1;
                indexCount += vertexIndicesCount;
                Mesh_AddTri(mesh, vertexCount - 3, vertexCount - 2, vertexCount - 1);
            } else if vertexIndicesCount == 4 {
                faceCount += 2;
                indexCount += vertexIndicesCount;
                Mesh_AddQuad(
                    mesh,
                    vertexCount - 4,
                    vertexCount - 3,
                    vertexCount - 2,
                    vertexCount - 1,
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
    mesh
}
