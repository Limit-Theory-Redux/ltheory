use std::sync::Mutex;

use once_cell::sync::Lazy;
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

// TODO: use [`std::cell::LazyCell`] when it's stabilized in Rust 1.80 on July 25
pub static TEXT_CTX: Lazy<Mutex<TextContext>> = Lazy::new(Default::default);
