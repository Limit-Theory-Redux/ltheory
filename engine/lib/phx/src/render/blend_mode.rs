//! Fixed-function enums of pipelines (`PipelineDesc`).

#[luajit_ffi_gen::luajit_ffi]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BlendMode {
    #[default]
    Disabled,
    Additive,
    Alpha,
    PreMultAlpha,
}

#[luajit_ffi_gen::luajit_ffi]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CullFace {
    #[default]
    None,
    Back,
    Front,
}
