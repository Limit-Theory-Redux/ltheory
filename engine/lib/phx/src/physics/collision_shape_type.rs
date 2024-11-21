use crate::math::Vec3;
use crate::render::Mesh;

pub enum CollisionShapeType {
    Box { half_extents: Vec3 },
    Sphere { radius: f32 },
    ConvexHull { mesh: Mesh },
    ConvexDecomposition { mesh: Mesh },
    Trimesh { mesh: Mesh },
}
