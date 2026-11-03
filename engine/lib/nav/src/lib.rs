//! Pure navigation algorithms: no FFI, no engine types, no game vocabulary.
//!
//! The crate is the Rust side of ship navigation. The engine wraps [`World`] for Lua; Lua kernels
//! that have a Rust twin are checked against it from `tests/`. See `README.md` and
//! `doc/engine/nav.md`.

mod agent;
mod avoidance;
mod behaviour;
mod controller;
mod obstacle;
mod world;

pub use agent::{AXIS_COUNT, AgentId, Axis, Body, Frame, Goal, Profile};
pub use avoidance::AvoidanceTuning;
pub use obstacle::Obstacle;
pub use world::World;
