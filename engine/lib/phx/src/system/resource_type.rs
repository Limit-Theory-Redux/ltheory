use internal::static_string;

use crate::common::*;

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Font,
    Mesh,
    Script,
    Shader,
    Sound,
    Tex1D,
    Tex2D,
    Tex3D,
    TexCube,
    Theme,
    Style,
    Other,
}
