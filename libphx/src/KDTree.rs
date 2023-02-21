use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
use glam::Vec2;

extern "C" {
    pub type Mesh;
    pub type Matrix;
    fn qsort(
        __base: *mut libc::c_void,
        __nel: libc::size_t,
        __width: libc::size_t,
        __compar: Option::<
            unsafe extern "C" fn(*const libc::c_void, *const libc::c_void) -> libc::c_int,
        >,
    );
    fn Draw_Box3(box_0: *const Box3f);
    fn Draw_Color(
        r: libc::c_float,
        g: libc::c_float,
        b: libc::c_float,
        a: libc::c_float,
    );
    fn Mesh_GetIndexCount(_: *mut Mesh) -> libc::c_int;
    fn Mesh_GetIndexData(_: *mut Mesh) -> *mut libc::c_int;
    fn Mesh_GetVertexData(_: *mut Mesh) -> *mut Vertex;
}
pub type uint64_t = libc::c_ulonglong;
pub type uint64 = uint64_t;
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
    pub id: uint64,
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
unsafe extern "C" fn Maxf(mut a: libc::c_float, mut b: libc::c_float) -> libc::c_float {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Minf(mut a: libc::c_float, mut b: libc::c_float) -> libc::c_float {
    return if a < b { a } else { b };
}
#[inline]
unsafe extern "C" fn Box3f_Create(mut lower: Vec3, mut upper: Vec3) -> Box3f {
    let mut result: Box3f = {
        let mut init = Box3f {
            lower: lower,
            upper: upper,
        };
        init
    };
    return result;
}
#[inline]
unsafe extern "C" fn Box3f_Union(mut a: Box3f, mut b: Box3f) -> Box3f {
    let mut this: Box3f = {
        let mut init = Box3f {
            lower: {
                let mut init = Vec3 {
                    x: Minf(a.lower.x, b.lower.x),
                    y: Minf(a.lower.y, b.lower.y),
                    z: Minf(a.lower.z, b.lower.z),
                };
                init
            },
            upper: {
                let mut init = Vec3 {
                    x: Maxf(a.upper.x, b.upper.x),
                    y: Maxf(a.upper.y, b.upper.y),
                    z: Maxf(a.upper.z, b.upper.z),
                };
                init
            },
        };
        init
    };
    return this;
}


#[no_mangle]
pub static mut kMaxLeafSize: libc::c_int = 64 as libc::c_int;
unsafe extern "C" fn compareLowerX(
    mut a: *const libc::c_void,
    mut b: *const libc::c_void,
) -> libc::c_int {
    return if (*(a as *const Box3f)).lower.x < (*(b as *const Box3f)).lower.x {
        -(1 as libc::c_int)
    } else {
        1 as libc::c_int
    };
}
unsafe extern "C" fn compareLowerY(
    mut a: *const libc::c_void,
    mut b: *const libc::c_void,
) -> libc::c_int {
    return if (*(a as *const Box3f)).lower.y < (*(b as *const Box3f)).lower.y {
        -(1 as libc::c_int)
    } else {
        1 as libc::c_int
    };
}
unsafe extern "C" fn compareLowerZ(
    mut a: *const libc::c_void,
    mut b: *const libc::c_void,
) -> libc::c_int {
    return if (*(a as *const Box3f)).lower.z < (*(b as *const Box3f)).lower.z {
        -(1 as libc::c_int)
    } else {
        1 as libc::c_int
    };
}
unsafe extern "C" fn Partition(
    mut boxes: *mut Box3f,
    mut boxCount: libc::c_int,
    mut dim: libc::c_int,
) -> *mut KDTree {
    let mut this: *mut KDTree = MemAlloc(
        ::core::mem::size_of::<KDTree>() as usize,
    ) as *mut KDTree;
    if boxCount <= kMaxLeafSize {
        (*this).box_0 = *boxes.offset(0);
        (*this).back = 0 as *mut KDTree;
        (*this).front = 0 as *mut KDTree;
        (*this).elems = 0 as *mut Node;
        let mut i: libc::c_int = 1 as libc::c_int;
        while i < boxCount {
            (*this).box_0 = Box3f_Union((*this).box_0, *boxes.offset(i as isize));
            i += 1;
        }
        let mut i_0: libc::c_int = 0 as libc::c_int;
        while i_0 < boxCount {
            let mut node: *mut Node = MemAlloc(
                ::core::mem::size_of::<Node>() as usize,
            ) as *mut Node;
            (*node).box_0 = *boxes.offset(i_0 as isize);
            (*node).next = (*this).elems;
            (*node).id = 0 as libc::c_int as uint64;
            (*this).elems = node;
            i_0 += 1;
        }
        return this;
    }
    if dim == 0 as libc::c_int {
        qsort(
            boxes as *mut libc::c_void,
            boxCount as libc::size_t,
            ::core::mem::size_of::<Box3f>() as usize,
            Some(
                compareLowerX
                    as unsafe extern "C" fn(
                        *const libc::c_void,
                        *const libc::c_void,
                    ) -> libc::c_int,
            ),
        );
    }
    if dim == 1 as libc::c_int {
        qsort(
            boxes as *mut libc::c_void,
            boxCount as libc::size_t,
            ::core::mem::size_of::<Box3f>() as usize,
            Some(
                compareLowerY
                    as unsafe extern "C" fn(
                        *const libc::c_void,
                        *const libc::c_void,
                    ) -> libc::c_int,
            ),
        );
    }
    if dim == 2 as libc::c_int {
        qsort(
            boxes as *mut libc::c_void,
            boxCount as libc::size_t,
            ::core::mem::size_of::<Box3f>() as usize,
            Some(
                compareLowerZ
                    as unsafe extern "C" fn(
                        *const libc::c_void,
                        *const libc::c_void,
                    ) -> libc::c_int,
            ),
        );
    }
    let mut boxCountBack: libc::c_int = boxCount / 2 as libc::c_int;
    let mut boxCountFront: libc::c_int = boxCount - boxCountBack;
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
        (dim + 1 as libc::c_int) % 3 as libc::c_int,
    );
    (*this)
        .front = Partition(
        boxesFront,
        boxCountFront,
        (dim + 1 as libc::c_int) % 3 as libc::c_int,
    );
    (*this).box_0 = Box3f_Union((*(*this).back).box_0, (*(*this).front).box_0);
    (*this).elems = 0 as *mut Node;
    MemFree(boxesBack as *const libc::c_void);
    MemFree(boxesFront as *const libc::c_void);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn KDTree_FromMesh(mut mesh: *mut Mesh) -> *mut KDTree {
    let indexCount: libc::c_int = Mesh_GetIndexCount(mesh);
    let mut indexData: *const libc::c_int = Mesh_GetIndexData(mesh);
    let mut vertexData: *const Vertex = Mesh_GetVertexData(mesh);
    let boxCount: libc::c_int = indexCount / 3 as libc::c_int;
    let mut boxes: *mut Box3f = MemAlloc(
        (::core::mem::size_of::<Box3f>())
            .wrapping_mul(boxCount as usize),
    ) as *mut Box3f;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < indexCount {
        let mut v0: *const Vertex = vertexData
            .offset(*indexData.offset((i + 0 as libc::c_int) as isize) as isize);
        let mut v1: *const Vertex = vertexData
            .offset(*indexData.offset((i + 1 as libc::c_int) as isize) as isize);
        let mut v2: *const Vertex = vertexData
            .offset(*indexData.offset((i + 2 as libc::c_int) as isize) as isize);
        *boxes
            .offset(
                (i / 3 as libc::c_int) as isize,
            ) = Box3f_Create(
            Vec3::min((*v0).p, Vec3::min((*v1).p, (*v2).p)),
            Vec3::max((*v0).p, Vec3::max((*v1).p, (*v2).p)),
        );
        i += 3 as libc::c_int;
    }
    let mut this: *mut KDTree = Partition(boxes, boxCount, 0 as libc::c_int);
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
pub unsafe extern "C" fn KDTree_GetMemory(mut this: *mut KDTree) -> libc::c_int {
    let mut memory: libc::c_int = ::core::mem::size_of::<KDTree>() as usize
        as libc::c_int;
    if !((*this).back).is_null() {
        memory += KDTree_GetMemory((*this).back);
    }
    if !((*this).front).is_null() {
        memory += KDTree_GetMemory((*this).front);
    }
    let mut elem: *mut Node = (*this).elems;
    while !elem.is_null() {
        memory = (memory as usize)
            .wrapping_add(::core::mem::size_of::<Node>()) as libc::c_int
            as libc::c_int;
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
    return 0 as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn KDTree_Draw(
    mut this: *mut KDTree,
    mut maxDepth: libc::c_int,
) {
    if maxDepth < 0 as libc::c_int {
        return;
    }
    Draw_Color(
        1 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
    );
    Draw_Box3(&mut (*this).box_0);
    if !((*this).back).is_null() {
        KDTree_Draw((*this).back, maxDepth - 1 as libc::c_int);
    }
    if !((*this).front).is_null() {
        KDTree_Draw((*this).front, maxDepth - 1 as libc::c_int);
    }
}
