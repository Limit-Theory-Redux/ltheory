use parley::layout::Alignment;

#[luajit_ffi_gen::luajit_ffi]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TextAlignment {
    Start,
    Middle,
    End,
    Justified,
}

impl From<TextAlignment> for Alignment {
    fn from(value: TextAlignment) -> Self {
        match value {
            TextAlignment::Start => Self::Start,
            TextAlignment::Middle => Self::Middle,
            TextAlignment::End => Self::End,
            TextAlignment::Justified => Self::Justified,
        }
    }
}
