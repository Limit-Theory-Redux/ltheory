use super::*;
use crate::math::*;
use internal::ConvertIntoString;

pub struct ShaderState {
    shader: *mut Shader,
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
        unsafe {
            Shader_Free(self.shader);
        }
    }
}

impl ShaderState {
    fn shader_get_variable(&self, name: &str) -> i32 {
        let c_name = std::ffi::CString::new(name).unwrap();
        unsafe { Shader_GetVariable(&mut *self.shader, c_name.as_ptr()) }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl ShaderState {
    pub fn create(shader: &mut Shader) -> ShaderState {
        ShaderState {
            shader: {
                Shader_Acquire(shader);
                shader as *mut _
            },
            elems: Vec::new(),
        }
    }

    pub fn from_shader_load(vert_name: &str, frag_name: &str) -> ShaderState {
        let c_vert_name = std::ffi::CString::new(vert_name).unwrap();
        let c_frag_name = std::ffi::CString::new(frag_name).unwrap();
        let shader = Box::into_raw(Shader_Load(c_vert_name.as_ptr(), c_frag_name.as_ptr()));
        let shader_state = Self::create(unsafe { &mut *shader });
        unsafe { Shader_Free(shader) };
        shader_state
    }

    pub fn set_float(&mut self, name: &str, x: f32) {
        self.elems
            .push((self.shader_get_variable(name), ShaderVarData::Float(x)));
    }

    pub fn set_float2(&mut self, name: &str, x: f32, y: f32) {
        self.elems.push((
            self.shader_get_variable(name),
            ShaderVarData::Float2(vec2(x, y)),
        ));
    }

    pub fn set_float3(&mut self, name: &str, x: f32, y: f32, z: f32) {
        self.elems.push((
            self.shader_get_variable(name),
            ShaderVarData::Float3(vec3(x, y, z)),
        ));
    }

    pub fn set_float4(&mut self, name: &str, x: f32, y: f32, z: f32, w: f32) {
        self.elems.push((
            self.shader_get_variable(name),
            ShaderVarData::Float4(vec4(x, y, z, w)),
        ));
    }

    pub fn set_int(&mut self, name: &str, x: i32) {
        self.elems
            .push((self.shader_get_variable(name), ShaderVarData::Int(x)));
    }

    pub fn set_int2(&mut self, name: &str, x: i32, y: i32) {
        self.elems.push((
            self.shader_get_variable(name),
            ShaderVarData::Int2(ivec2(x, y)),
        ));
    }

    pub fn set_int3(&mut self, name: &str, x: i32, y: i32, z: i32) {
        self.elems.push((
            self.shader_get_variable(name),
            ShaderVarData::Int3(ivec3(x, y, z)),
        ));
    }

    pub fn set_int4(&mut self, name: &str, x: i32, y: i32, z: i32, w: i32) {
        self.elems.push((
            self.shader_get_variable(name),
            ShaderVarData::Int4(ivec4(x, y, z, w)),
        ));
    }

    pub fn set_matrix(&mut self, name: &str, m: &Matrix) {
        self.elems
            .push((self.shader_get_variable(name), ShaderVarData::Matrix(*m)));
    }

    pub fn set_tex1d(&mut self, name: &str, t: &mut Tex1D) {
        self.elems
            .push((self.shader_get_variable(name), ShaderVarData::Tex1D(t)));
    }

    pub fn set_tex2d(&mut self, name: &str, t: &mut Tex2D) {
        self.elems
            .push((self.shader_get_variable(name), ShaderVarData::Tex2D(t)));
    }

    pub fn set_tex3d(&mut self, name: &str, t: &mut Tex3D) {
        self.elems
            .push((self.shader_get_variable(name), ShaderVarData::Tex3D(t)));
    }

    pub fn set_tex_cube(&mut self, name: &str, t: &mut TexCube) {
        self.elems
            .push((self.shader_get_variable(name), ShaderVarData::TexCube(t)));
    }

    pub fn start(&mut self) {
        unsafe {
            Shader_Start(&mut *self.shader);
        }

        // Apply uniforms.
        for (index, data) in self.elems.iter() {
            match data {
                ShaderVarData::Float(v) => glcheck!(gl::Uniform1f(*index, *v)),
                ShaderVarData::Float2(v) => glcheck!(gl::Uniform2f(*index, v.x, v.y)),
                ShaderVarData::Float3(v) => glcheck!(gl::Uniform3f(*index, v.x, v.y, v.z)),
                ShaderVarData::Float4(v) => glcheck!(gl::Uniform4f(*index, v.x, v.y, v.z, v.w)),
                ShaderVarData::Int(v) => glcheck!(gl::Uniform1i(*index, *v)),
                ShaderVarData::Int2(v) => glcheck!(gl::Uniform2i(*index, v.x, v.y)),
                ShaderVarData::Int3(v) => glcheck!(gl::Uniform3i(*index, v.x, v.y, v.z)),
                ShaderVarData::Int4(v) => glcheck!(gl::Uniform4i(*index, v.x, v.y, v.z, v.w)),
                ShaderVarData::Matrix(m) => Shader_ISetMatrix(*index, &m),
                ShaderVarData::Tex1D(t) => unsafe { Shader_ISetTex1D(*index, &mut **t) },
                ShaderVarData::Tex2D(t) => unsafe { Shader_ISetTex2D(*index, &mut **t) },
                ShaderVarData::Tex3D(t) => unsafe { Shader_ISetTex3D(*index, &mut **t) },
                ShaderVarData::TexCube(t) => unsafe { Shader_ISetTexCube(*index, &mut **t) },
            }
        }
    }

    pub fn stop(&mut self) {
        Shader_Stop(unsafe { &*self.shader });
    }
}
