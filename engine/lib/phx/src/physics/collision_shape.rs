use std::rc::{Rc, Weak};

use crate::math::{Box3, Vec3};
use crate::physics::*;
use crate::render::*;
use rapier3d_f64::prelude::nalgebra as na;
use rapier3d_f64::prelude::{self as rp, ColliderBuilder};

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
    Hull { points: Vec<na::Point3<rp::Real>> },
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
            CollisionShapeType::Hull { points } => {
                let scaled_points: Vec<na::Point3<rp::Real>> =
                    points.iter().map(|p| *p * (scale as rp::Real)).collect();
                ColliderBuilder::convex_hull(&scaled_points)
                    .expect("Convex hull computation failed")
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

    pub fn new_hull_from_mesh(mesh: &Mesh) -> CollisionShape {
        Self::new(
            1.0,
            CollisionShapeType::Hull {
                points: mesh.vertex.iter().map(|v| v.p.to_na_point()).collect(),
            },
        )
    }
}
