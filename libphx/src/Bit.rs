use libc;

#[no_mangle]
pub extern "C" fn Bit_And32(x: u32, y: u32) -> u32 {
    return x & y;
}

#[no_mangle]
pub extern "C" fn Bit_Or32(x: u32, y: u32) -> u32 {
    return x | y;
}

#[no_mangle]
pub extern "C" fn Bit_Xor32(x: u32, y: u32) -> u32 {
    return x ^ y;
}

#[no_mangle]
pub extern "C" fn Bit_Has32(x: u32, y: u32) -> bool {
    return x & y == y;
}

#[no_mangle]
pub extern "C" fn Bit_And64(x: u64, y: u64) -> u64 {
    return x & y;
}

#[no_mangle]
pub extern "C" fn Bit_Or64(x: u64, y: u64) -> u64 {
    return x | y;
}

#[no_mangle]
pub extern "C" fn Bit_Xor64(x: u64, y: u64) -> u64 {
    return x ^ y;
}

#[no_mangle]
pub extern "C" fn Bit_Has64(x: u64, y: u64) -> bool {
    return x & y == y;
}
