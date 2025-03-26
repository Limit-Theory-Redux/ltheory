const FNV32_INIT: u32 = 2166136261;
const FNV32_MUL: u32 = 16777619;
const FNV64_INIT: u64 = 14695981039346656037;
const FNV64_MUL: u64 = 1099511628211;

// const PRIME64_1: u64 = 11400714785074694791;
// const PRIME64_2: u64 = 14029467366897019727;
// const PRIME64_3: u64 = 1609587929392839161;
// const PRIME64_4: u64 = 9650029242287828579;
// const PRIME64_5: u64 = 2870177450012600261;

pub struct Hash;

#[luajit_ffi_gen::luajit_ffi]
impl Hash {
    /// Fowler–Noll–Vo
    #[bind(name = "FNV32")]
    pub fn fnv32(buf: &[u8]) -> u32 {
        let mut this = FNV32_INIT;
        buf.iter().for_each(|b| {
            this ^= *b as u32;
            this = this.wrapping_mul(FNV32_MUL);
        });
        this
    }

    #[bind(name = "FNV64")]
    pub fn fnv64(buf: &[u8]) -> u64 {
        let mut this = FNV64_INIT;
        buf.iter().for_each(|b| {
            this ^= *b as u64;
            this = this.wrapping_mul(FNV64_MUL);
        });
        this
    }

    #[bind(name = "FNVStr32")]
    pub fn fnv_str32(s: &str) -> u32 {
        let buf = s.as_bytes();
        Self::fnv32(buf)
    }

    #[bind(name = "FNVStr64")]
    pub fn fnv_str64(s: &str) -> u64 {
        let buf = s.as_bytes();
        Self::fnv64(buf)
    }

    #[bind(name = "FNV64_Init")]
    pub const fn fnv64_init() -> u64 {
        FNV64_INIT
    }

    #[bind(name = "FNV64_Incremental")]
    pub fn fnv64_incremental(mut this: u64, buf: &[u8]) -> u64 {
        buf.iter().for_each(|b| {
            this ^= *b as u64;
            this = this.wrapping_mul(FNV64_MUL);
        });
        this
    }

    // pub fn murmur3(key: &[u8]) -> u32 {
    //     let data: *const u8 = key as *const u8;
    //     let mut h1: u32 = 0xdeadbeef;
    //     let c1: u32 = 0xcc9e2d51;
    //     let c2: u32 = 0x1b873593;

    //     let nblocks: i32 = len / 4;
    //     let blocks: *const u32 = data.offset((nblocks * 4) as isize) as *const u32;

    //     let mut i: i32 = -nblocks;
    //     while i != 0 {
    //         let mut k1: u32 = *blocks.offset(i as isize);
    //         k1 = k1.wrapping_mul(c1);
    //         k1 = rotl32(k1, 15 as i8);
    //         k1 = k1.wrapping_mul(c2);

    //         h1 ^= k1;
    //         h1 = rotl32(h1, 13 as i8);
    //         h1 = h1.wrapping_mul(5_u32).wrapping_add(0xe6546b64);
    //         i += 1;
    //     }

    //     let tail: *const u8 = data.offset((nblocks * 4) as isize);
    //     let mut k1: u32 = 0;

    //     if len & 3 == 3 {
    //         k1 ^= ((*tail.offset(2) as i32) << 16) as u32;
    //     }
    //     if len & 3 >= 2 {
    //         k1 ^= ((*tail.offset(1) as i32) << 8) as u32;
    //     }
    //     if len & 3 >= 1 {
    //         k1 ^= *tail.offset(0) as u32;
    //         k1 = k1.wrapping_mul(c1);
    //         k1 = rotl32(k1, 15 as i8);
    //         k1 = k1.wrapping_mul(c2);
    //         h1 ^= k1;
    //     }

    //     h1 ^= len as u32;
    //     h1 = fmix32(h1);
    //     h1
    // }

    // #[bind(name = "XX64")]
    // pub fn xx64(buf: &[u8], seed: u64) -> u64 {
    //     let mut p: *const u8 = buf as *const u8;
    //     let end: *const u8 = p.offset(len as isize);
    //     let mut hash = if len >= 32 {
    //         let limit: *const u8 = end.offset(-(32));
    //         let mut v1: u64 = seed.wrapping_add(PRIME64_1).wrapping_add(PRIME64_2);
    //         let mut v2: u64 = seed.wrapping_add(PRIME64_2);
    //         let mut v3: u64 = seed.wrapping_add(0);
    //         let mut v4: u64 = seed.wrapping_sub(PRIME64_1);

    //         loop {
    //             v1 = xxh64_round(v1, *(p as *const u64));
    //             p = p.offset(8);
    //             v2 = xxh64_round(v2, *(p as *const u64));
    //             p = p.offset(8);
    //             v3 = xxh64_round(v3, *(p as *const u64));
    //             p = p.offset(8);
    //             v4 = xxh64_round(v4, *(p as *const u64));
    //             p = p.offset(8);
    //             if p > limit {
    //                 break;
    //             }
    //         }
    //         let mut hash = v1
    //             .rotate_left(1)
    //             .wrapping_add(v2.rotate_left(7))
    //             .wrapping_add(v3.rotate_left(12))
    //             .wrapping_add(v4.rotate_left(18));
    //         hash = xxh64_merge_round(hash, v1);
    //         hash = xxh64_merge_round(hash, v2);
    //         hash = xxh64_merge_round(hash, v3);
    //         xxh64_merge_round(hash, v4)
    //     } else {
    //         seed.wrapping_add(PRIME64_5)
    //     };

    //     hash = hash.wrapping_add(len as u64);

    //     while p.offset(8) <= end {
    //         let k1: u64 = xxh64_round(0, *(p as *const u64));
    //         hash ^= k1;
    //         hash = hash
    //             .rotate_left(27)
    //             .wrapping_mul(PRIME64_1)
    //             .wrapping_add(PRIME64_4);
    //         p = p.offset(8);
    //     }

    //     if p.offset(4) <= end {
    //         hash ^= (*(p as *mut u32) as u64).wrapping_mul(PRIME64_1);
    //         hash = hash
    //             .rotate_left(23)
    //             .wrapping_mul(PRIME64_2)
    //             .wrapping_add(PRIME64_3);
    //         p = p.offset(4);
    //     }

    //     while p < end {
    //         hash ^= (*p as u64).wrapping_mul(PRIME64_5);
    //         hash = hash.rotate_left(11).wrapping_mul(PRIME64_1);
    //         p = p.offset(1);
    //     }

    //     hash ^= hash >> 33;
    //     hash = hash.wrapping_mul(PRIME64_2);
    //     hash ^= hash >> 29;
    //     hash = hash.wrapping_mul(PRIME64_3);
    //     hash ^= hash >> 32;
    //     hash
    // }
}

// // --- Murmur3 --------------------------------------------------------------

// #[inline]
// fn rotl32(x: u32, r: i8) -> u32 {
//     (x << r as i32) | (x >> (32 - r as i32))
// }

// #[inline]
// fn fmix32(mut h: u32) -> u32 {
//     h ^= h >> 16;
//     h = h.wrapping_mul(0x85ebca6b);
//     h ^= h >> 13;
//     h = h.wrapping_mul(0xc2b2ae35);
//     h ^= h >> 16;
//     h
// }

// // --- XXHASH64 --------------------------------------------------------------
// //   https://github.com/Cyan4973/xxHash/blob/dev/xxhash.c
// // ---------------------------------------------------------------------------

// fn xxh64_round(mut acc: u64, val: u64) -> u64 {
//     acc = acc.wrapping_add(val.wrapping_mul(PRIME64_2));
//     acc = acc.rotate_left(31);
//     acc = acc.wrapping_mul(PRIME64_1);
//     acc
// }

// fn xxh64_merge_round(mut acc: u64, val: u64) -> u64 {
//     acc ^= xxh64_round(0, val);
//     acc = acc.wrapping_mul(PRIME64_1).wrapping_add(PRIME64_4);
//     acc
// }
