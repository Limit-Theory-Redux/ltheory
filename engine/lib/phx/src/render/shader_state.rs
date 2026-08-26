use glam::{ivec2, ivec3, ivec4, vec2, vec3, vec4};

use super::{Shader, ShaderVarData, Tex1D, Tex2D, Tex3D, TexCube};
use crate::math::Matrix;
use crate::render::Renderer;

pub struct ShaderState {
    shader: Shader,
    elems: Vec<(i32, ShaderVarData)>,
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

    pub fn from_shader_load(r: &mut Renderer, vs_name: &str, fs_name: &str) -> ShaderState {
        Self::new(&Shader::load(r, vs_name, fs_name))
    }

    pub fn set_float(&mut self, r: &mut Renderer, name: &str, x: f32) {
        if let Some(index) = self.shader.get_uniform_index(r, name) {
            self.elems.push((index, ShaderVarData::Float(x)));
        }
    }

    pub fn set_float2(&mut self, r: &mut Renderer, name: &str, x: f32, y: f32) {
        if let Some(index) = self.shader.get_uniform_index(r, name) {
            self.elems.push((index, ShaderVarData::Float2(vec2(x, y))));
        }
    }

    pub fn set_float3(&mut self, r: &mut Renderer, name: &str, x: f32, y: f32, z: f32) {
        if let Some(index) = self.shader.get_uniform_index(r, name) {
            self.elems
                .push((index, ShaderVarData::Float3(vec3(x, y, z))));
        }
    }

    pub fn set_float4(&mut self, r: &mut Renderer, name: &str, x: f32, y: f32, z: f32, w: f32) {
        if let Some(index) = self.shader.get_uniform_index(r, name) {
            self.elems
                .push((index, ShaderVarData::Float4(vec4(x, y, z, w))));
        }
    }

    pub fn set_int(&mut self, r: &mut Renderer, name: &str, x: i32) {
        if let Some(index) = self.shader.get_uniform_index(r, name) {
            self.elems.push((index, ShaderVarData::Int(x)));
        }
    }

    pub fn set_int2(&mut self, r: &mut Renderer, name: &str, x: i32, y: i32) {
        if let Some(index) = self.shader.get_uniform_index(r, name) {
            self.elems.push((index, ShaderVarData::Int2(ivec2(x, y))));
        }
    }

    pub fn set_int3(&mut self, r: &mut Renderer, name: &str, x: i32, y: i32, z: i32) {
        if let Some(index) = self.shader.get_uniform_index(r, name) {
            self.elems
                .push((index, ShaderVarData::Int3(ivec3(x, y, z))));
        }
    }

    pub fn set_int4(&mut self, r: &mut Renderer, name: &str, x: i32, y: i32, z: i32, w: i32) {
        if let Some(index) = self.shader.get_uniform_index(r, name) {
            self.elems
                .push((index, ShaderVarData::Int4(ivec4(x, y, z, w))));
        }
    }

    pub fn set_matrix(&mut self, r: &mut Renderer, name: &str, m: &Matrix) {
        if let Some(index) = self.shader.get_uniform_index(r, name) {
            self.elems.push((index, ShaderVarData::Matrix(m.clone())));
        }
    }

    pub fn set_tex1d(&mut self, r: &mut Renderer, name: &str, t: &mut Tex1D) {
        if let Some(index) = self.shader.get_uniform_index(r, name) {
            self.elems.push((index, ShaderVarData::Tex1D(t.clone())));
        }
    }

    pub fn set_tex2d(&mut self, r: &mut Renderer, name: &str, t: &mut Tex2D) {
        if let Some(index) = self.shader.get_uniform_index(r, name) {
            self.elems.push((index, ShaderVarData::Tex2D(t.clone())));
        }
    }

    pub fn set_tex3d(&mut self, r: &mut Renderer, name: &str, t: &mut Tex3D) {
        if let Some(index) = self.shader.get_uniform_index(r, name) {
            self.elems.push((index, ShaderVarData::Tex3D(t.clone())));
        }
    }

    pub fn set_tex_cube(&mut self, r: &mut Renderer, name: &str, t: &mut TexCube) {
        if let Some(index) = self.shader.get_uniform_index(r, name) {
            self.elems.push((index, ShaderVarData::TexCube(t.clone())));
        }
    }

    pub fn start(&mut self, r: &mut Renderer) {
        self.shader.start(r);

        // Apply uniforms.
        for (index, data) in self.elems.iter() {
            self.shader.index_set_uniform(r, *index, data.clone());
        }
    }

    pub fn stop(&mut self, r: &mut Renderer) {
        self.shader.stop(r);
    }

    pub fn shader(&mut self) -> &mut Shader {
        &mut self.shader
    }
}
