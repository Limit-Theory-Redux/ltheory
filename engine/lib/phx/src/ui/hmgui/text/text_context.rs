use std::sync::{LazyLock, Mutex};

use parley::{FontContext, LayoutContext};
use swash::scale::ScaleContext;

use crate::render::Color;

/// Persistent per application cache of text related information: fonts, layout and scale contexts.
#[derive(Default)]
pub struct TextContext {
    pub font: FontContext,
    pub layout: LayoutContext<Color>,
    pub scale: ScaleContext,
}

pub static TEXT_CTX: LazyLock<Mutex<TextContext>> = LazyLock::new(Default::default);
