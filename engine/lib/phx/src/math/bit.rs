pub struct Bit;

#[luajit_ffi_gen::luajit_ffi]
impl Bit {
    pub fn and32(x: u32, y: u32) -> u32 {
        x & y
    }

    pub fn or32(x: u32, y: u32) -> u32 {
        x | y
    }

    pub fn xor32(x: u32, y: u32) -> u32 {
        x ^ y
    }

    pub fn has32(x: u32, y: u32) -> bool {
        x & y == y
    }

    pub fn not32(x: u32) -> u32 {
        !x
    }

    pub fn and64(x: u64, y: u64) -> u64 {
        x & y
    }

    pub fn or64(x: u64, y: u64) -> u64 {
        x | y
    }

    pub fn xor64(x: u64, y: u64) -> u64 {
        x ^ y
    }

    pub fn has64(x: u64, y: u64) -> bool {
        x & y == y
    }

    pub fn not64(x: u64) -> u64 {
        !x
    }
}
