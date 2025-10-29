pub struct Bit;

#[luajit_ffi_gen::luajit_ffi]
impl Bit {
    // ----------------------
    // 32-bit operations
    // ----------------------
    pub fn and32(x: u32, y: u32) -> u32 {
        x & y
    }
    pub fn or32(x: u32, y: u32) -> u32 {
        x | y
    }
    pub fn xor32(x: u32, y: u32) -> u32 {
        x ^ y
    }
    pub fn not32(x: u32) -> u32 {
        !x
    }
    pub fn has32(x: u32, y: u32) -> bool {
        x & y == y
    }
    pub fn has_any32(x: u32, y: u32) -> bool {
        x & y != 0
    }
    pub fn lshift32(x: u32, n: u32) -> u32 {
        x << n
    }
    pub fn rshift32(x: u32, n: u32) -> u32 {
        x >> n
    }
    pub fn bitmask32(pos: u32) -> u32 {
        1 << pos
    }

    // ----------------------
    // 64-bit operations
    // ----------------------
    pub fn and64(x: u64, y: u64) -> u64 {
        x & y
    }
    pub fn or64(x: u64, y: u64) -> u64 {
        x | y
    }
    pub fn xor64(x: u64, y: u64) -> u64 {
        x ^ y
    }
    pub fn not64(x: u64) -> u64 {
        !x
    }
    pub fn has64(x: u64, y: u64) -> bool {
        x & y == y
    }
    pub fn has_any64(x: u64, y: u64) -> bool {
        x & y != 0
    }
    pub fn lshift64(x: u64, n: u32) -> u64 {
        x << n
    }
    pub fn rshift64(x: u64, n: u32) -> u64 {
        x >> n
    }
    pub fn bitmask64(pos: u32) -> u64 {
        1 << pos
    }
}
