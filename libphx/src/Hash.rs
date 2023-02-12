use ::libc;
pub type int8_t = libc::c_schar;
pub type uint8_t = libc::c_uchar;
pub type uint32_t = libc::c_uint;
pub type uint64_t = libc::c_ulonglong;
pub type uchar = libc::c_uchar;
pub type cstr = *const libc::c_char;
pub type int8 = int8_t;
pub type uint8 = uint8_t;
pub type uint32 = uint32_t;
pub type uint64 = uint64_t;
#[no_mangle]
pub unsafe extern "C" fn Hash_FNV32(
    mut buf: *const libc::c_void,
    mut len: libc::c_int,
) -> uint32 {
    let mut curr: *const uchar = buf as *const uchar;
    let mut end: *const uchar = curr.offset(len as isize);
    let mut self_0: uint32 = 2166136261 as libc::c_uint;
    while curr < end {
        let fresh0 = curr;
        curr = curr.offset(1);
        self_0 ^= *fresh0 as uint32;
        self_0 = (self_0 as libc::c_uint).wrapping_mul(16777619 as libc::c_uint)
            as uint32 as uint32;
    }
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Hash_FNV64(
    mut buf: *const libc::c_void,
    mut len: libc::c_int,
) -> uint64 {
    let mut curr: *const uchar = buf as *const uchar;
    let mut end: *const uchar = curr.offset(len as isize);
    let mut self_0: uint64 = 14695981039346656037 as libc::c_ulonglong;
    while curr < end {
        let fresh1 = curr;
        curr = curr.offset(1);
        self_0 ^= *fresh1 as uint64;
        self_0 = (self_0 as libc::c_ulonglong)
            .wrapping_mul(1099511628211 as libc::c_ulonglong) as uint64 as uint64;
    }
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Hash_FNVStr32(mut s: cstr) -> uint32 {
    let mut self_0: uint32 = 2166136261 as libc::c_uint;
    while *s != 0 {
        let fresh2 = s;
        s = s.offset(1);
        self_0 ^= *fresh2 as uint32;
        self_0 = (self_0 as libc::c_uint).wrapping_mul(16777619 as libc::c_uint)
            as uint32 as uint32;
    }
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Hash_FNVStr64(mut s: cstr) -> uint64 {
    let mut self_0: uint64 = 14695981039346656037 as libc::c_ulonglong;
    while *s != 0 {
        let fresh3 = s;
        s = s.offset(1);
        self_0 ^= *fresh3 as uint64;
        self_0 = (self_0 as libc::c_ulonglong)
            .wrapping_mul(1099511628211 as libc::c_ulonglong) as uint64 as uint64;
    }
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Hash_FNV64_Init() -> uint64 {
    return 14695981039346656037 as libc::c_ulonglong;
}
#[no_mangle]
pub unsafe extern "C" fn Hash_FNV64_Incremental(
    mut self_0: uint64,
    mut buf: *const libc::c_void,
    mut len: libc::c_int,
) -> uint64 {
    let mut curr: *const uchar = buf as *const uchar;
    let mut end: *const uchar = curr.offset(len as isize);
    while curr < end {
        let fresh4 = curr;
        curr = curr.offset(1);
        self_0 ^= *fresh4 as uint64;
        self_0 = (self_0 as libc::c_ulonglong)
            .wrapping_mul(1099511628211 as libc::c_ulonglong) as uint64 as uint64;
    }
    return self_0;
}
#[inline]
unsafe extern "C" fn rotl32(mut x: uint32, mut r: int8) -> uint32 {
    return x << r as libc::c_int | x >> 32 as libc::c_int - r as libc::c_int;
}
#[inline]
unsafe extern "C" fn fmix32(mut h: uint32) -> uint32 {
    h ^= h >> 16 as libc::c_int;
    h = (h as libc::c_uint).wrapping_mul(0x85ebca6b as libc::c_uint) as uint32 as uint32;
    h ^= h >> 13 as libc::c_int;
    h = (h as libc::c_uint).wrapping_mul(0xc2b2ae35 as libc::c_uint) as uint32 as uint32;
    h ^= h >> 16 as libc::c_int;
    return h;
}
#[no_mangle]
pub unsafe extern "C" fn Hash_Murmur3(
    mut key: *const libc::c_void,
    mut len: libc::c_int,
) -> uint32 {
    let mut data: *const uint8 = key as *const uint8;
    let mut h1: uint32 = 0xdeadbeef as libc::c_uint;
    let c1: uint32 = 0xcc9e2d51 as libc::c_uint;
    let c2: uint32 = 0x1b873593 as libc::c_int as uint32;
    let nblocks: libc::c_int = len / 4 as libc::c_int;
    let mut blocks: *const uint32 = data.offset((nblocks * 4 as libc::c_int) as isize)
        as *const uint32;
    let mut i: libc::c_int = -nblocks;
    while i != 0 {
        let mut k1: uint32 = *blocks.offset(i as isize);
        k1 = (k1 as libc::c_uint).wrapping_mul(c1) as uint32 as uint32;
        k1 = rotl32(k1, 15 as libc::c_int as int8);
        k1 = (k1 as libc::c_uint).wrapping_mul(c2) as uint32 as uint32;
        h1 ^= k1;
        h1 = rotl32(h1, 13 as libc::c_int as int8);
        h1 = h1
            .wrapping_mul(5 as libc::c_int as libc::c_uint)
            .wrapping_add(0xe6546b64 as libc::c_uint);
        i += 1;
    }
    let mut tail: *const uint8 = data.offset((nblocks * 4 as libc::c_int) as isize);
    let mut k1_0: uint32 = 0 as libc::c_int as uint32;
    let mut current_block_14: u64;
    match len & 3 as libc::c_int {
        3 => {
            k1_0
                ^= ((*tail.offset(2 as libc::c_int as isize) as libc::c_int)
                    << 16 as libc::c_int) as libc::c_uint;
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
            k1_0
                ^= ((*tail.offset(1 as libc::c_int as isize) as libc::c_int)
                    << 8 as libc::c_int) as libc::c_uint;
            current_block_14 = 15333892231877469626;
        }
        _ => {}
    }
    match current_block_14 {
        15333892231877469626 => {
            k1_0 ^= *tail.offset(0 as libc::c_int as isize) as libc::c_uint;
            k1_0 = (k1_0 as libc::c_uint).wrapping_mul(c1) as uint32 as uint32;
            k1_0 = rotl32(k1_0, 15 as libc::c_int as int8);
            k1_0 = (k1_0 as libc::c_uint).wrapping_mul(c2) as uint32 as uint32;
            h1 ^= k1_0;
        }
        _ => {}
    }
    h1 ^= len as libc::c_uint;
    h1 = fmix32(h1);
    return h1;
}
static mut PRIME64_1: uint64 = 11400714785074694791 as libc::c_ulonglong;
static mut PRIME64_2: uint64 = 14029467366897019727 as libc::c_ulonglong;
static mut PRIME64_3: uint64 = 1609587929392839161 as libc::c_ulonglong;
static mut PRIME64_4: uint64 = 9650029242287828579 as libc::c_ulonglong;
static mut PRIME64_5: uint64 = 2870177450012600261 as libc::c_ulonglong;
unsafe extern "C" fn XXH64_round(mut acc: uint64, mut val: uint64) -> uint64 {
    acc = (acc as libc::c_ulonglong).wrapping_add(val.wrapping_mul(PRIME64_2)) as uint64
        as uint64;
    acc = acc << 31 as libc::c_int | acc >> 64 as libc::c_int - 31 as libc::c_int;
    acc = (acc as libc::c_ulonglong).wrapping_mul(PRIME64_1) as uint64 as uint64;
    return acc;
}
unsafe extern "C" fn XXH64_mergeRound(mut acc: uint64, mut val: uint64) -> uint64 {
    val = XXH64_round(0 as libc::c_int as uint64, val);
    acc ^= val;
    acc = acc.wrapping_mul(PRIME64_1).wrapping_add(PRIME64_4);
    return acc;
}
#[no_mangle]
pub unsafe extern "C" fn Hash_XX64(
    mut buf: *const libc::c_void,
    mut len: libc::c_int,
    mut seed: uint64,
) -> uint64 {
    let mut p: *const uint8 = buf as *const uint8;
    let mut end: *const uint8 = p.offset(len as isize);
    let mut hash: uint64 = 0;
    if len >= 32 as libc::c_int {
        let limit: *const uint8 = end.offset(-(32 as libc::c_int as isize));
        let mut v1: uint64 = seed.wrapping_add(PRIME64_1).wrapping_add(PRIME64_2);
        let mut v2: uint64 = seed.wrapping_add(PRIME64_2);
        let mut v3: uint64 = seed.wrapping_add(0 as libc::c_int as libc::c_ulonglong);
        let mut v4: uint64 = seed.wrapping_sub(PRIME64_1);
        loop {
            v1 = XXH64_round(v1, *(p as *const uint64));
            p = p.offset(8 as libc::c_int as isize);
            v2 = XXH64_round(v2, *(p as *const uint64));
            p = p.offset(8 as libc::c_int as isize);
            v3 = XXH64_round(v3, *(p as *const uint64));
            p = p.offset(8 as libc::c_int as isize);
            v4 = XXH64_round(v4, *(p as *const uint64));
            p = p.offset(8 as libc::c_int as isize);
            if !(p <= limit) {
                break;
            }
        }
        hash = (v1 << 1 as libc::c_int | v1 >> 64 as libc::c_int - 1 as libc::c_int)
            .wrapping_add(
                v2 << 7 as libc::c_int | v2 >> 64 as libc::c_int - 7 as libc::c_int,
            )
            .wrapping_add(
                v3 << 12 as libc::c_int | v3 >> 64 as libc::c_int - 12 as libc::c_int,
            )
            .wrapping_add(
                v4 << 18 as libc::c_int | v4 >> 64 as libc::c_int - 18 as libc::c_int,
            );
        hash = XXH64_mergeRound(hash, v1);
        hash = XXH64_mergeRound(hash, v2);
        hash = XXH64_mergeRound(hash, v3);
        hash = XXH64_mergeRound(hash, v4);
    } else {
        hash = seed.wrapping_add(PRIME64_5);
    }
    hash = (hash as libc::c_ulonglong).wrapping_add(len as uint64) as uint64 as uint64;
    while p.offset(8 as libc::c_int as isize) <= end {
        let k1: uint64 = XXH64_round(0 as libc::c_int as uint64, *(p as *const uint64));
        hash ^= k1;
        hash = (hash << 27 as libc::c_int
            | hash >> 64 as libc::c_int - 27 as libc::c_int)
            .wrapping_mul(PRIME64_1)
            .wrapping_add(PRIME64_4);
        p = p.offset(8 as libc::c_int as isize);
    }
    if p.offset(4 as libc::c_int as isize) <= end {
        hash ^= (*(p as *mut uint32) as uint64).wrapping_mul(PRIME64_1);
        hash = (hash << 23 as libc::c_int
            | hash >> 64 as libc::c_int - 23 as libc::c_int)
            .wrapping_mul(PRIME64_2)
            .wrapping_add(PRIME64_3);
        p = p.offset(4 as libc::c_int as isize);
    }
    while p < end {
        hash ^= (*p as libc::c_ulonglong).wrapping_mul(PRIME64_5);
        hash = (hash << 11 as libc::c_int
            | hash >> 64 as libc::c_int - 11 as libc::c_int)
            .wrapping_mul(PRIME64_1);
        p = p.offset(1);
    }
    hash ^= hash >> 33 as libc::c_int;
    hash = (hash as libc::c_ulonglong).wrapping_mul(PRIME64_2) as uint64 as uint64;
    hash ^= hash >> 29 as libc::c_int;
    hash = (hash as libc::c_ulonglong).wrapping_mul(PRIME64_3) as uint64 as uint64;
    hash ^= hash >> 32 as libc::c_int;
    return hash;
}
