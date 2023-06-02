mod c2rust;

use std::ffi::CString;

pub use c2rust::*;

/// Convert one type to another.
/// Simplified version of the From crate to convert C data to Rust.
///
/// Rust GAT in action!
pub trait Convert {
    type T<'a>
    where
        Self: 'a;

    fn convert(&self) -> Self::T<'_>;
}
