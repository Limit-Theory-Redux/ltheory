mod lua2rust;
mod rust2lua;

pub use lua2rust::*;
pub use rust2lua::*;

pub trait Convert {
    type T;

    fn convert(&self) -> Self::T;
}
