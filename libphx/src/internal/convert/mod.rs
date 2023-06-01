mod lua2rust;

pub use lua2rust::*;

pub trait Convert {
    type T;

    fn convert(&self) -> Self::T;
}
