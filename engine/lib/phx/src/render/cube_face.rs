#[luajit_ffi_gen::luajit_ffi(with_impl = true, repr = "u32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CubeFace {
    PX,
    NX,
    PY,
    NY,
    PZ,
    NZ,
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
