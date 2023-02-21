use libc;

#[no_mangle]
pub extern "C" fn Bit_And32(mut x: u32, mut y: u32) -> u32 {
    return x & y;
}

#[no_mangle]
pub extern "C" fn Bit_Or32(mut x: u32, mut y: u32) -> u32 {
    return x | y;
}

#[no_mangle]
pub extern "C" fn Bit_Xor32(mut x: u32, mut y: u32) -> u32 {
    return x ^ y;
}

#[no_mangle]
pub extern "C" fn Bit_Has32(mut x: u32, mut y: u32) -> bool {
    return x & y == y;
}

#[no_mangle]
pub extern "C" fn Bit_And64(mut x: u64, mut y: u64) -> u64 {
    return x & y;
}

#[no_mangle]
pub extern "C" fn Bit_Or64(mut x: u64, mut y: u64) -> u64 {
    return x | y;
}

#[no_mangle]
pub extern "C" fn Bit_Xor64(mut x: u64, mut y: u64) -> u64 {
    return x ^ y;
}

#[no_mangle]
pub extern "C" fn Bit_Has64(mut x: u64, mut y: u64) -> bool {
    return x & y == y;
}
