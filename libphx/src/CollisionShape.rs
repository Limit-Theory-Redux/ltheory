use crate::Math::{Box3, Vec3};
use crate::Mesh::*;
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
    Hull { b: Box<Mesh> },
    Compound(),
}

pub struct CollisionShape {
    pub cacheIndex: i32,
    pub scale: f32,
    pub shape: CollisionShapeType,
    pub collider: rp::Collider,
}

impl CollisionShape {
    fn new(spec: CollisionShapeType) -> Box<CollisionShape> {
        let builder = match spec {
            CollisionShapeType::Box { halfExtents } => {
                ColliderBuilder::cuboid(halfExtents.x, halfExtents.y, halfExtents.z)
            }
            CollisionShapeType::Sphere { radius } => ColliderBuilder::ball(radius),
            _ => ColliderBuilder::ball(1.0), // TODO: Implement remaining types.
        };
        Box::new(CollisionShape {
            cacheIndex: 0,
            scale: 0.0,
            shape: spec,
            collider: builder.build(),
        })
    }

    pub fn newBox(halfExtents: &Vec3) -> Box<CollisionShape> {
        Self::new(CollisionShapeType::Box {
            halfExtents: *halfExtents,
        })
    }

    pub fn newBoxFromMesh(mesh: &mut Mesh) -> Box<CollisionShape> {
        let mut bounds = Box3::default();
        unsafe { Mesh_GetBound(mesh, &mut bounds) };
        Self::new(CollisionShapeType::Box {
            halfExtents: Vec3::new(
                f32::max(f32::abs(bounds.upper.x), f32::abs(bounds.lower.x)),
                f32::max(f32::abs(bounds.upper.y), f32::abs(bounds.lower.y)),
                f32::max(f32::abs(bounds.upper.z), f32::abs(bounds.lower.z)),
            ),
        })
    }

    pub fn newSphere(radius: f32) -> Box<CollisionShape> {
        Self::new(CollisionShapeType::Sphere { radius: radius })
    }

    pub fn newSphereFromMesh(mesh: &mut Mesh) -> Box<CollisionShape> {
        Self::new(CollisionShapeType::Sphere {
            radius: Mesh_GetRadius(mesh),
        })
    }

    pub fn newHullFromMesh(mesh: &mut Mesh) -> Box<CollisionShape> {
        // TODO
        Self::new(CollisionShapeType::Sphere { radius: 1.0f32 })
    }
}
