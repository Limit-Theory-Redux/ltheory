use super::*;
use crate::common::*;
use crate::internal::*;
use crate::math::*;
use crate::*;

use std::ffi::CString;

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

fn Obj_Fatal(message: &str, s: *mut ParseState) {
    unsafe {
        let mut len: i32 = 0;
        let mut ch: *const libc::c_char = (*s).lineStart;
        while ch < (*s).endOfData && *ch as i32 != '\r' as i32 && *ch as i32 != '\n' as i32 {
            ch = ch.offset(1);
            len += 1;
        }
        let line: *mut libc::c_char = MemAlloc((len + 1) as usize) as *mut libc::c_char;
        MemCpy(line as *mut _, (*s).lineStart as *const _, len as usize);
        *line.offset(len as isize) = 0 as libc::c_char;

        panic!(
            "{message}. Line {}\n{:?}",
            (*s).lineNumber,
            CStr::from_ptr(line)
        );
    }
}

unsafe extern "C" fn ConsumeRestOfLine(s: *mut ParseState) -> bool {
    let oldPosition: *const libc::c_char = (*s).cursor;
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

unsafe extern "C" fn ConsumeWhitespace(s: *mut ParseState) -> bool {
    let oldPosition: *const libc::c_char = (*s).cursor;
    while (*s).cursor < (*s).endOfData
        && (*(*s).cursor as i32 == ' ' as i32 || *(*s).cursor as i32 == '\t' as i32)
    {
        (*s).cursor = ((*s).cursor).offset(1);
    }
    (*s).cursor != oldPosition
}

unsafe extern "C" fn ConsumeToken(
    token: *mut libc::c_char,
    tokenLen: i32,
    s: *mut ParseState,
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

unsafe extern "C" fn ConsumeFloat(value: *mut f32, s: *mut ParseState) -> bool {
    let mut afterFloat: *mut libc::c_char = std::ptr::null_mut();
    let f: f32 = libc::strtof((*s).cursor, &mut afterFloat);
    if std::io::Error::last_os_error().raw_os_error().unwrap_or(0) == 34 {
        Obj_Fatal("Parsed float in .obj data is out of range.", s);
    }
    if afterFloat != (*s).cursor as *mut libc::c_char {
        (*s).cursor = afterFloat;
        *value = f;
        return true;
    }
    false
}

unsafe extern "C" fn ConsumeInt(value: *mut i32, s: *mut ParseState) -> bool {
    let mut afterInt: *mut libc::c_char = std::ptr::null_mut();
    let i: i32 = libc::strtol((*s).cursor, &mut afterInt, 10) as i32;
    if std::io::Error::last_os_error().raw_os_error().unwrap_or(0) == 34 {
        Obj_Fatal("Parsed int in .obj data is out of range.", s);
    }
    if afterInt != (*s).cursor as *mut libc::c_char {
        (*s).cursor = afterInt;
        *value = i;
        return true;
    }
    false
}

unsafe extern "C" fn ConsumeCharacter(character: libc::c_char, s: *mut ParseState) -> bool {
    if (*s).cursor < (*s).endOfData && *(*s).cursor as i32 == character as i32 {
        (*s).cursor = ((*s).cursor).offset(1);
        return true;
    }
    false
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_FromObj(bytes: *const libc::c_char) -> Box<Mesh> {
    let bytesSize: i32 = bytes.as_str().len() as i32;

    let mut s: ParseState = ParseState {
        cursor: bytes,
        endOfData: bytes.offset(bytesSize as isize),
        lineStart: std::ptr::null(),
        lineNumber: 0,
    };

    let mut mesh = Mesh_Create();
    let mut vertexCount: i32 = 0;
    let mut indexCount: i32 = 0;
    let mut _faceCount: i32 = 0;

    let mut positions: Vec<Vec3> = Vec::new();
    let mut uvs: Vec<Vec2> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();

    positions.reserve((0.008f32 * bytesSize as f32) as usize);
    uvs.reserve((0.008f32 * bytesSize as f32) as usize);
    normals.reserve((0.008f32 * bytesSize as f32) as usize);
    Mesh_ReserveIndexData(&mut *mesh, (0.050f32 * bytesSize as f32) as i32);
    Mesh_ReserveVertexData(&mut *mesh, (0.050f32 * bytesSize as f32) as i32);

    loop {
        s.lineStart = s.cursor;
        s.lineNumber += 1;

        let mut token: [libc::c_char; 16] = [0; 16];

        ConsumeWhitespace(&mut s);
        ConsumeToken(token.as_mut_ptr(), 16, &mut s);
        ConsumeWhitespace(&mut s);

        let token_str = token.as_ptr().as_string();

        if token_str == "" {
            if s.cursor >= s.endOfData {
                break;
            }
        } else if token_str == "v" {
            if positions.len() == i32::MAX as usize {
                Obj_Fatal(
                    ".obj data contains more vertex positions than will fit in a vector.",
                    &mut s,
                );
            }

            let mut p = Vec3::ZERO;
            if !(ConsumeFloat(&mut p.x, &mut s) as i32 != 0
                && ConsumeFloat(&mut p.y, &mut s) as i32 != 0
                && ConsumeFloat(&mut p.z, &mut s) as i32 != 0)
            {
                Obj_Fatal("Failed to parse geometric vertex from .obj data.", &mut s);
            }

            positions.push(p);
        } else if token_str == "vt" {
            if uvs.len() == i32::MAX as usize {
                Obj_Fatal(
                    ".obj data contains more UVs than will fit in an ArrayList.",
                    &mut s,
                );
            }

            let mut uv = Vec2::ZERO;
            if !(ConsumeFloat(&mut uv.x, &mut s) as i32 != 0
                && ConsumeFloat(&mut uv.y, &mut s) as i32 != 0)
            {
                Obj_Fatal("Failed to parse texture vertex from .obj data.", &mut s);
            }

            uvs.push(uv);
        } else if token_str == "vn" {
            if normals.len() == i32::MAX as usize {
                Obj_Fatal(
                    ".obj data contains more normals than will fit in an ArrayList.",
                    &mut s,
                );
            }

            let mut n = Vec3::ZERO;
            if !(ConsumeFloat(&mut n.x, &mut s) as i32 != 0
                && ConsumeFloat(&mut n.y, &mut s) as i32 != 0
                && ConsumeFloat(&mut n.z, &mut s) as i32 != 0)
            {
                Obj_Fatal("Failed to parse vertex normal from .obj data.", &mut s);
            }

            normals.push(n);
        } else if token_str == "f" {
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
                let face: *mut VertexIndices = &mut *vertexIndices
                    .as_mut_ptr()
                    .offset(vertexIndicesCount as isize)
                    as *mut VertexIndices;

                (*face).iUV = i32::MIN;
                (*face).iN = i32::MIN;

                if !ConsumeInt(&mut (*face).iP, &mut s) {
                    Obj_Fatal("Failed to parse face vertex index from .obj data.", &mut s);
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
                        ".obj data contains more vertex indices than will fit in an ArrayList.",
                        &mut s,
                    );
                }

                let face: *mut VertexIndices = &mut *vertexIndices.as_mut_ptr().offset(i as isize);
                let mut vertex: Vertex = Vertex {
                    p: Vec3::ZERO,
                    n: Vec3::ZERO,
                    uv: Vec2::ZERO,
                };

                (*face).iP += if (*face).iP < 0 {
                    positions.len() as i32
                } else {
                    i32::MAX
                };
                if (*face).iP < 0 || (*face).iP >= positions.len() as i32 {
                    Obj_Fatal("Face vertex index is out of range in .obj data", &mut s);
                }

                vertex.p = positions[(*face).iP as usize];
                if (*face).iN != i32::MIN {
                    (*face).iN += if (*face).iN < 0 {
                        normals.len() as i32
                    } else {
                        i32::MAX
                    };
                    if (*face).iN < 0 || (*face).iN >= normals.len() as i32 {
                        Obj_Fatal("Face normal index is out of range in .obj data", &mut s);
                    }
                    vertex.n = normals[(*face).iN as usize];
                }

                if (*face).iUV != i32::MIN {
                    (*face).iUV += if (*face).iUV < 0 {
                        uvs.len() as i32
                    } else {
                        i32::MAX
                    };
                    if (*face).iUV < 0 || (*face).iUV >= uvs.len() as i32 {
                        Obj_Fatal("Face UV index is out of range in .obj data", &mut s);
                    }
                    vertex.uv = uvs[(*face).iUV as usize];
                }

                vertexCount += 1;
                Mesh_AddVertexRaw(&mut *mesh, &mut vertex);
            }

            if indexCount >= i32::MAX - vertexIndicesCount {
                Obj_Fatal(
                    ".obj data contains more vertex indices than will fit in an ArrayList",
                    &mut s,
                );
            }

            let vertices: *mut Vertex = Mesh_GetVertexData(&mut *mesh);
            let verticesLen: i32 = Mesh_GetVertexCount(&mut *mesh);
            for i in 0..vertexIndicesCount {
                for j in (i + 1)..vertexIndicesCount {
                    let p1: Vec3 =
                        (*vertices.offset((verticesLen - vertexIndicesCount + i) as isize)).p;
                    let p2: Vec3 =
                        (*vertices.offset((verticesLen - vertexIndicesCount + j) as isize)).p;
                    if p1 == p2 {
                        Obj_Fatal(".obj data contains a degenerate polygon.", &mut s);
                    }
                }
            }

            if vertexIndicesCount == 3 {
                _faceCount += 1;
                indexCount += vertexIndicesCount;
                Mesh_AddTri(
                    &mut *mesh,
                    vertexCount - 3,
                    vertexCount - 2,
                    vertexCount - 1,
                );
            } else if vertexIndicesCount == 4 {
                _faceCount += 2;
                indexCount += vertexIndicesCount;
                Mesh_AddQuad(
                    &mut *mesh,
                    vertexCount - 4,
                    vertexCount - 3,
                    vertexCount - 2,
                    vertexCount - 1,
                );
            } else {
                Obj_Fatal(
                    ".obj data has an unexpected number of vertices in a face",
                    &mut s,
                );
            }
        } else if !(token_str == "#"
            || token_str == "f"
            || token_str == "s"
            || token_str == "p"
            || token_str == "l"
            || token_str == "g"
            || token_str == "o"
            || token_str == "maplib"
            || token_str == "usemap"
            || token_str == "usemtl"
            || token_str == "mtllib")
        {
            Obj_Fatal("Unsupported token in .obj data.", &mut s);
        }
        ConsumeRestOfLine(&mut s);
    }
    mesh
}
