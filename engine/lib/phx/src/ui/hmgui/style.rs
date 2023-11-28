use std::collections::HashMap;
use std::ops::Deref;

use glam::Vec4;

use crate::render::Font;

use super::{HmGuiProperty, HmGuiPropertyId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HmGuiStyleId(usize);

impl Deref for HmGuiStyleId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Default)]
pub struct HmGuiStyle {
    pub properties: HashMap<HmGuiPropertyId, HmGuiProperty>,
}
