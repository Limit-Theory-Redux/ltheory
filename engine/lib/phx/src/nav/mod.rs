//! The engine's navigation world for Lua: [`NavWorld`] steers agents that are bodies of one
//! physics world, with the algorithms of the `nav` crate. Design: `doc/engine/nav.md`.

mod nav_axis;
mod nav_world;

pub use nav_axis::*;
pub use nav_world::*;
