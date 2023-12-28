use std::rc::{Rc, Weak};

use crate::math::{Box3, Vec3};
use crate::render::*;
use rapier3d::prelude::nalgebra as na;
use rapier3d::prelude::{self as rp, ColliderBuilder};

pub type CollisionGroup = i32;
pub type CollisionMask = i32;

pub const CollisionGroup_Null: CollisionGroup = 0 << 0;
pub const CollisionGroup_Default: CollisionGroup = 1 << 0;
pub const CollisionGroup_Trigger: CollisionGroup = 1 << 1;

pub const CollisionMask_Null: CollisionMask = 0 << 0;
pub const CollisionMask_All: CollisionMask = !CollisionGroup_Null;
pub const CollisionMask_NoTriggers: CollisionMask = !CollisionGroup_Trigger;

pub enum CollisionShapeType {
    Box { halfExtents: Vec3 },
    Sphere { radius: f32 },
    Hull { mesh: Box<Mesh> },
}

impl CollisionShapeType {
    pub fn clone(&mut self) -> CollisionShapeType {
        match self {
            CollisionShapeType::Box { halfExtents } => CollisionShapeType::Box {
                halfExtents: *halfExtents,
            },
            CollisionShapeType::Sphere { radius } => CollisionShapeType::Sphere { radius: *radius },
            CollisionShapeType::Hull { mesh } => CollisionShapeType::Hull {
                mesh: unsafe { Mesh_Clone(&mut **mesh) },
            },
        }
    }
}

pub struct CollisionShape {
    pub scale: f32,
    pub shape: CollisionShapeType,
    pub collider: rp::Collider,
}

impl CollisionShape {
    pub(crate) fn new(scale: f32, shape: CollisionShapeType) -> CollisionShape {
        let builder = match shape {
            CollisionShapeType::Box { halfExtents } => ColliderBuilder::cuboid(
                halfExtents.x * scale,
                halfExtents.y * scale,
                halfExtents.z * scale,
            ),
            CollisionShapeType::Sphere { radius } => ColliderBuilder::ball(radius * scale),
            _ => ColliderBuilder::ball(1.0), // TODO: Implement remaining types.
        };
        CollisionShape {
            scale,
            shape,
            collider: builder.restitution(0.4).build(),
        }
    }

    pub fn new_box(halfExtents: &Vec3) -> CollisionShape {
        Self::new(
            1.0,
            CollisionShapeType::Box {
                halfExtents: *halfExtents,
            },
        )
    }

    pub fn new_box_from_mesh(mesh: &mut Mesh) -> CollisionShape {
        let mut bounds = Box3::default();
        unsafe { Mesh_GetBound(mesh, &mut bounds) };
        Self::new(
            1.0,
            CollisionShapeType::Box {
                halfExtents: Vec3::new(
                    f32::max(f32::abs(bounds.upper.x), f32::abs(bounds.lower.x)),
                    f32::max(f32::abs(bounds.upper.y), f32::abs(bounds.lower.y)),
                    f32::max(f32::abs(bounds.upper.z), f32::abs(bounds.lower.z)),
                ),
            },
        )
    }

    pub fn new_sphere(radius: f32) -> CollisionShape {
        Self::new(1.0, CollisionShapeType::Sphere { radius: radius })
    }

    pub fn new_sphere_from_mesh(mesh: &mut Mesh) -> CollisionShape {
        Self::new(
            1.0,
            CollisionShapeType::Sphere {
                radius: Mesh_GetRadius(mesh),
            },
        )
    }

    pub fn new_hull_from_mesh(mesh: Box<Mesh>) -> CollisionShape {
        Self::new(1.0, CollisionShapeType::Hull { mesh })
    }
}
