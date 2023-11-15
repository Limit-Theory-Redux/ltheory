pub type TexWrapMode = i32;

#[no_mangle]
pub static TexWrapMode_Clamp: TexWrapMode = 1; //gl::CLAMP_TO_EDGE as TexWrapMode;

#[no_mangle]
pub static TexWrapMode_MirrorClamp: TexWrapMode = 1; //gl::MIRROR_CLAMP_TO_EDGE as TexWrapMode;

#[no_mangle]
pub static TexWrapMode_MirrorRepeat: TexWrapMode = 1; //gl::MIRRORED_REPEAT as TexWrapMode;

#[no_mangle]
pub static TexWrapMode_Repeat: TexWrapMode = 1; //gl::REPEAT as TexWrapMode;
