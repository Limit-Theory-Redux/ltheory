



// Fowler–Noll–Vo
#[no_mangle]
pub unsafe extern "C" fn Hash_FNV32(buf: *const libc::c_void, len: i32) -> u32 {
    let mut curr: *const libc::c_uchar = buf as *const libc::c_uchar;
    let end: *const libc::c_uchar = curr.offset(len as isize);
    let mut this: u32 = 2166136261;
    while curr < end {
        let fresh0 = curr;
        curr = curr.offset(1);
        this ^= *fresh0 as u32;
        this = this.wrapping_mul(16777619);
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn Hash_FNV64(buf: *const libc::c_void, len: i32) -> u64 {
    let mut curr: *const libc::c_uchar = buf as *const libc::c_uchar;
    let end: *const libc::c_uchar = curr.offset(len as isize);
    let mut this: u64 = 14695981039346656037;
    while curr < end {
        let fresh1 = curr;
        curr = curr.offset(1);
        this ^= *fresh1 as u64;
        this = this.wrapping_mul(1099511628211);
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn Hash_FNVStr32(mut s: *const libc::c_char) -> u32 {
    let mut this: u32 = 2166136261;
    while *s != 0 {
        let fresh2 = s;
        s = s.offset(1);
        this ^= *fresh2 as u32;
        this = this.wrapping_mul(16777619);
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn Hash_FNVStr64(mut s: *const libc::c_char) -> u64 {
    let mut this: u64 = 14695981039346656037;
    while *s != 0 {
        let fresh3 = s;
        s = s.offset(1);
        this ^= *fresh3 as u64;
        this = this.wrapping_mul(1099511628211);
    }
    this
}

#[no_mangle]
pub extern "C" fn Hash_FNV64_Init() -> u64 {
    14695981039346656037
}

#[no_mangle]
pub unsafe extern "C" fn Hash_FNV64_Incremental(
    mut this: u64,
    buf: *const libc::c_void,
    len: i32,
) -> u64 {
    let mut curr: *const libc::c_uchar = buf as *const libc::c_uchar;
    let end: *const libc::c_uchar = curr.offset(len as isize);
    while curr < end {
        let fresh4 = curr;
        curr = curr.offset(1);
        this ^= *fresh4 as u64;
        this = this.wrapping_mul(1099511628211);
    }
    this
}

/* --- Murmur3 -------------------------------------------------------------- */

#[inline]
extern "C" fn rotl32(x: u32, r: i8) -> u32 {
    x << r as i32 | x >> 32 - r as i32
}

#[inline]
extern "C" fn fmix32(mut h: u32) -> u32 {
    h ^= h >> 16;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> 16;
    h
}

#[no_mangle]
pub unsafe extern "C" fn Hash_Murmur3(key: *const libc::c_void, len: i32) -> u32 {
    let data: *const u8 = key as *const u8;
    let mut h1: u32 = 0xdeadbeef;
    let c1: u32 = 0xcc9e2d51;
    let c2: u32 = 0x1b873593;

    let nblocks: i32 = len / 4;
    let blocks: *const u32 = data.offset((nblocks * 4) as isize) as *const u32;

    let mut i: i32 = -nblocks;
    while i != 0 {
        let mut k1: u32 = *blocks.offset(i as isize);
        k1 = k1.wrapping_mul(c1);
        k1 = rotl32(k1, 15 as i8);
        k1 = k1.wrapping_mul(c2);

        h1 ^= k1;
        h1 = rotl32(h1, 13 as i8);
        h1 = h1.wrapping_mul(5_u32).wrapping_add(0xe6546b64);
        i += 1;
    }

    let tail: *const u8 = data.offset((nblocks * 4) as isize);
    let mut k1: u32 = 0;

    if len & 3 == 3 {
        k1 ^= ((*tail.offset(2) as i32) << 16) as u32;
    }
    if len & 3 >= 2 {
        k1 ^= ((*tail.offset(1) as i32) << 8) as u32;
    }
    if len & 3 >= 1 {
        k1 ^= *tail.offset(0) as u32;
        k1 = k1.wrapping_mul(c1);
        k1 = rotl32(k1, 15 as i8);
        k1 = k1.wrapping_mul(c2);
        h1 ^= k1;
    }

    h1 ^= len as u32;
    h1 = fmix32(h1);
    h1
}

/* --- XXHASH64 --------------------------------------------------------------
 *   https://github.com/Cyan4973/xxHash/blob/dev/xxhash.c
 * -------------------------------------------------------------------------- */

const PRIME64_1: u64 = 11400714785074694791;
const PRIME64_2: u64 = 14029467366897019727;
const PRIME64_3: u64 = 1609587929392839161;
const PRIME64_4: u64 = 9650029242287828579;
const PRIME64_5: u64 = 2870177450012600261;

fn XXH64_round(mut acc: u64, val: u64) -> u64 {
    acc = acc.wrapping_add(val.wrapping_mul(PRIME64_2));
    acc = acc << 31 | acc >> 64 - 31;
    acc = acc.wrapping_mul(PRIME64_1);
    acc
}

fn XXH64_mergeRound(mut acc: u64, val: u64) -> u64 {
    acc ^= XXH64_round(0, val);
    acc = acc.wrapping_mul(PRIME64_1).wrapping_add(PRIME64_4);
    acc
}

#[no_mangle]
pub unsafe extern "C" fn Hash_XX64(buf: *const libc::c_void, len: i32, seed: u64) -> u64 {
    let mut p: *const u8 = buf as *const u8;
    let end: *const u8 = p.offset(len as isize);
    let mut hash: u64 = 0;

    if len >= 32 {
        let limit: *const u8 = end.offset(-(32));
        let mut v1: u64 = seed.wrapping_add(PRIME64_1).wrapping_add(PRIME64_2);
        let mut v2: u64 = seed.wrapping_add(PRIME64_2);
        let mut v3: u64 = seed.wrapping_add(0);
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
            if p > limit {
                break;
            }
        }
        hash = (v1 << 1 | v1 >> 64 - 1)
            .wrapping_add(v2 << 7 | v2 >> 64 - 7)
            .wrapping_add(v3 << 12 | v3 >> 64 - 12)
            .wrapping_add(v4 << 18 | v4 >> 64 - 18);
        hash = XXH64_mergeRound(hash, v1);
        hash = XXH64_mergeRound(hash, v2);
        hash = XXH64_mergeRound(hash, v3);
        hash = XXH64_mergeRound(hash, v4);
    } else {
        hash = seed.wrapping_add(PRIME64_5);
    }

    hash = hash.wrapping_add(len as u64);

    while p.offset(8) <= end {
        let k1: u64 = XXH64_round(0, *(p as *const u64));
        hash ^= k1;
        hash = (hash << 27 | hash >> 64 - 27)
            .wrapping_mul(PRIME64_1)
            .wrapping_add(PRIME64_4);
        p = p.offset(8);
    }

    if p.offset(4) <= end {
        hash ^= (*(p as *mut u32) as u64).wrapping_mul(PRIME64_1);
        hash = (hash << 23 | hash >> 64 - 23)
            .wrapping_mul(PRIME64_2)
            .wrapping_add(PRIME64_3);
        p = p.offset(4);
    }

    while p < end {
        hash ^= (*p as u64).wrapping_mul(PRIME64_5);
        hash = (hash << 11 | hash >> 64 - 11_i32).wrapping_mul(PRIME64_1);
        p = p.offset(1);
    }

    hash ^= hash >> 33;
    hash = hash.wrapping_mul(PRIME64_2);
    hash ^= hash >> 29;
    hash = hash.wrapping_mul(PRIME64_3);
    hash ^= hash >> 32;
    hash
}
