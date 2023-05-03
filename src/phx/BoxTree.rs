use crate::phx::internal::Memory::*;
use crate::phx::Common::*;
use crate::phx::Draw::*;
use crate::phx::Math::Box3;
use crate::phx::Math::Vec2;
use crate::phx::Math::Vec3;
use crate::phx::Matrix::*;
use crate::phx::Mesh::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct BoxTree {
    pub root: *mut Node,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Node {
    pub box3: Box3,
    pub data: *mut libc::c_void,
    pub sub: [*mut Node; 2],
}

#[inline]
unsafe extern "C" fn Node_Create(box3: Box3, data: *mut libc::c_void) -> *mut Node {
    let this = MemNew!(Node);
    (*this).box3 = box3;
    (*this).sub[0] = std::ptr::null_mut();
    (*this).sub[1] = std::ptr::null_mut();
    (*this).data = data;
    this
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_Create() -> *mut BoxTree {
    let this = MemNew!(BoxTree);
    (*this).root = std::ptr::null_mut();
    this
}

unsafe extern "C" fn Node_Free(this: *mut Node) {
    if !((*this).sub[0]).is_null() {
        Node_Free((*this).sub[0]);
    }
    if !((*this).sub[1]).is_null() {
        Node_Free((*this).sub[1]);
    }
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_Free(this: *mut BoxTree) {
    if !((*this).root).is_null() {
        Node_Free((*this).root);
    }
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_FromMesh(mesh: &mut Mesh) -> *mut BoxTree {
    let this: *mut BoxTree = BoxTree_Create();
    let indexCount: i32 = Mesh_GetIndexCount(mesh);
    let indexData: *const i32 = Mesh_GetIndexData(mesh);
    let vertexData: *const Vertex = Mesh_GetVertexData(mesh);

    for i in (0..indexCount).step_by(3) {
        let v0: *const Vertex = vertexData.offset(*indexData.offset((i + 0) as isize) as isize);
        let v1: *const Vertex = vertexData.offset(*indexData.offset((i + 1) as isize) as isize);
        let v2: *const Vertex = vertexData.offset(*indexData.offset((i + 2) as isize) as isize);
        let box3: Box3 = Box3::new(
            Vec3::min((*v0).p, Vec3::min((*v1).p, (*v2).p)),
            Vec3::max((*v0).p, Vec3::max((*v1).p, (*v2).p)),
        );
        BoxTree_Add(&mut *this, box3, std::ptr::null_mut());
    }
    this
}

#[inline]
extern "C" fn Cost(box3: Box3) -> f32 {
    box3.volume()
}

#[inline]
extern "C" fn CostMerge(a: Box3, b: Box3) -> f32 {
    Cost(Box3::union(a, b))
}

unsafe extern "C" fn Node_Merge(this: Option<&mut Node>, src: *mut Node, prev: *mut *mut Node) {
    if this.is_none() {
        *prev = src;
        return;
    }
    let this = this.unwrap();

    /* Leaf node. */
    if (this.sub[0]).is_null() {
        let parent: *mut Node =
            Node_Create(Box3::union(this.box3, (*src).box3), std::ptr::null_mut());
        *prev = parent;
        (*parent).sub[0] = this;
        (*parent).sub[1] = src;
        // this = parent;
        return;
    }

    /* Contained by current sub-tree. */
    if Box3::contains(this.box3, (*src).box3) {
        let cost0: f32 = CostMerge((*this.sub[0]).box3, (*src).box3) + Cost((*this.sub[1]).box3);
        let cost1: f32 = CostMerge((*this.sub[1]).box3, (*src).box3) + Cost((*this.sub[0]).box3);
        if cost0 < cost1 {
            Node_Merge(
                this.sub[0].as_mut(),
                src,
                &mut *(this.sub).as_mut_ptr().offset(0),
            );
        } else {
            Node_Merge(
                this.sub[1].as_mut(),
                src,
                &mut *(this.sub).as_mut_ptr().offset(1),
            );
        }
    } else {
        /* Not contained, need new parent. */
        let parent: *mut Node =
            Node_Create(Box3::union(this.box3, (*src).box3), std::ptr::null_mut());
        *prev = parent;

        let costBase: f32 = Cost(this.box3) + Cost((*src).box3);
        let cost0: f32 = CostMerge((*this.sub[0]).box3, (*src).box3) + Cost((*this.sub[1]).box3);
        let cost1: f32 = CostMerge((*this.sub[1]).box3, (*src).box3) + Cost((*this.sub[0]).box3);

        if costBase <= cost0 && costBase <= cost1 {
            (*parent).sub[0] = this;
            (*parent).sub[1] = src;
        } else if cost0 <= costBase && cost0 <= cost1 {
            (*parent).sub[0] = this.sub[0];
            (*parent).sub[1] = this.sub[1];
            MemFree(this as *mut _ as *const _);
            Node_Merge(
                (*parent).sub[0].as_mut(),
                src,
                &mut *((*parent).sub).as_mut_ptr().offset(0),
            );
        } else {
            (*parent).sub[0] = this.sub[0];
            (*parent).sub[1] = this.sub[1];
            MemFree(this as *mut _ as *const _);
            Node_Merge(
                (*parent).sub[1].as_mut(),
                src,
                &mut *((*parent).sub).as_mut_ptr().offset(1),
            );
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_Add(this: &mut BoxTree, box3: Box3, data: *mut libc::c_void) {
    Node_Merge(this.root.as_mut(), Node_Create(box3, data), &mut this.root);
}

unsafe extern "C" fn Node_GetMemory(this: &mut Node) -> i32 {
    let mut memory: i32 = std::mem::size_of::<Node>() as i32;
    if !(this.sub[0]).is_null() {
        memory += Node_GetMemory(&mut *this.sub[0]);
    }
    if !(this.sub[1]).is_null() {
        memory += Node_GetMemory(&mut *this.sub[1]);
    }
    memory
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_GetMemory(this: &mut BoxTree) -> i32 {
    let mut memory: i32 = std::mem::size_of::<BoxTree>() as i32;
    if !(this.root).is_null() {
        memory += Node_GetMemory(&mut *this.root);
    }
    memory
}

unsafe extern "C" fn Node_IntersectRay(this: &mut Node, o: Vec3, di: Vec3) -> bool {
    if !this.box3.intersects_ray(o, di) {
        return false;
    }

    if !(this.sub[0]).is_null() {
        if Node_IntersectRay(&mut *this.sub[0], o, di) {
            return true;
        }
        if Node_IntersectRay(&mut *this.sub[1], o, di) {
            return true;
        }
        false
    } else {
        true
    }
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_IntersectRay(
    this: &mut BoxTree,
    matrix: &mut Matrix,
    ro: &Vec3,
    rd: &Vec3,
) -> bool {
    if (this.root).is_null() {
        return false;
    }
    let inv = Matrix_Inverse(matrix);
    let mut invRo = Vec3::ZERO;
    Matrix_MulPoint(inv.as_ref(), &mut invRo, ro.x, ro.y, ro.z);
    let mut invRd = Vec3::ZERO;
    Matrix_MulDir(inv.as_ref(), &mut invRd, rd.x, rd.y, rd.z);
    Node_IntersectRay(&mut *this.root, invRo, invRd.recip())
}

unsafe extern "C" fn BoxTree_DrawNode(this: &mut Node, maxDepth: i32) {
    if maxDepth < 0 {
        return;
    }
    if !(this.sub[0]).is_null() || !(this.sub[1]).is_null() {
        Draw_Color(1.0f32, 1.0f32, 1.0f32, 1.0f32);
        Draw_Box3(&mut this.box3);
    } else {
        Draw_Color(0.0f32, 1.0f32, 0.0f32, 1.0f32);
        Draw_Box3(&mut this.box3);
    }
    if !(this.sub[0]).is_null() {
        BoxTree_DrawNode(&mut *this.sub[0], maxDepth - 1);
    }
    if !(this.sub[1]).is_null() {
        BoxTree_DrawNode(&mut *this.sub[1], maxDepth - 1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn BoxTree_Draw(this: &mut BoxTree, maxDepth: i32) {
    if !(this.root).is_null() {
        BoxTree_DrawNode(&mut *this.root, maxDepth);
    }
}
