use std::rc::{Rc, Weak};

use crate::math::{Box3, Vec3};
use crate::physics::*;
use crate::render::*;
use rapier3d::prelude::nalgebra as na;
use rapier3d::prelude::{self as rp, ColliderBuilder};

pub type CollisionGroup = u32;
pub type CollisionMask = u32;

pub const CollisionGroup_Null: CollisionGroup = 0 << 0;
pub const CollisionGroup_Default: CollisionGroup = 1 << 0;

pub const CollisionMask_Null: CollisionMask = 0 << 0;
pub const CollisionMask_All: CollisionMask = !CollisionGroup_Null;

#[derive(Clone)]
pub enum CollisionShapeType {
    Box { half_extents: Vec3 },
    Sphere { radius: f32 },
    Hull { points: Vec<na::Point3<f32>> },
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
                half_extents.x * scale,
                half_extents.y * scale,
                half_extents.z * scale,
            ),
            CollisionShapeType::Sphere { radius } => ColliderBuilder::ball(radius * scale),
            CollisionShapeType::Hull { points } => {
                ColliderBuilder::convex_hull(points).expect("Convex hull computation failed")
            }
        };

        // Rapier does not support 0 mass, so we just set it to a negligible value here.
        let collider = builder.restitution(0.4).mass(0.0000001).build();
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

    pub fn new_hull_from_mesh(mesh: &Mesh) -> CollisionShape {
        Self::new(
            1.0,
            CollisionShapeType::Hull {
                points: mesh.vertex.iter().map(|v| v.p.to_na_point()).collect(),
            },
        )
    }
}
