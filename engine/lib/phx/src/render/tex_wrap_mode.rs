#[luajit_ffi_gen::luajit_ffi(repr = "u32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TexWrapMode {
    Clamp,
    MirrorClamp,
    MirrorRepeat,
    Repeat,
}
