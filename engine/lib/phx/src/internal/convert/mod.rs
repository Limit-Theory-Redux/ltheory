mod c2rust;

pub use c2rust::*;

pub trait ConvertIntoString {
    fn as_str(&self) -> &str;

    fn as_string(&self) -> String {
        self.as_str().to_string()
    }
}
