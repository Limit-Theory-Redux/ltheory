#![allow(unsafe_code)] // TODO: remove

use luajit_ffi_gen::luajit_ffi;

#[allow(dead_code)]
mod helpers;

#[derive(Default)]
pub struct ResultTest {}

#[luajit_ffi(gen_dir = "./tests/out/ffi_gen", meta_dir = "./tests/out/ffi_meta")]
impl ResultTest {
    pub fn result_primitive() -> Result<u8, u8> {
        Ok(42)
    }

    pub fn result_err() -> Result<u8, u8> {
        Err(13)
    }

    pub fn result_option() -> Result<Option<u8>, u8> {
        Ok(Some(42))
    }

    pub fn result_string() -> Result<String, u8> {
        Ok("hello".to_string())
    }
}

#[test]
fn test_result() {
    use internal::ConvertIntoString;

    unsafe {
        let val = ResultTest_ResultPrimitive();
        assert_eq!(val, 42);

        let val = ResultTest_ResultOption();
        assert_eq!(val.as_ref().cloned(), Some(42));

        let val = ResultTest_ResultString();
        assert_eq!(val.as_str(), "hello");
    }
}

#[test]
#[should_panic]
fn test_impl_return_error() {
    unsafe {
        ResultTest_ResultErr();
    }
}
