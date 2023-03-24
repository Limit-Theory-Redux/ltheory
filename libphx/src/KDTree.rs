use crate::internal::Memory::*;
use crate::Common::*;
use crate::Draw::*;
use crate::Math::Box3;
use crate::Math::Vec2;
use crate::Math::Vec3;
use crate::Matrix::*;
use crate::Mesh::*;
use libc;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct KDTree {
    pub box_0: Box3,
    pub back: *mut KDTree,
    pub front: *mut KDTree,
    pub elems: *mut Node,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Node {
    pub next: *mut Node,
    pub id: u64,
    pub box_0: Box3,
}

#[no_mangle]
pub static kMaxLeafSize: i32 = 64;

unsafe extern "C" fn compareLowerX(mut a: *const libc::c_void, mut b: *const libc::c_void) -> i32 {
    if (*(a as *const Box3)).lower.x < (*(b as *const Box3)).lower.x {
        -1
    } else {
        1
    }
}

unsafe extern "C" fn compareLowerY(mut a: *const libc::c_void, mut b: *const libc::c_void) -> i32 {
    if (*(a as *const Box3)).lower.y < (*(b as *const Box3)).lower.y {
        -1
    } else {
        1
    }
}

unsafe extern "C" fn compareLowerZ(mut a: *const libc::c_void, mut b: *const libc::c_void) -> i32 {
    if (*(a as *const Box3)).lower.z < (*(b as *const Box3)).lower.z {
        -1
    } else {
        1
    }
}

unsafe extern "C" fn Partition(
    mut boxes: *mut Box3,
    mut boxCount: i32,
    mut dim: i32,
) -> *mut KDTree {
    let mut this = MemNew!(KDTree);
    if boxCount <= kMaxLeafSize {
        (*this).box_0 = *boxes.offset(0);
        (*this).back = std::ptr::null_mut();
        (*this).front = std::ptr::null_mut();
        (*this).elems = std::ptr::null_mut();
        let mut i: i32 = 1;
        while i < boxCount {
            (*this).box_0 = Box3::union((*this).box_0, *boxes.offset(i as isize));
            i += 1;
        }
        let mut i_0: i32 = 0;
        while i_0 < boxCount {
            let mut node = MemNew!(Node);
            (*node).box_0 = *boxes.offset(i_0 as isize);
            (*node).next = (*this).elems;
            (*node).id = 0;
            (*this).elems = node;
            i_0 += 1;
        }
        return this;
    }
    if dim == 0 {
        libc::qsort(
            boxes as *mut _,
            boxCount as usize,
            std::mem::size_of::<Box3>(),
            Some(
                compareLowerX
                    as unsafe extern "C" fn(*const libc::c_void, *const libc::c_void) -> i32,
            ),
        );
    }
    if dim == 1 {
        libc::qsort(
            boxes as *mut _,
            boxCount as usize,
            std::mem::size_of::<Box3>(),
            Some(
                compareLowerY
                    as unsafe extern "C" fn(*const libc::c_void, *const libc::c_void) -> i32,
            ),
        );
    }
    if dim == 2 {
        libc::qsort(
            boxes as *mut _,
            boxCount as usize,
            std::mem::size_of::<Box3>(),
            Some(
                compareLowerZ
                    as unsafe extern "C" fn(*const libc::c_void, *const libc::c_void) -> i32,
            ),
        );
    }
    let mut boxCountBack: i32 = boxCount / 2;
    let mut boxCountFront: i32 = boxCount - boxCountBack;
    let mut boxesBack: *mut Box3 = MemNewArray!(Box3, boxCountBack);
    let mut boxesFront: *mut Box3 = MemNewArray!(Box3, boxCountFront);
    MemCpy(
        boxesBack as *mut _,
        boxes as *const _,
        (boxCountBack as usize).wrapping_mul(std::mem::size_of::<Box3>()),
    );
    MemCpy(
        boxesFront as *mut _,
        boxes.offset(boxCountBack as isize) as *const _,
        (boxCountFront as usize).wrapping_mul(std::mem::size_of::<Box3>()),
    );
    (*this).back = Partition(boxesBack, boxCountBack, (dim + 1) % 3);
    (*this).front = Partition(boxesFront, boxCountFront, (dim + 1) % 3);
    (*this).box_0 = Box3::union((*(*this).back).box_0, (*(*this).front).box_0);
    (*this).elems = std::ptr::null_mut();
    MemFree(boxesBack as *const _);
    MemFree(boxesFront as *const _);
    this
}

#[no_mangle]
pub unsafe extern "C" fn KDTree_FromMesh(mut mesh: *mut Mesh) -> *mut KDTree {
    let indexCount: i32 = Mesh_GetIndexCount(mesh);
    let mut indexData: *const i32 = Mesh_GetIndexData(mesh);
    let mut vertexData: *const Vertex = Mesh_GetVertexData(mesh);
    let boxCount: i32 = indexCount / 3;
    let mut boxes: *mut Box3 = MemNewArray!(Box3, boxCount);
    let mut i: i32 = 0;
    while i < indexCount {
        let mut v0: *const Vertex = vertexData.offset(*indexData.offset((i + 0) as isize) as isize);
        let mut v1: *const Vertex = vertexData.offset(*indexData.offset((i + 1) as isize) as isize);
        let mut v2: *const Vertex = vertexData.offset(*indexData.offset((i + 2) as isize) as isize);
        *boxes.offset((i / 3) as isize) = Box3::new(
            Vec3::min((*v0).p, Vec3::min((*v1).p, (*v2).p)),
            Vec3::max((*v0).p, Vec3::max((*v1).p, (*v2).p)),
        );
        i += 3;
    }
    let this: *mut KDTree = Partition(boxes, boxCount, 0);
    MemFree(boxes as *const _);
    this
}

#[no_mangle]
pub unsafe extern "C" fn KDTree_Free(this: *mut KDTree) {
    if !((*this).back).is_null() {
        KDTree_Free((*this).back);
    }
    if !((*this).front).is_null() {
        KDTree_Free((*this).front);
    }
    let mut elem: *mut Node = (*this).elems;
    while !elem.is_null() {
        let mut next: *mut Node = (*elem).next;
        MemFree(elem as *const _);
        elem = next;
    }
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn KDTree_GetMemory(this: *mut KDTree) -> i32 {
    let mut memory: i32 = std::mem::size_of::<KDTree>() as i32;
    if !((*this).back).is_null() {
        memory += KDTree_GetMemory((*this).back);
    }
    if !((*this).front).is_null() {
        memory += KDTree_GetMemory((*this).front);
    }
    let mut elem: *mut Node = (*this).elems;
    while !elem.is_null() {
        memory = (memory as usize).wrapping_add(std::mem::size_of::<Node>()) as i32;
        elem = (*elem).next;
    }
    memory
}

#[no_mangle]
pub unsafe extern "C" fn KDTree_IntersectRay(
    mut _this: *mut KDTree,
    mut _m: *mut Matrix,
    mut _a: *const Vec3,
    mut _b: *const Vec3,
) -> bool {
    false
}

#[no_mangle]
pub unsafe extern "C" fn KDTree_Draw(this: *mut KDTree, mut maxDepth: i32) {
    if maxDepth < 0 {
        return;
    }
    Draw_Color(1.0f32, 1.0f32, 1.0f32, 1.0f32);
    Draw_Box3(&mut (*this).box_0);
    if !((*this).back).is_null() {
        KDTree_Draw((*this).back, maxDepth - 1);
    }
    if !((*this).front).is_null() {
        KDTree_Draw((*this).front, maxDepth - 1);
    }
}
