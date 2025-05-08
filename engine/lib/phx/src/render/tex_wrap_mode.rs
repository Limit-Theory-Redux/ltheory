use super::gl;

#[luajit_ffi_gen::luajit_ffi(repr = "u32")]
#[derive(Debug, Clone, Copy)]
pub enum TexWrapMode {
    Clamp = gl::CLAMP_TO_EDGE,
    MirrorClamp = gl::MIRROR_CLAMP_TO_EDGE,
    MirrorRepeat = gl::MIRRORED_REPEAT,
    Repeat = gl::REPEAT,
}
