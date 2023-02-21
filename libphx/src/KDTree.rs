use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
use glam::Vec2;

extern "C" {
    pub type Mesh;
    pub type Matrix;
    fn qsort(
        __base: *mut libc::c_void,
        __nel: usize,
        __width: usize,
        __compar: Option::<
            unsafe extern "C" fn(*const libc::c_void, *const libc::c_void) -> i32,
        >,
    );
    fn Draw_Box3(box_0: *const Box3f);
    fn Draw_Color(
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    );
    fn Mesh_GetIndexCount(_: *mut Mesh) -> i32;
    fn Mesh_GetIndexData(_: *mut Mesh) -> *mut i32;
    fn Mesh_GetVertexData(_: *mut Mesh) -> *mut Vertex;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KDTree {
    pub box_0: Box3f,
    pub back: *mut KDTree,
    pub front: *mut KDTree,
    pub elems: *mut Node,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Node {
    pub next: *mut Node,
    pub id: u64,
    pub box_0: Box3f,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Box3f {
    pub lower: Vec3,
    pub upper: Vec3,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub p: Vec3,
    pub n: Vec3,
    pub uv: Vec2,
}
#[inline]
unsafe extern "C" fn Maxf(mut a: f32, mut b: f32) -> f32 {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Minf(mut a: f32, mut b: f32) -> f32 {
    return if a < b { a } else { b };
}
#[inline]
unsafe extern "C" fn Box3f_Create(mut lower: Vec3, mut upper: Vec3) -> Box3f {
    let mut result: Box3f =  Box3f {
            lower: lower,
            upper: upper,
        };
    return result;
}
#[inline]
unsafe extern "C" fn Box3f_Union(mut a: Box3f, mut b: Box3f) -> Box3f {
    let mut this: Box3f =  Box3f {
            lower:  Vec3 {
                    x: Minf(a.lower.x, b.lower.x),
                    y: Minf(a.lower.y, b.lower.y),
                    z: Minf(a.lower.z, b.lower.z),
                },
            upper:  Vec3 {
                    x: Maxf(a.upper.x, b.upper.x),
                    y: Maxf(a.upper.y, b.upper.y),
                    z: Maxf(a.upper.z, b.upper.z),
                },
        };
    return this;
}


#[no_mangle]
pub static mut kMaxLeafSize: i32 = 64 as i32;
unsafe extern "C" fn compareLowerX(
    mut a: *const libc::c_void,
    mut b: *const libc::c_void,
) -> i32 {
    return if (*(a as *const Box3f)).lower.x < (*(b as *const Box3f)).lower.x {
        -(1 as i32)
    } else {
        1 as i32
    };
}
unsafe extern "C" fn compareLowerY(
    mut a: *const libc::c_void,
    mut b: *const libc::c_void,
) -> i32 {
    return if (*(a as *const Box3f)).lower.y < (*(b as *const Box3f)).lower.y {
        -(1 as i32)
    } else {
        1 as i32
    };
}
unsafe extern "C" fn compareLowerZ(
    mut a: *const libc::c_void,
    mut b: *const libc::c_void,
) -> i32 {
    return if (*(a as *const Box3f)).lower.z < (*(b as *const Box3f)).lower.z {
        -(1 as i32)
    } else {
        1 as i32
    };
}
unsafe extern "C" fn Partition(
    mut boxes: *mut Box3f,
    mut boxCount: i32,
    mut dim: i32,
) -> *mut KDTree {
    let mut this: *mut KDTree = MemAlloc(
        ::core::mem::size_of::<KDTree>() as usize,
    ) as *mut KDTree;
    if boxCount <= kMaxLeafSize {
        (*this).box_0 = *boxes.offset(0);
        (*this).back = 0 as *mut KDTree;
        (*this).front = 0 as *mut KDTree;
        (*this).elems = 0 as *mut Node;
        let mut i: i32 = 1 as i32;
        while i < boxCount {
            (*this).box_0 = Box3f_Union((*this).box_0, *boxes.offset(i as isize));
            i += 1;
        }
        let mut i_0: i32 = 0 as i32;
        while i_0 < boxCount {
            let mut node: *mut Node = MemAlloc(
                ::core::mem::size_of::<Node>() as usize,
            ) as *mut Node;
            (*node).box_0 = *boxes.offset(i_0 as isize);
            (*node).next = (*this).elems;
            (*node).id = 0 as i32 as u64;
            (*this).elems = node;
            i_0 += 1;
        }
        return this;
    }
    if dim == 0 as i32 {
        qsort(
            boxes as *mut libc::c_void,
            boxCount as usize,
            ::core::mem::size_of::<Box3f>() as usize,
            Some(
                compareLowerX
                    as unsafe extern "C" fn(
                        *const libc::c_void,
                        *const libc::c_void,
                    ) -> i32,
            ),
        );
    }
    if dim == 1 as i32 {
        qsort(
            boxes as *mut libc::c_void,
            boxCount as usize,
            ::core::mem::size_of::<Box3f>() as usize,
            Some(
                compareLowerY
                    as unsafe extern "C" fn(
                        *const libc::c_void,
                        *const libc::c_void,
                    ) -> i32,
            ),
        );
    }
    if dim == 2 as i32 {
        qsort(
            boxes as *mut libc::c_void,
            boxCount as usize,
            ::core::mem::size_of::<Box3f>() as usize,
            Some(
                compareLowerZ
                    as unsafe extern "C" fn(
                        *const libc::c_void,
                        *const libc::c_void,
                    ) -> i32,
            ),
        );
    }
    let mut boxCountBack: i32 = boxCount / 2 as i32;
    let mut boxCountFront: i32 = boxCount - boxCountBack;
    let mut boxesBack: *mut Box3f = MemAlloc(
        (::core::mem::size_of::<Box3f>())
            .wrapping_mul(boxCountBack as usize),
    ) as *mut Box3f;
    let mut boxesFront: *mut Box3f = MemAlloc(
        (::core::mem::size_of::<Box3f>())
            .wrapping_mul(boxCountFront as usize),
    ) as *mut Box3f;
    MemCpy(
        boxesBack as *mut libc::c_void,
        boxes as *const libc::c_void,
        (boxCountBack as usize).wrapping_mul(::core::mem::size_of::<Box3f>()),
    );
    MemCpy(
        boxesFront as *mut libc::c_void,
        boxes.offset(boxCountBack as isize) as *const libc::c_void,
        (boxCountFront as usize).wrapping_mul(::core::mem::size_of::<Box3f>()),
    );
    (*this)
        .back = Partition(
        boxesBack,
        boxCountBack,
        (dim + 1 as i32) % 3 as i32,
    );
    (*this)
        .front = Partition(
        boxesFront,
        boxCountFront,
        (dim + 1 as i32) % 3 as i32,
    );
    (*this).box_0 = Box3f_Union((*(*this).back).box_0, (*(*this).front).box_0);
    (*this).elems = 0 as *mut Node;
    MemFree(boxesBack as *const libc::c_void);
    MemFree(boxesFront as *const libc::c_void);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn KDTree_FromMesh(mut mesh: *mut Mesh) -> *mut KDTree {
    let indexCount: i32 = Mesh_GetIndexCount(mesh);
    let mut indexData: *const i32 = Mesh_GetIndexData(mesh);
    let mut vertexData: *const Vertex = Mesh_GetVertexData(mesh);
    let boxCount: i32 = indexCount / 3 as i32;
    let mut boxes: *mut Box3f = MemAlloc(
        (::core::mem::size_of::<Box3f>())
            .wrapping_mul(boxCount as usize),
    ) as *mut Box3f;
    let mut i: i32 = 0 as i32;
    while i < indexCount {
        let mut v0: *const Vertex = vertexData
            .offset(*indexData.offset((i + 0 as i32) as isize) as isize);
        let mut v1: *const Vertex = vertexData
            .offset(*indexData.offset((i + 1 as i32) as isize) as isize);
        let mut v2: *const Vertex = vertexData
            .offset(*indexData.offset((i + 2 as i32) as isize) as isize);
        *boxes
            .offset(
                (i / 3 as i32) as isize,
            ) = Box3f_Create(
            Vec3::min((*v0).p, Vec3::min((*v1).p, (*v2).p)),
            Vec3::max((*v0).p, Vec3::max((*v1).p, (*v2).p)),
        );
        i += 3 as i32;
    }
    let mut this: *mut KDTree = Partition(boxes, boxCount, 0 as i32);
    MemFree(boxes as *const libc::c_void);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn KDTree_Free(mut this: *mut KDTree) {
    if !((*this).back).is_null() {
        KDTree_Free((*this).back);
    }
    if !((*this).front).is_null() {
        KDTree_Free((*this).front);
    }
    let mut elem: *mut Node = (*this).elems;
    while !elem.is_null() {
        let mut next: *mut Node = (*elem).next;
        MemFree(elem as *const libc::c_void);
        elem = next;
    }
    MemFree(this as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn KDTree_GetMemory(mut this: *mut KDTree) -> i32 {
    let mut memory: i32 = ::core::mem::size_of::<KDTree>() as usize
        as i32;
    if !((*this).back).is_null() {
        memory += KDTree_GetMemory((*this).back);
    }
    if !((*this).front).is_null() {
        memory += KDTree_GetMemory((*this).front);
    }
    let mut elem: *mut Node = (*this).elems;
    while !elem.is_null() {
        memory = (memory as usize)
            .wrapping_add(::core::mem::size_of::<Node>()) as i32
            as i32;
        elem = (*elem).next;
    }
    return memory;
}
#[no_mangle]
pub unsafe extern "C" fn KDTree_IntersectRay(
    mut this: *mut KDTree,
    mut m: *mut Matrix,
    mut a: *const Vec3,
    mut b: *const Vec3,
) -> bool {
    return 0 as i32 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn KDTree_Draw(
    mut this: *mut KDTree,
    mut maxDepth: i32,
) {
    if maxDepth < 0 as i32 {
        return;
    }
    Draw_Color(
        1.0f32,
        1.0f32,
        1.0f32,
        1.0f32,
    );
    Draw_Box3(&mut (*this).box_0);
    if !((*this).back).is_null() {
        KDTree_Draw((*this).back, maxDepth - 1 as i32);
    }
    if !((*this).front).is_null() {
        KDTree_Draw((*this).front, maxDepth - 1 as i32);
    }
}
