use internal::*;

use crate::math::*;
use crate::render::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Octree {
    pub box_0: Box3,
    pub child: [*mut Octree; 8],
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
pub unsafe extern "C" fn Octree_Create(box_0: Box3) -> *mut Octree {
    let this = MemNew!(Octree);
    MemZero(this as *mut _, std::mem::size_of::<Octree>());
    (*this).box_0 = box_0;
    this
}

#[no_mangle]
pub unsafe extern "C" fn Octree_Free(this: *mut Octree) {
    let mut i: i32 = 0;
    while i < 8 {
        if !((*this).child[i as usize]).is_null() {
            Octree_Free((*this).child[i as usize]);
        }
        i += 1;
    }
    let mut elem: *mut Node = (*this).elems;
    while !elem.is_null() {
        let next: *mut Node = (*elem).next;
        MemFree(elem as *const _);
        elem = next;
    }
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn Octree_FromMesh(mesh: &mut Mesh) -> *mut Octree {
    let mut mesh_box: Box3 = Box3 {
        lower: Vec3::ZERO,
        upper: Vec3::ZERO,
    };
    Mesh_GetBound(mesh, &mut mesh_box);
    let this: *mut Octree = Octree_Create(mesh_box);

    let index_data = mesh.get_index_data();
    let vertex_data = mesh.get_vertex_data();

    for i in (0..index_data.len()).step_by(3) {
        let v0 = &vertex_data[index_data[i] as usize];
        let v1 = &vertex_data[index_data[i + 1] as usize];
        let v2 = &vertex_data[index_data[i + 2] as usize];
        let box_0: Box3 = Box3::new(
            Vec3::min(v0.p, Vec3::min(v1.p, v2.p)),
            Vec3::max(v0.p, Vec3::max(v1.p, v2.p)),
        );

        Octree_Add(&mut *this, box_0, (i / 3) as u32);
    }
    this
}

#[allow(non_snake_case)] // TODO: remove this and fix all warnings
unsafe extern "C" fn Octree_GetAvgLoadImpl(this: &mut Octree, load: *mut f64, nodes: *mut f64) {
    *nodes += 1.0;
    let mut elem: *mut Node = this.elems;
    while !elem.is_null() {
        *load += 1.0;
        elem = (*elem).next;
    }
    let mut i: i32 = 0;
    while i < 8 {
        if !(this.child[i as usize]).is_null() {
            Octree_GetAvgLoadImpl(&mut *this.child[i as usize], load, nodes);
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Octree_GetAvgLoad(this: &mut Octree) -> f64 {
    let mut load: f64 = 0.0;
    let mut nodes: f64 = 0.0;
    Octree_GetAvgLoadImpl(this, &mut load, &mut nodes);
    load / nodes
}

#[no_mangle]
pub unsafe extern "C" fn Octree_GetMaxLoad(this: &mut Octree) -> i32 {
    let mut load: i32 = 0;
    let mut elem: *mut Node = this.elems;
    while !elem.is_null() {
        load += 1;
        elem = (*elem).next;
    }
    let mut i: i32 = 0;
    while i < 8 {
        if !(this.child[i as usize]).is_null() {
            load = f64::max(
                load as f64,
                Octree_GetMaxLoad(&mut *this.child[i as usize]) as f64,
            ) as i32;
        }
        i += 1;
    }
    load
}

#[no_mangle]
pub unsafe extern "C" fn Octree_GetMemory(this: &mut Octree) -> i32 {
    let mut memory: i32 = std::mem::size_of::<Octree>() as i32;
    let mut i: i32 = 0;
    while i < 8 {
        if !(this.child[i as usize]).is_null() {
            memory += Octree_GetMemory(&mut *this.child[i as usize]);
        }
        i += 1;
    }
    let mut elem: *mut Node = this.elems;
    while !elem.is_null() {
        memory = (memory as usize).wrapping_add(std::mem::size_of::<Node>()) as i32;
        elem = (*elem).next;
    }
    memory
}

#[allow(non_snake_case)] // TODO: remove this and fix all warnings
unsafe extern "C" fn Octree_IntersectRayImpl(this: &mut Octree, o: Vec3, di: Vec3) -> bool {
    if !this.box_0.intersects_ray(o, di) {
        return false;
    }
    let mut elem: *mut Node = this.elems;
    while !elem.is_null() {
        if (*elem).box_0.intersects_ray(o, di) {
            return true;
        }
        elem = (*elem).next;
    }
    let mut i: i32 = 0;
    while i < 8 {
        if !(this.child[i as usize]).is_null()
            && Octree_IntersectRayImpl(&mut *this.child[i as usize], o, di)
        {
            return true;
        }
        i += 1;
    }
    false
}

#[no_mangle]
pub unsafe extern "C" fn Octree_IntersectRay(
    this: &mut Octree,
    matrix: &mut Matrix,
    ro: &Vec3,
    rd: &Vec3,
) -> bool {
    let inv = Matrix_Inverse(matrix);
    let mut inv_ro = Vec3::ZERO;
    Matrix_MulPoint(inv.as_ref(), &mut inv_ro, ro.x, ro.y, ro.z);
    let mut inv_rd = Vec3::ZERO;
    Matrix_MulDir(inv.as_ref(), &mut inv_rd, rd.x, rd.y, rd.z);
    Octree_IntersectRayImpl(this, inv_ro, inv_rd.recip())
}

#[allow(non_snake_case)] // TODO: remove this and fix all warnings
unsafe extern "C" fn Octree_Insert(this: &mut Octree, box_0: Box3, id: u32) {
    let elem = MemNew!(Node);
    (*elem).box_0 = box_0;
    (*elem).id = id as u64;
    (*elem).next = this.elems;
    this.elems = elem;
}

#[allow(non_snake_case)] // TODO: remove this and fix all warnings
unsafe extern "C" fn Octree_AddDepth(this: &mut Octree, box_0: Box3, id: u32) {
    let l: *const Vec3 = &mut this.box_0.lower;
    let u: *const Vec3 = &mut this.box_0.upper;
    let c: Vec3 = this.box_0.center();
    let child_bound: [Box3; 8] = [
        Box3 {
            lower: Vec3 {
                x: (*l).x,
                y: (*l).y,
                z: (*l).z,
            },
            upper: Vec3 {
                x: c.x,
                y: c.y,
                z: c.z,
            },
        },
        Box3 {
            lower: Vec3 {
                x: c.x,
                y: (*l).y,
                z: (*l).z,
            },
            upper: Vec3 {
                x: (*u).x,
                y: c.y,
                z: c.z,
            },
        },
        Box3 {
            lower: Vec3 {
                x: (*l).x,
                y: c.y,
                z: (*l).z,
            },
            upper: Vec3 {
                x: c.x,
                y: (*u).y,
                z: c.z,
            },
        },
        Box3 {
            lower: Vec3 {
                x: c.x,
                y: c.y,
                z: (*l).z,
            },
            upper: Vec3 {
                x: (*u).x,
                y: (*u).y,
                z: c.z,
            },
        },
        Box3 {
            lower: Vec3 {
                x: (*l).x,
                y: (*l).y,
                z: c.z,
            },
            upper: Vec3 {
                x: c.x,
                y: c.y,
                z: (*u).z,
            },
        },
        Box3 {
            lower: Vec3 {
                x: c.x,
                y: (*l).y,
                z: c.z,
            },
            upper: Vec3 {
                x: (*u).x,
                y: c.y,
                z: (*u).z,
            },
        },
        Box3 {
            lower: Vec3 {
                x: (*l).x,
                y: c.y,
                z: c.z,
            },
            upper: Vec3 {
                x: c.x,
                y: (*u).y,
                z: (*u).z,
            },
        },
        Box3 {
            lower: Vec3 {
                x: c.x,
                y: c.y,
                z: c.z,
            },
            upper: Vec3 {
                x: (*u).x,
                y: (*u).y,
                z: (*u).z,
            },
        },
    ];
    let mut intersections: i32 = 0;
    let mut last_intersection: i32 = 0;
    let mut i: i32 = 0;
    while i < 8 {
        if Box3::intersects_box(box_0, child_bound[i as usize]) {
            intersections += 1;
            last_intersection = i;
        }
        i += 1;
    }
    if intersections == 0 {
        return;
    }
    if intersections == 1 {
        if (this.child[last_intersection as usize]).is_null() {
            this.child[last_intersection as usize] =
                Octree_Create(child_bound[last_intersection as usize]);
        }
        Octree_AddDepth(
            &mut *this.child[last_intersection as usize],
            Box3::intersection(box_0, child_bound[last_intersection as usize]),
            id,
        );
        return;
    }
    Octree_Insert(this, box_0, id);
}

#[no_mangle]
pub unsafe extern "C" fn Octree_Add(this: &mut Octree, box_0: Box3, id: u32) {
    Octree_AddDepth(this, box_0, id);
}

#[no_mangle]
pub unsafe extern "C" fn Octree_Draw(this: &mut Octree) {
    Draw_Color(1.0f32, 1.0f32, 1.0f32, 1.0f32);
    Draw_Box3(&this.box_0);
    Draw_Color(0.0f32, 1.0f32, 0.0f32, 1.0f32);
    let mut elem: *mut Node = this.elems;
    while !elem.is_null() {
        Draw_Box3(&(*elem).box_0);
        elem = (*elem).next;
    }
    let mut i: i32 = 0;
    while i < 8 {
        if !(this.child[i as usize]).is_null() {
            Octree_Draw(&mut *this.child[i as usize]);
        }
        i += 1;
    }
}
