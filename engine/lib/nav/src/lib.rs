//! Pure navigation algorithms: no FFI, no engine types, no game vocabulary.
//!
//! The crate is the Rust side of ship navigation. The engine wraps [`World`] for Lua; Lua kernels
//! that have a Rust twin are checked against it from `tests/`. See `README.md`.

mod world;

pub use world::World;
