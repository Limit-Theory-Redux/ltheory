use super::gl;

#[luajit_ffi_gen::luajit_ffi(with_impl = true, repr = "u32")]
#[derive(Debug, Clone, Copy)]
pub enum CubeFace {
    PX = gl::TEXTURE_CUBE_MAP_POSITIVE_X,
    NX = gl::TEXTURE_CUBE_MAP_NEGATIVE_X,
    PY = gl::TEXTURE_CUBE_MAP_POSITIVE_Y,
    NY = gl::TEXTURE_CUBE_MAP_NEGATIVE_Y,
    PZ = gl::TEXTURE_CUBE_MAP_POSITIVE_Z,
    NZ = gl::TEXTURE_CUBE_MAP_NEGATIVE_Z,
}

pub const CUBE_FACES: [CubeFace; 6] = [
    CubeFace::PX,
    CubeFace::NX,
    CubeFace::PY,
    CubeFace::NY,
    CubeFace::PZ,
    CubeFace::NZ,
];

#[luajit_ffi_gen::luajit_ffi]
impl CubeFace {
    pub fn get(index: i32) -> CubeFace {
        CUBE_FACES[index as usize]
    }
}
