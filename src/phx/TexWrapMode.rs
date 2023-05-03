use crate::phx::GL::gl;

pub type TexWrapMode = i32;

#[no_mangle]
pub static TexWrapMode_Clamp: TexWrapMode = gl::CLAMP_TO_EDGE as TexWrapMode;

#[no_mangle]
pub static TexWrapMode_MirrorClamp: TexWrapMode = gl::MIRROR_CLAMP_TO_EDGE as TexWrapMode;

#[no_mangle]
pub static TexWrapMode_MirrorRepeat: TexWrapMode = gl::MIRRORED_REPEAT as TexWrapMode;

#[no_mangle]
pub static TexWrapMode_Repeat: TexWrapMode = gl::REPEAT as TexWrapMode;
