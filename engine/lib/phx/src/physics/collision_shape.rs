use crate::math::{Box3, Vec3};
use crate::physics::*;
use crate::render::*;
use rapier3d_f64::prelude::nalgebra as na;
use rapier3d_f64::prelude::{self as rp, ColliderBuilder};

pub type CollisionGroup = u32;
pub type CollisionMask = u32;

pub const COLLISION_GROUP_NULL: CollisionGroup = 0;
pub const COLLISION_GROUP_DEFAULT: CollisionGroup = 1;

pub const COLLISION_MASK_NULL: CollisionMask = 0;
pub const COLLISION_MASK_ALL: CollisionMask = !COLLISION_GROUP_NULL;

#[derive(Clone)]
pub enum CollisionShapeType {
    Box {
        half_extents: Vec3,
    },
    Sphere {
        radius: f32,
    },
    ConvexHull {
        points: Vec<na::Point3<rp::Real>>,
    },
    ConvexDecomposition {
        vertices: Vec<na::Point3<rp::Real>>,
        indices: Vec<[u32; 3]>,
    },
    Trimesh {
        vertices: Vec<na::Point3<rp::Real>>,
        indices: Vec<[u32; 3]>,
    },
}

pub struct CollisionShape {
    pub scale: f32,
    pub shape: CollisionShapeType,
    pub collider: rp::Collider,
}

impl CollisionShape {
    pub(crate) fn new(scale: f32, shape: CollisionShapeType) -> CollisionShape {
        let builder = match &shape {
            CollisionShapeType::Box { half_extents } => ColliderBuilder::cuboid(
                (half_extents.x * scale) as rp::Real,
                (half_extents.y * scale) as rp::Real,
                (half_extents.z * scale) as rp::Real,
            ),
            CollisionShapeType::Sphere { radius } => {
                ColliderBuilder::ball((radius * scale) as rp::Real)
            }
            CollisionShapeType::ConvexHull { points } => {
                let scaled_points: Vec<na::Point3<rp::Real>> =
                    points.iter().map(|p| *p * (scale as rp::Real)).collect();
                ColliderBuilder::convex_hull(&scaled_points)
                    .expect("Convex hull computation failed")
            }
            CollisionShapeType::ConvexDecomposition { vertices, indices } => {
                let scaled_vertices: Vec<na::Point3<rp::Real>> =
                    vertices.iter().map(|p| *p * (scale as rp::Real)).collect();
                ColliderBuilder::convex_decomposition(&scaled_vertices, indices)
            }
            CollisionShapeType::Trimesh { vertices, indices } => {
                let scaled_vertices: Vec<na::Point3<rp::Real>> =
                    vertices.iter().map(|p| *p * (scale as rp::Real)).collect();
                ColliderBuilder::trimesh(scaled_vertices, indices.clone())
            }
        };

        let collider = builder.restitution(0.4).mass(1.0).build();
        CollisionShape {
            scale,
            shape,
            collider,
        }
    }

    pub fn new_box(half_extents: &Vec3) -> CollisionShape {
        Self::new(
            1.0,
            CollisionShapeType::Box {
                half_extents: *half_extents,
            },
        )
    }

    pub fn new_box_from_mesh(mesh: &mut Mesh) -> CollisionShape {
        let mut bounds = Box3::default();
        Mesh_GetBound(mesh, &mut bounds);
        Self::new(
            1.0,
            CollisionShapeType::Box {
                half_extents: Vec3::new(
                    f32::max(f32::abs(bounds.upper.x), f32::abs(bounds.lower.x)),
                    f32::max(f32::abs(bounds.upper.y), f32::abs(bounds.lower.y)),
                    f32::max(f32::abs(bounds.upper.z), f32::abs(bounds.lower.z)),
                ),
            },
        )
    }

    pub fn new_sphere(radius: f32) -> CollisionShape {
        Self::new(1.0, CollisionShapeType::Sphere { radius })
    }

    pub fn new_sphere_from_mesh(mesh: &mut Mesh) -> CollisionShape {
        Self::new(
            1.0,
            CollisionShapeType::Sphere {
                radius: Mesh_GetRadius(mesh),
            },
        )
    }

    pub fn new_convex_hull_from_mesh(mesh: &Mesh) -> CollisionShape {
        Self::new(
            1.0,
            CollisionShapeType::ConvexHull {
                points: Self::convert_vertices(mesh),
            },
        )
    }

    pub fn new_convex_decomposition_from_mesh(mesh: &Mesh) -> CollisionShape {
        Self::new(
            1.0,
            CollisionShapeType::ConvexDecomposition {
                vertices: Self::convert_vertices(mesh),
                indices: Self::convert_indices(mesh),
            },
        )
    }

    pub fn new_trimesh_from_mesh(mesh: &Mesh) -> CollisionShape {
        Self::new(
            1.0,
            CollisionShapeType::Trimesh {
                vertices: Self::convert_vertices(mesh),
                indices: Self::convert_indices(mesh),
            },
        )
    }

    fn convert_vertices(mesh: &Mesh) -> Vec<na::Point3<rp::Real>> {
        mesh.vertex.iter().map(|v| v.p.to_na_point()).collect()
    }

    fn convert_indices(mesh: &Mesh) -> Vec<[u32; 3]> {
        let mesh_indices = &mesh.index[..mesh.index.len() - (mesh.index.len() % 3)];
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
