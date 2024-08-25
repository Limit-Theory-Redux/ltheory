use super::*;
use crate::rf::Rf;

/* TODO : Merge meshes into single IBO/VBO so that we can skip all the rebinds
 *        (profiling shows that they are a huge perf drain in the rendering
 *         pipeline) */

/// A basic container for abstracting LOD rendering behavior. LodMesh consists of any number of (Mesh,
/// distMin, distMax) tuples. Drawing a requires passing a *distance squared* argument that is used to
/// determine which component(s) of the LodMesh to draw.
///
/// This object is cheap to clone, as the underlying data structure is reference counted.
#[derive(Clone)]
pub struct LodMesh {
    lod_levels: Rf<Vec<LodMeshEntry>>,
}

struct LodMeshEntry {
    mesh: Mesh,
    distance_squared_min: f32,
    distance_squared_max: f32,
}

#[luajit_ffi_gen::luajit_ffi]
impl LodMesh {
    #[bind(name = "Create")]
    pub fn new() -> LodMesh {
        LodMesh {
            lod_levels: Rf::new(Vec::new()),
        }
    }

    // This simply forwards calls from Lua to the Clone trait.
    #[bind(name = "Clone")]
    fn clone_impl(&self) -> LodMesh {
        self.clone()
    }

    pub fn add(&mut self, mesh: Mesh, distance_min: f32, distance_max: f32) {
        self.lod_levels.as_mut().push(LodMeshEntry {
            mesh,
            distance_squared_min: distance_min * distance_min,
            distance_squared_max: distance_max * distance_max,
        });
    }

    pub fn draw(&mut self, distance_squared: f32) {
        for level in &mut *self.lod_levels.as_mut() {
            if level.distance_squared_min <= distance_squared
                && distance_squared <= level.distance_squared_max
            {
                level.mesh.draw();
                break;
            }
        }
    }

    pub fn get(&mut self, distance_squared: f32) -> Option<Mesh> {
        for level in &mut *self.lod_levels.as_mut() {
            if level.distance_squared_min <= distance_squared
                && distance_squared <= level.distance_squared_max
            {
                return Some(level.mesh.clone());
            }
        }

        None
    }
}
