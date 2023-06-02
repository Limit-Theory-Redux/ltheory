use std::rc::{Rc, Weak};

use crate::math::{Box3, Vec3};
use crate::mesh::*;
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

#[derive(Clone)]
pub enum CollisionShapeType {
    Box { halfExtents: Vec3 },
    Sphere { radius: f32 },
    Hull { mesh: Weak<Mesh> },
    Compound(),
}

pub struct CollisionShape {
    pub scale: f32,
    pub shape: CollisionShapeType,
    pub collider: rp::Collider,
}

impl CollisionShape {
    pub(crate) fn new(scale: f32, shape: CollisionShapeType) -> Box<CollisionShape> {
        let builder = match shape {
            CollisionShapeType::Box { halfExtents } => ColliderBuilder::cuboid(
                halfExtents.x * scale,
                halfExtents.y * scale,
                halfExtents.z * scale,
            ),
            CollisionShapeType::Sphere { radius } => ColliderBuilder::ball(radius * scale),
            _ => ColliderBuilder::ball(1.0), // TODO: Implement remaining types.
        };
        Box::new(CollisionShape {
            scale: scale,
            shape: shape,
            collider: builder.build(),
        })
    }

    pub fn newBox(halfExtents: &Vec3) -> Box<CollisionShape> {
        Self::new(
            1.0,
            CollisionShapeType::Box {
                halfExtents: *halfExtents,
            },
        )
    }

    pub fn newBoxFromMesh(mesh: &mut Mesh) -> Box<CollisionShape> {
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

    pub fn newSphere(radius: f32) -> Box<CollisionShape> {
        Self::new(1.0, CollisionShapeType::Sphere { radius: radius })
    }

    pub fn newSphereFromMesh(mesh: &mut Mesh) -> Box<CollisionShape> {
        Self::new(
            1.0,
            CollisionShapeType::Sphere {
                radius: Mesh_GetRadius(mesh),
            },
        )
    }

    pub fn newHullFromMesh(mesh: Rc<Mesh>) -> Box<CollisionShape> {
        Self::new(
            1.0,
            CollisionShapeType::Hull {
                mesh: Rc::downgrade(&mesh),
            },
        )
    }
}
