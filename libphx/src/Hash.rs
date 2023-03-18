use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use libc;
pub type uchar = libc::c_uchar;

#[no_mangle]
pub unsafe extern "C" fn Hash_FNV32(mut buf: *const libc::c_void, mut len: i32) -> u32 {
    let mut curr: *const uchar = buf as *const uchar;
    let mut end: *const uchar = curr.offset(len as isize);
    let mut this: u32 = 2166136261_u32;
    while curr < end {
        let fresh0 = curr;
        curr = curr.offset(1);
        this ^= *fresh0 as u32;
        this = this.wrapping_mul(16777619_u32);
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn Hash_FNV64(mut buf: *const libc::c_void, mut len: i32) -> u64 {
    let mut curr: *const uchar = buf as *const uchar;
    let mut end: *const uchar = curr.offset(len as isize);
    let mut this: u64 = 14695981039346656037_u64;
    while curr < end {
        let fresh1 = curr;
        curr = curr.offset(1);
        this ^= *fresh1 as u64;
        this = this.wrapping_mul(1099511628211_u64);
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn Hash_FNVStr32(mut s: *const libc::c_char) -> u32 {
    let mut this: u32 = 2166136261_u32;
    while *s != 0 {
        let fresh2 = s;
        s = s.offset(1);
        this ^= *fresh2 as u32;
        this = this.wrapping_mul(16777619_u32);
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn Hash_FNVStr64(mut s: *const libc::c_char) -> u64 {
    let mut this: u64 = 14695981039346656037_u64;
    while *s != 0 {
        let fresh3 = s;
        s = s.offset(1);
        this ^= *fresh3 as u64;
        this = this.wrapping_mul(1099511628211_u64);
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn Hash_FNV64_Init() -> u64 {
    14695981039346656037_u64
}

#[no_mangle]
pub unsafe extern "C" fn Hash_FNV64_Incremental(
    mut this: u64,
    mut buf: *const libc::c_void,
    mut len: i32,
) -> u64 {
    let mut curr: *const uchar = buf as *const uchar;
    let mut end: *const uchar = curr.offset(len as isize);
    while curr < end {
        let fresh4 = curr;
        curr = curr.offset(1);
        this ^= *fresh4 as u64;
        this = this.wrapping_mul(1099511628211_u64);
    }
    this
}

#[inline]
unsafe extern "C" fn rotl32(mut x: u32, mut r: i8) -> u32 {
    x << r as i32 | x >> 32_i32 - r as i32
}

#[inline]
unsafe extern "C" fn fmix32(mut h: u32) -> u32 {
    h ^= h >> 16_i32;
    h = h.wrapping_mul(0x85ebca6b_u32);
    h ^= h >> 13_i32;
    h = h.wrapping_mul(0xc2b2ae35_u32);
    h ^= h >> 16_i32;
    h
}

#[no_mangle]
pub unsafe extern "C" fn Hash_Murmur3(mut key: *const libc::c_void, mut len: i32) -> u32 {
    let mut data: *const u8 = key as *const u8;
    let mut h1: u32 = 0xdeadbeef_u32;
    let c1: u32 = 0xcc9e2d51_u32;
    let c2: u32 = 0x1b873593_u32;
    let nblocks: i32 = len / 4_i32;
    let mut blocks: *const u32 = data.offset((nblocks * 4_i32) as isize) as *const u32;
    let mut i: i32 = -nblocks;
    while i != 0 {
        let mut k1: u32 = *blocks.offset(i as isize);
        k1 = k1.wrapping_mul(c1);
        k1 = rotl32(k1, 15_i32 as i8);
        k1 = k1.wrapping_mul(c2);
        h1 ^= k1;
        h1 = rotl32(h1, 13_i32 as i8);
        h1 = h1.wrapping_mul(5_u32).wrapping_add(0xe6546b64_u32);
        i += 1;
    }
    let mut tail: *const u8 = data.offset((nblocks * 4_i32) as isize);
    let mut k1_0: u32 = 0_u32;
    let mut current_block_14: u64;
    match len & 3_i32 {
        3 => {
            k1_0 ^= ((*tail.offset(2) as i32) << 16_i32) as u32;
            current_block_14 = 1337185109221498832;
        }
        2 => {
            current_block_14 = 1337185109221498832;
        }
        1 => {
            current_block_14 = 15333892231877469626;
        }
        _ => {
            current_block_14 = 12039483399334584727;
        }
    }
    match current_block_14 {
        1337185109221498832 => {
            k1_0 ^= ((*tail.offset(1) as i32) << 8_i32) as u32;
            current_block_14 = 15333892231877469626;
        }
        _ => {}
    }
    match current_block_14 {
        15333892231877469626 => {
            k1_0 ^= *tail.offset(0) as u32;
            k1_0 = k1_0.wrapping_mul(c1);
            k1_0 = rotl32(k1_0, 15_i32 as i8);
            k1_0 = k1_0.wrapping_mul(c2);
            h1 ^= k1_0;
        }
        _ => {}
    }
    h1 ^= len as u32;
    h1 = fmix32(h1);
    h1
}
static mut PRIME64_1: u64 = 11400714785074694791_u64;

static mut PRIME64_2: u64 = 14029467366897019727_u64;

static mut PRIME64_3: u64 = 1609587929392839161_u64;

static mut PRIME64_4: u64 = 9650029242287828579_u64;

static mut PRIME64_5: u64 = 2870177450012600261_u64;

unsafe extern "C" fn XXH64_round(mut acc: u64, mut val: u64) -> u64 {
    acc = acc.wrapping_add(val.wrapping_mul(PRIME64_2));
    acc = acc << 31_i32 | acc >> 64_i32 - 31_i32;
    acc = acc.wrapping_mul(PRIME64_1);
    acc
}

unsafe extern "C" fn XXH64_mergeRound(mut acc: u64, mut val: u64) -> u64 {
    val = XXH64_round(0_i32 as u64, val);
    acc ^= val;
    acc = acc.wrapping_mul(PRIME64_1).wrapping_add(PRIME64_4);
    acc
}

#[no_mangle]
pub unsafe extern "C" fn Hash_XX64(
    mut buf: *const libc::c_void,
    mut len: i32,
    mut seed: u64,
) -> u64 {
    let mut p: *const u8 = buf as *const u8;
    let mut end: *const u8 = p.offset(len as isize);
    let mut hash: u64 = 0;
    if len >= 32_i32 {
        let limit: *const u8 = end.offset(-(32));
        let mut v1: u64 = seed.wrapping_add(PRIME64_1).wrapping_add(PRIME64_2);
        let mut v2: u64 = seed.wrapping_add(PRIME64_2);
        let mut v3: u64 = seed.wrapping_add(0_u64);
        let mut v4: u64 = seed.wrapping_sub(PRIME64_1);
        loop {
            v1 = XXH64_round(v1, *(p as *const u64));
            p = p.offset(8);
            v2 = XXH64_round(v2, *(p as *const u64));
            p = p.offset(8);
            v3 = XXH64_round(v3, *(p as *const u64));
            p = p.offset(8);
            v4 = XXH64_round(v4, *(p as *const u64));
            p = p.offset(8);
            if !(p <= limit) {
                break;
            }
        }
        hash = (v1 << 1_i32 | v1 >> 64_i32 - 1_i32)
            .wrapping_add(v2 << 7_i32 | v2 >> 64_i32 - 7_i32)
            .wrapping_add(v3 << 12_i32 | v3 >> 64_i32 - 12_i32)
            .wrapping_add(v4 << 18_i32 | v4 >> 64_i32 - 18_i32);
        hash = XXH64_mergeRound(hash, v1);
        hash = XXH64_mergeRound(hash, v2);
        hash = XXH64_mergeRound(hash, v3);
        hash = XXH64_mergeRound(hash, v4);
    } else {
        hash = seed.wrapping_add(PRIME64_5);
    }
    hash = hash.wrapping_add(len as u64);
    while p.offset(8) <= end {
        let k1: u64 = XXH64_round(0_i32 as u64, *(p as *const u64));
        hash ^= k1;
        hash = (hash << 27_i32 | hash >> 64_i32 - 27_i32)
            .wrapping_mul(PRIME64_1)
            .wrapping_add(PRIME64_4);
        p = p.offset(8);
    }
    if p.offset(4) <= end {
        hash ^= (*(p as *mut u32) as u64).wrapping_mul(PRIME64_1);
        hash = (hash << 23_i32 | hash >> 64_i32 - 23_i32)
            .wrapping_mul(PRIME64_2)
            .wrapping_add(PRIME64_3);
        p = p.offset(4);
    }
    while p < end {
        hash ^= (*p as u64).wrapping_mul(PRIME64_5);
        hash = (hash << 11_i32 | hash >> 64_i32 - 11_i32).wrapping_mul(PRIME64_1);
        p = p.offset(1);
    }
    hash ^= hash >> 33_i32;
    hash = hash.wrapping_mul(PRIME64_2);
    hash ^= hash >> 29_i32;
    hash = hash.wrapping_mul(PRIME64_3);
    hash ^= hash >> 32_i32;
    hash
}
