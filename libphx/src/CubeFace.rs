use crate::GL::gl;
pub type CubeFace = i32;

#[no_mangle]
pub static CubeFace_PX: CubeFace = gl::TEXTURE_CUBE_MAP_POSITIVE_X as CubeFace;

#[no_mangle]
pub static CubeFace_NX: CubeFace = gl::TEXTURE_CUBE_MAP_NEGATIVE_X as CubeFace;

#[no_mangle]
pub static CubeFace_PY: CubeFace = gl::TEXTURE_CUBE_MAP_POSITIVE_Y as CubeFace;

#[no_mangle]
pub static CubeFace_NY: CubeFace = gl::TEXTURE_CUBE_MAP_NEGATIVE_Y as CubeFace;

#[no_mangle]
pub static CubeFace_PZ: CubeFace = gl::TEXTURE_CUBE_MAP_POSITIVE_Z as CubeFace;

#[no_mangle]
pub static CubeFace_NZ: CubeFace = gl::TEXTURE_CUBE_MAP_NEGATIVE_Z as CubeFace;

static mut kFaces: [CubeFace; 6] = [
    CubeFace_PX,
    CubeFace_NX,
    CubeFace_PY,
    CubeFace_NY,
    CubeFace_PZ,
    CubeFace_NZ,
];

#[no_mangle]
pub unsafe extern "C" fn CubeFace_Get(index: i32) -> CubeFace {
    kFaces[index as usize]
}
