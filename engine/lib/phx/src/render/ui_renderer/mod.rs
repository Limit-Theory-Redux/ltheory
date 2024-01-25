mod image;
mod layer;
mod panel;
mod rect;
mod renderer;
mod text;

use self::image::*;
use layer::*;
use panel::*;
use rect::*;
pub use renderer::*;
use text::*;

macro_rules! def_id {
    ($name:ident) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
        pub struct $name(usize);

        impl $name {
            pub fn inc(&self) -> Self {
                Self(self.0 + 1)
            }

            pub fn dec(&self) -> Option<Self> {
                if self.0 > 0 {
                    Some(Self(self.0 - 1))
                } else {
                    None
                }
            }
        }

        impl std::ops::Deref for $name {
            type Target = usize;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<usize> for $name {
            fn from(value: usize) -> Self {
                Self(value)
            }
        }
    };
}

def_id!(UIRendererLayerId);
def_id!(UIRendererImageId);
def_id!(UIRendererPanelId);
def_id!(UIRendererRectId);
def_id!(UIRendererTextId);
