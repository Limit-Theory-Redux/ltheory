use std::sync::Arc;

use glam::{ivec2, ivec3, ivec4, vec2, vec3, vec4};

use super::{is_command_mode, Shader, ShaderVarData, Tex1D, Tex2D, Tex3D, TexCube};
use crate::math::Matrix;

/// Stores both index (for direct GL mode) and name (for command mode).
/// This ensures compatibility with both rendering paths.
struct UniformEntry {
    index: i32,
    name: Arc<str>,
    data: ShaderVarData,
}

/// ShaderState stores uniform values for later application.
///
/// Dual-mode design:
/// - Direct GL mode: uses cached indices for fast uniform setting
/// - Command mode: uses names because render thread has different indices
pub struct ShaderState {
    shader: Shader,
    elems: Vec<UniformEntry>,
}

#[luajit_ffi_gen::luajit_ffi]
impl ShaderState {
    #[bind(name = "Create")]
    pub fn new(shader: &Shader) -> ShaderState {
        ShaderState {
            shader: shader.clone(),
            elems: Vec::new(),
        }
    }

    pub fn from_shader_load(vs_name: &str, fs_name: &str) -> ShaderState {
        Self::new(&Shader::load(vs_name, fs_name))
    }

    pub fn set_float(&mut self, name: &str, x: f32) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push(UniformEntry {
                index,
                name: Arc::from(name),
                data: ShaderVarData::Float(x),
            });
        }
    }

    pub fn set_float2(&mut self, name: &str, x: f32, y: f32) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push(UniformEntry {
                index,
                name: Arc::from(name),
                data: ShaderVarData::Float2(vec2(x, y)),
            });
        }
    }

    pub fn set_float3(&mut self, name: &str, x: f32, y: f32, z: f32) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push(UniformEntry {
                index,
                name: Arc::from(name),
                data: ShaderVarData::Float3(vec3(x, y, z)),
            });
        }
    }

    pub fn set_float4(&mut self, name: &str, x: f32, y: f32, z: f32, w: f32) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push(UniformEntry {
                index,
                name: Arc::from(name),
                data: ShaderVarData::Float4(vec4(x, y, z, w)),
            });
        }
    }

    pub fn set_int(&mut self, name: &str, x: i32) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push(UniformEntry {
                index,
                name: Arc::from(name),
                data: ShaderVarData::Int(x),
            });
        }
    }

    pub fn set_int2(&mut self, name: &str, x: i32, y: i32) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push(UniformEntry {
                index,
                name: Arc::from(name),
                data: ShaderVarData::Int2(ivec2(x, y)),
            });
        }
    }

    pub fn set_int3(&mut self, name: &str, x: i32, y: i32, z: i32) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push(UniformEntry {
                index,
                name: Arc::from(name),
                data: ShaderVarData::Int3(ivec3(x, y, z)),
            });
        }
    }

    pub fn set_int4(&mut self, name: &str, x: i32, y: i32, z: i32, w: i32) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push(UniformEntry {
                index,
                name: Arc::from(name),
                data: ShaderVarData::Int4(ivec4(x, y, z, w)),
            });
        }
    }

    pub fn set_matrix(&mut self, name: &str, m: &Matrix) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push(UniformEntry {
                index,
                name: Arc::from(name),
                data: ShaderVarData::Matrix(m.clone()),
            });
        }
    }

    pub fn set_tex1d(&mut self, name: &str, t: &mut Tex1D) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push(UniformEntry {
                index,
                name: Arc::from(name),
                data: ShaderVarData::Tex1D(t.clone()),
            });
        }
    }

    pub fn set_tex2d(&mut self, name: &str, t: &mut Tex2D) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push(UniformEntry {
                index,
                name: Arc::from(name),
                data: ShaderVarData::Tex2D(t.clone()),
            });
        }
    }

    pub fn set_tex3d(&mut self, name: &str, t: &mut Tex3D) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push(UniformEntry {
                index,
                name: Arc::from(name),
                data: ShaderVarData::Tex3D(t.clone()),
            });
        }
    }

    pub fn set_tex_cube(&mut self, name: &str, t: &mut TexCube) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push(UniformEntry {
                index,
                name: Arc::from(name),
                data: ShaderVarData::TexCube(t.clone()),
            });
        }
    }

    pub fn start(&mut self) {
        self.shader.start();

        // Apply uniforms using the appropriate method for current mode:
        // - Direct GL mode: use index for fast, direct uniform setting
        // - Command mode: use name because render thread has different indices
        if is_command_mode() {
            for entry in self.elems.iter() {
                self.shader
                    .name_set_uniform_on_shared(&entry.name, entry.data.clone());
            }
        } else {
            for entry in self.elems.iter() {
                self.shader.index_set_uniform(entry.index, entry.data.clone());
            }
        }
    }

    pub fn stop(&mut self) {
        self.shader.stop();
    }

    pub fn shader(&mut self) -> &mut Shader {
        &mut self.shader
    }
}
