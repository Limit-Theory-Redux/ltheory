use std::rc::{Rc, Weak};
use std::collections::HashMap;

use crate::math::{Box3, Vec3};
use crate::physics::*;
use crate::render::*;
use rapier3d_f64::na::vector;
use rapier3d_f64::prelude::nalgebra as na;
use rapier3d_f64::prelude as rp;

pub type CollisionGroup = u32;
pub type CollisionMask = u32;

pub const CollisionGroup_Null: CollisionGroup = 0 << 0;
pub const CollisionGroup_Default: CollisionGroup = 1 << 0;

pub const CollisionMask_Null: CollisionMask = 0 << 0;
pub const CollisionMask_All: CollisionMask = !CollisionGroup_Null;

// TODO: Replace all of these with a SharedShape inside?
pub enum CollisionShapeType {
    Box {
        half_extents: Vec3,
    },
    Sphere {
        radius: f32,
    },
    ConvexHull {
        // ref-counted, TODO: Replace this with Rf<MeshData> when Mesh is rewritten to use Rf.
        mesh: *mut Mesh,
    },
    ConvexDecomposition {
        // ref-counted,  TODO: Replace this with Rf<MeshData> when Mesh is rewritten to use Rf.
        mesh: *mut Mesh,
    },
    Trimesh {
        // ref-counted,  TODO: Replace this with Rf<MeshData> when Mesh is rewritten to use Rf.
        mesh: *mut Mesh,
    },
}

// pub struct CollisionShape {
//     pub scale: f32,
//     pub shape_type: CollisionShapeType,
//     pub shape: rp::SharedShape,
// }

// impl CollisionShape {
//     // TODO: Replace these with separate functions.
//     pub(crate) fn new(scale: f32, shape_type: CollisionShapeType) -> CollisionShape {
//         let shape = match &shape_type {
//             CollisionShapeType::Box { half_extents } => rp::SharedShape::cuboid(
//                 (half_extents.x * scale) as rp::Real,
//                 (half_extents.y * scale) as rp::Real,
//                 (half_extents.z * scale) as rp::Real,
//             ),
//             CollisionShapeType::Sphere { radius } => {
//                 rp::SharedShape::ball((radius * scale) as rp::Real)
//             }
//             CollisionShapeType::ConvexHull { mesh } => {
//                 let mesh = unsafe { &**mesh };
//                 let vertices = Self::convert_vertices(scale, mesh);
//                 rp::SharedShape::convex_hull(&vertices)
//                     .expect("Convex hull computation failed")
//             }
//             CollisionShapeType::ConvexDecomposition { mesh } => {
//                 let mesh = unsafe { &**mesh };
//                 let vertices = Self::convert_vertices(scale, mesh);
//                 let indices = Self::convert_indices(mesh);
//                 rp::SharedShape::convex_decomposition(&vertices, &indices)
//             }
//             CollisionShapeType::Trimesh { mesh } => {
//                 let mesh = unsafe { &**mesh };
//                 let vertices = Self::convert_vertices(scale, mesh);
//                 let indices = Self::convert_indices(mesh);
//                 rp::SharedShape::trimesh(vertices, indices)
//             }
//         };

//         CollisionShape {
//             scale,
//             shape_type,
//             shape,
//         }
//     }

//     // Use https://github.com/dimforge/bevy_rapier/blob/master/src/geometry/collider_impl.rs#L524 as reference
//     pub fn make_scaled(mut self, scale: f32) -> Self {
//         let ratio = scale / self.scale;
//         let scale_factor = rp::Vector::new(ratio as rp::Real, ratio as rp::Real, ratio as rp::Real);

//         let new_shape = self.shape.make_mut();
//         let shape = match &self.shape_type {
//             CollisionShapeType::Box { .. } => rp::SharedShape::new(self.shape.as_cuboid_mut().unwrap().scaled(&scale_factor)),
//             CollisionShapeType::Sphere { .. } => rp::SharedShape::new(self.shape.as_ball_mut().unwrap().scaled(&scale_factor, 10).unwrap().unwrap_left()),
//             CollisionShapeType::ConvexHull { .. } => rp::SharedShape::new(self.shape.as_convex_polyhedron_mut().unwrap().scaled(&scale_factor).unwrap()),
//             CollisionShapeType::ConvexDecomposition { .. } => {
//                 let c = self.shape.as_compound_mut().unwrap();
//                 let mut scaled = Vec::with_capacity(c.shapes().len());

//                 // TODO: We know that these are all going to be ConvexMesh.
//                 for (tra, rot, shape) in c.shapes() {
//                     scaled.push((
//                         (tra * scale, rot).into(),
//                         shape.raw_scale_by(scale, num_subdivisions)?,
//                     ));
//                 }
//                 rp::SharedShape::compound(scaled)
//             },
//             CollisionShapeType::Trimesh { .. } => rp::SharedShape::new(self.shape.as_trimesh_mut().unwrap().scaled(&scale_factor)),
//         };

//         CollisionShape {
//             scale,
//             shape_type: self.shape_type,
//             shape,
//         }
//     }

//     fn convert_vertices(scale: f32, mesh: &Mesh) -> Vec<na::Point3<rp::Real>> {
//         mesh.vertex.iter().map(|v| (v.p * scale).to_na_point()).collect()
//     }

//     fn convert_indices(mesh: &Mesh) -> Vec<[u32; 3]> {
//         let mesh_indices = &mesh.index[..mesh.index.len() - (mesh.index.len() % 3)];
//         let mut indices: Vec<[u32; 3]> = Vec::new();
//         indices.reserve(mesh_indices.len() / 3);
//         for i in 0..mesh_indices.len() / 3 {
//             indices.push([
//                 mesh_indices[i * 3] as u32,
//                 mesh_indices[i * 3 + 1] as u32,
//                 mesh_indices[i * 3 + 2] as u32,
//             ])
//         }
//         indices
//     }
// }

impl Drop for CollisionShapeType {
    fn drop(&mut self) {
        // TODO: While Mesh is still not using luajit-ffi-gen, we need to free it explicitly to decrement the refcount.
        match self {
            CollisionShapeType::ConvexHull { mesh } => Mesh_Free(*mesh),
            CollisionShapeType::ConvexDecomposition { mesh } => Mesh_Free(*mesh),
            CollisionShapeType::Trimesh { mesh } => Mesh_Free(*mesh),
            _ => {}
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
struct ShapeCacheKey {
    mesh: *const Mesh,
    scale: u32,
}

impl ShapeCacheKey {
    fn new(mesh: &Mesh, scale: f32) -> ShapeCacheKey {
        ShapeCacheKey {
            mesh: mesh as *const Mesh,
            scale: scale.to_bits(),
        }
    }
}

pub struct ShapeCache {
    convex_hull_unscaled_cache: HashMap<*const Mesh, rp::SharedShape>,
    trimesh_unscaled_cache: HashMap<*const Mesh, rp::SharedShape>,

    convex_hull_cache: HashMap<ShapeCacheKey, rp::SharedShape>,
    convex_decomposition_cache: HashMap<ShapeCacheKey, rp::SharedShape>,
    trimesh_cache: HashMap<ShapeCacheKey, rp::SharedShape>,
}

impl ShapeCache {
    pub fn new() -> ShapeCache {
        ShapeCache {
            convex_hull_unscaled_cache: HashMap::new(),
            trimesh_unscaled_cache: HashMap::new(),
            convex_hull_cache: HashMap::new(),
            convex_decomposition_cache: HashMap::new(),
            trimesh_cache: HashMap::new(),
        }
    }

    pub fn get(&mut self, scale: f32, shape_type: &CollisionShapeType) -> rp::SharedShape {
        match shape_type {
            CollisionShapeType::Box { half_extents } => rp::SharedShape::cuboid(
                (half_extents.x * scale) as rp::Real,
                (half_extents.y * scale) as rp::Real,
                (half_extents.z * scale) as rp::Real,
            ),
            CollisionShapeType::Sphere { radius } => {
                rp::SharedShape::ball((radius * scale) as rp::Real)
            }
            CollisionShapeType::ConvexHull { mesh } => {
                let mesh = unsafe { &**mesh };
                self.convex_hull_cache.entry(ShapeCacheKey::new(mesh, scale)).or_insert_with(|| {
                    // Get the unscaled shape (and generate it if needed).
                    let unscaled_shape = self.convex_hull_unscaled_cache.entry(mesh as *const _).or_insert_with(|| {
                        let vertices = Self::convert_vertices(1.0, mesh);
                        rp::SharedShape::convex_hull(&vertices).expect("Convex hull computation failed")
                    });

                    // Scale the unscaled shape.
                    if scale != 1.0 {
                        let scale_factor = rp::Vector::new(scale as rp::Real, scale as rp::Real, scale as rp::Real);
                        rp::SharedShape::new(unscaled_shape.as_convex_polyhedron().unwrap().clone().scaled(&scale_factor).unwrap())
                    } else {
                        unscaled_shape.clone()
                    }
                }).clone()
            }
            CollisionShapeType::ConvexDecomposition { mesh } => {
                let mesh = unsafe { &**mesh };
                self.convex_decomposition_cache.entry(ShapeCacheKey::new(mesh, scale)).or_insert_with(|| {
                    let vertices = Self::convert_vertices(scale, mesh);
                    let indices = Self::convert_indices(mesh);
                    rp::SharedShape::convex_decomposition(&vertices, &indices)
                }).clone()
            }
            CollisionShapeType::Trimesh { mesh } => {
                let mesh = unsafe { &**mesh };
                self.trimesh_cache.entry(ShapeCacheKey::new(mesh, scale)).or_insert_with(|| {
                    // Get the unscaled shape (and generate it if needed).
                    let unscaled_shape = self.trimesh_unscaled_cache.entry(mesh as *const _).or_insert_with(|| {
                        let vertices = Self::convert_vertices(1.0, mesh);
                        let indices = Self::convert_indices(mesh);
                        rp::SharedShape::trimesh(vertices, indices)
                    });

                    // Scale the unscaled shape.
                    if scale != 1.0 {
                        let scale_factor = rp::Vector::new(scale as rp::Real, scale as rp::Real, scale as rp::Real);
                        rp::SharedShape::new(unscaled_shape.as_trimesh().unwrap().clone().scaled(&scale_factor))
                    } else {
                        unscaled_shape.clone()
                    }
                }).clone()
            }
        }
    }

    pub fn evict(&mut self, scale: f32, mesh: &Mesh) {
        let key = ShapeCacheKey::new(mesh, scale);
        self.convex_hull_cache.remove(&key);
        self.convex_decomposition_cache.remove(&key);
        self.trimesh_cache.remove(&key);
    }

    pub fn evict_all(&mut self, mesh: &Mesh) {
        self.convex_hull_unscaled_cache.remove(&(mesh as *const _));
        self.trimesh_unscaled_cache.remove(&(mesh as *const _));
        // self.convex_hull_cache.iter_mut().filter(predicate)
    }

    fn convert_vertices(mesh: &Mesh) -> Vec<na::Point3<rp::Real>> {
        mesh.get_vertex_data()
            .iter()
            .map(|v| v.p.to_na_point())
            .collect()
    }

    fn convert_indices(mesh: &Mesh) -> Vec<[u32; 3]> {
        let mesh_indices = mesh.get_index_data();
        let mesh_indices = &mesh_indices[..mesh_indices.len() - (mesh_indices.len() % 3)];
        let mut indices: Vec<[u32; 3]> = Vec::with_capacity(mesh_indices.len() / 3);

        for i in 0..mesh_indices.len() / 3 {
            indices.push([
                mesh_indices[i * 3] as u32,
                mesh_indices[i * 3 + 1] as u32,
                mesh_indices[i * 3 + 2] as u32,
            ])
        }
        indices
    }
}
