use std::collections::HashMap;

use rapier3d_f64::prelude as rp;
use rapier3d_f64::prelude::nalgebra as na;

use crate::physics::*;
use crate::render::*;

#[derive(PartialEq, Eq, Hash, Debug)]
struct ShapeCacheKey {
    mesh: MeshCacheKey,
    scale: u32,
}

impl ShapeCacheKey {
    fn new(mesh: MeshCacheKey, scale: f32) -> ShapeCacheKey {
        ShapeCacheKey {
            mesh: mesh,
            scale: scale.to_bits(),
        }
    }
}

pub struct ShapeCache {
    convex_hull_unscaled_cache: HashMap<MeshCacheKey, rp::SharedShape>,
    trimesh_unscaled_cache: HashMap<MeshCacheKey, rp::SharedShape>,

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

    // This function retrieves a new rp::SharedShape, either cloned from the existing cache, or
    // created from the underlying mesh data.
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
                let mesh_key = mesh.get_cache_key();
                self.convex_hull_cache
                    .entry(ShapeCacheKey::new(mesh_key, scale))
                    .or_insert_with(|| {
                        // Get the unscaled shape (and generate it if needed).
                        let unscaled_shape = self
                            .convex_hull_unscaled_cache
                            .entry(mesh_key)
                            .or_insert_with(|| {
                                let vertices = Self::convert_vertices(1.0, mesh);
                                rp::SharedShape::convex_hull(&vertices)
                                    .expect("Convex hull computation failed")
                            });

                        // Scale the unscaled shape.
                        if scale != 1.0 {
                            let scale_factor = rp::Vector::new(
                                scale as rp::Real,
                                scale as rp::Real,
                                scale as rp::Real,
                            );
                            rp::SharedShape::new(
                                unscaled_shape
                                    .as_convex_polyhedron()
                                    .unwrap()
                                    .clone()
                                    .scaled(&scale_factor)
                                    .unwrap(),
                            )
                        } else {
                            unscaled_shape.clone()
                        }
                    })
                    .clone()
            }
            CollisionShapeType::ConvexDecomposition { mesh } => {
                let mesh_key = mesh.get_cache_key();
                self.convex_decomposition_cache
                    .entry(ShapeCacheKey::new(mesh_key, scale))
                    .or_insert_with(|| {
                        let vertices = Self::convert_vertices(scale, mesh);
                        let indices = Self::convert_indices(mesh);
                        rp::SharedShape::convex_decomposition(&vertices, &indices)
                    })
                    .clone()
            }
            CollisionShapeType::Trimesh { mesh } => {
                let mesh_key = mesh.get_cache_key();
                self.trimesh_cache
                    .entry(ShapeCacheKey::new(mesh_key, scale))
                    .or_insert_with(|| {
                        // Get the unscaled shape (and generate it if needed).
                        let unscaled_shape = self
                            .trimesh_unscaled_cache
                            .entry(mesh_key)
                            .or_insert_with(|| {
                                let vertices = Self::convert_vertices(1.0, mesh);
                                let indices = Self::convert_indices(mesh);
                                rp::SharedShape::trimesh(vertices, indices)
                            });

                        // Scale the unscaled shape.
                        if scale != 1.0 {
                            let scale_factor = rp::Vector::new(
                                scale as rp::Real,
                                scale as rp::Real,
                                scale as rp::Real,
                            );
                            rp::SharedShape::new(
                                unscaled_shape
                                    .as_trimesh()
                                    .unwrap()
                                    .clone()
                                    .scaled(&scale_factor),
                            )
                        } else {
                            unscaled_shape.clone()
                        }
                    })
                    .clone()
            }
        }
    }

    pub fn evict(&mut self, scale: f32, mesh: &Mesh) {
        let key = ShapeCacheKey::new(mesh.get_cache_key(), scale);
        self.convex_hull_cache.remove(&key);
        self.convex_decomposition_cache.remove(&key);
        self.trimesh_cache.remove(&key);
    }

    pub fn evict_all(&mut self, mesh: &Mesh) {
        let mesh_cache_key = mesh.get_cache_key();
        self.convex_hull_unscaled_cache.remove(&mesh_cache_key);
        self.trimesh_unscaled_cache.remove(&mesh_cache_key);
    }

    fn convert_vertices(scale: f32, mesh: &Mesh) -> Vec<na::Point3<rp::Real>> {
        mesh.get_vertex_data()
            .iter()
            .map(|v| (v.p * scale).to_na_point())
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
