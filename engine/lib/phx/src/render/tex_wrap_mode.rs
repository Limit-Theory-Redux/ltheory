#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy)]
pub enum TexWrapMode {
    Clamp = 0x812F,        // gl::CLAMP_TO_EDGE
    MirrorClamp = 0x8743,  // gl::MIRROR_CLAMP_TO_EDGE
    MirrorRepeat = 0x8370, // gl::MIRRORED_REPEAT
    Repeat = 0x2901,       // gl::REPEAT
}
