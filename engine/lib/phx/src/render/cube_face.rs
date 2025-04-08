#[luajit_ffi_gen::luajit_ffi(with_impl = true)]
#[derive(Debug, Clone, Copy)]
pub enum CubeFace {
    PX = 0x8515, // gl::TEXTURE_CUBE_MAP_POSITIVE_X,
    NX = 0x8516, // gl::TEXTURE_CUBE_MAP_NEGATIVE_X,
    PY = 0x8517, // gl::TEXTURE_CUBE_MAP_POSITIVE_Y,
    NY = 0x8518, // gl::TEXTURE_CUBE_MAP_NEGATIVE_Y,
    PZ = 0x8519, // gl::TEXTURE_CUBE_MAP_POSITIVE_Z,
    NZ = 0x851A, // gl::TEXTURE_CUBE_MAP_NEGATIVE_Z,
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
