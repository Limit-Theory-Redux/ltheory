use super::*;
use crate::math::*;
use internal::ConvertIntoString;

pub struct ShaderState {
    shader: Shader,
    elems: Vec<(i32, ShaderVarData)>,
}

impl Drop for ShaderState {
    fn drop(&mut self) {
        for (_, data) in self.elems.iter() {
            match data {
                ShaderVarData::Tex1D(t) => unsafe {
                    Tex1D_Free(*t);
                },
                ShaderVarData::Tex2D(t) => unsafe {
                    Tex2D_Free(*t);
                },
                ShaderVarData::Tex3D(t) => unsafe {
                    Tex3D_Free(*t);
                },
                ShaderVarData::TexCube(t) => unsafe {
                    TexCube_Free(*t);
                },
                _ => {}
            }
        }
    }
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
            self.elems.push((index, ShaderVarData::Float(x)));
        }
    }

    pub fn set_float2(&mut self, name: &str, x: f32, y: f32) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push((index, ShaderVarData::Float2(vec2(x, y))));
        }
    }

    pub fn set_float3(&mut self, name: &str, x: f32, y: f32, z: f32) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems
                .push((index, ShaderVarData::Float3(vec3(x, y, z))));
        }
    }

    pub fn set_float4(&mut self, name: &str, x: f32, y: f32, z: f32, w: f32) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems
                .push((index, ShaderVarData::Float4(vec4(x, y, z, w))));
        }
    }

    pub fn set_int(&mut self, name: &str, x: i32) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push((index, ShaderVarData::Int(x)));
        }
    }

    pub fn set_int2(&mut self, name: &str, x: i32, y: i32) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push((index, ShaderVarData::Int2(ivec2(x, y))));
        }
    }

    pub fn set_int3(&mut self, name: &str, x: i32, y: i32, z: i32) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems
                .push((index, ShaderVarData::Int3(ivec3(x, y, z))));
        }
    }

    pub fn set_int4(&mut self, name: &str, x: i32, y: i32, z: i32, w: i32) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems
                .push((index, ShaderVarData::Int4(ivec4(x, y, z, w))));
        }
    }

    pub fn set_matrix(&mut self, name: &str, m: &Matrix) {
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push((index, ShaderVarData::Matrix(*m)));
        }
    }

    pub fn set_tex1d(&mut self, name: &str, t: &mut Tex1D) {
        Tex1D_Acquire(t);
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push((index, ShaderVarData::Tex1D(t)));
        }
    }

    pub fn set_tex2d(&mut self, name: &str, t: &mut Tex2D) {
        Tex2D_Acquire(t);
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push((index, ShaderVarData::Tex2D(t)));
        }
    }

    pub fn set_tex3d(&mut self, name: &str, t: &mut Tex3D) {
        Tex3D_Acquire(t);
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push((index, ShaderVarData::Tex3D(t)));
        }
    }

    pub fn set_tex_cube(&mut self, name: &str, t: &mut TexCube) {
        TexCube_Acquire(t);
        if let Some(index) = self.shader.get_uniform_index(name) {
            self.elems.push((index, ShaderVarData::TexCube(t)));
        }
    }

    pub fn start(&mut self) {
        self.shader.start();

        // Apply uniforms.
        for (index, data) in self.elems.iter() {
            Shader::index_set_uniform(*index, data);
        }
    }

    pub fn stop(&mut self) {
        self.shader.stop();
    }
}
