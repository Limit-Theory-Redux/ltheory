use ::libc;
extern "C" {
    pub type __sFILEX;
    fn memcmp(
        _: *const libc::c_void,
        _: *const libc::c_void,
        _: libc::c_ulong,
    ) -> libc::c_int;
    fn memcpy(
        _: *mut libc::c_void,
        _: *const libc::c_void,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn memmove(
        _: *mut libc::c_void,
        _: *const libc::c_void,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn abs(_: libc::c_int) -> libc::c_int;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn realloc(_: *mut libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
    fn fclose(_: *mut FILE) -> libc::c_int;
    fn fwrite(
        _: *const libc::c_void,
        _: libc::c_ulong,
        _: libc::c_ulong,
        _: *mut FILE,
    ) -> libc::c_ulong;
    fn fopen(_: *const libc::c_char, _: *const libc::c_char) -> *mut FILE;
    fn sprintf(_: *mut libc::c_char, _: *const libc::c_char, _: ...) -> libc::c_int;
    fn frexp(_: libc::c_double, _: *mut libc::c_int) -> libc::c_double;
}
pub type __builtin_va_list = *mut libc::c_char;
pub type __int64_t = libc::c_longlong;
pub type __darwin_size_t = libc::c_ulong;
pub type __darwin_va_list = __builtin_va_list;
pub type __darwin_off_t = __int64_t;
pub type size_t = __darwin_size_t;
pub type uchar = libc::c_uchar;
pub type cstr = *const libc::c_char;
pub type stbiw_uint32 = libc::c_uint;
pub type FILE = __sFILE;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sFILE {
    pub _p: *mut libc::c_uchar,
    pub _r: libc::c_int,
    pub _w: libc::c_int,
    pub _flags: libc::c_short,
    pub _file: libc::c_short,
    pub _bf: __sbuf,
    pub _lbfsize: libc::c_int,
    pub _cookie: *mut libc::c_void,
    pub _close: Option::<unsafe extern "C" fn(*mut libc::c_void) -> libc::c_int>,
    pub _read: Option::<
        unsafe extern "C" fn(
            *mut libc::c_void,
            *mut libc::c_char,
            libc::c_int,
        ) -> libc::c_int,
    >,
    pub _seek: Option::<
        unsafe extern "C" fn(*mut libc::c_void, fpos_t, libc::c_int) -> fpos_t,
    >,
    pub _write: Option::<
        unsafe extern "C" fn(
            *mut libc::c_void,
            *const libc::c_char,
            libc::c_int,
        ) -> libc::c_int,
    >,
    pub _ub: __sbuf,
    pub _extra: *mut __sFILEX,
    pub _ur: libc::c_int,
    pub _ubuf: [libc::c_uchar; 3],
    pub _nbuf: [libc::c_uchar; 1],
    pub _lb: __sbuf,
    pub _blksize: libc::c_int,
    pub _offset: fpos_t,
}
pub type fpos_t = __darwin_off_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sbuf {
    pub _base: *mut libc::c_uchar,
    pub _size: libc::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__write_context {
    pub func: Option::<stbi_write_func>,
    pub context: *mut libc::c_void,
    pub buffer: [libc::c_uchar; 64],
    pub buf_used: libc::c_int,
}
pub type stbi_write_func = unsafe extern "C" fn(
    *mut libc::c_void,
    *mut libc::c_void,
    libc::c_int,
) -> ();
pub type va_list = __darwin_va_list;
#[no_mangle]
pub static mut stbi_write_tga_with_rle: libc::c_int = 1 as libc::c_int;
#[no_mangle]
pub static mut stbi_write_png_compression_level: libc::c_int = 8 as libc::c_int;
#[no_mangle]
pub static mut stbi_write_force_png_filter: libc::c_int = -(1 as libc::c_int);
#[no_mangle]
pub unsafe extern "C" fn stbi_write_png(
    mut filename: *const libc::c_char,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut comp: libc::c_int,
    mut data: *const libc::c_void,
    mut stride_bytes: libc::c_int,
) -> libc::c_int {
    let mut f: *mut FILE = 0 as *mut FILE;
    let mut len: libc::c_int = 0;
    let mut png: *mut libc::c_uchar = stbi_write_png_to_mem(
        data as *const libc::c_uchar,
        stride_bytes,
        x,
        y,
        comp,
        &mut len,
    );
    if png.is_null() {
        return 0 as libc::c_int;
    }
    f = stbiw__fopen(filename, b"wb\0" as *const u8 as *const libc::c_char);
    if f.is_null() {
        free(png as *mut libc::c_void);
        return 0 as libc::c_int;
    }
    fwrite(
        png as *const libc::c_void,
        1 as libc::c_int as libc::c_ulong,
        len as libc::c_ulong,
        f,
    );
    fclose(f);
    free(png as *mut libc::c_void);
    return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_write_png_to_mem(
    mut pixels: *const libc::c_uchar,
    mut stride_bytes: libc::c_int,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut n: libc::c_int,
    mut out_len: *mut libc::c_int,
) -> *mut libc::c_uchar {
    let mut force_filter: libc::c_int = stbi_write_force_png_filter;
    let mut ctype: [libc::c_int; 5] = [
        -(1 as libc::c_int),
        0 as libc::c_int,
        4 as libc::c_int,
        2 as libc::c_int,
        6 as libc::c_int,
    ];
    let mut sig: [libc::c_uchar; 8] = [
        137 as libc::c_int as libc::c_uchar,
        80 as libc::c_int as libc::c_uchar,
        78 as libc::c_int as libc::c_uchar,
        71 as libc::c_int as libc::c_uchar,
        13 as libc::c_int as libc::c_uchar,
        10 as libc::c_int as libc::c_uchar,
        26 as libc::c_int as libc::c_uchar,
        10 as libc::c_int as libc::c_uchar,
    ];
    let mut out: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
    let mut o: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
    let mut filt: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
    let mut zlib: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
    let mut line_buffer: *mut libc::c_schar = 0 as *mut libc::c_schar;
    let mut j: libc::c_int = 0;
    let mut zlen: libc::c_int = 0;
    if stride_bytes == 0 as libc::c_int {
        stride_bytes = x * n;
    }
    if force_filter >= 5 as libc::c_int {
        force_filter = -(1 as libc::c_int);
    }
    filt = malloc(((x * n + 1 as libc::c_int) * y) as libc::c_ulong)
        as *mut libc::c_uchar;
    if filt.is_null() {
        return 0 as *mut libc::c_uchar;
    }
    line_buffer = malloc((x * n) as libc::c_ulong) as *mut libc::c_schar;
    if line_buffer.is_null() {
        free(filt as *mut libc::c_void);
        return 0 as *mut libc::c_uchar;
    }
    j = 0 as libc::c_int;
    while j < y {
        let mut filter_type: libc::c_int = 0;
        if force_filter > -(1 as libc::c_int) {
            filter_type = force_filter;
            stbiw__encode_png_line(
                pixels as *mut libc::c_uchar,
                stride_bytes,
                x,
                y,
                j,
                n,
                force_filter,
                line_buffer,
            );
        } else {
            let mut best_filter: libc::c_int = 0 as libc::c_int;
            let mut best_filter_val: libc::c_int = 0x7fffffff as libc::c_int;
            let mut est: libc::c_int = 0;
            let mut i: libc::c_int = 0;
            filter_type = 0 as libc::c_int;
            while filter_type < 5 as libc::c_int {
                stbiw__encode_png_line(
                    pixels as *mut libc::c_uchar,
                    stride_bytes,
                    x,
                    y,
                    j,
                    n,
                    filter_type,
                    line_buffer,
                );
                est = 0 as libc::c_int;
                i = 0 as libc::c_int;
                while i < x * n {
                    est += abs(*line_buffer.offset(i as isize) as libc::c_int);
                    i += 1;
                }
                if est < best_filter_val {
                    best_filter_val = est;
                    best_filter = filter_type;
                }
                filter_type += 1;
            }
            if filter_type != best_filter {
                stbiw__encode_png_line(
                    pixels as *mut libc::c_uchar,
                    stride_bytes,
                    x,
                    y,
                    j,
                    n,
                    best_filter,
                    line_buffer,
                );
                filter_type = best_filter;
            }
        }
        *filt
            .offset(
                (j * (x * n + 1 as libc::c_int)) as isize,
            ) = filter_type as libc::c_uchar;
        memmove(
            filt
                .offset((j * (x * n + 1 as libc::c_int)) as isize)
                .offset(1 as libc::c_int as isize) as *mut libc::c_void,
            line_buffer as *const libc::c_void,
            (x * n) as libc::c_ulong,
        );
        j += 1;
    }
    free(line_buffer as *mut libc::c_void);
    zlib = stbi_zlib_compress(
        filt,
        y * (x * n + 1 as libc::c_int),
        &mut zlen,
        stbi_write_png_compression_level,
    );
    free(filt as *mut libc::c_void);
    if zlib.is_null() {
        return 0 as *mut libc::c_uchar;
    }
    out = malloc(
        (8 as libc::c_int + 12 as libc::c_int + 13 as libc::c_int + 12 as libc::c_int
            + zlen + 12 as libc::c_int) as libc::c_ulong,
    ) as *mut libc::c_uchar;
    if out.is_null() {
        return 0 as *mut libc::c_uchar;
    }
    *out_len = 8 as libc::c_int + 12 as libc::c_int + 13 as libc::c_int
        + 12 as libc::c_int + zlen + 12 as libc::c_int;
    o = out;
    memmove(
        o as *mut libc::c_void,
        sig.as_mut_ptr() as *const libc::c_void,
        8 as libc::c_int as libc::c_ulong,
    );
    o = o.offset(8 as libc::c_int as isize);
    *o
        .offset(
            0 as libc::c_int as isize,
        ) = (13 as libc::c_int >> 24 as libc::c_int & 0xff as libc::c_int)
        as libc::c_uchar;
    *o
        .offset(
            1 as libc::c_int as isize,
        ) = (13 as libc::c_int >> 16 as libc::c_int & 0xff as libc::c_int)
        as libc::c_uchar;
    *o
        .offset(
            2 as libc::c_int as isize,
        ) = (13 as libc::c_int >> 8 as libc::c_int & 0xff as libc::c_int)
        as libc::c_uchar;
    *o
        .offset(
            3 as libc::c_int as isize,
        ) = (13 as libc::c_int & 0xff as libc::c_int) as libc::c_uchar;
    o = o.offset(4 as libc::c_int as isize);
    *o
        .offset(
            0 as libc::c_int as isize,
        ) = ((*::core::mem::transmute::<
        &[u8; 5],
        &[libc::c_char; 5],
    >(b"IHDR\0"))[0 as libc::c_int as usize] as libc::c_int & 0xff as libc::c_int)
        as libc::c_uchar;
    *o
        .offset(
            1 as libc::c_int as isize,
        ) = ((*::core::mem::transmute::<
        &[u8; 5],
        &[libc::c_char; 5],
    >(b"IHDR\0"))[1 as libc::c_int as usize] as libc::c_int & 0xff as libc::c_int)
        as libc::c_uchar;
    *o
        .offset(
            2 as libc::c_int as isize,
        ) = ((*::core::mem::transmute::<
        &[u8; 5],
        &[libc::c_char; 5],
    >(b"IHDR\0"))[2 as libc::c_int as usize] as libc::c_int & 0xff as libc::c_int)
        as libc::c_uchar;
    *o
        .offset(
            3 as libc::c_int as isize,
        ) = ((*::core::mem::transmute::<
        &[u8; 5],
        &[libc::c_char; 5],
    >(b"IHDR\0"))[3 as libc::c_int as usize] as libc::c_int & 0xff as libc::c_int)
        as libc::c_uchar;
    o = o.offset(4 as libc::c_int as isize);
    *o
        .offset(
            0 as libc::c_int as isize,
        ) = (x >> 24 as libc::c_int & 0xff as libc::c_int) as libc::c_uchar;
    *o
        .offset(
            1 as libc::c_int as isize,
        ) = (x >> 16 as libc::c_int & 0xff as libc::c_int) as libc::c_uchar;
    *o
        .offset(
            2 as libc::c_int as isize,
        ) = (x >> 8 as libc::c_int & 0xff as libc::c_int) as libc::c_uchar;
    *o.offset(3 as libc::c_int as isize) = (x & 0xff as libc::c_int) as libc::c_uchar;
    o = o.offset(4 as libc::c_int as isize);
    *o
        .offset(
            0 as libc::c_int as isize,
        ) = (y >> 24 as libc::c_int & 0xff as libc::c_int) as libc::c_uchar;
    *o
        .offset(
            1 as libc::c_int as isize,
        ) = (y >> 16 as libc::c_int & 0xff as libc::c_int) as libc::c_uchar;
    *o
        .offset(
            2 as libc::c_int as isize,
        ) = (y >> 8 as libc::c_int & 0xff as libc::c_int) as libc::c_uchar;
    *o.offset(3 as libc::c_int as isize) = (y & 0xff as libc::c_int) as libc::c_uchar;
    o = o.offset(4 as libc::c_int as isize);
    let fresh0 = o;
    o = o.offset(1);
    *fresh0 = 8 as libc::c_int as libc::c_uchar;
    let fresh1 = o;
    o = o.offset(1);
    *fresh1 = (ctype[n as usize] & 0xff as libc::c_int) as libc::c_uchar;
    let fresh2 = o;
    o = o.offset(1);
    *fresh2 = 0 as libc::c_int as libc::c_uchar;
    let fresh3 = o;
    o = o.offset(1);
    *fresh3 = 0 as libc::c_int as libc::c_uchar;
    let fresh4 = o;
    o = o.offset(1);
    *fresh4 = 0 as libc::c_int as libc::c_uchar;
    stbiw__wpcrc(&mut o, 13 as libc::c_int);
    *o
        .offset(
            0 as libc::c_int as isize,
        ) = (zlen >> 24 as libc::c_int & 0xff as libc::c_int) as libc::c_uchar;
    *o
        .offset(
            1 as libc::c_int as isize,
        ) = (zlen >> 16 as libc::c_int & 0xff as libc::c_int) as libc::c_uchar;
    *o
        .offset(
            2 as libc::c_int as isize,
        ) = (zlen >> 8 as libc::c_int & 0xff as libc::c_int) as libc::c_uchar;
    *o.offset(3 as libc::c_int as isize) = (zlen & 0xff as libc::c_int) as libc::c_uchar;
    o = o.offset(4 as libc::c_int as isize);
    *o
        .offset(
            0 as libc::c_int as isize,
        ) = ((*::core::mem::transmute::<
        &[u8; 5],
        &[libc::c_char; 5],
    >(b"IDAT\0"))[0 as libc::c_int as usize] as libc::c_int & 0xff as libc::c_int)
        as libc::c_uchar;
    *o
        .offset(
            1 as libc::c_int as isize,
        ) = ((*::core::mem::transmute::<
        &[u8; 5],
        &[libc::c_char; 5],
    >(b"IDAT\0"))[1 as libc::c_int as usize] as libc::c_int & 0xff as libc::c_int)
        as libc::c_uchar;
    *o
        .offset(
            2 as libc::c_int as isize,
        ) = ((*::core::mem::transmute::<
        &[u8; 5],
        &[libc::c_char; 5],
    >(b"IDAT\0"))[2 as libc::c_int as usize] as libc::c_int & 0xff as libc::c_int)
        as libc::c_uchar;
    *o
        .offset(
            3 as libc::c_int as isize,
        ) = ((*::core::mem::transmute::<
        &[u8; 5],
        &[libc::c_char; 5],
    >(b"IDAT\0"))[3 as libc::c_int as usize] as libc::c_int & 0xff as libc::c_int)
        as libc::c_uchar;
    o = o.offset(4 as libc::c_int as isize);
    memmove(o as *mut libc::c_void, zlib as *const libc::c_void, zlen as libc::c_ulong);
    o = o.offset(zlen as isize);
    free(zlib as *mut libc::c_void);
    stbiw__wpcrc(&mut o, zlen);
    *o
        .offset(
            0 as libc::c_int as isize,
        ) = (0 as libc::c_int >> 24 as libc::c_int & 0xff as libc::c_int)
        as libc::c_uchar;
    *o
        .offset(
            1 as libc::c_int as isize,
        ) = (0 as libc::c_int >> 16 as libc::c_int & 0xff as libc::c_int)
        as libc::c_uchar;
    *o
        .offset(
            2 as libc::c_int as isize,
        ) = (0 as libc::c_int >> 8 as libc::c_int & 0xff as libc::c_int)
        as libc::c_uchar;
    *o
        .offset(
            3 as libc::c_int as isize,
        ) = (0 as libc::c_int & 0xff as libc::c_int) as libc::c_uchar;
    o = o.offset(4 as libc::c_int as isize);
    *o
        .offset(
            0 as libc::c_int as isize,
        ) = ((*::core::mem::transmute::<
        &[u8; 5],
        &[libc::c_char; 5],
    >(b"IEND\0"))[0 as libc::c_int as usize] as libc::c_int & 0xff as libc::c_int)
        as libc::c_uchar;
    *o
        .offset(
            1 as libc::c_int as isize,
        ) = ((*::core::mem::transmute::<
        &[u8; 5],
        &[libc::c_char; 5],
    >(b"IEND\0"))[1 as libc::c_int as usize] as libc::c_int & 0xff as libc::c_int)
        as libc::c_uchar;
    *o
        .offset(
            2 as libc::c_int as isize,
        ) = ((*::core::mem::transmute::<
        &[u8; 5],
        &[libc::c_char; 5],
    >(b"IEND\0"))[2 as libc::c_int as usize] as libc::c_int & 0xff as libc::c_int)
        as libc::c_uchar;
    *o
        .offset(
            3 as libc::c_int as isize,
        ) = ((*::core::mem::transmute::<
        &[u8; 5],
        &[libc::c_char; 5],
    >(b"IEND\0"))[3 as libc::c_int as usize] as libc::c_int & 0xff as libc::c_int)
        as libc::c_uchar;
    o = o.offset(4 as libc::c_int as isize);
    stbiw__wpcrc(&mut o, 0 as libc::c_int);
    return out;
}
unsafe extern "C" fn stbiw__wpcrc(
    mut data: *mut *mut libc::c_uchar,
    mut len: libc::c_int,
) {
    let mut crc: libc::c_uint = stbiw__crc32(
        (*data).offset(-(len as isize)).offset(-(4 as libc::c_int as isize)),
        len + 4 as libc::c_int,
    );
    *(*data)
        .offset(
            0 as libc::c_int as isize,
        ) = (crc >> 24 as libc::c_int & 0xff as libc::c_int as libc::c_uint)
        as libc::c_uchar;
    *(*data)
        .offset(
            1 as libc::c_int as isize,
        ) = (crc >> 16 as libc::c_int & 0xff as libc::c_int as libc::c_uint)
        as libc::c_uchar;
    *(*data)
        .offset(
            2 as libc::c_int as isize,
        ) = (crc >> 8 as libc::c_int & 0xff as libc::c_int as libc::c_uint)
        as libc::c_uchar;
    *(*data)
        .offset(
            3 as libc::c_int as isize,
        ) = (crc & 0xff as libc::c_int as libc::c_uint) as libc::c_uchar;
    *data = (*data).offset(4 as libc::c_int as isize);
}
unsafe extern "C" fn stbiw__crc32(
    mut buffer: *mut libc::c_uchar,
    mut len: libc::c_int,
) -> libc::c_uint {
    static mut crc_table: [libc::c_uint; 256] = [
        0 as libc::c_int as libc::c_uint,
        0x77073096 as libc::c_int as libc::c_uint,
        0xee0e612c as libc::c_uint,
        0x990951ba as libc::c_uint,
        0x76dc419 as libc::c_int as libc::c_uint,
        0x706af48f as libc::c_int as libc::c_uint,
        0xe963a535 as libc::c_uint,
        0x9e6495a3 as libc::c_uint,
        0xedb8832 as libc::c_int as libc::c_uint,
        0x79dcb8a4 as libc::c_int as libc::c_uint,
        0xe0d5e91e as libc::c_uint,
        0x97d2d988 as libc::c_uint,
        0x9b64c2b as libc::c_int as libc::c_uint,
        0x7eb17cbd as libc::c_int as libc::c_uint,
        0xe7b82d07 as libc::c_uint,
        0x90bf1d91 as libc::c_uint,
        0x1db71064 as libc::c_int as libc::c_uint,
        0x6ab020f2 as libc::c_int as libc::c_uint,
        0xf3b97148 as libc::c_uint,
        0x84be41de as libc::c_uint,
        0x1adad47d as libc::c_int as libc::c_uint,
        0x6ddde4eb as libc::c_int as libc::c_uint,
        0xf4d4b551 as libc::c_uint,
        0x83d385c7 as libc::c_uint,
        0x136c9856 as libc::c_int as libc::c_uint,
        0x646ba8c0 as libc::c_int as libc::c_uint,
        0xfd62f97a as libc::c_uint,
        0x8a65c9ec as libc::c_uint,
        0x14015c4f as libc::c_int as libc::c_uint,
        0x63066cd9 as libc::c_int as libc::c_uint,
        0xfa0f3d63 as libc::c_uint,
        0x8d080df5 as libc::c_uint,
        0x3b6e20c8 as libc::c_int as libc::c_uint,
        0x4c69105e as libc::c_int as libc::c_uint,
        0xd56041e4 as libc::c_uint,
        0xa2677172 as libc::c_uint,
        0x3c03e4d1 as libc::c_int as libc::c_uint,
        0x4b04d447 as libc::c_int as libc::c_uint,
        0xd20d85fd as libc::c_uint,
        0xa50ab56b as libc::c_uint,
        0x35b5a8fa as libc::c_int as libc::c_uint,
        0x42b2986c as libc::c_int as libc::c_uint,
        0xdbbbc9d6 as libc::c_uint,
        0xacbcf940 as libc::c_uint,
        0x32d86ce3 as libc::c_int as libc::c_uint,
        0x45df5c75 as libc::c_int as libc::c_uint,
        0xdcd60dcf as libc::c_uint,
        0xabd13d59 as libc::c_uint,
        0x26d930ac as libc::c_int as libc::c_uint,
        0x51de003a as libc::c_int as libc::c_uint,
        0xc8d75180 as libc::c_uint,
        0xbfd06116 as libc::c_uint,
        0x21b4f4b5 as libc::c_int as libc::c_uint,
        0x56b3c423 as libc::c_int as libc::c_uint,
        0xcfba9599 as libc::c_uint,
        0xb8bda50f as libc::c_uint,
        0x2802b89e as libc::c_int as libc::c_uint,
        0x5f058808 as libc::c_int as libc::c_uint,
        0xc60cd9b2 as libc::c_uint,
        0xb10be924 as libc::c_uint,
        0x2f6f7c87 as libc::c_int as libc::c_uint,
        0x58684c11 as libc::c_int as libc::c_uint,
        0xc1611dab as libc::c_uint,
        0xb6662d3d as libc::c_uint,
        0x76dc4190 as libc::c_int as libc::c_uint,
        0x1db7106 as libc::c_int as libc::c_uint,
        0x98d220bc as libc::c_uint,
        0xefd5102a as libc::c_uint,
        0x71b18589 as libc::c_int as libc::c_uint,
        0x6b6b51f as libc::c_int as libc::c_uint,
        0x9fbfe4a5 as libc::c_uint,
        0xe8b8d433 as libc::c_uint,
        0x7807c9a2 as libc::c_int as libc::c_uint,
        0xf00f934 as libc::c_int as libc::c_uint,
        0x9609a88e as libc::c_uint,
        0xe10e9818 as libc::c_uint,
        0x7f6a0dbb as libc::c_int as libc::c_uint,
        0x86d3d2d as libc::c_int as libc::c_uint,
        0x91646c97 as libc::c_uint,
        0xe6635c01 as libc::c_uint,
        0x6b6b51f4 as libc::c_int as libc::c_uint,
        0x1c6c6162 as libc::c_int as libc::c_uint,
        0x856530d8 as libc::c_uint,
        0xf262004e as libc::c_uint,
        0x6c0695ed as libc::c_int as libc::c_uint,
        0x1b01a57b as libc::c_int as libc::c_uint,
        0x8208f4c1 as libc::c_uint,
        0xf50fc457 as libc::c_uint,
        0x65b0d9c6 as libc::c_int as libc::c_uint,
        0x12b7e950 as libc::c_int as libc::c_uint,
        0x8bbeb8ea as libc::c_uint,
        0xfcb9887c as libc::c_uint,
        0x62dd1ddf as libc::c_int as libc::c_uint,
        0x15da2d49 as libc::c_int as libc::c_uint,
        0x8cd37cf3 as libc::c_uint,
        0xfbd44c65 as libc::c_uint,
        0x4db26158 as libc::c_int as libc::c_uint,
        0x3ab551ce as libc::c_int as libc::c_uint,
        0xa3bc0074 as libc::c_uint,
        0xd4bb30e2 as libc::c_uint,
        0x4adfa541 as libc::c_int as libc::c_uint,
        0x3dd895d7 as libc::c_int as libc::c_uint,
        0xa4d1c46d as libc::c_uint,
        0xd3d6f4fb as libc::c_uint,
        0x4369e96a as libc::c_int as libc::c_uint,
        0x346ed9fc as libc::c_int as libc::c_uint,
        0xad678846 as libc::c_uint,
        0xda60b8d0 as libc::c_uint,
        0x44042d73 as libc::c_int as libc::c_uint,
        0x33031de5 as libc::c_int as libc::c_uint,
        0xaa0a4c5f as libc::c_uint,
        0xdd0d7cc9 as libc::c_uint,
        0x5005713c as libc::c_int as libc::c_uint,
        0x270241aa as libc::c_int as libc::c_uint,
        0xbe0b1010 as libc::c_uint,
        0xc90c2086 as libc::c_uint,
        0x5768b525 as libc::c_int as libc::c_uint,
        0x206f85b3 as libc::c_int as libc::c_uint,
        0xb966d409 as libc::c_uint,
        0xce61e49f as libc::c_uint,
        0x5edef90e as libc::c_int as libc::c_uint,
        0x29d9c998 as libc::c_int as libc::c_uint,
        0xb0d09822 as libc::c_uint,
        0xc7d7a8b4 as libc::c_uint,
        0x59b33d17 as libc::c_int as libc::c_uint,
        0x2eb40d81 as libc::c_int as libc::c_uint,
        0xb7bd5c3b as libc::c_uint,
        0xc0ba6cad as libc::c_uint,
        0xedb88320 as libc::c_uint,
        0x9abfb3b6 as libc::c_uint,
        0x3b6e20c as libc::c_int as libc::c_uint,
        0x74b1d29a as libc::c_int as libc::c_uint,
        0xead54739 as libc::c_uint,
        0x9dd277af as libc::c_uint,
        0x4db2615 as libc::c_int as libc::c_uint,
        0x73dc1683 as libc::c_int as libc::c_uint,
        0xe3630b12 as libc::c_uint,
        0x94643b84 as libc::c_uint,
        0xd6d6a3e as libc::c_int as libc::c_uint,
        0x7a6a5aa8 as libc::c_int as libc::c_uint,
        0xe40ecf0b as libc::c_uint,
        0x9309ff9d as libc::c_uint,
        0xa00ae27 as libc::c_int as libc::c_uint,
        0x7d079eb1 as libc::c_int as libc::c_uint,
        0xf00f9344 as libc::c_uint,
        0x8708a3d2 as libc::c_uint,
        0x1e01f268 as libc::c_int as libc::c_uint,
        0x6906c2fe as libc::c_int as libc::c_uint,
        0xf762575d as libc::c_uint,
        0x806567cb as libc::c_uint,
        0x196c3671 as libc::c_int as libc::c_uint,
        0x6e6b06e7 as libc::c_int as libc::c_uint,
        0xfed41b76 as libc::c_uint,
        0x89d32be0 as libc::c_uint,
        0x10da7a5a as libc::c_int as libc::c_uint,
        0x67dd4acc as libc::c_int as libc::c_uint,
        0xf9b9df6f as libc::c_uint,
        0x8ebeeff9 as libc::c_uint,
        0x17b7be43 as libc::c_int as libc::c_uint,
        0x60b08ed5 as libc::c_int as libc::c_uint,
        0xd6d6a3e8 as libc::c_uint,
        0xa1d1937e as libc::c_uint,
        0x38d8c2c4 as libc::c_int as libc::c_uint,
        0x4fdff252 as libc::c_int as libc::c_uint,
        0xd1bb67f1 as libc::c_uint,
        0xa6bc5767 as libc::c_uint,
        0x3fb506dd as libc::c_int as libc::c_uint,
        0x48b2364b as libc::c_int as libc::c_uint,
        0xd80d2bda as libc::c_uint,
        0xaf0a1b4c as libc::c_uint,
        0x36034af6 as libc::c_int as libc::c_uint,
        0x41047a60 as libc::c_int as libc::c_uint,
        0xdf60efc3 as libc::c_uint,
        0xa867df55 as libc::c_uint,
        0x316e8eef as libc::c_int as libc::c_uint,
        0x4669be79 as libc::c_int as libc::c_uint,
        0xcb61b38c as libc::c_uint,
        0xbc66831a as libc::c_uint,
        0x256fd2a0 as libc::c_int as libc::c_uint,
        0x5268e236 as libc::c_int as libc::c_uint,
        0xcc0c7795 as libc::c_uint,
        0xbb0b4703 as libc::c_uint,
        0x220216b9 as libc::c_int as libc::c_uint,
        0x5505262f as libc::c_int as libc::c_uint,
        0xc5ba3bbe as libc::c_uint,
        0xb2bd0b28 as libc::c_uint,
        0x2bb45a92 as libc::c_int as libc::c_uint,
        0x5cb36a04 as libc::c_int as libc::c_uint,
        0xc2d7ffa7 as libc::c_uint,
        0xb5d0cf31 as libc::c_uint,
        0x2cd99e8b as libc::c_int as libc::c_uint,
        0x5bdeae1d as libc::c_int as libc::c_uint,
        0x9b64c2b0 as libc::c_uint,
        0xec63f226 as libc::c_uint,
        0x756aa39c as libc::c_int as libc::c_uint,
        0x26d930a as libc::c_int as libc::c_uint,
        0x9c0906a9 as libc::c_uint,
        0xeb0e363f as libc::c_uint,
        0x72076785 as libc::c_int as libc::c_uint,
        0x5005713 as libc::c_int as libc::c_uint,
        0x95bf4a82 as libc::c_uint,
        0xe2b87a14 as libc::c_uint,
        0x7bb12bae as libc::c_int as libc::c_uint,
        0xcb61b38 as libc::c_int as libc::c_uint,
        0x92d28e9b as libc::c_uint,
        0xe5d5be0d as libc::c_uint,
        0x7cdcefb7 as libc::c_int as libc::c_uint,
        0xbdbdf21 as libc::c_int as libc::c_uint,
        0x86d3d2d4 as libc::c_uint,
        0xf1d4e242 as libc::c_uint,
        0x68ddb3f8 as libc::c_int as libc::c_uint,
        0x1fda836e as libc::c_int as libc::c_uint,
        0x81be16cd as libc::c_uint,
        0xf6b9265b as libc::c_uint,
        0x6fb077e1 as libc::c_int as libc::c_uint,
        0x18b74777 as libc::c_int as libc::c_uint,
        0x88085ae6 as libc::c_uint,
        0xff0f6a70 as libc::c_uint,
        0x66063bca as libc::c_int as libc::c_uint,
        0x11010b5c as libc::c_int as libc::c_uint,
        0x8f659eff as libc::c_uint,
        0xf862ae69 as libc::c_uint,
        0x616bffd3 as libc::c_int as libc::c_uint,
        0x166ccf45 as libc::c_int as libc::c_uint,
        0xa00ae278 as libc::c_uint,
        0xd70dd2ee as libc::c_uint,
        0x4e048354 as libc::c_int as libc::c_uint,
        0x3903b3c2 as libc::c_int as libc::c_uint,
        0xa7672661 as libc::c_uint,
        0xd06016f7 as libc::c_uint,
        0x4969474d as libc::c_int as libc::c_uint,
        0x3e6e77db as libc::c_int as libc::c_uint,
        0xaed16a4a as libc::c_uint,
        0xd9d65adc as libc::c_uint,
        0x40df0b66 as libc::c_int as libc::c_uint,
        0x37d83bf0 as libc::c_int as libc::c_uint,
        0xa9bcae53 as libc::c_uint,
        0xdebb9ec5 as libc::c_uint,
        0x47b2cf7f as libc::c_int as libc::c_uint,
        0x30b5ffe9 as libc::c_int as libc::c_uint,
        0xbdbdf21c as libc::c_uint,
        0xcabac28a as libc::c_uint,
        0x53b39330 as libc::c_int as libc::c_uint,
        0x24b4a3a6 as libc::c_int as libc::c_uint,
        0xbad03605 as libc::c_uint,
        0xcdd70693 as libc::c_uint,
        0x54de5729 as libc::c_int as libc::c_uint,
        0x23d967bf as libc::c_int as libc::c_uint,
        0xb3667a2e as libc::c_uint,
        0xc4614ab8 as libc::c_uint,
        0x5d681b02 as libc::c_int as libc::c_uint,
        0x2a6f2b94 as libc::c_int as libc::c_uint,
        0xb40bbe37 as libc::c_uint,
        0xc30c8ea1 as libc::c_uint,
        0x5a05df1b as libc::c_int as libc::c_uint,
        0x2d02ef8d as libc::c_int as libc::c_uint,
    ];
    let mut crc: libc::c_uint = !(0 as libc::c_uint);
    let mut i: libc::c_int = 0;
    i = 0 as libc::c_int;
    while i < len {
        crc = crc >> 8 as libc::c_int
            ^ crc_table[(*buffer.offset(i as isize) as libc::c_uint
                ^ crc & 0xff as libc::c_int as libc::c_uint) as usize];
        i += 1;
    }
    return !crc;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_zlib_compress(
    mut data: *mut libc::c_uchar,
    mut data_len: libc::c_int,
    mut out_len: *mut libc::c_int,
    mut quality: libc::c_int,
) -> *mut libc::c_uchar {
    static mut lengthc: [libc::c_ushort; 30] = [
        3 as libc::c_int as libc::c_ushort,
        4 as libc::c_int as libc::c_ushort,
        5 as libc::c_int as libc::c_ushort,
        6 as libc::c_int as libc::c_ushort,
        7 as libc::c_int as libc::c_ushort,
        8 as libc::c_int as libc::c_ushort,
        9 as libc::c_int as libc::c_ushort,
        10 as libc::c_int as libc::c_ushort,
        11 as libc::c_int as libc::c_ushort,
        13 as libc::c_int as libc::c_ushort,
        15 as libc::c_int as libc::c_ushort,
        17 as libc::c_int as libc::c_ushort,
        19 as libc::c_int as libc::c_ushort,
        23 as libc::c_int as libc::c_ushort,
        27 as libc::c_int as libc::c_ushort,
        31 as libc::c_int as libc::c_ushort,
        35 as libc::c_int as libc::c_ushort,
        43 as libc::c_int as libc::c_ushort,
        51 as libc::c_int as libc::c_ushort,
        59 as libc::c_int as libc::c_ushort,
        67 as libc::c_int as libc::c_ushort,
        83 as libc::c_int as libc::c_ushort,
        99 as libc::c_int as libc::c_ushort,
        115 as libc::c_int as libc::c_ushort,
        131 as libc::c_int as libc::c_ushort,
        163 as libc::c_int as libc::c_ushort,
        195 as libc::c_int as libc::c_ushort,
        227 as libc::c_int as libc::c_ushort,
        258 as libc::c_int as libc::c_ushort,
        259 as libc::c_int as libc::c_ushort,
    ];
    static mut lengtheb: [libc::c_uchar; 29] = [
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        2 as libc::c_int as libc::c_uchar,
        2 as libc::c_int as libc::c_uchar,
        2 as libc::c_int as libc::c_uchar,
        2 as libc::c_int as libc::c_uchar,
        3 as libc::c_int as libc::c_uchar,
        3 as libc::c_int as libc::c_uchar,
        3 as libc::c_int as libc::c_uchar,
        3 as libc::c_int as libc::c_uchar,
        4 as libc::c_int as libc::c_uchar,
        4 as libc::c_int as libc::c_uchar,
        4 as libc::c_int as libc::c_uchar,
        4 as libc::c_int as libc::c_uchar,
        5 as libc::c_int as libc::c_uchar,
        5 as libc::c_int as libc::c_uchar,
        5 as libc::c_int as libc::c_uchar,
        5 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
    ];
    static mut distc: [libc::c_ushort; 31] = [
        1 as libc::c_int as libc::c_ushort,
        2 as libc::c_int as libc::c_ushort,
        3 as libc::c_int as libc::c_ushort,
        4 as libc::c_int as libc::c_ushort,
        5 as libc::c_int as libc::c_ushort,
        7 as libc::c_int as libc::c_ushort,
        9 as libc::c_int as libc::c_ushort,
        13 as libc::c_int as libc::c_ushort,
        17 as libc::c_int as libc::c_ushort,
        25 as libc::c_int as libc::c_ushort,
        33 as libc::c_int as libc::c_ushort,
        49 as libc::c_int as libc::c_ushort,
        65 as libc::c_int as libc::c_ushort,
        97 as libc::c_int as libc::c_ushort,
        129 as libc::c_int as libc::c_ushort,
        193 as libc::c_int as libc::c_ushort,
        257 as libc::c_int as libc::c_ushort,
        385 as libc::c_int as libc::c_ushort,
        513 as libc::c_int as libc::c_ushort,
        769 as libc::c_int as libc::c_ushort,
        1025 as libc::c_int as libc::c_ushort,
        1537 as libc::c_int as libc::c_ushort,
        2049 as libc::c_int as libc::c_ushort,
        3073 as libc::c_int as libc::c_ushort,
        4097 as libc::c_int as libc::c_ushort,
        6145 as libc::c_int as libc::c_ushort,
        8193 as libc::c_int as libc::c_ushort,
        12289 as libc::c_int as libc::c_ushort,
        16385 as libc::c_int as libc::c_ushort,
        24577 as libc::c_int as libc::c_ushort,
        32768 as libc::c_int as libc::c_ushort,
    ];
    static mut disteb: [libc::c_uchar; 30] = [
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        2 as libc::c_int as libc::c_uchar,
        2 as libc::c_int as libc::c_uchar,
        3 as libc::c_int as libc::c_uchar,
        3 as libc::c_int as libc::c_uchar,
        4 as libc::c_int as libc::c_uchar,
        4 as libc::c_int as libc::c_uchar,
        5 as libc::c_int as libc::c_uchar,
        5 as libc::c_int as libc::c_uchar,
        6 as libc::c_int as libc::c_uchar,
        6 as libc::c_int as libc::c_uchar,
        7 as libc::c_int as libc::c_uchar,
        7 as libc::c_int as libc::c_uchar,
        8 as libc::c_int as libc::c_uchar,
        8 as libc::c_int as libc::c_uchar,
        9 as libc::c_int as libc::c_uchar,
        9 as libc::c_int as libc::c_uchar,
        10 as libc::c_int as libc::c_uchar,
        10 as libc::c_int as libc::c_uchar,
        11 as libc::c_int as libc::c_uchar,
        11 as libc::c_int as libc::c_uchar,
        12 as libc::c_int as libc::c_uchar,
        12 as libc::c_int as libc::c_uchar,
        13 as libc::c_int as libc::c_uchar,
        13 as libc::c_int as libc::c_uchar,
    ];
    let mut bitbuf: libc::c_uint = 0 as libc::c_int as libc::c_uint;
    let mut i: libc::c_int = 0;
    let mut j: libc::c_int = 0;
    let mut bitcount: libc::c_int = 0 as libc::c_int;
    let mut out: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
    let mut hash_table: *mut *mut *mut libc::c_uchar = malloc(
        (16384 as libc::c_int as libc::c_ulong)
            .wrapping_mul(
                ::core::mem::size_of::<*mut *mut libc::c_uchar>() as libc::c_ulong,
            ),
    ) as *mut *mut *mut libc::c_uchar;
    if hash_table.is_null() {
        return 0 as *mut libc::c_uchar;
    }
    if quality < 5 as libc::c_int {
        quality = 5 as libc::c_int;
    }
    if out.is_null()
        || *(out as *mut libc::c_void as *mut libc::c_int)
            .offset(-(2 as libc::c_int as isize))
            .offset(1 as libc::c_int as isize) + 1 as libc::c_int
            >= *(out as *mut libc::c_void as *mut libc::c_int)
                .offset(-(2 as libc::c_int as isize))
                .offset(0 as libc::c_int as isize)
    {
        stbiw__sbgrowf(
            &mut out as *mut *mut libc::c_uchar as *mut *mut libc::c_void,
            1 as libc::c_int,
            ::core::mem::size_of::<libc::c_uchar>() as libc::c_ulong as libc::c_int,
        );
    } else {};
    let ref mut fresh5 = *(out as *mut libc::c_void as *mut libc::c_int)
        .offset(-(2 as libc::c_int as isize))
        .offset(1 as libc::c_int as isize);
    let fresh6 = *fresh5;
    *fresh5 = *fresh5 + 1;
    *out.offset(fresh6 as isize) = 0x78 as libc::c_int as libc::c_uchar;
    if out.is_null()
        || *(out as *mut libc::c_void as *mut libc::c_int)
            .offset(-(2 as libc::c_int as isize))
            .offset(1 as libc::c_int as isize) + 1 as libc::c_int
            >= *(out as *mut libc::c_void as *mut libc::c_int)
                .offset(-(2 as libc::c_int as isize))
                .offset(0 as libc::c_int as isize)
    {
        stbiw__sbgrowf(
            &mut out as *mut *mut libc::c_uchar as *mut *mut libc::c_void,
            1 as libc::c_int,
            ::core::mem::size_of::<libc::c_uchar>() as libc::c_ulong as libc::c_int,
        );
    } else {};
    let ref mut fresh7 = *(out as *mut libc::c_void as *mut libc::c_int)
        .offset(-(2 as libc::c_int as isize))
        .offset(1 as libc::c_int as isize);
    let fresh8 = *fresh7;
    *fresh7 = *fresh7 + 1;
    *out.offset(fresh8 as isize) = 0x5e as libc::c_int as libc::c_uchar;
    bitbuf |= ((1 as libc::c_int) << bitcount) as libc::c_uint;
    bitcount += 1 as libc::c_int;
    out = stbiw__zlib_flushf(out, &mut bitbuf, &mut bitcount);
    bitbuf |= ((1 as libc::c_int) << bitcount) as libc::c_uint;
    bitcount += 2 as libc::c_int;
    out = stbiw__zlib_flushf(out, &mut bitbuf, &mut bitcount);
    i = 0 as libc::c_int;
    while i < 16384 as libc::c_int {
        let ref mut fresh9 = *hash_table.offset(i as isize);
        *fresh9 = 0 as *mut *mut libc::c_uchar;
        i += 1;
    }
    i = 0 as libc::c_int;
    while i < data_len - 3 as libc::c_int {
        let mut h: libc::c_int = (stbiw__zhash(data.offset(i as isize))
            & (16384 as libc::c_int - 1 as libc::c_int) as libc::c_uint) as libc::c_int;
        let mut best: libc::c_int = 3 as libc::c_int;
        let mut bestloc: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
        let mut hlist: *mut *mut libc::c_uchar = *hash_table.offset(h as isize);
        let mut n: libc::c_int = if !hlist.is_null() {
            *(hlist as *mut libc::c_void as *mut libc::c_int)
                .offset(-(2 as libc::c_int as isize))
                .offset(1 as libc::c_int as isize)
        } else {
            0 as libc::c_int
        };
        j = 0 as libc::c_int;
        while j < n {
            if (*hlist.offset(j as isize)).offset_from(data) as libc::c_long
                > (i - 32768 as libc::c_int) as libc::c_long
            {
                let mut d: libc::c_int = stbiw__zlib_countm(
                    *hlist.offset(j as isize),
                    data.offset(i as isize),
                    data_len - i,
                ) as libc::c_int;
                if d >= best {
                    best = d;
                    bestloc = *hlist.offset(j as isize);
                }
            }
            j += 1;
        }
        if !(*hash_table.offset(h as isize)).is_null()
            && *(*hash_table.offset(h as isize) as *mut libc::c_void as *mut libc::c_int)
                .offset(-(2 as libc::c_int as isize))
                .offset(1 as libc::c_int as isize) == 2 as libc::c_int * quality
        {
            memmove(
                *hash_table.offset(h as isize) as *mut libc::c_void,
                (*hash_table.offset(h as isize)).offset(quality as isize)
                    as *const libc::c_void,
                (::core::mem::size_of::<*mut libc::c_uchar>() as libc::c_ulong)
                    .wrapping_mul(quality as libc::c_ulong),
            );
            *(*hash_table.offset(h as isize) as *mut libc::c_void as *mut libc::c_int)
                .offset(-(2 as libc::c_int as isize))
                .offset(1 as libc::c_int as isize) = quality;
        }
        if (*hash_table.offset(h as isize)).is_null()
            || *(*hash_table.offset(h as isize) as *mut libc::c_void as *mut libc::c_int)
                .offset(-(2 as libc::c_int as isize))
                .offset(1 as libc::c_int as isize) + 1 as libc::c_int
                >= *(*hash_table.offset(h as isize) as *mut libc::c_void
                    as *mut libc::c_int)
                    .offset(-(2 as libc::c_int as isize))
                    .offset(0 as libc::c_int as isize)
        {
            stbiw__sbgrowf(
                &mut *hash_table.offset(h as isize) as *mut *mut *mut libc::c_uchar
                    as *mut *mut libc::c_void,
                1 as libc::c_int,
                ::core::mem::size_of::<*mut libc::c_uchar>() as libc::c_ulong
                    as libc::c_int,
            );
        } else {};
        let ref mut fresh10 = *(*hash_table.offset(h as isize) as *mut libc::c_void
            as *mut libc::c_int)
            .offset(-(2 as libc::c_int as isize))
            .offset(1 as libc::c_int as isize);
        let fresh11 = *fresh10;
        *fresh10 = *fresh10 + 1;
        let ref mut fresh12 = *(*hash_table.offset(h as isize)).offset(fresh11 as isize);
        *fresh12 = data.offset(i as isize);
        if !bestloc.is_null() {
            h = (stbiw__zhash(data.offset(i as isize).offset(1 as libc::c_int as isize))
                & (16384 as libc::c_int - 1 as libc::c_int) as libc::c_uint)
                as libc::c_int;
            hlist = *hash_table.offset(h as isize);
            n = if !hlist.is_null() {
                *(hlist as *mut libc::c_void as *mut libc::c_int)
                    .offset(-(2 as libc::c_int as isize))
                    .offset(1 as libc::c_int as isize)
            } else {
                0 as libc::c_int
            };
            j = 0 as libc::c_int;
            while j < n {
                if (*hlist.offset(j as isize)).offset_from(data) as libc::c_long
                    > (i - 32767 as libc::c_int) as libc::c_long
                {
                    let mut e: libc::c_int = stbiw__zlib_countm(
                        *hlist.offset(j as isize),
                        data.offset(i as isize).offset(1 as libc::c_int as isize),
                        data_len - i - 1 as libc::c_int,
                    ) as libc::c_int;
                    if e > best {
                        bestloc = 0 as *mut libc::c_uchar;
                        break;
                    }
                }
                j += 1;
            }
        }
        if !bestloc.is_null() {
            let mut d_0: libc::c_int = data.offset(i as isize).offset_from(bestloc)
                as libc::c_long as libc::c_int;
            j = 0 as libc::c_int;
            while best
                > lengthc[(j + 1 as libc::c_int) as usize] as libc::c_int
                    - 1 as libc::c_int
            {
                j += 1;
            }
            if j + 257 as libc::c_int <= 143 as libc::c_int {
                bitbuf
                    |= (stbiw__zlib_bitrev(
                        0x30 as libc::c_int + (j + 257 as libc::c_int),
                        8 as libc::c_int,
                    ) << bitcount) as libc::c_uint;
                bitcount += 8 as libc::c_int;
                out = stbiw__zlib_flushf(out, &mut bitbuf, &mut bitcount);
            } else {
                if j + 257 as libc::c_int <= 255 as libc::c_int {
                    bitbuf
                        |= (stbiw__zlib_bitrev(
                            0x190 as libc::c_int + (j + 257 as libc::c_int)
                                - 144 as libc::c_int,
                            9 as libc::c_int,
                        ) << bitcount) as libc::c_uint;
                    bitcount += 9 as libc::c_int;
                    out = stbiw__zlib_flushf(out, &mut bitbuf, &mut bitcount);
                } else {
                    if j + 257 as libc::c_int <= 279 as libc::c_int {
                        bitbuf
                            |= (stbiw__zlib_bitrev(
                                0 as libc::c_int + (j + 257 as libc::c_int)
                                    - 256 as libc::c_int,
                                7 as libc::c_int,
                            ) << bitcount) as libc::c_uint;
                        bitcount += 7 as libc::c_int;
                        out = stbiw__zlib_flushf(out, &mut bitbuf, &mut bitcount);
                    } else {
                        bitbuf
                            |= (stbiw__zlib_bitrev(
                                0xc0 as libc::c_int + (j + 257 as libc::c_int)
                                    - 280 as libc::c_int,
                                8 as libc::c_int,
                            ) << bitcount) as libc::c_uint;
                        bitcount += 8 as libc::c_int;
                        out = stbiw__zlib_flushf(out, &mut bitbuf, &mut bitcount);
                    };
                };
            };
            if lengtheb[j as usize] != 0 {
                bitbuf
                    |= ((best - lengthc[j as usize] as libc::c_int) << bitcount)
                        as libc::c_uint;
                bitcount += lengtheb[j as usize] as libc::c_int;
                out = stbiw__zlib_flushf(out, &mut bitbuf, &mut bitcount);
            }
            j = 0 as libc::c_int;
            while d_0
                > distc[(j + 1 as libc::c_int) as usize] as libc::c_int
                    - 1 as libc::c_int
            {
                j += 1;
            }
            bitbuf
                |= (stbiw__zlib_bitrev(j, 5 as libc::c_int) << bitcount) as libc::c_uint;
            bitcount += 5 as libc::c_int;
            out = stbiw__zlib_flushf(out, &mut bitbuf, &mut bitcount);
            if disteb[j as usize] != 0 {
                bitbuf
                    |= ((d_0 - distc[j as usize] as libc::c_int) << bitcount)
                        as libc::c_uint;
                bitcount += disteb[j as usize] as libc::c_int;
                out = stbiw__zlib_flushf(out, &mut bitbuf, &mut bitcount);
            }
            i += best;
        } else {
            if *data.offset(i as isize) as libc::c_int <= 143 as libc::c_int {
                bitbuf
                    |= (stbiw__zlib_bitrev(
                        0x30 as libc::c_int + *data.offset(i as isize) as libc::c_int,
                        8 as libc::c_int,
                    ) << bitcount) as libc::c_uint;
                bitcount += 8 as libc::c_int;
                out = stbiw__zlib_flushf(out, &mut bitbuf, &mut bitcount);
            } else {
                bitbuf
                    |= (stbiw__zlib_bitrev(
                        0x190 as libc::c_int + *data.offset(i as isize) as libc::c_int
                            - 144 as libc::c_int,
                        9 as libc::c_int,
                    ) << bitcount) as libc::c_uint;
                bitcount += 9 as libc::c_int;
                out = stbiw__zlib_flushf(out, &mut bitbuf, &mut bitcount);
            };
            i += 1;
        }
    }
    while i < data_len {
        if *data.offset(i as isize) as libc::c_int <= 143 as libc::c_int {
            bitbuf
                |= (stbiw__zlib_bitrev(
                    0x30 as libc::c_int + *data.offset(i as isize) as libc::c_int,
                    8 as libc::c_int,
                ) << bitcount) as libc::c_uint;
            bitcount += 8 as libc::c_int;
            out = stbiw__zlib_flushf(out, &mut bitbuf, &mut bitcount);
        } else {
            bitbuf
                |= (stbiw__zlib_bitrev(
                    0x190 as libc::c_int + *data.offset(i as isize) as libc::c_int
                        - 144 as libc::c_int,
                    9 as libc::c_int,
                ) << bitcount) as libc::c_uint;
            bitcount += 9 as libc::c_int;
            out = stbiw__zlib_flushf(out, &mut bitbuf, &mut bitcount);
        };
        i += 1;
    }
    if 256 as libc::c_int <= 143 as libc::c_int {
        bitbuf
            |= (stbiw__zlib_bitrev(
                0x30 as libc::c_int + 256 as libc::c_int,
                8 as libc::c_int,
            ) << bitcount) as libc::c_uint;
        bitcount += 8 as libc::c_int;
        out = stbiw__zlib_flushf(out, &mut bitbuf, &mut bitcount);
    } else {
        if 256 as libc::c_int <= 255 as libc::c_int {
            bitbuf
                |= (stbiw__zlib_bitrev(
                    0x190 as libc::c_int + 256 as libc::c_int - 144 as libc::c_int,
                    9 as libc::c_int,
                ) << bitcount) as libc::c_uint;
            bitcount += 9 as libc::c_int;
            out = stbiw__zlib_flushf(out, &mut bitbuf, &mut bitcount);
        } else {
            if 256 as libc::c_int <= 279 as libc::c_int {
                bitbuf
                    |= (stbiw__zlib_bitrev(
                        0 as libc::c_int + 256 as libc::c_int - 256 as libc::c_int,
                        7 as libc::c_int,
                    ) << bitcount) as libc::c_uint;
                bitcount += 7 as libc::c_int;
                out = stbiw__zlib_flushf(out, &mut bitbuf, &mut bitcount);
            } else {
                bitbuf
                    |= (stbiw__zlib_bitrev(
                        0xc0 as libc::c_int + 256 as libc::c_int - 280 as libc::c_int,
                        8 as libc::c_int,
                    ) << bitcount) as libc::c_uint;
                bitcount += 8 as libc::c_int;
                out = stbiw__zlib_flushf(out, &mut bitbuf, &mut bitcount);
            };
        };
    };
    while bitcount != 0 {
        bitbuf |= ((0 as libc::c_int) << bitcount) as libc::c_uint;
        bitcount += 1 as libc::c_int;
        out = stbiw__zlib_flushf(out, &mut bitbuf, &mut bitcount);
    }
    i = 0 as libc::c_int;
    while i < 16384 as libc::c_int {
        if !(*hash_table.offset(i as isize)).is_null() {
            free(
                (*hash_table.offset(i as isize) as *mut libc::c_void as *mut libc::c_int)
                    .offset(-(2 as libc::c_int as isize)) as *mut libc::c_void,
            );
        } else {};
        i += 1;
    }
    free(hash_table as *mut libc::c_void);
    if *(out as *mut libc::c_void as *mut libc::c_int)
        .offset(-(2 as libc::c_int as isize))
        .offset(1 as libc::c_int as isize)
        > data_len + 2 as libc::c_int
            + (data_len + 32766 as libc::c_int) / 32767 as libc::c_int * 5 as libc::c_int
    {
        *(out as *mut libc::c_void as *mut libc::c_int)
            .offset(-(2 as libc::c_int as isize))
            .offset(1 as libc::c_int as isize) = 2 as libc::c_int;
        j = 0 as libc::c_int;
        while j < data_len {
            let mut blocklen: libc::c_int = data_len - j;
            if blocklen > 32767 as libc::c_int {
                blocklen = 32767 as libc::c_int;
            }
            if out.is_null()
                || *(out as *mut libc::c_void as *mut libc::c_int)
                    .offset(-(2 as libc::c_int as isize))
                    .offset(1 as libc::c_int as isize) + 1 as libc::c_int
                    >= *(out as *mut libc::c_void as *mut libc::c_int)
                        .offset(-(2 as libc::c_int as isize))
                        .offset(0 as libc::c_int as isize)
            {
                stbiw__sbgrowf(
                    &mut out as *mut *mut libc::c_uchar as *mut *mut libc::c_void,
                    1 as libc::c_int,
                    ::core::mem::size_of::<libc::c_uchar>() as libc::c_ulong
                        as libc::c_int,
                );
            } else {};
            let ref mut fresh13 = *(out as *mut libc::c_void as *mut libc::c_int)
                .offset(-(2 as libc::c_int as isize))
                .offset(1 as libc::c_int as isize);
            let fresh14 = *fresh13;
            *fresh13 = *fresh13 + 1;
            *out
                .offset(
                    fresh14 as isize,
                ) = (data_len - j == blocklen) as libc::c_int as libc::c_uchar;
            if out.is_null()
                || *(out as *mut libc::c_void as *mut libc::c_int)
                    .offset(-(2 as libc::c_int as isize))
                    .offset(1 as libc::c_int as isize) + 1 as libc::c_int
                    >= *(out as *mut libc::c_void as *mut libc::c_int)
                        .offset(-(2 as libc::c_int as isize))
                        .offset(0 as libc::c_int as isize)
            {
                stbiw__sbgrowf(
                    &mut out as *mut *mut libc::c_uchar as *mut *mut libc::c_void,
                    1 as libc::c_int,
                    ::core::mem::size_of::<libc::c_uchar>() as libc::c_ulong
                        as libc::c_int,
                );
            } else {};
            let ref mut fresh15 = *(out as *mut libc::c_void as *mut libc::c_int)
                .offset(-(2 as libc::c_int as isize))
                .offset(1 as libc::c_int as isize);
            let fresh16 = *fresh15;
            *fresh15 = *fresh15 + 1;
            *out
                .offset(
                    fresh16 as isize,
                ) = (blocklen & 0xff as libc::c_int) as libc::c_uchar;
            if out.is_null()
                || *(out as *mut libc::c_void as *mut libc::c_int)
                    .offset(-(2 as libc::c_int as isize))
                    .offset(1 as libc::c_int as isize) + 1 as libc::c_int
                    >= *(out as *mut libc::c_void as *mut libc::c_int)
                        .offset(-(2 as libc::c_int as isize))
                        .offset(0 as libc::c_int as isize)
            {
                stbiw__sbgrowf(
                    &mut out as *mut *mut libc::c_uchar as *mut *mut libc::c_void,
                    1 as libc::c_int,
                    ::core::mem::size_of::<libc::c_uchar>() as libc::c_ulong
                        as libc::c_int,
                );
            } else {};
            let ref mut fresh17 = *(out as *mut libc::c_void as *mut libc::c_int)
                .offset(-(2 as libc::c_int as isize))
                .offset(1 as libc::c_int as isize);
            let fresh18 = *fresh17;
            *fresh17 = *fresh17 + 1;
            *out
                .offset(
                    fresh18 as isize,
                ) = (blocklen >> 8 as libc::c_int & 0xff as libc::c_int)
                as libc::c_uchar;
            if out.is_null()
                || *(out as *mut libc::c_void as *mut libc::c_int)
                    .offset(-(2 as libc::c_int as isize))
                    .offset(1 as libc::c_int as isize) + 1 as libc::c_int
                    >= *(out as *mut libc::c_void as *mut libc::c_int)
                        .offset(-(2 as libc::c_int as isize))
                        .offset(0 as libc::c_int as isize)
            {
                stbiw__sbgrowf(
                    &mut out as *mut *mut libc::c_uchar as *mut *mut libc::c_void,
                    1 as libc::c_int,
                    ::core::mem::size_of::<libc::c_uchar>() as libc::c_ulong
                        as libc::c_int,
                );
            } else {};
            let ref mut fresh19 = *(out as *mut libc::c_void as *mut libc::c_int)
                .offset(-(2 as libc::c_int as isize))
                .offset(1 as libc::c_int as isize);
            let fresh20 = *fresh19;
            *fresh19 = *fresh19 + 1;
            *out
                .offset(
                    fresh20 as isize,
                ) = (!blocklen & 0xff as libc::c_int) as libc::c_uchar;
            if out.is_null()
                || *(out as *mut libc::c_void as *mut libc::c_int)
                    .offset(-(2 as libc::c_int as isize))
                    .offset(1 as libc::c_int as isize) + 1 as libc::c_int
                    >= *(out as *mut libc::c_void as *mut libc::c_int)
                        .offset(-(2 as libc::c_int as isize))
                        .offset(0 as libc::c_int as isize)
            {
                stbiw__sbgrowf(
                    &mut out as *mut *mut libc::c_uchar as *mut *mut libc::c_void,
                    1 as libc::c_int,
                    ::core::mem::size_of::<libc::c_uchar>() as libc::c_ulong
                        as libc::c_int,
                );
            } else {};
            let ref mut fresh21 = *(out as *mut libc::c_void as *mut libc::c_int)
                .offset(-(2 as libc::c_int as isize))
                .offset(1 as libc::c_int as isize);
            let fresh22 = *fresh21;
            *fresh21 = *fresh21 + 1;
            *out
                .offset(
                    fresh22 as isize,
                ) = (!blocklen >> 8 as libc::c_int & 0xff as libc::c_int)
                as libc::c_uchar;
            memcpy(
                out
                    .offset(
                        *(out as *mut libc::c_void as *mut libc::c_int)
                            .offset(-(2 as libc::c_int as isize))
                            .offset(1 as libc::c_int as isize) as isize,
                    ) as *mut libc::c_void,
                data.offset(j as isize) as *const libc::c_void,
                blocklen as libc::c_ulong,
            );
            *(out as *mut libc::c_void as *mut libc::c_int)
                .offset(-(2 as libc::c_int as isize))
                .offset(1 as libc::c_int as isize) += blocklen;
            j += blocklen;
        }
    }
    let mut s1: libc::c_uint = 1 as libc::c_int as libc::c_uint;
    let mut s2: libc::c_uint = 0 as libc::c_int as libc::c_uint;
    let mut blocklen_0: libc::c_int = data_len % 5552 as libc::c_int;
    j = 0 as libc::c_int;
    while j < data_len {
        i = 0 as libc::c_int;
        while i < blocklen_0 {
            s1 = s1.wrapping_add(*data.offset((j + i) as isize) as libc::c_uint);
            s2 = s2.wrapping_add(s1);
            i += 1;
        }
        s1 = s1.wrapping_rem(65521 as libc::c_int as libc::c_uint);
        s2 = s2.wrapping_rem(65521 as libc::c_int as libc::c_uint);
        j += blocklen_0;
        blocklen_0 = 5552 as libc::c_int;
    }
    if out.is_null()
        || *(out as *mut libc::c_void as *mut libc::c_int)
            .offset(-(2 as libc::c_int as isize))
            .offset(1 as libc::c_int as isize) + 1 as libc::c_int
            >= *(out as *mut libc::c_void as *mut libc::c_int)
                .offset(-(2 as libc::c_int as isize))
                .offset(0 as libc::c_int as isize)
    {
        stbiw__sbgrowf(
            &mut out as *mut *mut libc::c_uchar as *mut *mut libc::c_void,
            1 as libc::c_int,
            ::core::mem::size_of::<libc::c_uchar>() as libc::c_ulong as libc::c_int,
        );
    } else {};
    let ref mut fresh23 = *(out as *mut libc::c_void as *mut libc::c_int)
        .offset(-(2 as libc::c_int as isize))
        .offset(1 as libc::c_int as isize);
    let fresh24 = *fresh23;
    *fresh23 = *fresh23 + 1;
    *out
        .offset(
            fresh24 as isize,
        ) = (s2 >> 8 as libc::c_int & 0xff as libc::c_int as libc::c_uint)
        as libc::c_uchar;
    if out.is_null()
        || *(out as *mut libc::c_void as *mut libc::c_int)
            .offset(-(2 as libc::c_int as isize))
            .offset(1 as libc::c_int as isize) + 1 as libc::c_int
            >= *(out as *mut libc::c_void as *mut libc::c_int)
                .offset(-(2 as libc::c_int as isize))
                .offset(0 as libc::c_int as isize)
    {
        stbiw__sbgrowf(
            &mut out as *mut *mut libc::c_uchar as *mut *mut libc::c_void,
            1 as libc::c_int,
            ::core::mem::size_of::<libc::c_uchar>() as libc::c_ulong as libc::c_int,
        );
    } else {};
    let ref mut fresh25 = *(out as *mut libc::c_void as *mut libc::c_int)
        .offset(-(2 as libc::c_int as isize))
        .offset(1 as libc::c_int as isize);
    let fresh26 = *fresh25;
    *fresh25 = *fresh25 + 1;
    *out
        .offset(
            fresh26 as isize,
        ) = (s2 & 0xff as libc::c_int as libc::c_uint) as libc::c_uchar;
    if out.is_null()
        || *(out as *mut libc::c_void as *mut libc::c_int)
            .offset(-(2 as libc::c_int as isize))
            .offset(1 as libc::c_int as isize) + 1 as libc::c_int
            >= *(out as *mut libc::c_void as *mut libc::c_int)
                .offset(-(2 as libc::c_int as isize))
                .offset(0 as libc::c_int as isize)
    {
        stbiw__sbgrowf(
            &mut out as *mut *mut libc::c_uchar as *mut *mut libc::c_void,
            1 as libc::c_int,
            ::core::mem::size_of::<libc::c_uchar>() as libc::c_ulong as libc::c_int,
        );
    } else {};
    let ref mut fresh27 = *(out as *mut libc::c_void as *mut libc::c_int)
        .offset(-(2 as libc::c_int as isize))
        .offset(1 as libc::c_int as isize);
    let fresh28 = *fresh27;
    *fresh27 = *fresh27 + 1;
    *out
        .offset(
            fresh28 as isize,
        ) = (s1 >> 8 as libc::c_int & 0xff as libc::c_int as libc::c_uint)
        as libc::c_uchar;
    if out.is_null()
        || *(out as *mut libc::c_void as *mut libc::c_int)
            .offset(-(2 as libc::c_int as isize))
            .offset(1 as libc::c_int as isize) + 1 as libc::c_int
            >= *(out as *mut libc::c_void as *mut libc::c_int)
                .offset(-(2 as libc::c_int as isize))
                .offset(0 as libc::c_int as isize)
    {
        stbiw__sbgrowf(
            &mut out as *mut *mut libc::c_uchar as *mut *mut libc::c_void,
            1 as libc::c_int,
            ::core::mem::size_of::<libc::c_uchar>() as libc::c_ulong as libc::c_int,
        );
    } else {};
    let ref mut fresh29 = *(out as *mut libc::c_void as *mut libc::c_int)
        .offset(-(2 as libc::c_int as isize))
        .offset(1 as libc::c_int as isize);
    let fresh30 = *fresh29;
    *fresh29 = *fresh29 + 1;
    *out
        .offset(
            fresh30 as isize,
        ) = (s1 & 0xff as libc::c_int as libc::c_uint) as libc::c_uchar;
    *out_len = *(out as *mut libc::c_void as *mut libc::c_int)
        .offset(-(2 as libc::c_int as isize))
        .offset(1 as libc::c_int as isize);
    memmove(
        (out as *mut libc::c_void as *mut libc::c_int)
            .offset(-(2 as libc::c_int as isize)) as *mut libc::c_void,
        out as *const libc::c_void,
        *out_len as libc::c_ulong,
    );
    return (out as *mut libc::c_void as *mut libc::c_int)
        .offset(-(2 as libc::c_int as isize)) as *mut libc::c_uchar;
}
unsafe extern "C" fn stbiw__sbgrowf(
    mut arr: *mut *mut libc::c_void,
    mut increment: libc::c_int,
    mut itemsize: libc::c_int,
) -> *mut libc::c_void {
    let mut m: libc::c_int = if !(*arr).is_null() {
        2 as libc::c_int
            * *(*arr as *mut libc::c_int)
                .offset(-(2 as libc::c_int as isize))
                .offset(0 as libc::c_int as isize) + increment
    } else {
        increment + 1 as libc::c_int
    };
    let mut p: *mut libc::c_void = realloc(
        (if !(*arr).is_null() {
            (*arr as *mut libc::c_int).offset(-(2 as libc::c_int as isize))
        } else {
            0 as *mut libc::c_int
        }) as *mut libc::c_void,
        ((itemsize * m) as libc::c_ulong)
            .wrapping_add(
                (::core::mem::size_of::<libc::c_int>() as libc::c_ulong)
                    .wrapping_mul(2 as libc::c_int as libc::c_ulong),
            ),
    );
    if !p.is_null() {
        if (*arr).is_null() {
            *(p as *mut libc::c_int)
                .offset(1 as libc::c_int as isize) = 0 as libc::c_int;
        }
        *arr = (p as *mut libc::c_int).offset(2 as libc::c_int as isize)
            as *mut libc::c_void;
        *(*arr as *mut libc::c_int)
            .offset(-(2 as libc::c_int as isize))
            .offset(0 as libc::c_int as isize) = m;
    }
    return *arr;
}
unsafe extern "C" fn stbiw__zlib_flushf(
    mut data: *mut libc::c_uchar,
    mut bitbuffer: *mut libc::c_uint,
    mut bitcount: *mut libc::c_int,
) -> *mut libc::c_uchar {
    while *bitcount >= 8 as libc::c_int {
        if data.is_null()
            || *(data as *mut libc::c_void as *mut libc::c_int)
                .offset(-(2 as libc::c_int as isize))
                .offset(1 as libc::c_int as isize) + 1 as libc::c_int
                >= *(data as *mut libc::c_void as *mut libc::c_int)
                    .offset(-(2 as libc::c_int as isize))
                    .offset(0 as libc::c_int as isize)
        {
            stbiw__sbgrowf(
                &mut data as *mut *mut libc::c_uchar as *mut *mut libc::c_void,
                1 as libc::c_int,
                ::core::mem::size_of::<libc::c_uchar>() as libc::c_ulong as libc::c_int,
            );
        } else {};
        let ref mut fresh31 = *(data as *mut libc::c_void as *mut libc::c_int)
            .offset(-(2 as libc::c_int as isize))
            .offset(1 as libc::c_int as isize);
        let fresh32 = *fresh31;
        *fresh31 = *fresh31 + 1;
        *data
            .offset(
                fresh32 as isize,
            ) = (*bitbuffer & 0xff as libc::c_int as libc::c_uint) as libc::c_uchar;
        *bitbuffer >>= 8 as libc::c_int;
        *bitcount -= 8 as libc::c_int;
    }
    return data;
}
unsafe extern "C" fn stbiw__zlib_bitrev(
    mut code: libc::c_int,
    mut codebits: libc::c_int,
) -> libc::c_int {
    let mut res: libc::c_int = 0 as libc::c_int;
    loop {
        let fresh33 = codebits;
        codebits = codebits - 1;
        if !(fresh33 != 0) {
            break;
        }
        res = res << 1 as libc::c_int | code & 1 as libc::c_int;
        code >>= 1 as libc::c_int;
    }
    return res;
}
unsafe extern "C" fn stbiw__zhash(mut data: *mut libc::c_uchar) -> libc::c_uint {
    let mut hash: stbiw_uint32 = (*data.offset(0 as libc::c_int as isize) as libc::c_int
        + ((*data.offset(1 as libc::c_int as isize) as libc::c_int) << 8 as libc::c_int)
        + ((*data.offset(2 as libc::c_int as isize) as libc::c_int)
            << 16 as libc::c_int)) as stbiw_uint32;
    hash ^= hash << 3 as libc::c_int;
    hash = (hash as libc::c_uint).wrapping_add(hash >> 5 as libc::c_int) as stbiw_uint32
        as stbiw_uint32;
    hash ^= hash << 4 as libc::c_int;
    hash = (hash as libc::c_uint).wrapping_add(hash >> 17 as libc::c_int) as stbiw_uint32
        as stbiw_uint32;
    hash ^= hash << 25 as libc::c_int;
    hash = (hash as libc::c_uint).wrapping_add(hash >> 6 as libc::c_int) as stbiw_uint32
        as stbiw_uint32;
    return hash;
}
unsafe extern "C" fn stbiw__zlib_countm(
    mut a: *mut libc::c_uchar,
    mut b: *mut libc::c_uchar,
    mut limit: libc::c_int,
) -> libc::c_uint {
    let mut i: libc::c_int = 0;
    i = 0 as libc::c_int;
    while i < limit && i < 258 as libc::c_int {
        if *a.offset(i as isize) as libc::c_int != *b.offset(i as isize) as libc::c_int {
            break;
        }
        i += 1;
    }
    return i as libc::c_uint;
}
unsafe extern "C" fn stbiw__encode_png_line(
    mut pixels: *mut libc::c_uchar,
    mut stride_bytes: libc::c_int,
    mut width: libc::c_int,
    mut height: libc::c_int,
    mut y: libc::c_int,
    mut n: libc::c_int,
    mut filter_type: libc::c_int,
    mut line_buffer: *mut libc::c_schar,
) {
    static mut mapping: [libc::c_int; 5] = [
        0 as libc::c_int,
        1 as libc::c_int,
        2 as libc::c_int,
        3 as libc::c_int,
        4 as libc::c_int,
    ];
    static mut firstmap: [libc::c_int; 5] = [
        0 as libc::c_int,
        1 as libc::c_int,
        0 as libc::c_int,
        5 as libc::c_int,
        6 as libc::c_int,
    ];
    let mut mymap: *mut libc::c_int = if y != 0 as libc::c_int {
        mapping.as_mut_ptr()
    } else {
        firstmap.as_mut_ptr()
    };
    let mut i: libc::c_int = 0;
    let mut type_0: libc::c_int = *mymap.offset(filter_type as isize);
    let mut z: *mut libc::c_uchar = pixels
        .offset(
            (stride_bytes
                * (if stbi__flip_vertically_on_write != 0 {
                    height - 1 as libc::c_int - y
                } else {
                    y
                })) as isize,
        );
    let mut signed_stride: libc::c_int = if stbi__flip_vertically_on_write != 0 {
        -stride_bytes
    } else {
        stride_bytes
    };
    if type_0 == 0 as libc::c_int {
        memcpy(
            line_buffer as *mut libc::c_void,
            z as *const libc::c_void,
            (width * n) as libc::c_ulong,
        );
        return;
    }
    i = 0 as libc::c_int;
    while i < n {
        match type_0 {
            1 => {
                *line_buffer.offset(i as isize) = *z.offset(i as isize) as libc::c_schar;
            }
            2 => {
                *line_buffer
                    .offset(
                        i as isize,
                    ) = (*z.offset(i as isize) as libc::c_int
                    - *z.offset((i - signed_stride) as isize) as libc::c_int)
                    as libc::c_schar;
            }
            3 => {
                *line_buffer
                    .offset(
                        i as isize,
                    ) = (*z.offset(i as isize) as libc::c_int
                    - (*z.offset((i - signed_stride) as isize) as libc::c_int
                        >> 1 as libc::c_int)) as libc::c_schar;
            }
            4 => {
                *line_buffer
                    .offset(
                        i as isize,
                    ) = (*z.offset(i as isize) as libc::c_int
                    - stbiw__paeth(
                        0 as libc::c_int,
                        *z.offset((i - signed_stride) as isize) as libc::c_int,
                        0 as libc::c_int,
                    ) as libc::c_int) as libc::c_schar;
            }
            5 => {
                *line_buffer.offset(i as isize) = *z.offset(i as isize) as libc::c_schar;
            }
            6 => {
                *line_buffer.offset(i as isize) = *z.offset(i as isize) as libc::c_schar;
            }
            _ => {}
        }
        i += 1;
    }
    match type_0 {
        1 => {
            i = n;
            while i < width * n {
                *line_buffer
                    .offset(
                        i as isize,
                    ) = (*z.offset(i as isize) as libc::c_int
                    - *z.offset((i - n) as isize) as libc::c_int) as libc::c_schar;
                i += 1;
            }
        }
        2 => {
            i = n;
            while i < width * n {
                *line_buffer
                    .offset(
                        i as isize,
                    ) = (*z.offset(i as isize) as libc::c_int
                    - *z.offset((i - signed_stride) as isize) as libc::c_int)
                    as libc::c_schar;
                i += 1;
            }
        }
        3 => {
            i = n;
            while i < width * n {
                *line_buffer
                    .offset(
                        i as isize,
                    ) = (*z.offset(i as isize) as libc::c_int
                    - (*z.offset((i - n) as isize) as libc::c_int
                        + *z.offset((i - signed_stride) as isize) as libc::c_int
                        >> 1 as libc::c_int)) as libc::c_schar;
                i += 1;
            }
        }
        4 => {
            i = n;
            while i < width * n {
                *line_buffer
                    .offset(
                        i as isize,
                    ) = (*z.offset(i as isize) as libc::c_int
                    - stbiw__paeth(
                        *z.offset((i - n) as isize) as libc::c_int,
                        *z.offset((i - signed_stride) as isize) as libc::c_int,
                        *z.offset((i - signed_stride - n) as isize) as libc::c_int,
                    ) as libc::c_int) as libc::c_schar;
                i += 1;
            }
        }
        5 => {
            i = n;
            while i < width * n {
                *line_buffer
                    .offset(
                        i as isize,
                    ) = (*z.offset(i as isize) as libc::c_int
                    - (*z.offset((i - n) as isize) as libc::c_int >> 1 as libc::c_int))
                    as libc::c_schar;
                i += 1;
            }
        }
        6 => {
            i = n;
            while i < width * n {
                *line_buffer
                    .offset(
                        i as isize,
                    ) = (*z.offset(i as isize) as libc::c_int
                    - stbiw__paeth(
                        *z.offset((i - n) as isize) as libc::c_int,
                        0 as libc::c_int,
                        0 as libc::c_int,
                    ) as libc::c_int) as libc::c_schar;
                i += 1;
            }
        }
        _ => {}
    };
}
static mut stbi__flip_vertically_on_write: libc::c_int = 0 as libc::c_int;
unsafe extern "C" fn stbiw__paeth(
    mut a: libc::c_int,
    mut b: libc::c_int,
    mut c: libc::c_int,
) -> libc::c_uchar {
    let mut p: libc::c_int = a + b - c;
    let mut pa: libc::c_int = abs(p - a);
    let mut pb: libc::c_int = abs(p - b);
    let mut pc: libc::c_int = abs(p - c);
    if pa <= pb && pa <= pc {
        return (a & 0xff as libc::c_int) as libc::c_uchar;
    }
    if pb <= pc {
        return (b & 0xff as libc::c_int) as libc::c_uchar;
    }
    return (c & 0xff as libc::c_int) as libc::c_uchar;
}
unsafe extern "C" fn stbiw__fopen(
    mut filename: *const libc::c_char,
    mut mode: *const libc::c_char,
) -> *mut FILE {
    let mut f: *mut FILE = 0 as *mut FILE;
    f = fopen(filename, mode);
    return f;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_write_bmp(
    mut filename: *const libc::c_char,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut comp: libc::c_int,
    mut data: *const libc::c_void,
) -> libc::c_int {
    let mut s: stbi__write_context = {
        let mut init = stbi__write_context {
            func: None,
            context: 0 as *mut libc::c_void,
            buffer: [0; 64],
            buf_used: 0,
        };
        init
    };
    if stbi__start_write_file(&mut s, filename) != 0 {
        let mut r: libc::c_int = stbi_write_bmp_core(&mut s, x, y, comp, data);
        stbi__end_write_file(&mut s);
        return r;
    } else {
        return 0 as libc::c_int
    };
}
unsafe extern "C" fn stbi_write_bmp_core(
    mut s: *mut stbi__write_context,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut comp: libc::c_int,
    mut data: *const libc::c_void,
) -> libc::c_int {
    if comp != 4 as libc::c_int {
        let mut pad: libc::c_int = -x * 3 as libc::c_int & 3 as libc::c_int;
        return stbiw__outfile(
            s,
            -(1 as libc::c_int),
            -(1 as libc::c_int),
            x,
            y,
            comp,
            1 as libc::c_int,
            data as *mut libc::c_void,
            0 as libc::c_int,
            pad,
            b"11 4 22 44 44 22 444444\0" as *const u8 as *const libc::c_char,
            'B' as i32,
            'M' as i32,
            14 as libc::c_int + 40 as libc::c_int + (x * 3 as libc::c_int + pad) * y,
            0 as libc::c_int,
            0 as libc::c_int,
            14 as libc::c_int + 40 as libc::c_int,
            40 as libc::c_int,
            x,
            y,
            1 as libc::c_int,
            24 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
        );
    } else {
        return stbiw__outfile(
            s,
            -(1 as libc::c_int),
            -(1 as libc::c_int),
            x,
            y,
            comp,
            1 as libc::c_int,
            data as *mut libc::c_void,
            1 as libc::c_int,
            0 as libc::c_int,
            b"11 4 22 44 44 22 444444 4444 4 444 444 444 444\0" as *const u8
                as *const libc::c_char,
            'B' as i32,
            'M' as i32,
            14 as libc::c_int + 108 as libc::c_int + x * y * 4 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            14 as libc::c_int + 108 as libc::c_int,
            108 as libc::c_int,
            x,
            y,
            1 as libc::c_int,
            32 as libc::c_int,
            3 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0xff0000 as libc::c_int,
            0xff00 as libc::c_int,
            0xff as libc::c_int,
            0xff000000 as libc::c_uint,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
        )
    };
}
unsafe extern "C" fn stbiw__outfile(
    mut s: *mut stbi__write_context,
    mut rgb_dir: libc::c_int,
    mut vdir: libc::c_int,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut comp: libc::c_int,
    mut expand_mono: libc::c_int,
    mut data: *mut libc::c_void,
    mut alpha: libc::c_int,
    mut pad: libc::c_int,
    mut fmt: *const libc::c_char,
    mut args: ...
) -> libc::c_int {
    if y < 0 as libc::c_int || x < 0 as libc::c_int {
        return 0 as libc::c_int
    } else {
        let mut v: va_list = 0 as *mut libc::c_char;
        v = args.clone();
        stbiw__writefv(s, fmt, v);
        stbiw__write_pixels(s, rgb_dir, vdir, x, y, comp, data, alpha, pad, expand_mono);
        return 1 as libc::c_int;
    };
}
unsafe extern "C" fn stbiw__write_pixels(
    mut s: *mut stbi__write_context,
    mut rgb_dir: libc::c_int,
    mut vdir: libc::c_int,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut comp: libc::c_int,
    mut data: *mut libc::c_void,
    mut write_alpha: libc::c_int,
    mut scanline_pad: libc::c_int,
    mut expand_mono: libc::c_int,
) {
    let mut zero: stbiw_uint32 = 0 as libc::c_int as stbiw_uint32;
    let mut i: libc::c_int = 0;
    let mut j: libc::c_int = 0;
    let mut j_end: libc::c_int = 0;
    if y <= 0 as libc::c_int {
        return;
    }
    if stbi__flip_vertically_on_write != 0 {
        vdir *= -(1 as libc::c_int);
    }
    if vdir < 0 as libc::c_int {
        j_end = -(1 as libc::c_int);
        j = y - 1 as libc::c_int;
    } else {
        j_end = y;
        j = 0 as libc::c_int;
    }
    while j != j_end {
        i = 0 as libc::c_int;
        while i < x {
            let mut d: *mut libc::c_uchar = (data as *mut libc::c_uchar)
                .offset(((j * x + i) * comp) as isize);
            stbiw__write_pixel(s, rgb_dir, comp, write_alpha, expand_mono, d);
            i += 1;
        }
        stbiw__write_flush(s);
        ((*s).func)
            .expect(
                "non-null function pointer",
            )(
            (*s).context,
            &mut zero as *mut stbiw_uint32 as *mut libc::c_void,
            scanline_pad,
        );
        j += vdir;
    }
}
unsafe extern "C" fn stbiw__write_flush(mut s: *mut stbi__write_context) {
    if (*s).buf_used != 0 {
        ((*s).func)
            .expect(
                "non-null function pointer",
            )(
            (*s).context,
            &mut (*s).buffer as *mut [libc::c_uchar; 64] as *mut libc::c_void,
            (*s).buf_used,
        );
        (*s).buf_used = 0 as libc::c_int;
    }
}
unsafe extern "C" fn stbiw__write_pixel(
    mut s: *mut stbi__write_context,
    mut rgb_dir: libc::c_int,
    mut comp: libc::c_int,
    mut write_alpha: libc::c_int,
    mut expand_mono: libc::c_int,
    mut d: *mut libc::c_uchar,
) {
    let mut bg: [libc::c_uchar; 3] = [
        255 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        255 as libc::c_int as libc::c_uchar,
    ];
    let mut px: [libc::c_uchar; 3] = [0; 3];
    let mut k: libc::c_int = 0;
    if write_alpha < 0 as libc::c_int {
        stbiw__write1(s, *d.offset((comp - 1 as libc::c_int) as isize));
    }
    let mut current_block_9: u64;
    match comp {
        2 | 1 => {
            if expand_mono != 0 {
                stbiw__write3(
                    s,
                    *d.offset(0 as libc::c_int as isize),
                    *d.offset(0 as libc::c_int as isize),
                    *d.offset(0 as libc::c_int as isize),
                );
            } else {
                stbiw__write1(s, *d.offset(0 as libc::c_int as isize));
            }
            current_block_9 = 13586036798005543211;
        }
        4 => {
            if write_alpha == 0 {
                k = 0 as libc::c_int;
                while k < 3 as libc::c_int {
                    px[k
                        as usize] = (bg[k as usize] as libc::c_int
                        + (*d.offset(k as isize) as libc::c_int
                            - bg[k as usize] as libc::c_int)
                            * *d.offset(3 as libc::c_int as isize) as libc::c_int
                            / 255 as libc::c_int) as libc::c_uchar;
                    k += 1;
                }
                stbiw__write3(
                    s,
                    px[(1 as libc::c_int - rgb_dir) as usize],
                    px[1 as libc::c_int as usize],
                    px[(1 as libc::c_int + rgb_dir) as usize],
                );
                current_block_9 = 13586036798005543211;
            } else {
                current_block_9 = 17770042771771916326;
            }
        }
        3 => {
            current_block_9 = 17770042771771916326;
        }
        _ => {
            current_block_9 = 13586036798005543211;
        }
    }
    match current_block_9 {
        17770042771771916326 => {
            stbiw__write3(
                s,
                *d.offset((1 as libc::c_int - rgb_dir) as isize),
                *d.offset(1 as libc::c_int as isize),
                *d.offset((1 as libc::c_int + rgb_dir) as isize),
            );
        }
        _ => {}
    }
    if write_alpha > 0 as libc::c_int {
        stbiw__write1(s, *d.offset((comp - 1 as libc::c_int) as isize));
    }
}
unsafe extern "C" fn stbiw__write1(
    mut s: *mut stbi__write_context,
    mut a: libc::c_uchar,
) {
    if ((*s).buf_used as size_t).wrapping_add(1 as libc::c_int as libc::c_ulong)
        > ::core::mem::size_of::<[libc::c_uchar; 64]>() as libc::c_ulong
    {
        stbiw__write_flush(s);
    }
    let fresh34 = (*s).buf_used;
    (*s).buf_used = (*s).buf_used + 1;
    (*s).buffer[fresh34 as usize] = a;
}
unsafe extern "C" fn stbiw__write3(
    mut s: *mut stbi__write_context,
    mut a: libc::c_uchar,
    mut b: libc::c_uchar,
    mut c: libc::c_uchar,
) {
    let mut n: libc::c_int = 0;
    if ((*s).buf_used as size_t).wrapping_add(3 as libc::c_int as libc::c_ulong)
        > ::core::mem::size_of::<[libc::c_uchar; 64]>() as libc::c_ulong
    {
        stbiw__write_flush(s);
    }
    n = (*s).buf_used;
    (*s).buf_used = n + 3 as libc::c_int;
    (*s).buffer[(n + 0 as libc::c_int) as usize] = a;
    (*s).buffer[(n + 1 as libc::c_int) as usize] = b;
    (*s).buffer[(n + 2 as libc::c_int) as usize] = c;
}
unsafe extern "C" fn stbiw__writefv(
    mut s: *mut stbi__write_context,
    mut fmt: *const libc::c_char,
    mut v: va_list,
) {
    while *fmt != 0 {
        let fresh35 = fmt;
        fmt = fmt.offset(1);
        match *fresh35 as libc::c_int {
            32 => {}
            49 => {
                let mut x: libc::c_uchar = (v.arg::<libc::c_int>() & 0xff as libc::c_int)
                    as libc::c_uchar;
                ((*s).func)
                    .expect(
                        "non-null function pointer",
                    )(
                    (*s).context,
                    &mut x as *mut libc::c_uchar as *mut libc::c_void,
                    1 as libc::c_int,
                );
            }
            50 => {
                let mut x_0: libc::c_int = v.arg::<libc::c_int>();
                let mut b: [libc::c_uchar; 2] = [0; 2];
                b[0 as libc::c_int
                    as usize] = (x_0 & 0xff as libc::c_int) as libc::c_uchar;
                b[1 as libc::c_int
                    as usize] = (x_0 >> 8 as libc::c_int & 0xff as libc::c_int)
                    as libc::c_uchar;
                ((*s).func)
                    .expect(
                        "non-null function pointer",
                    )(
                    (*s).context,
                    b.as_mut_ptr() as *mut libc::c_void,
                    2 as libc::c_int,
                );
            }
            52 => {
                let mut x_1: stbiw_uint32 = v.arg::<libc::c_int>() as stbiw_uint32;
                let mut b_0: [libc::c_uchar; 4] = [0; 4];
                b_0[0 as libc::c_int
                    as usize] = (x_1 & 0xff as libc::c_int as libc::c_uint)
                    as libc::c_uchar;
                b_0[1 as libc::c_int
                    as usize] = (x_1 >> 8 as libc::c_int
                    & 0xff as libc::c_int as libc::c_uint) as libc::c_uchar;
                b_0[2 as libc::c_int
                    as usize] = (x_1 >> 16 as libc::c_int
                    & 0xff as libc::c_int as libc::c_uint) as libc::c_uchar;
                b_0[3 as libc::c_int
                    as usize] = (x_1 >> 24 as libc::c_int
                    & 0xff as libc::c_int as libc::c_uint) as libc::c_uchar;
                ((*s).func)
                    .expect(
                        "non-null function pointer",
                    )(
                    (*s).context,
                    b_0.as_mut_ptr() as *mut libc::c_void,
                    4 as libc::c_int,
                );
            }
            _ => return,
        }
    }
}
unsafe extern "C" fn stbi__end_write_file(mut s: *mut stbi__write_context) {
    fclose((*s).context as *mut FILE);
}
unsafe extern "C" fn stbi__start_write_file(
    mut s: *mut stbi__write_context,
    mut filename: *const libc::c_char,
) -> libc::c_int {
    let mut f: *mut FILE = stbiw__fopen(
        filename,
        b"wb\0" as *const u8 as *const libc::c_char,
    );
    stbi__start_write_callbacks(
        s,
        Some(
            stbi__stdio_write
                as unsafe extern "C" fn(
                    *mut libc::c_void,
                    *mut libc::c_void,
                    libc::c_int,
                ) -> (),
        ),
        f as *mut libc::c_void,
    );
    return (f != 0 as *mut libc::c_void as *mut FILE) as libc::c_int;
}
unsafe extern "C" fn stbi__stdio_write(
    mut context: *mut libc::c_void,
    mut data: *mut libc::c_void,
    mut size: libc::c_int,
) {
    fwrite(
        data,
        1 as libc::c_int as libc::c_ulong,
        size as libc::c_ulong,
        context as *mut FILE,
    );
}
unsafe extern "C" fn stbi__start_write_callbacks(
    mut s: *mut stbi__write_context,
    mut c: Option::<stbi_write_func>,
    mut context: *mut libc::c_void,
) {
    (*s).func = c;
    (*s).context = context;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_write_tga(
    mut filename: *const libc::c_char,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut comp: libc::c_int,
    mut data: *const libc::c_void,
) -> libc::c_int {
    let mut s: stbi__write_context = {
        let mut init = stbi__write_context {
            func: None,
            context: 0 as *mut libc::c_void,
            buffer: [0; 64],
            buf_used: 0,
        };
        init
    };
    if stbi__start_write_file(&mut s, filename) != 0 {
        let mut r: libc::c_int = stbi_write_tga_core(
            &mut s,
            x,
            y,
            comp,
            data as *mut libc::c_void,
        );
        stbi__end_write_file(&mut s);
        return r;
    } else {
        return 0 as libc::c_int
    };
}
unsafe extern "C" fn stbi_write_tga_core(
    mut s: *mut stbi__write_context,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut comp: libc::c_int,
    mut data: *mut libc::c_void,
) -> libc::c_int {
    let mut has_alpha: libc::c_int = (comp == 2 as libc::c_int
        || comp == 4 as libc::c_int) as libc::c_int;
    let mut colorbytes: libc::c_int = if has_alpha != 0 {
        comp - 1 as libc::c_int
    } else {
        comp
    };
    let mut format: libc::c_int = if colorbytes < 2 as libc::c_int {
        3 as libc::c_int
    } else {
        2 as libc::c_int
    };
    if y < 0 as libc::c_int || x < 0 as libc::c_int {
        return 0 as libc::c_int;
    }
    if stbi_write_tga_with_rle == 0 {
        return stbiw__outfile(
            s,
            -(1 as libc::c_int),
            -(1 as libc::c_int),
            x,
            y,
            comp,
            0 as libc::c_int,
            data,
            has_alpha,
            0 as libc::c_int,
            b"111 221 2222 11\0" as *const u8 as *const libc::c_char,
            0 as libc::c_int,
            0 as libc::c_int,
            format,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            x,
            y,
            (colorbytes + has_alpha) * 8 as libc::c_int,
            has_alpha * 8 as libc::c_int,
        )
    } else {
        let mut i: libc::c_int = 0;
        let mut j: libc::c_int = 0;
        let mut k: libc::c_int = 0;
        let mut jend: libc::c_int = 0;
        let mut jdir: libc::c_int = 0;
        stbiw__writef(
            s,
            b"111 221 2222 11\0" as *const u8 as *const libc::c_char,
            0 as libc::c_int,
            0 as libc::c_int,
            format + 8 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            x,
            y,
            (colorbytes + has_alpha) * 8 as libc::c_int,
            has_alpha * 8 as libc::c_int,
        );
        if stbi__flip_vertically_on_write != 0 {
            j = 0 as libc::c_int;
            jend = y;
            jdir = 1 as libc::c_int;
        } else {
            j = y - 1 as libc::c_int;
            jend = -(1 as libc::c_int);
            jdir = -(1 as libc::c_int);
        }
        while j != jend {
            let mut row: *mut libc::c_uchar = (data as *mut libc::c_uchar)
                .offset((j * x * comp) as isize);
            let mut len: libc::c_int = 0;
            i = 0 as libc::c_int;
            while i < x {
                let mut begin: *mut libc::c_uchar = row.offset((i * comp) as isize);
                let mut diff: libc::c_int = 1 as libc::c_int;
                len = 1 as libc::c_int;
                if i < x - 1 as libc::c_int {
                    len += 1;
                    diff = memcmp(
                        begin as *const libc::c_void,
                        row.offset(((i + 1 as libc::c_int) * comp) as isize)
                            as *const libc::c_void,
                        comp as libc::c_ulong,
                    );
                    if diff != 0 {
                        let mut prev: *const libc::c_uchar = begin;
                        k = i + 2 as libc::c_int;
                        while k < x && len < 128 as libc::c_int {
                            if memcmp(
                                prev as *const libc::c_void,
                                row.offset((k * comp) as isize) as *const libc::c_void,
                                comp as libc::c_ulong,
                            ) != 0
                            {
                                prev = prev.offset(comp as isize);
                                len += 1;
                                k += 1;
                            } else {
                                len -= 1;
                                break;
                            }
                        }
                    } else {
                        k = i + 2 as libc::c_int;
                        while k < x && len < 128 as libc::c_int {
                            if !(memcmp(
                                begin as *const libc::c_void,
                                row.offset((k * comp) as isize) as *const libc::c_void,
                                comp as libc::c_ulong,
                            ) == 0)
                            {
                                break;
                            }
                            len += 1;
                            k += 1;
                        }
                    }
                }
                if diff != 0 {
                    let mut header: libc::c_uchar = (len - 1 as libc::c_int
                        & 0xff as libc::c_int) as libc::c_uchar;
                    stbiw__write1(s, header);
                    k = 0 as libc::c_int;
                    while k < len {
                        stbiw__write_pixel(
                            s,
                            -(1 as libc::c_int),
                            comp,
                            has_alpha,
                            0 as libc::c_int,
                            begin.offset((k * comp) as isize),
                        );
                        k += 1;
                    }
                } else {
                    let mut header_0: libc::c_uchar = (len - 129 as libc::c_int
                        & 0xff as libc::c_int) as libc::c_uchar;
                    stbiw__write1(s, header_0);
                    stbiw__write_pixel(
                        s,
                        -(1 as libc::c_int),
                        comp,
                        has_alpha,
                        0 as libc::c_int,
                        begin,
                    );
                }
                i += len;
            }
            j += jdir;
        }
        stbiw__write_flush(s);
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbiw__writef(
    mut s: *mut stbi__write_context,
    mut fmt: *const libc::c_char,
    mut args: ...
) {
    let mut v: va_list = 0 as *mut libc::c_char;
    v = args.clone();
    stbiw__writefv(s, fmt, v);
}
#[no_mangle]
pub unsafe extern "C" fn stbi_write_hdr(
    mut filename: *const libc::c_char,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut comp: libc::c_int,
    mut data: *const libc::c_float,
) -> libc::c_int {
    let mut s: stbi__write_context = {
        let mut init = stbi__write_context {
            func: None,
            context: 0 as *mut libc::c_void,
            buffer: [0; 64],
            buf_used: 0,
        };
        init
    };
    if stbi__start_write_file(&mut s, filename) != 0 {
        let mut r: libc::c_int = stbi_write_hdr_core(
            &mut s,
            x,
            y,
            comp,
            data as *mut libc::c_float,
        );
        stbi__end_write_file(&mut s);
        return r;
    } else {
        return 0 as libc::c_int
    };
}
unsafe extern "C" fn stbi_write_hdr_core(
    mut s: *mut stbi__write_context,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut comp: libc::c_int,
    mut data: *mut libc::c_float,
) -> libc::c_int {
    if y <= 0 as libc::c_int || x <= 0 as libc::c_int || data.is_null() {
        return 0 as libc::c_int
    } else {
        let mut scratch: *mut libc::c_uchar = malloc(
            (x * 4 as libc::c_int) as libc::c_ulong,
        ) as *mut libc::c_uchar;
        let mut i: libc::c_int = 0;
        let mut len: libc::c_int = 0;
        let mut buffer: [libc::c_char; 128] = [0; 128];
        let mut header: [libc::c_char; 66] = *::core::mem::transmute::<
            &[u8; 66],
            &mut [libc::c_char; 66],
        >(b"#?RADIANCE\n# Written by stb_image_write.h\nFORMAT=32-bit_rle_rgbe\n\0");
        ((*s).func)
            .expect(
                "non-null function pointer",
            )(
            (*s).context,
            header.as_mut_ptr() as *mut libc::c_void,
            (::core::mem::size_of::<[libc::c_char; 66]>() as libc::c_ulong)
                .wrapping_sub(1 as libc::c_int as libc::c_ulong) as libc::c_int,
        );
        len = sprintf(
            buffer.as_mut_ptr(),
            b"EXPOSURE=          1.0000000000000\n\n-Y %d +X %d\n\0" as *const u8
                as *const libc::c_char,
            y,
            x,
        );
        ((*s).func)
            .expect(
                "non-null function pointer",
            )((*s).context, buffer.as_mut_ptr() as *mut libc::c_void, len);
        i = 0 as libc::c_int;
        while i < y {
            stbiw__write_hdr_scanline(
                s,
                x,
                comp,
                scratch,
                data
                    .offset(
                        (comp * x
                            * (if stbi__flip_vertically_on_write != 0 {
                                y - 1 as libc::c_int - i
                            } else {
                                i
                            })) as isize,
                    ),
            );
            i += 1;
        }
        free(scratch as *mut libc::c_void);
        return 1 as libc::c_int;
    };
}
unsafe extern "C" fn stbiw__write_hdr_scanline(
    mut s: *mut stbi__write_context,
    mut width: libc::c_int,
    mut ncomp: libc::c_int,
    mut scratch: *mut libc::c_uchar,
    mut scanline: *mut libc::c_float,
) {
    let mut scanlineheader: [libc::c_uchar; 4] = [
        2 as libc::c_int as libc::c_uchar,
        2 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
    ];
    let mut rgbe: [libc::c_uchar; 4] = [0; 4];
    let mut linear: [libc::c_float; 3] = [0.; 3];
    let mut x: libc::c_int = 0;
    scanlineheader[2 as libc::c_int
        as usize] = ((width & 0xff00 as libc::c_int) >> 8 as libc::c_int)
        as libc::c_uchar;
    scanlineheader[3 as libc::c_int
        as usize] = (width & 0xff as libc::c_int) as libc::c_uchar;
    if width < 8 as libc::c_int || width >= 32768 as libc::c_int {
        x = 0 as libc::c_int;
        while x < width {
            match ncomp {
                4 | 3 => {
                    linear[2 as libc::c_int
                        as usize] = *scanline
                        .offset((x * ncomp + 2 as libc::c_int) as isize);
                    linear[1 as libc::c_int
                        as usize] = *scanline
                        .offset((x * ncomp + 1 as libc::c_int) as isize);
                    linear[0 as libc::c_int
                        as usize] = *scanline
                        .offset((x * ncomp + 0 as libc::c_int) as isize);
                }
                _ => {
                    linear[2 as libc::c_int
                        as usize] = *scanline
                        .offset((x * ncomp + 0 as libc::c_int) as isize);
                    linear[1 as libc::c_int
                        as usize] = linear[2 as libc::c_int as usize];
                    linear[0 as libc::c_int
                        as usize] = linear[1 as libc::c_int as usize];
                }
            }
            stbiw__linear_to_rgbe(rgbe.as_mut_ptr(), linear.as_mut_ptr());
            ((*s).func)
                .expect(
                    "non-null function pointer",
                )(
                (*s).context,
                rgbe.as_mut_ptr() as *mut libc::c_void,
                4 as libc::c_int,
            );
            x += 1;
        }
    } else {
        let mut c: libc::c_int = 0;
        let mut r: libc::c_int = 0;
        x = 0 as libc::c_int;
        while x < width {
            match ncomp {
                4 | 3 => {
                    linear[2 as libc::c_int
                        as usize] = *scanline
                        .offset((x * ncomp + 2 as libc::c_int) as isize);
                    linear[1 as libc::c_int
                        as usize] = *scanline
                        .offset((x * ncomp + 1 as libc::c_int) as isize);
                    linear[0 as libc::c_int
                        as usize] = *scanline
                        .offset((x * ncomp + 0 as libc::c_int) as isize);
                }
                _ => {
                    linear[2 as libc::c_int
                        as usize] = *scanline
                        .offset((x * ncomp + 0 as libc::c_int) as isize);
                    linear[1 as libc::c_int
                        as usize] = linear[2 as libc::c_int as usize];
                    linear[0 as libc::c_int
                        as usize] = linear[1 as libc::c_int as usize];
                }
            }
            stbiw__linear_to_rgbe(rgbe.as_mut_ptr(), linear.as_mut_ptr());
            *scratch
                .offset(
                    (x + width * 0 as libc::c_int) as isize,
                ) = rgbe[0 as libc::c_int as usize];
            *scratch
                .offset(
                    (x + width * 1 as libc::c_int) as isize,
                ) = rgbe[1 as libc::c_int as usize];
            *scratch
                .offset(
                    (x + width * 2 as libc::c_int) as isize,
                ) = rgbe[2 as libc::c_int as usize];
            *scratch
                .offset(
                    (x + width * 3 as libc::c_int) as isize,
                ) = rgbe[3 as libc::c_int as usize];
            x += 1;
        }
        ((*s).func)
            .expect(
                "non-null function pointer",
            )(
            (*s).context,
            scanlineheader.as_mut_ptr() as *mut libc::c_void,
            4 as libc::c_int,
        );
        c = 0 as libc::c_int;
        while c < 4 as libc::c_int {
            let mut comp: *mut libc::c_uchar = &mut *scratch.offset((width * c) as isize)
                as *mut libc::c_uchar;
            x = 0 as libc::c_int;
            while x < width {
                r = x;
                while (r + 2 as libc::c_int) < width {
                    if *comp.offset(r as isize) as libc::c_int
                        == *comp.offset((r + 1 as libc::c_int) as isize) as libc::c_int
                        && *comp.offset(r as isize) as libc::c_int
                            == *comp.offset((r + 2 as libc::c_int) as isize)
                                as libc::c_int
                    {
                        break;
                    }
                    r += 1;
                }
                if r + 2 as libc::c_int >= width {
                    r = width;
                }
                while x < r {
                    let mut len: libc::c_int = r - x;
                    if len > 128 as libc::c_int {
                        len = 128 as libc::c_int;
                    }
                    stbiw__write_dump_data(s, len, &mut *comp.offset(x as isize));
                    x += len;
                }
                if (r + 2 as libc::c_int) < width {
                    while r < width
                        && *comp.offset(r as isize) as libc::c_int
                            == *comp.offset(x as isize) as libc::c_int
                    {
                        r += 1;
                    }
                    while x < r {
                        let mut len_0: libc::c_int = r - x;
                        if len_0 > 127 as libc::c_int {
                            len_0 = 127 as libc::c_int;
                        }
                        stbiw__write_run_data(s, len_0, *comp.offset(x as isize));
                        x += len_0;
                    }
                }
            }
            c += 1;
        }
    };
}
unsafe extern "C" fn stbiw__write_run_data(
    mut s: *mut stbi__write_context,
    mut length: libc::c_int,
    mut databyte: libc::c_uchar,
) {
    let mut lengthbyte: libc::c_uchar = (length + 128 as libc::c_int
        & 0xff as libc::c_int) as libc::c_uchar;
    ((*s).func)
        .expect(
            "non-null function pointer",
        )(
        (*s).context,
        &mut lengthbyte as *mut libc::c_uchar as *mut libc::c_void,
        1 as libc::c_int,
    );
    ((*s).func)
        .expect(
            "non-null function pointer",
        )(
        (*s).context,
        &mut databyte as *mut libc::c_uchar as *mut libc::c_void,
        1 as libc::c_int,
    );
}
unsafe extern "C" fn stbiw__write_dump_data(
    mut s: *mut stbi__write_context,
    mut length: libc::c_int,
    mut data: *mut libc::c_uchar,
) {
    let mut lengthbyte: libc::c_uchar = (length & 0xff as libc::c_int) as libc::c_uchar;
    ((*s).func)
        .expect(
            "non-null function pointer",
        )(
        (*s).context,
        &mut lengthbyte as *mut libc::c_uchar as *mut libc::c_void,
        1 as libc::c_int,
    );
    ((*s).func)
        .expect(
            "non-null function pointer",
        )((*s).context, data as *mut libc::c_void, length);
}
unsafe extern "C" fn stbiw__linear_to_rgbe(
    mut rgbe: *mut libc::c_uchar,
    mut linear: *mut libc::c_float,
) {
    let mut exponent: libc::c_int = 0;
    let mut maxcomp: libc::c_float = if *linear.offset(0 as libc::c_int as isize)
        > (if *linear.offset(1 as libc::c_int as isize)
            > *linear.offset(2 as libc::c_int as isize)
        {
            *linear.offset(1 as libc::c_int as isize)
        } else {
            *linear.offset(2 as libc::c_int as isize)
        })
    {
        *linear.offset(0 as libc::c_int as isize)
    } else if *linear.offset(1 as libc::c_int as isize)
        > *linear.offset(2 as libc::c_int as isize)
    {
        *linear.offset(1 as libc::c_int as isize)
    } else {
        *linear.offset(2 as libc::c_int as isize)
    };
    if maxcomp < 1e-32f32 {
        let ref mut fresh36 = *rgbe.offset(3 as libc::c_int as isize);
        *fresh36 = 0 as libc::c_int as libc::c_uchar;
        let ref mut fresh37 = *rgbe.offset(2 as libc::c_int as isize);
        *fresh37 = *fresh36;
        let ref mut fresh38 = *rgbe.offset(1 as libc::c_int as isize);
        *fresh38 = *fresh37;
        *rgbe.offset(0 as libc::c_int as isize) = *fresh38;
    } else {
        let mut normalize: libc::c_float = frexp(
            maxcomp as libc::c_double,
            &mut exponent,
        ) as libc::c_float * 256.0f32 / maxcomp;
        *rgbe
            .offset(
                0 as libc::c_int as isize,
            ) = (*linear.offset(0 as libc::c_int as isize) * normalize) as libc::c_uchar;
        *rgbe
            .offset(
                1 as libc::c_int as isize,
            ) = (*linear.offset(1 as libc::c_int as isize) * normalize) as libc::c_uchar;
        *rgbe
            .offset(
                2 as libc::c_int as isize,
            ) = (*linear.offset(2 as libc::c_int as isize) * normalize) as libc::c_uchar;
        *rgbe
            .offset(
                3 as libc::c_int as isize,
            ) = (exponent + 128 as libc::c_int) as libc::c_uchar;
    };
}
#[no_mangle]
pub unsafe extern "C" fn stbi_write_jpg(
    mut filename: *const libc::c_char,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut comp: libc::c_int,
    mut data: *const libc::c_void,
    mut quality: libc::c_int,
) -> libc::c_int {
    let mut s: stbi__write_context = {
        let mut init = stbi__write_context {
            func: None,
            context: 0 as *mut libc::c_void,
            buffer: [0; 64],
            buf_used: 0,
        };
        init
    };
    if stbi__start_write_file(&mut s, filename) != 0 {
        let mut r: libc::c_int = stbi_write_jpg_core(&mut s, x, y, comp, data, quality);
        stbi__end_write_file(&mut s);
        return r;
    } else {
        return 0 as libc::c_int
    };
}
unsafe extern "C" fn stbi_write_jpg_core(
    mut s: *mut stbi__write_context,
    mut width: libc::c_int,
    mut height: libc::c_int,
    mut comp: libc::c_int,
    mut data: *const libc::c_void,
    mut quality: libc::c_int,
) -> libc::c_int {
    static mut std_dc_luminance_nrcodes: [libc::c_uchar; 17] = [
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        5 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
    ];
    static mut std_dc_luminance_values: [libc::c_uchar; 12] = [
        0 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        2 as libc::c_int as libc::c_uchar,
        3 as libc::c_int as libc::c_uchar,
        4 as libc::c_int as libc::c_uchar,
        5 as libc::c_int as libc::c_uchar,
        6 as libc::c_int as libc::c_uchar,
        7 as libc::c_int as libc::c_uchar,
        8 as libc::c_int as libc::c_uchar,
        9 as libc::c_int as libc::c_uchar,
        10 as libc::c_int as libc::c_uchar,
        11 as libc::c_int as libc::c_uchar,
    ];
    static mut std_ac_luminance_nrcodes: [libc::c_uchar; 17] = [
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        2 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        3 as libc::c_int as libc::c_uchar,
        3 as libc::c_int as libc::c_uchar,
        2 as libc::c_int as libc::c_uchar,
        4 as libc::c_int as libc::c_uchar,
        3 as libc::c_int as libc::c_uchar,
        5 as libc::c_int as libc::c_uchar,
        5 as libc::c_int as libc::c_uchar,
        4 as libc::c_int as libc::c_uchar,
        4 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        0x7d as libc::c_int as libc::c_uchar,
    ];
    static mut std_ac_luminance_values: [libc::c_uchar; 162] = [
        0x1 as libc::c_int as libc::c_uchar,
        0x2 as libc::c_int as libc::c_uchar,
        0x3 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0x4 as libc::c_int as libc::c_uchar,
        0x11 as libc::c_int as libc::c_uchar,
        0x5 as libc::c_int as libc::c_uchar,
        0x12 as libc::c_int as libc::c_uchar,
        0x21 as libc::c_int as libc::c_uchar,
        0x31 as libc::c_int as libc::c_uchar,
        0x41 as libc::c_int as libc::c_uchar,
        0x6 as libc::c_int as libc::c_uchar,
        0x13 as libc::c_int as libc::c_uchar,
        0x51 as libc::c_int as libc::c_uchar,
        0x61 as libc::c_int as libc::c_uchar,
        0x7 as libc::c_int as libc::c_uchar,
        0x22 as libc::c_int as libc::c_uchar,
        0x71 as libc::c_int as libc::c_uchar,
        0x14 as libc::c_int as libc::c_uchar,
        0x32 as libc::c_int as libc::c_uchar,
        0x81 as libc::c_int as libc::c_uchar,
        0x91 as libc::c_int as libc::c_uchar,
        0xa1 as libc::c_int as libc::c_uchar,
        0x8 as libc::c_int as libc::c_uchar,
        0x23 as libc::c_int as libc::c_uchar,
        0x42 as libc::c_int as libc::c_uchar,
        0xb1 as libc::c_int as libc::c_uchar,
        0xc1 as libc::c_int as libc::c_uchar,
        0x15 as libc::c_int as libc::c_uchar,
        0x52 as libc::c_int as libc::c_uchar,
        0xd1 as libc::c_int as libc::c_uchar,
        0xf0 as libc::c_int as libc::c_uchar,
        0x24 as libc::c_int as libc::c_uchar,
        0x33 as libc::c_int as libc::c_uchar,
        0x62 as libc::c_int as libc::c_uchar,
        0x72 as libc::c_int as libc::c_uchar,
        0x82 as libc::c_int as libc::c_uchar,
        0x9 as libc::c_int as libc::c_uchar,
        0xa as libc::c_int as libc::c_uchar,
        0x16 as libc::c_int as libc::c_uchar,
        0x17 as libc::c_int as libc::c_uchar,
        0x18 as libc::c_int as libc::c_uchar,
        0x19 as libc::c_int as libc::c_uchar,
        0x1a as libc::c_int as libc::c_uchar,
        0x25 as libc::c_int as libc::c_uchar,
        0x26 as libc::c_int as libc::c_uchar,
        0x27 as libc::c_int as libc::c_uchar,
        0x28 as libc::c_int as libc::c_uchar,
        0x29 as libc::c_int as libc::c_uchar,
        0x2a as libc::c_int as libc::c_uchar,
        0x34 as libc::c_int as libc::c_uchar,
        0x35 as libc::c_int as libc::c_uchar,
        0x36 as libc::c_int as libc::c_uchar,
        0x37 as libc::c_int as libc::c_uchar,
        0x38 as libc::c_int as libc::c_uchar,
        0x39 as libc::c_int as libc::c_uchar,
        0x3a as libc::c_int as libc::c_uchar,
        0x43 as libc::c_int as libc::c_uchar,
        0x44 as libc::c_int as libc::c_uchar,
        0x45 as libc::c_int as libc::c_uchar,
        0x46 as libc::c_int as libc::c_uchar,
        0x47 as libc::c_int as libc::c_uchar,
        0x48 as libc::c_int as libc::c_uchar,
        0x49 as libc::c_int as libc::c_uchar,
        0x4a as libc::c_int as libc::c_uchar,
        0x53 as libc::c_int as libc::c_uchar,
        0x54 as libc::c_int as libc::c_uchar,
        0x55 as libc::c_int as libc::c_uchar,
        0x56 as libc::c_int as libc::c_uchar,
        0x57 as libc::c_int as libc::c_uchar,
        0x58 as libc::c_int as libc::c_uchar,
        0x59 as libc::c_int as libc::c_uchar,
        0x5a as libc::c_int as libc::c_uchar,
        0x63 as libc::c_int as libc::c_uchar,
        0x64 as libc::c_int as libc::c_uchar,
        0x65 as libc::c_int as libc::c_uchar,
        0x66 as libc::c_int as libc::c_uchar,
        0x67 as libc::c_int as libc::c_uchar,
        0x68 as libc::c_int as libc::c_uchar,
        0x69 as libc::c_int as libc::c_uchar,
        0x6a as libc::c_int as libc::c_uchar,
        0x73 as libc::c_int as libc::c_uchar,
        0x74 as libc::c_int as libc::c_uchar,
        0x75 as libc::c_int as libc::c_uchar,
        0x76 as libc::c_int as libc::c_uchar,
        0x77 as libc::c_int as libc::c_uchar,
        0x78 as libc::c_int as libc::c_uchar,
        0x79 as libc::c_int as libc::c_uchar,
        0x7a as libc::c_int as libc::c_uchar,
        0x83 as libc::c_int as libc::c_uchar,
        0x84 as libc::c_int as libc::c_uchar,
        0x85 as libc::c_int as libc::c_uchar,
        0x86 as libc::c_int as libc::c_uchar,
        0x87 as libc::c_int as libc::c_uchar,
        0x88 as libc::c_int as libc::c_uchar,
        0x89 as libc::c_int as libc::c_uchar,
        0x8a as libc::c_int as libc::c_uchar,
        0x92 as libc::c_int as libc::c_uchar,
        0x93 as libc::c_int as libc::c_uchar,
        0x94 as libc::c_int as libc::c_uchar,
        0x95 as libc::c_int as libc::c_uchar,
        0x96 as libc::c_int as libc::c_uchar,
        0x97 as libc::c_int as libc::c_uchar,
        0x98 as libc::c_int as libc::c_uchar,
        0x99 as libc::c_int as libc::c_uchar,
        0x9a as libc::c_int as libc::c_uchar,
        0xa2 as libc::c_int as libc::c_uchar,
        0xa3 as libc::c_int as libc::c_uchar,
        0xa4 as libc::c_int as libc::c_uchar,
        0xa5 as libc::c_int as libc::c_uchar,
        0xa6 as libc::c_int as libc::c_uchar,
        0xa7 as libc::c_int as libc::c_uchar,
        0xa8 as libc::c_int as libc::c_uchar,
        0xa9 as libc::c_int as libc::c_uchar,
        0xaa as libc::c_int as libc::c_uchar,
        0xb2 as libc::c_int as libc::c_uchar,
        0xb3 as libc::c_int as libc::c_uchar,
        0xb4 as libc::c_int as libc::c_uchar,
        0xb5 as libc::c_int as libc::c_uchar,
        0xb6 as libc::c_int as libc::c_uchar,
        0xb7 as libc::c_int as libc::c_uchar,
        0xb8 as libc::c_int as libc::c_uchar,
        0xb9 as libc::c_int as libc::c_uchar,
        0xba as libc::c_int as libc::c_uchar,
        0xc2 as libc::c_int as libc::c_uchar,
        0xc3 as libc::c_int as libc::c_uchar,
        0xc4 as libc::c_int as libc::c_uchar,
        0xc5 as libc::c_int as libc::c_uchar,
        0xc6 as libc::c_int as libc::c_uchar,
        0xc7 as libc::c_int as libc::c_uchar,
        0xc8 as libc::c_int as libc::c_uchar,
        0xc9 as libc::c_int as libc::c_uchar,
        0xca as libc::c_int as libc::c_uchar,
        0xd2 as libc::c_int as libc::c_uchar,
        0xd3 as libc::c_int as libc::c_uchar,
        0xd4 as libc::c_int as libc::c_uchar,
        0xd5 as libc::c_int as libc::c_uchar,
        0xd6 as libc::c_int as libc::c_uchar,
        0xd7 as libc::c_int as libc::c_uchar,
        0xd8 as libc::c_int as libc::c_uchar,
        0xd9 as libc::c_int as libc::c_uchar,
        0xda as libc::c_int as libc::c_uchar,
        0xe1 as libc::c_int as libc::c_uchar,
        0xe2 as libc::c_int as libc::c_uchar,
        0xe3 as libc::c_int as libc::c_uchar,
        0xe4 as libc::c_int as libc::c_uchar,
        0xe5 as libc::c_int as libc::c_uchar,
        0xe6 as libc::c_int as libc::c_uchar,
        0xe7 as libc::c_int as libc::c_uchar,
        0xe8 as libc::c_int as libc::c_uchar,
        0xe9 as libc::c_int as libc::c_uchar,
        0xea as libc::c_int as libc::c_uchar,
        0xf1 as libc::c_int as libc::c_uchar,
        0xf2 as libc::c_int as libc::c_uchar,
        0xf3 as libc::c_int as libc::c_uchar,
        0xf4 as libc::c_int as libc::c_uchar,
        0xf5 as libc::c_int as libc::c_uchar,
        0xf6 as libc::c_int as libc::c_uchar,
        0xf7 as libc::c_int as libc::c_uchar,
        0xf8 as libc::c_int as libc::c_uchar,
        0xf9 as libc::c_int as libc::c_uchar,
        0xfa as libc::c_int as libc::c_uchar,
    ];
    static mut std_dc_chrominance_nrcodes: [libc::c_uchar; 17] = [
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        3 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
    ];
    static mut std_dc_chrominance_values: [libc::c_uchar; 12] = [
        0 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        2 as libc::c_int as libc::c_uchar,
        3 as libc::c_int as libc::c_uchar,
        4 as libc::c_int as libc::c_uchar,
        5 as libc::c_int as libc::c_uchar,
        6 as libc::c_int as libc::c_uchar,
        7 as libc::c_int as libc::c_uchar,
        8 as libc::c_int as libc::c_uchar,
        9 as libc::c_int as libc::c_uchar,
        10 as libc::c_int as libc::c_uchar,
        11 as libc::c_int as libc::c_uchar,
    ];
    static mut std_ac_chrominance_nrcodes: [libc::c_uchar; 17] = [
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        2 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        2 as libc::c_int as libc::c_uchar,
        4 as libc::c_int as libc::c_uchar,
        4 as libc::c_int as libc::c_uchar,
        3 as libc::c_int as libc::c_uchar,
        4 as libc::c_int as libc::c_uchar,
        7 as libc::c_int as libc::c_uchar,
        5 as libc::c_int as libc::c_uchar,
        4 as libc::c_int as libc::c_uchar,
        4 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        2 as libc::c_int as libc::c_uchar,
        0x77 as libc::c_int as libc::c_uchar,
    ];
    static mut std_ac_chrominance_values: [libc::c_uchar; 162] = [
        0 as libc::c_int as libc::c_uchar,
        0x1 as libc::c_int as libc::c_uchar,
        0x2 as libc::c_int as libc::c_uchar,
        0x3 as libc::c_int as libc::c_uchar,
        0x11 as libc::c_int as libc::c_uchar,
        0x4 as libc::c_int as libc::c_uchar,
        0x5 as libc::c_int as libc::c_uchar,
        0x21 as libc::c_int as libc::c_uchar,
        0x31 as libc::c_int as libc::c_uchar,
        0x6 as libc::c_int as libc::c_uchar,
        0x12 as libc::c_int as libc::c_uchar,
        0x41 as libc::c_int as libc::c_uchar,
        0x51 as libc::c_int as libc::c_uchar,
        0x7 as libc::c_int as libc::c_uchar,
        0x61 as libc::c_int as libc::c_uchar,
        0x71 as libc::c_int as libc::c_uchar,
        0x13 as libc::c_int as libc::c_uchar,
        0x22 as libc::c_int as libc::c_uchar,
        0x32 as libc::c_int as libc::c_uchar,
        0x81 as libc::c_int as libc::c_uchar,
        0x8 as libc::c_int as libc::c_uchar,
        0x14 as libc::c_int as libc::c_uchar,
        0x42 as libc::c_int as libc::c_uchar,
        0x91 as libc::c_int as libc::c_uchar,
        0xa1 as libc::c_int as libc::c_uchar,
        0xb1 as libc::c_int as libc::c_uchar,
        0xc1 as libc::c_int as libc::c_uchar,
        0x9 as libc::c_int as libc::c_uchar,
        0x23 as libc::c_int as libc::c_uchar,
        0x33 as libc::c_int as libc::c_uchar,
        0x52 as libc::c_int as libc::c_uchar,
        0xf0 as libc::c_int as libc::c_uchar,
        0x15 as libc::c_int as libc::c_uchar,
        0x62 as libc::c_int as libc::c_uchar,
        0x72 as libc::c_int as libc::c_uchar,
        0xd1 as libc::c_int as libc::c_uchar,
        0xa as libc::c_int as libc::c_uchar,
        0x16 as libc::c_int as libc::c_uchar,
        0x24 as libc::c_int as libc::c_uchar,
        0x34 as libc::c_int as libc::c_uchar,
        0xe1 as libc::c_int as libc::c_uchar,
        0x25 as libc::c_int as libc::c_uchar,
        0xf1 as libc::c_int as libc::c_uchar,
        0x17 as libc::c_int as libc::c_uchar,
        0x18 as libc::c_int as libc::c_uchar,
        0x19 as libc::c_int as libc::c_uchar,
        0x1a as libc::c_int as libc::c_uchar,
        0x26 as libc::c_int as libc::c_uchar,
        0x27 as libc::c_int as libc::c_uchar,
        0x28 as libc::c_int as libc::c_uchar,
        0x29 as libc::c_int as libc::c_uchar,
        0x2a as libc::c_int as libc::c_uchar,
        0x35 as libc::c_int as libc::c_uchar,
        0x36 as libc::c_int as libc::c_uchar,
        0x37 as libc::c_int as libc::c_uchar,
        0x38 as libc::c_int as libc::c_uchar,
        0x39 as libc::c_int as libc::c_uchar,
        0x3a as libc::c_int as libc::c_uchar,
        0x43 as libc::c_int as libc::c_uchar,
        0x44 as libc::c_int as libc::c_uchar,
        0x45 as libc::c_int as libc::c_uchar,
        0x46 as libc::c_int as libc::c_uchar,
        0x47 as libc::c_int as libc::c_uchar,
        0x48 as libc::c_int as libc::c_uchar,
        0x49 as libc::c_int as libc::c_uchar,
        0x4a as libc::c_int as libc::c_uchar,
        0x53 as libc::c_int as libc::c_uchar,
        0x54 as libc::c_int as libc::c_uchar,
        0x55 as libc::c_int as libc::c_uchar,
        0x56 as libc::c_int as libc::c_uchar,
        0x57 as libc::c_int as libc::c_uchar,
        0x58 as libc::c_int as libc::c_uchar,
        0x59 as libc::c_int as libc::c_uchar,
        0x5a as libc::c_int as libc::c_uchar,
        0x63 as libc::c_int as libc::c_uchar,
        0x64 as libc::c_int as libc::c_uchar,
        0x65 as libc::c_int as libc::c_uchar,
        0x66 as libc::c_int as libc::c_uchar,
        0x67 as libc::c_int as libc::c_uchar,
        0x68 as libc::c_int as libc::c_uchar,
        0x69 as libc::c_int as libc::c_uchar,
        0x6a as libc::c_int as libc::c_uchar,
        0x73 as libc::c_int as libc::c_uchar,
        0x74 as libc::c_int as libc::c_uchar,
        0x75 as libc::c_int as libc::c_uchar,
        0x76 as libc::c_int as libc::c_uchar,
        0x77 as libc::c_int as libc::c_uchar,
        0x78 as libc::c_int as libc::c_uchar,
        0x79 as libc::c_int as libc::c_uchar,
        0x7a as libc::c_int as libc::c_uchar,
        0x82 as libc::c_int as libc::c_uchar,
        0x83 as libc::c_int as libc::c_uchar,
        0x84 as libc::c_int as libc::c_uchar,
        0x85 as libc::c_int as libc::c_uchar,
        0x86 as libc::c_int as libc::c_uchar,
        0x87 as libc::c_int as libc::c_uchar,
        0x88 as libc::c_int as libc::c_uchar,
        0x89 as libc::c_int as libc::c_uchar,
        0x8a as libc::c_int as libc::c_uchar,
        0x92 as libc::c_int as libc::c_uchar,
        0x93 as libc::c_int as libc::c_uchar,
        0x94 as libc::c_int as libc::c_uchar,
        0x95 as libc::c_int as libc::c_uchar,
        0x96 as libc::c_int as libc::c_uchar,
        0x97 as libc::c_int as libc::c_uchar,
        0x98 as libc::c_int as libc::c_uchar,
        0x99 as libc::c_int as libc::c_uchar,
        0x9a as libc::c_int as libc::c_uchar,
        0xa2 as libc::c_int as libc::c_uchar,
        0xa3 as libc::c_int as libc::c_uchar,
        0xa4 as libc::c_int as libc::c_uchar,
        0xa5 as libc::c_int as libc::c_uchar,
        0xa6 as libc::c_int as libc::c_uchar,
        0xa7 as libc::c_int as libc::c_uchar,
        0xa8 as libc::c_int as libc::c_uchar,
        0xa9 as libc::c_int as libc::c_uchar,
        0xaa as libc::c_int as libc::c_uchar,
        0xb2 as libc::c_int as libc::c_uchar,
        0xb3 as libc::c_int as libc::c_uchar,
        0xb4 as libc::c_int as libc::c_uchar,
        0xb5 as libc::c_int as libc::c_uchar,
        0xb6 as libc::c_int as libc::c_uchar,
        0xb7 as libc::c_int as libc::c_uchar,
        0xb8 as libc::c_int as libc::c_uchar,
        0xb9 as libc::c_int as libc::c_uchar,
        0xba as libc::c_int as libc::c_uchar,
        0xc2 as libc::c_int as libc::c_uchar,
        0xc3 as libc::c_int as libc::c_uchar,
        0xc4 as libc::c_int as libc::c_uchar,
        0xc5 as libc::c_int as libc::c_uchar,
        0xc6 as libc::c_int as libc::c_uchar,
        0xc7 as libc::c_int as libc::c_uchar,
        0xc8 as libc::c_int as libc::c_uchar,
        0xc9 as libc::c_int as libc::c_uchar,
        0xca as libc::c_int as libc::c_uchar,
        0xd2 as libc::c_int as libc::c_uchar,
        0xd3 as libc::c_int as libc::c_uchar,
        0xd4 as libc::c_int as libc::c_uchar,
        0xd5 as libc::c_int as libc::c_uchar,
        0xd6 as libc::c_int as libc::c_uchar,
        0xd7 as libc::c_int as libc::c_uchar,
        0xd8 as libc::c_int as libc::c_uchar,
        0xd9 as libc::c_int as libc::c_uchar,
        0xda as libc::c_int as libc::c_uchar,
        0xe2 as libc::c_int as libc::c_uchar,
        0xe3 as libc::c_int as libc::c_uchar,
        0xe4 as libc::c_int as libc::c_uchar,
        0xe5 as libc::c_int as libc::c_uchar,
        0xe6 as libc::c_int as libc::c_uchar,
        0xe7 as libc::c_int as libc::c_uchar,
        0xe8 as libc::c_int as libc::c_uchar,
        0xe9 as libc::c_int as libc::c_uchar,
        0xea as libc::c_int as libc::c_uchar,
        0xf2 as libc::c_int as libc::c_uchar,
        0xf3 as libc::c_int as libc::c_uchar,
        0xf4 as libc::c_int as libc::c_uchar,
        0xf5 as libc::c_int as libc::c_uchar,
        0xf6 as libc::c_int as libc::c_uchar,
        0xf7 as libc::c_int as libc::c_uchar,
        0xf8 as libc::c_int as libc::c_uchar,
        0xf9 as libc::c_int as libc::c_uchar,
        0xfa as libc::c_int as libc::c_uchar,
    ];
    static mut YDC_HT: [[libc::c_ushort; 2]; 256] = [
        [0 as libc::c_int as libc::c_ushort, 2 as libc::c_int as libc::c_ushort],
        [2 as libc::c_int as libc::c_ushort, 3 as libc::c_int as libc::c_ushort],
        [3 as libc::c_int as libc::c_ushort, 3 as libc::c_int as libc::c_ushort],
        [4 as libc::c_int as libc::c_ushort, 3 as libc::c_int as libc::c_ushort],
        [5 as libc::c_int as libc::c_ushort, 3 as libc::c_int as libc::c_ushort],
        [6 as libc::c_int as libc::c_ushort, 3 as libc::c_int as libc::c_ushort],
        [14 as libc::c_int as libc::c_ushort, 4 as libc::c_int as libc::c_ushort],
        [30 as libc::c_int as libc::c_ushort, 5 as libc::c_int as libc::c_ushort],
        [62 as libc::c_int as libc::c_ushort, 6 as libc::c_int as libc::c_ushort],
        [126 as libc::c_int as libc::c_ushort, 7 as libc::c_int as libc::c_ushort],
        [254 as libc::c_int as libc::c_ushort, 8 as libc::c_int as libc::c_ushort],
        [510 as libc::c_int as libc::c_ushort, 9 as libc::c_int as libc::c_ushort],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
    ];
    static mut UVDC_HT: [[libc::c_ushort; 2]; 256] = [
        [0 as libc::c_int as libc::c_ushort, 2 as libc::c_int as libc::c_ushort],
        [1 as libc::c_int as libc::c_ushort, 2 as libc::c_int as libc::c_ushort],
        [2 as libc::c_int as libc::c_ushort, 2 as libc::c_int as libc::c_ushort],
        [6 as libc::c_int as libc::c_ushort, 3 as libc::c_int as libc::c_ushort],
        [14 as libc::c_int as libc::c_ushort, 4 as libc::c_int as libc::c_ushort],
        [30 as libc::c_int as libc::c_ushort, 5 as libc::c_int as libc::c_ushort],
        [62 as libc::c_int as libc::c_ushort, 6 as libc::c_int as libc::c_ushort],
        [126 as libc::c_int as libc::c_ushort, 7 as libc::c_int as libc::c_ushort],
        [254 as libc::c_int as libc::c_ushort, 8 as libc::c_int as libc::c_ushort],
        [510 as libc::c_int as libc::c_ushort, 9 as libc::c_int as libc::c_ushort],
        [1022 as libc::c_int as libc::c_ushort, 10 as libc::c_int as libc::c_ushort],
        [2046 as libc::c_int as libc::c_ushort, 11 as libc::c_int as libc::c_ushort],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
        [0; 2],
    ];
    static mut YAC_HT: [[libc::c_ushort; 2]; 256] = [
        [10 as libc::c_int as libc::c_ushort, 4 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 2 as libc::c_int as libc::c_ushort],
        [1 as libc::c_int as libc::c_ushort, 2 as libc::c_int as libc::c_ushort],
        [4 as libc::c_int as libc::c_ushort, 3 as libc::c_int as libc::c_ushort],
        [11 as libc::c_int as libc::c_ushort, 4 as libc::c_int as libc::c_ushort],
        [26 as libc::c_int as libc::c_ushort, 5 as libc::c_int as libc::c_ushort],
        [120 as libc::c_int as libc::c_ushort, 7 as libc::c_int as libc::c_ushort],
        [248 as libc::c_int as libc::c_ushort, 8 as libc::c_int as libc::c_ushort],
        [1014 as libc::c_int as libc::c_ushort, 10 as libc::c_int as libc::c_ushort],
        [65410 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65411 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [12 as libc::c_int as libc::c_ushort, 4 as libc::c_int as libc::c_ushort],
        [27 as libc::c_int as libc::c_ushort, 5 as libc::c_int as libc::c_ushort],
        [121 as libc::c_int as libc::c_ushort, 7 as libc::c_int as libc::c_ushort],
        [502 as libc::c_int as libc::c_ushort, 9 as libc::c_int as libc::c_ushort],
        [2038 as libc::c_int as libc::c_ushort, 11 as libc::c_int as libc::c_ushort],
        [65412 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65413 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65414 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65415 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65416 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [28 as libc::c_int as libc::c_ushort, 5 as libc::c_int as libc::c_ushort],
        [249 as libc::c_int as libc::c_ushort, 8 as libc::c_int as libc::c_ushort],
        [1015 as libc::c_int as libc::c_ushort, 10 as libc::c_int as libc::c_ushort],
        [4084 as libc::c_int as libc::c_ushort, 12 as libc::c_int as libc::c_ushort],
        [65417 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65418 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65419 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65420 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65421 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65422 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [58 as libc::c_int as libc::c_ushort, 6 as libc::c_int as libc::c_ushort],
        [503 as libc::c_int as libc::c_ushort, 9 as libc::c_int as libc::c_ushort],
        [4085 as libc::c_int as libc::c_ushort, 12 as libc::c_int as libc::c_ushort],
        [65423 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65424 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65425 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65426 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65427 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65428 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65429 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [59 as libc::c_int as libc::c_ushort, 6 as libc::c_int as libc::c_ushort],
        [1016 as libc::c_int as libc::c_ushort, 10 as libc::c_int as libc::c_ushort],
        [65430 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65431 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65432 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65433 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65434 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65435 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65436 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65437 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [122 as libc::c_int as libc::c_ushort, 7 as libc::c_int as libc::c_ushort],
        [2039 as libc::c_int as libc::c_ushort, 11 as libc::c_int as libc::c_ushort],
        [65438 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65439 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65440 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65441 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65442 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65443 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65444 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65445 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [123 as libc::c_int as libc::c_ushort, 7 as libc::c_int as libc::c_ushort],
        [4086 as libc::c_int as libc::c_ushort, 12 as libc::c_int as libc::c_ushort],
        [65446 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65447 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65448 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65449 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65450 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65451 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65452 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65453 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [250 as libc::c_int as libc::c_ushort, 8 as libc::c_int as libc::c_ushort],
        [4087 as libc::c_int as libc::c_ushort, 12 as libc::c_int as libc::c_ushort],
        [65454 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65455 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65456 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65457 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65458 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65459 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65460 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65461 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [504 as libc::c_int as libc::c_ushort, 9 as libc::c_int as libc::c_ushort],
        [32704 as libc::c_int as libc::c_ushort, 15 as libc::c_int as libc::c_ushort],
        [65462 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65463 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65464 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65465 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65466 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65467 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65468 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65469 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [505 as libc::c_int as libc::c_ushort, 9 as libc::c_int as libc::c_ushort],
        [65470 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65471 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65472 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65473 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65474 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65475 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65476 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65477 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65478 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [506 as libc::c_int as libc::c_ushort, 9 as libc::c_int as libc::c_ushort],
        [65479 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65480 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65481 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65482 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65483 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65484 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65485 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65486 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65487 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [1017 as libc::c_int as libc::c_ushort, 10 as libc::c_int as libc::c_ushort],
        [65488 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65489 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65490 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65491 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65492 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65493 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65494 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65495 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65496 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [1018 as libc::c_int as libc::c_ushort, 10 as libc::c_int as libc::c_ushort],
        [65497 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65498 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65499 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65500 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65501 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65502 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65503 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65504 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65505 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [2040 as libc::c_int as libc::c_ushort, 11 as libc::c_int as libc::c_ushort],
        [65506 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65507 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65508 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65509 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65510 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65511 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65512 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65513 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65514 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [65515 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65516 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65517 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65518 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65519 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65520 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65521 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65522 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65523 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65524 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [2041 as libc::c_int as libc::c_ushort, 11 as libc::c_int as libc::c_ushort],
        [65525 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65526 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65527 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65528 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65529 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65530 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65531 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65532 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65533 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65534 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
    ];
    static mut UVAC_HT: [[libc::c_ushort; 2]; 256] = [
        [0 as libc::c_int as libc::c_ushort, 2 as libc::c_int as libc::c_ushort],
        [1 as libc::c_int as libc::c_ushort, 2 as libc::c_int as libc::c_ushort],
        [4 as libc::c_int as libc::c_ushort, 3 as libc::c_int as libc::c_ushort],
        [10 as libc::c_int as libc::c_ushort, 4 as libc::c_int as libc::c_ushort],
        [24 as libc::c_int as libc::c_ushort, 5 as libc::c_int as libc::c_ushort],
        [25 as libc::c_int as libc::c_ushort, 5 as libc::c_int as libc::c_ushort],
        [56 as libc::c_int as libc::c_ushort, 6 as libc::c_int as libc::c_ushort],
        [120 as libc::c_int as libc::c_ushort, 7 as libc::c_int as libc::c_ushort],
        [500 as libc::c_int as libc::c_ushort, 9 as libc::c_int as libc::c_ushort],
        [1014 as libc::c_int as libc::c_ushort, 10 as libc::c_int as libc::c_ushort],
        [4084 as libc::c_int as libc::c_ushort, 12 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [11 as libc::c_int as libc::c_ushort, 4 as libc::c_int as libc::c_ushort],
        [57 as libc::c_int as libc::c_ushort, 6 as libc::c_int as libc::c_ushort],
        [246 as libc::c_int as libc::c_ushort, 8 as libc::c_int as libc::c_ushort],
        [501 as libc::c_int as libc::c_ushort, 9 as libc::c_int as libc::c_ushort],
        [2038 as libc::c_int as libc::c_ushort, 11 as libc::c_int as libc::c_ushort],
        [4085 as libc::c_int as libc::c_ushort, 12 as libc::c_int as libc::c_ushort],
        [65416 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65417 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65418 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65419 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [26 as libc::c_int as libc::c_ushort, 5 as libc::c_int as libc::c_ushort],
        [247 as libc::c_int as libc::c_ushort, 8 as libc::c_int as libc::c_ushort],
        [1015 as libc::c_int as libc::c_ushort, 10 as libc::c_int as libc::c_ushort],
        [4086 as libc::c_int as libc::c_ushort, 12 as libc::c_int as libc::c_ushort],
        [32706 as libc::c_int as libc::c_ushort, 15 as libc::c_int as libc::c_ushort],
        [65420 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65421 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65422 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65423 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65424 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [27 as libc::c_int as libc::c_ushort, 5 as libc::c_int as libc::c_ushort],
        [248 as libc::c_int as libc::c_ushort, 8 as libc::c_int as libc::c_ushort],
        [1016 as libc::c_int as libc::c_ushort, 10 as libc::c_int as libc::c_ushort],
        [4087 as libc::c_int as libc::c_ushort, 12 as libc::c_int as libc::c_ushort],
        [65425 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65426 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65427 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65428 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65429 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65430 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [58 as libc::c_int as libc::c_ushort, 6 as libc::c_int as libc::c_ushort],
        [502 as libc::c_int as libc::c_ushort, 9 as libc::c_int as libc::c_ushort],
        [65431 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65432 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65433 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65434 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65435 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65436 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65437 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65438 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [59 as libc::c_int as libc::c_ushort, 6 as libc::c_int as libc::c_ushort],
        [1017 as libc::c_int as libc::c_ushort, 10 as libc::c_int as libc::c_ushort],
        [65439 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65440 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65441 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65442 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65443 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65444 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65445 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65446 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [121 as libc::c_int as libc::c_ushort, 7 as libc::c_int as libc::c_ushort],
        [2039 as libc::c_int as libc::c_ushort, 11 as libc::c_int as libc::c_ushort],
        [65447 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65448 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65449 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65450 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65451 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65452 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65453 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65454 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [122 as libc::c_int as libc::c_ushort, 7 as libc::c_int as libc::c_ushort],
        [2040 as libc::c_int as libc::c_ushort, 11 as libc::c_int as libc::c_ushort],
        [65455 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65456 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65457 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65458 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65459 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65460 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65461 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65462 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [249 as libc::c_int as libc::c_ushort, 8 as libc::c_int as libc::c_ushort],
        [65463 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65464 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65465 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65466 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65467 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65468 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65469 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65470 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65471 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [503 as libc::c_int as libc::c_ushort, 9 as libc::c_int as libc::c_ushort],
        [65472 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65473 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65474 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65475 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65476 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65477 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65478 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65479 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65480 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [504 as libc::c_int as libc::c_ushort, 9 as libc::c_int as libc::c_ushort],
        [65481 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65482 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65483 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65484 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65485 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65486 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65487 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65488 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65489 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [505 as libc::c_int as libc::c_ushort, 9 as libc::c_int as libc::c_ushort],
        [65490 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65491 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65492 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65493 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65494 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65495 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65496 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65497 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65498 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [506 as libc::c_int as libc::c_ushort, 9 as libc::c_int as libc::c_ushort],
        [65499 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65500 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65501 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65502 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65503 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65504 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65505 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65506 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65507 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [2041 as libc::c_int as libc::c_ushort, 11 as libc::c_int as libc::c_ushort],
        [65508 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65509 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65510 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65511 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65512 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65513 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65514 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65515 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65516 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [16352 as libc::c_int as libc::c_ushort, 14 as libc::c_int as libc::c_ushort],
        [65517 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65518 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65519 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65520 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65521 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65522 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65523 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65524 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65525 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [1018 as libc::c_int as libc::c_ushort, 10 as libc::c_int as libc::c_ushort],
        [32707 as libc::c_int as libc::c_ushort, 15 as libc::c_int as libc::c_ushort],
        [65526 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65527 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65528 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65529 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65530 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65531 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65532 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65533 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [65534 as libc::c_int as libc::c_ushort, 16 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
        [0 as libc::c_int as libc::c_ushort, 0 as libc::c_int as libc::c_ushort],
    ];
    static mut YQT: [libc::c_int; 64] = [
        16 as libc::c_int,
        11 as libc::c_int,
        10 as libc::c_int,
        16 as libc::c_int,
        24 as libc::c_int,
        40 as libc::c_int,
        51 as libc::c_int,
        61 as libc::c_int,
        12 as libc::c_int,
        12 as libc::c_int,
        14 as libc::c_int,
        19 as libc::c_int,
        26 as libc::c_int,
        58 as libc::c_int,
        60 as libc::c_int,
        55 as libc::c_int,
        14 as libc::c_int,
        13 as libc::c_int,
        16 as libc::c_int,
        24 as libc::c_int,
        40 as libc::c_int,
        57 as libc::c_int,
        69 as libc::c_int,
        56 as libc::c_int,
        14 as libc::c_int,
        17 as libc::c_int,
        22 as libc::c_int,
        29 as libc::c_int,
        51 as libc::c_int,
        87 as libc::c_int,
        80 as libc::c_int,
        62 as libc::c_int,
        18 as libc::c_int,
        22 as libc::c_int,
        37 as libc::c_int,
        56 as libc::c_int,
        68 as libc::c_int,
        109 as libc::c_int,
        103 as libc::c_int,
        77 as libc::c_int,
        24 as libc::c_int,
        35 as libc::c_int,
        55 as libc::c_int,
        64 as libc::c_int,
        81 as libc::c_int,
        104 as libc::c_int,
        113 as libc::c_int,
        92 as libc::c_int,
        49 as libc::c_int,
        64 as libc::c_int,
        78 as libc::c_int,
        87 as libc::c_int,
        103 as libc::c_int,
        121 as libc::c_int,
        120 as libc::c_int,
        101 as libc::c_int,
        72 as libc::c_int,
        92 as libc::c_int,
        95 as libc::c_int,
        98 as libc::c_int,
        112 as libc::c_int,
        100 as libc::c_int,
        103 as libc::c_int,
        99 as libc::c_int,
    ];
    static mut UVQT: [libc::c_int; 64] = [
        17 as libc::c_int,
        18 as libc::c_int,
        24 as libc::c_int,
        47 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        18 as libc::c_int,
        21 as libc::c_int,
        26 as libc::c_int,
        66 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        24 as libc::c_int,
        26 as libc::c_int,
        56 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        47 as libc::c_int,
        66 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
        99 as libc::c_int,
    ];
    static mut aasf: [libc::c_float; 8] = [
        1.0f32 * 2.828427125f32,
        1.387039845f32 * 2.828427125f32,
        1.306562965f32 * 2.828427125f32,
        1.175875602f32 * 2.828427125f32,
        1.0f32 * 2.828427125f32,
        0.785694958f32 * 2.828427125f32,
        0.541196100f32 * 2.828427125f32,
        0.275899379f32 * 2.828427125f32,
    ];
    let mut row: libc::c_int = 0;
    let mut col: libc::c_int = 0;
    let mut i: libc::c_int = 0;
    let mut k: libc::c_int = 0;
    let mut subsample: libc::c_int = 0;
    let mut fdtbl_Y: [libc::c_float; 64] = [0.; 64];
    let mut fdtbl_UV: [libc::c_float; 64] = [0.; 64];
    let mut YTable: [libc::c_uchar; 64] = [0; 64];
    let mut UVTable: [libc::c_uchar; 64] = [0; 64];
    if data.is_null() || width == 0 || height == 0 || comp > 4 as libc::c_int
        || comp < 1 as libc::c_int
    {
        return 0 as libc::c_int;
    }
    quality = if quality != 0 { quality } else { 90 as libc::c_int };
    subsample = if quality <= 90 as libc::c_int {
        1 as libc::c_int
    } else {
        0 as libc::c_int
    };
    quality = if quality < 1 as libc::c_int {
        1 as libc::c_int
    } else if quality > 100 as libc::c_int {
        100 as libc::c_int
    } else {
        quality
    };
    quality = if quality < 50 as libc::c_int {
        5000 as libc::c_int / quality
    } else {
        200 as libc::c_int - quality * 2 as libc::c_int
    };
    i = 0 as libc::c_int;
    while i < 64 as libc::c_int {
        let mut uvti: libc::c_int = 0;
        let mut yti: libc::c_int = (YQT[i as usize] * quality + 50 as libc::c_int)
            / 100 as libc::c_int;
        YTable[stbiw__jpg_ZigZag[i as usize]
            as usize] = (if yti < 1 as libc::c_int {
            1 as libc::c_int
        } else if yti > 255 as libc::c_int {
            255 as libc::c_int
        } else {
            yti
        }) as libc::c_uchar;
        uvti = (UVQT[i as usize] * quality + 50 as libc::c_int) / 100 as libc::c_int;
        UVTable[stbiw__jpg_ZigZag[i as usize]
            as usize] = (if uvti < 1 as libc::c_int {
            1 as libc::c_int
        } else if uvti > 255 as libc::c_int {
            255 as libc::c_int
        } else {
            uvti
        }) as libc::c_uchar;
        i += 1;
    }
    row = 0 as libc::c_int;
    k = 0 as libc::c_int;
    while row < 8 as libc::c_int {
        col = 0 as libc::c_int;
        while col < 8 as libc::c_int {
            fdtbl_Y[k
                as usize] = 1 as libc::c_int as libc::c_float
                / (YTable[stbiw__jpg_ZigZag[k as usize] as usize] as libc::c_int
                    as libc::c_float * aasf[row as usize] * aasf[col as usize]);
            fdtbl_UV[k
                as usize] = 1 as libc::c_int as libc::c_float
                / (UVTable[stbiw__jpg_ZigZag[k as usize] as usize] as libc::c_int
                    as libc::c_float * aasf[row as usize] * aasf[col as usize]);
            col += 1;
            k += 1;
        }
        row += 1;
    }
    static mut head0: [libc::c_uchar; 25] = [
        0xff as libc::c_int as libc::c_uchar,
        0xd8 as libc::c_int as libc::c_uchar,
        0xff as libc::c_int as libc::c_uchar,
        0xe0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0x10 as libc::c_int as libc::c_uchar,
        'J' as i32 as libc::c_uchar,
        'F' as i32 as libc::c_uchar,
        'I' as i32 as libc::c_uchar,
        'F' as i32 as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0xff as libc::c_int as libc::c_uchar,
        0xdb as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0x84 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
    ];
    static mut head2: [libc::c_uchar; 14] = [
        0xff as libc::c_int as libc::c_uchar,
        0xda as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0xc as libc::c_int as libc::c_uchar,
        3 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        2 as libc::c_int as libc::c_uchar,
        0x11 as libc::c_int as libc::c_uchar,
        3 as libc::c_int as libc::c_uchar,
        0x11 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0x3f as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
    ];
    let head1: [libc::c_uchar; 24] = [
        0xff as libc::c_int as libc::c_uchar,
        0xc0 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        0x11 as libc::c_int as libc::c_uchar,
        8 as libc::c_int as libc::c_uchar,
        (height >> 8 as libc::c_int) as libc::c_uchar,
        (height & 0xff as libc::c_int) as libc::c_uchar,
        (width >> 8 as libc::c_int) as libc::c_uchar,
        (width & 0xff as libc::c_int) as libc::c_uchar,
        3 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        (if subsample != 0 { 0x22 as libc::c_int } else { 0x11 as libc::c_int })
            as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
        2 as libc::c_int as libc::c_uchar,
        0x11 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        3 as libc::c_int as libc::c_uchar,
        0x11 as libc::c_int as libc::c_uchar,
        1 as libc::c_int as libc::c_uchar,
        0xff as libc::c_int as libc::c_uchar,
        0xc4 as libc::c_int as libc::c_uchar,
        0x1 as libc::c_int as libc::c_uchar,
        0xa2 as libc::c_int as libc::c_uchar,
        0 as libc::c_int as libc::c_uchar,
    ];
    ((*s).func)
        .expect(
            "non-null function pointer",
        )(
        (*s).context,
        head0.as_ptr() as *mut libc::c_void,
        ::core::mem::size_of::<[libc::c_uchar; 25]>() as libc::c_ulong as libc::c_int,
    );
    ((*s).func)
        .expect(
            "non-null function pointer",
        )(
        (*s).context,
        YTable.as_mut_ptr() as *mut libc::c_void,
        ::core::mem::size_of::<[libc::c_uchar; 64]>() as libc::c_ulong as libc::c_int,
    );
    stbiw__putc(s, 1 as libc::c_int as libc::c_uchar);
    ((*s).func)
        .expect(
            "non-null function pointer",
        )(
        (*s).context,
        UVTable.as_mut_ptr() as *mut libc::c_void,
        ::core::mem::size_of::<[libc::c_uchar; 64]>() as libc::c_ulong as libc::c_int,
    );
    ((*s).func)
        .expect(
            "non-null function pointer",
        )(
        (*s).context,
        head1.as_ptr() as *mut libc::c_void,
        ::core::mem::size_of::<[libc::c_uchar; 24]>() as libc::c_ulong as libc::c_int,
    );
    ((*s).func)
        .expect(
            "non-null function pointer",
        )(
        (*s).context,
        std_dc_luminance_nrcodes.as_ptr().offset(1 as libc::c_int as isize)
            as *mut libc::c_void,
        (::core::mem::size_of::<[libc::c_uchar; 17]>() as libc::c_ulong)
            .wrapping_sub(1 as libc::c_int as libc::c_ulong) as libc::c_int,
    );
    ((*s).func)
        .expect(
            "non-null function pointer",
        )(
        (*s).context,
        std_dc_luminance_values.as_ptr() as *mut libc::c_void,
        ::core::mem::size_of::<[libc::c_uchar; 12]>() as libc::c_ulong as libc::c_int,
    );
    stbiw__putc(s, 0x10 as libc::c_int as libc::c_uchar);
    ((*s).func)
        .expect(
            "non-null function pointer",
        )(
        (*s).context,
        std_ac_luminance_nrcodes.as_ptr().offset(1 as libc::c_int as isize)
            as *mut libc::c_void,
        (::core::mem::size_of::<[libc::c_uchar; 17]>() as libc::c_ulong)
            .wrapping_sub(1 as libc::c_int as libc::c_ulong) as libc::c_int,
    );
    ((*s).func)
        .expect(
            "non-null function pointer",
        )(
        (*s).context,
        std_ac_luminance_values.as_ptr() as *mut libc::c_void,
        ::core::mem::size_of::<[libc::c_uchar; 162]>() as libc::c_ulong as libc::c_int,
    );
    stbiw__putc(s, 1 as libc::c_int as libc::c_uchar);
    ((*s).func)
        .expect(
            "non-null function pointer",
        )(
        (*s).context,
        std_dc_chrominance_nrcodes.as_ptr().offset(1 as libc::c_int as isize)
            as *mut libc::c_void,
        (::core::mem::size_of::<[libc::c_uchar; 17]>() as libc::c_ulong)
            .wrapping_sub(1 as libc::c_int as libc::c_ulong) as libc::c_int,
    );
    ((*s).func)
        .expect(
            "non-null function pointer",
        )(
        (*s).context,
        std_dc_chrominance_values.as_ptr() as *mut libc::c_void,
        ::core::mem::size_of::<[libc::c_uchar; 12]>() as libc::c_ulong as libc::c_int,
    );
    stbiw__putc(s, 0x11 as libc::c_int as libc::c_uchar);
    ((*s).func)
        .expect(
            "non-null function pointer",
        )(
        (*s).context,
        std_ac_chrominance_nrcodes.as_ptr().offset(1 as libc::c_int as isize)
            as *mut libc::c_void,
        (::core::mem::size_of::<[libc::c_uchar; 17]>() as libc::c_ulong)
            .wrapping_sub(1 as libc::c_int as libc::c_ulong) as libc::c_int,
    );
    ((*s).func)
        .expect(
            "non-null function pointer",
        )(
        (*s).context,
        std_ac_chrominance_values.as_ptr() as *mut libc::c_void,
        ::core::mem::size_of::<[libc::c_uchar; 162]>() as libc::c_ulong as libc::c_int,
    );
    ((*s).func)
        .expect(
            "non-null function pointer",
        )(
        (*s).context,
        head2.as_ptr() as *mut libc::c_void,
        ::core::mem::size_of::<[libc::c_uchar; 14]>() as libc::c_ulong as libc::c_int,
    );
    static mut fillBits: [libc::c_ushort; 2] = [
        0x7f as libc::c_int as libc::c_ushort,
        7 as libc::c_int as libc::c_ushort,
    ];
    let mut DCY: libc::c_int = 0 as libc::c_int;
    let mut DCU: libc::c_int = 0 as libc::c_int;
    let mut DCV: libc::c_int = 0 as libc::c_int;
    let mut bitBuf: libc::c_int = 0 as libc::c_int;
    let mut bitCnt: libc::c_int = 0 as libc::c_int;
    let mut ofsG: libc::c_int = if comp > 2 as libc::c_int {
        1 as libc::c_int
    } else {
        0 as libc::c_int
    };
    let mut ofsB: libc::c_int = if comp > 2 as libc::c_int {
        2 as libc::c_int
    } else {
        0 as libc::c_int
    };
    let mut dataR: *const libc::c_uchar = data as *const libc::c_uchar;
    let mut dataG: *const libc::c_uchar = dataR.offset(ofsG as isize);
    let mut dataB: *const libc::c_uchar = dataR.offset(ofsB as isize);
    let mut x: libc::c_int = 0;
    let mut y: libc::c_int = 0;
    let mut pos: libc::c_int = 0;
    if subsample != 0 {
        y = 0 as libc::c_int;
        while y < height {
            x = 0 as libc::c_int;
            while x < width {
                let mut Y: [libc::c_float; 256] = [0.; 256];
                let mut U: [libc::c_float; 256] = [0.; 256];
                let mut V: [libc::c_float; 256] = [0.; 256];
                row = y;
                pos = 0 as libc::c_int;
                while row < y + 16 as libc::c_int {
                    let mut clamped_row: libc::c_int = if row < height {
                        row
                    } else {
                        height - 1 as libc::c_int
                    };
                    let mut base_p: libc::c_int = (if stbi__flip_vertically_on_write != 0
                    {
                        height - 1 as libc::c_int - clamped_row
                    } else {
                        clamped_row
                    }) * width * comp;
                    col = x;
                    while col < x + 16 as libc::c_int {
                        let mut p: libc::c_int = base_p
                            + (if col < width { col } else { width - 1 as libc::c_int })
                                * comp;
                        let mut r: libc::c_float = *dataR.offset(p as isize)
                            as libc::c_float;
                        let mut g: libc::c_float = *dataG.offset(p as isize)
                            as libc::c_float;
                        let mut b: libc::c_float = *dataB.offset(p as isize)
                            as libc::c_float;
                        Y[pos
                            as usize] = 0.29900f32 * r + 0.58700f32 * g + 0.11400f32 * b
                            - 128 as libc::c_int as libc::c_float;
                        U[pos
                            as usize] = -0.16874f32 * r - 0.33126f32 * g
                            + 0.50000f32 * b;
                        V[pos
                            as usize] = 0.50000f32 * r - 0.41869f32 * g - 0.08131f32 * b;
                        col += 1;
                        pos += 1;
                    }
                    row += 1;
                }
                DCY = stbiw__jpg_processDU(
                    s,
                    &mut bitBuf,
                    &mut bitCnt,
                    Y.as_mut_ptr().offset(0 as libc::c_int as isize),
                    16 as libc::c_int,
                    fdtbl_Y.as_mut_ptr(),
                    DCY,
                    YDC_HT.as_ptr(),
                    YAC_HT.as_ptr(),
                );
                DCY = stbiw__jpg_processDU(
                    s,
                    &mut bitBuf,
                    &mut bitCnt,
                    Y.as_mut_ptr().offset(8 as libc::c_int as isize),
                    16 as libc::c_int,
                    fdtbl_Y.as_mut_ptr(),
                    DCY,
                    YDC_HT.as_ptr(),
                    YAC_HT.as_ptr(),
                );
                DCY = stbiw__jpg_processDU(
                    s,
                    &mut bitBuf,
                    &mut bitCnt,
                    Y.as_mut_ptr().offset(128 as libc::c_int as isize),
                    16 as libc::c_int,
                    fdtbl_Y.as_mut_ptr(),
                    DCY,
                    YDC_HT.as_ptr(),
                    YAC_HT.as_ptr(),
                );
                DCY = stbiw__jpg_processDU(
                    s,
                    &mut bitBuf,
                    &mut bitCnt,
                    Y.as_mut_ptr().offset(136 as libc::c_int as isize),
                    16 as libc::c_int,
                    fdtbl_Y.as_mut_ptr(),
                    DCY,
                    YDC_HT.as_ptr(),
                    YAC_HT.as_ptr(),
                );
                let mut subU: [libc::c_float; 64] = [0.; 64];
                let mut subV: [libc::c_float; 64] = [0.; 64];
                let mut yy: libc::c_int = 0;
                let mut xx: libc::c_int = 0;
                yy = 0 as libc::c_int;
                pos = 0 as libc::c_int;
                while yy < 8 as libc::c_int {
                    xx = 0 as libc::c_int;
                    while xx < 8 as libc::c_int {
                        let mut j: libc::c_int = yy * 32 as libc::c_int
                            + xx * 2 as libc::c_int;
                        subU[pos
                            as usize] = (U[(j + 0 as libc::c_int) as usize]
                            + U[(j + 1 as libc::c_int) as usize]
                            + U[(j + 16 as libc::c_int) as usize]
                            + U[(j + 17 as libc::c_int) as usize]) * 0.25f32;
                        subV[pos
                            as usize] = (V[(j + 0 as libc::c_int) as usize]
                            + V[(j + 1 as libc::c_int) as usize]
                            + V[(j + 16 as libc::c_int) as usize]
                            + V[(j + 17 as libc::c_int) as usize]) * 0.25f32;
                        xx += 1;
                        pos += 1;
                    }
                    yy += 1;
                }
                DCU = stbiw__jpg_processDU(
                    s,
                    &mut bitBuf,
                    &mut bitCnt,
                    subU.as_mut_ptr(),
                    8 as libc::c_int,
                    fdtbl_UV.as_mut_ptr(),
                    DCU,
                    UVDC_HT.as_ptr(),
                    UVAC_HT.as_ptr(),
                );
                DCV = stbiw__jpg_processDU(
                    s,
                    &mut bitBuf,
                    &mut bitCnt,
                    subV.as_mut_ptr(),
                    8 as libc::c_int,
                    fdtbl_UV.as_mut_ptr(),
                    DCV,
                    UVDC_HT.as_ptr(),
                    UVAC_HT.as_ptr(),
                );
                x += 16 as libc::c_int;
            }
            y += 16 as libc::c_int;
        }
    } else {
        y = 0 as libc::c_int;
        while y < height {
            x = 0 as libc::c_int;
            while x < width {
                let mut Y_0: [libc::c_float; 64] = [0.; 64];
                let mut U_0: [libc::c_float; 64] = [0.; 64];
                let mut V_0: [libc::c_float; 64] = [0.; 64];
                row = y;
                pos = 0 as libc::c_int;
                while row < y + 8 as libc::c_int {
                    let mut clamped_row_0: libc::c_int = if row < height {
                        row
                    } else {
                        height - 1 as libc::c_int
                    };
                    let mut base_p_0: libc::c_int = (if stbi__flip_vertically_on_write
                        != 0
                    {
                        height - 1 as libc::c_int - clamped_row_0
                    } else {
                        clamped_row_0
                    }) * width * comp;
                    col = x;
                    while col < x + 8 as libc::c_int {
                        let mut p_0: libc::c_int = base_p_0
                            + (if col < width { col } else { width - 1 as libc::c_int })
                                * comp;
                        let mut r_0: libc::c_float = *dataR.offset(p_0 as isize)
                            as libc::c_float;
                        let mut g_0: libc::c_float = *dataG.offset(p_0 as isize)
                            as libc::c_float;
                        let mut b_0: libc::c_float = *dataB.offset(p_0 as isize)
                            as libc::c_float;
                        Y_0[pos
                            as usize] = 0.29900f32 * r_0 + 0.58700f32 * g_0
                            + 0.11400f32 * b_0 - 128 as libc::c_int as libc::c_float;
                        U_0[pos
                            as usize] = -0.16874f32 * r_0 - 0.33126f32 * g_0
                            + 0.50000f32 * b_0;
                        V_0[pos
                            as usize] = 0.50000f32 * r_0 - 0.41869f32 * g_0
                            - 0.08131f32 * b_0;
                        col += 1;
                        pos += 1;
                    }
                    row += 1;
                }
                DCY = stbiw__jpg_processDU(
                    s,
                    &mut bitBuf,
                    &mut bitCnt,
                    Y_0.as_mut_ptr(),
                    8 as libc::c_int,
                    fdtbl_Y.as_mut_ptr(),
                    DCY,
                    YDC_HT.as_ptr(),
                    YAC_HT.as_ptr(),
                );
                DCU = stbiw__jpg_processDU(
                    s,
                    &mut bitBuf,
                    &mut bitCnt,
                    U_0.as_mut_ptr(),
                    8 as libc::c_int,
                    fdtbl_UV.as_mut_ptr(),
                    DCU,
                    UVDC_HT.as_ptr(),
                    UVAC_HT.as_ptr(),
                );
                DCV = stbiw__jpg_processDU(
                    s,
                    &mut bitBuf,
                    &mut bitCnt,
                    V_0.as_mut_ptr(),
                    8 as libc::c_int,
                    fdtbl_UV.as_mut_ptr(),
                    DCV,
                    UVDC_HT.as_ptr(),
                    UVAC_HT.as_ptr(),
                );
                x += 8 as libc::c_int;
            }
            y += 8 as libc::c_int;
        }
    }
    stbiw__jpg_writeBits(s, &mut bitBuf, &mut bitCnt, fillBits.as_ptr());
    stbiw__putc(s, 0xff as libc::c_int as libc::c_uchar);
    stbiw__putc(s, 0xd9 as libc::c_int as libc::c_uchar);
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbiw__putc(mut s: *mut stbi__write_context, mut c: libc::c_uchar) {
    ((*s).func)
        .expect(
            "non-null function pointer",
        )(
        (*s).context,
        &mut c as *mut libc::c_uchar as *mut libc::c_void,
        1 as libc::c_int,
    );
}
unsafe extern "C" fn stbiw__jpg_writeBits(
    mut s: *mut stbi__write_context,
    mut bitBufP: *mut libc::c_int,
    mut bitCntP: *mut libc::c_int,
    mut bs: *const libc::c_ushort,
) {
    let mut bitBuf: libc::c_int = *bitBufP;
    let mut bitCnt: libc::c_int = *bitCntP;
    bitCnt += *bs.offset(1 as libc::c_int as isize) as libc::c_int;
    bitBuf
        |= (*bs.offset(0 as libc::c_int as isize) as libc::c_int)
            << 24 as libc::c_int - bitCnt;
    while bitCnt >= 8 as libc::c_int {
        let mut c: libc::c_uchar = (bitBuf >> 16 as libc::c_int & 255 as libc::c_int)
            as libc::c_uchar;
        stbiw__putc(s, c);
        if c as libc::c_int == 255 as libc::c_int {
            stbiw__putc(s, 0 as libc::c_int as libc::c_uchar);
        }
        bitBuf <<= 8 as libc::c_int;
        bitCnt -= 8 as libc::c_int;
    }
    *bitBufP = bitBuf;
    *bitCntP = bitCnt;
}
unsafe extern "C" fn stbiw__jpg_processDU(
    mut s: *mut stbi__write_context,
    mut bitBuf: *mut libc::c_int,
    mut bitCnt: *mut libc::c_int,
    mut CDU: *mut libc::c_float,
    mut du_stride: libc::c_int,
    mut fdtbl: *mut libc::c_float,
    mut DC: libc::c_int,
    mut HTDC: *const [libc::c_ushort; 2],
    mut HTAC: *const [libc::c_ushort; 2],
) -> libc::c_int {
    let EOB: [libc::c_ushort; 2] = [
        (*HTAC.offset(0 as libc::c_int as isize))[0 as libc::c_int as usize],
        (*HTAC.offset(0 as libc::c_int as isize))[1 as libc::c_int as usize],
    ];
    let M16zeroes: [libc::c_ushort; 2] = [
        (*HTAC.offset(0xf0 as libc::c_int as isize))[0 as libc::c_int as usize],
        (*HTAC.offset(0xf0 as libc::c_int as isize))[1 as libc::c_int as usize],
    ];
    let mut dataOff: libc::c_int = 0;
    let mut i: libc::c_int = 0;
    let mut j: libc::c_int = 0;
    let mut n: libc::c_int = 0;
    let mut diff: libc::c_int = 0;
    let mut end0pos: libc::c_int = 0;
    let mut x: libc::c_int = 0;
    let mut y: libc::c_int = 0;
    let mut DU: [libc::c_int; 64] = [0; 64];
    dataOff = 0 as libc::c_int;
    n = du_stride * 8 as libc::c_int;
    while dataOff < n {
        stbiw__jpg_DCT(
            &mut *CDU.offset(dataOff as isize),
            &mut *CDU.offset((dataOff + 1 as libc::c_int) as isize),
            &mut *CDU.offset((dataOff + 2 as libc::c_int) as isize),
            &mut *CDU.offset((dataOff + 3 as libc::c_int) as isize),
            &mut *CDU.offset((dataOff + 4 as libc::c_int) as isize),
            &mut *CDU.offset((dataOff + 5 as libc::c_int) as isize),
            &mut *CDU.offset((dataOff + 6 as libc::c_int) as isize),
            &mut *CDU.offset((dataOff + 7 as libc::c_int) as isize),
        );
        dataOff += du_stride;
    }
    dataOff = 0 as libc::c_int;
    while dataOff < 8 as libc::c_int {
        stbiw__jpg_DCT(
            &mut *CDU.offset(dataOff as isize),
            &mut *CDU.offset((dataOff + du_stride) as isize),
            &mut *CDU.offset((dataOff + du_stride * 2 as libc::c_int) as isize),
            &mut *CDU.offset((dataOff + du_stride * 3 as libc::c_int) as isize),
            &mut *CDU.offset((dataOff + du_stride * 4 as libc::c_int) as isize),
            &mut *CDU.offset((dataOff + du_stride * 5 as libc::c_int) as isize),
            &mut *CDU.offset((dataOff + du_stride * 6 as libc::c_int) as isize),
            &mut *CDU.offset((dataOff + du_stride * 7 as libc::c_int) as isize),
        );
        dataOff += 1;
    }
    y = 0 as libc::c_int;
    j = 0 as libc::c_int;
    while y < 8 as libc::c_int {
        x = 0 as libc::c_int;
        while x < 8 as libc::c_int {
            let mut v: libc::c_float = 0.;
            i = y * du_stride + x;
            v = *CDU.offset(i as isize) * *fdtbl.offset(j as isize);
            DU[stbiw__jpg_ZigZag[j as usize]
                as usize] = (if v < 0 as libc::c_int as libc::c_float {
                v - 0.5f32
            } else {
                v + 0.5f32
            }) as libc::c_int;
            x += 1;
            j += 1;
        }
        y += 1;
    }
    diff = DU[0 as libc::c_int as usize] - DC;
    if diff == 0 as libc::c_int {
        stbiw__jpg_writeBits(
            s,
            bitBuf,
            bitCnt,
            (*HTDC.offset(0 as libc::c_int as isize)).as_ptr(),
        );
    } else {
        let mut bits: [libc::c_ushort; 2] = [0; 2];
        stbiw__jpg_calcBits(diff, bits.as_mut_ptr());
        stbiw__jpg_writeBits(
            s,
            bitBuf,
            bitCnt,
            (*HTDC.offset(bits[1 as libc::c_int as usize] as isize)).as_ptr(),
        );
        stbiw__jpg_writeBits(s, bitBuf, bitCnt, bits.as_mut_ptr());
    }
    end0pos = 63 as libc::c_int;
    while end0pos > 0 as libc::c_int && DU[end0pos as usize] == 0 as libc::c_int {
        end0pos -= 1;
    }
    if end0pos == 0 as libc::c_int {
        stbiw__jpg_writeBits(s, bitBuf, bitCnt, EOB.as_ptr());
        return DU[0 as libc::c_int as usize];
    }
    i = 1 as libc::c_int;
    while i <= end0pos {
        let mut startpos: libc::c_int = i;
        let mut nrzeroes: libc::c_int = 0;
        let mut bits_0: [libc::c_ushort; 2] = [0; 2];
        while DU[i as usize] == 0 as libc::c_int && i <= end0pos {
            i += 1;
        }
        nrzeroes = i - startpos;
        if nrzeroes >= 16 as libc::c_int {
            let mut lng: libc::c_int = nrzeroes >> 4 as libc::c_int;
            let mut nrmarker: libc::c_int = 0;
            nrmarker = 1 as libc::c_int;
            while nrmarker <= lng {
                stbiw__jpg_writeBits(s, bitBuf, bitCnt, M16zeroes.as_ptr());
                nrmarker += 1;
            }
            nrzeroes &= 15 as libc::c_int;
        }
        stbiw__jpg_calcBits(DU[i as usize], bits_0.as_mut_ptr());
        stbiw__jpg_writeBits(
            s,
            bitBuf,
            bitCnt,
            (*HTAC
                .offset(
                    ((nrzeroes << 4 as libc::c_int)
                        + bits_0[1 as libc::c_int as usize] as libc::c_int) as isize,
                ))
                .as_ptr(),
        );
        stbiw__jpg_writeBits(s, bitBuf, bitCnt, bits_0.as_mut_ptr());
        i += 1;
    }
    if end0pos != 63 as libc::c_int {
        stbiw__jpg_writeBits(s, bitBuf, bitCnt, EOB.as_ptr());
    }
    return DU[0 as libc::c_int as usize];
}
unsafe extern "C" fn stbiw__jpg_calcBits(
    mut val: libc::c_int,
    mut bits: *mut libc::c_ushort,
) {
    let mut tmp1: libc::c_int = if val < 0 as libc::c_int { -val } else { val };
    val = if val < 0 as libc::c_int { val - 1 as libc::c_int } else { val };
    *bits.offset(1 as libc::c_int as isize) = 1 as libc::c_int as libc::c_ushort;
    loop {
        tmp1 >>= 1 as libc::c_int;
        if !(tmp1 != 0) {
            break;
        }
        let ref mut fresh39 = *bits.offset(1 as libc::c_int as isize);
        *fresh39 = (*fresh39).wrapping_add(1);
    }
    *bits
        .offset(
            0 as libc::c_int as isize,
        ) = (val
        & ((1 as libc::c_int) << *bits.offset(1 as libc::c_int as isize) as libc::c_int)
            - 1 as libc::c_int) as libc::c_ushort;
}
static mut stbiw__jpg_ZigZag: [libc::c_uchar; 64] = [
    0 as libc::c_int as libc::c_uchar,
    1 as libc::c_int as libc::c_uchar,
    5 as libc::c_int as libc::c_uchar,
    6 as libc::c_int as libc::c_uchar,
    14 as libc::c_int as libc::c_uchar,
    15 as libc::c_int as libc::c_uchar,
    27 as libc::c_int as libc::c_uchar,
    28 as libc::c_int as libc::c_uchar,
    2 as libc::c_int as libc::c_uchar,
    4 as libc::c_int as libc::c_uchar,
    7 as libc::c_int as libc::c_uchar,
    13 as libc::c_int as libc::c_uchar,
    16 as libc::c_int as libc::c_uchar,
    26 as libc::c_int as libc::c_uchar,
    29 as libc::c_int as libc::c_uchar,
    42 as libc::c_int as libc::c_uchar,
    3 as libc::c_int as libc::c_uchar,
    8 as libc::c_int as libc::c_uchar,
    12 as libc::c_int as libc::c_uchar,
    17 as libc::c_int as libc::c_uchar,
    25 as libc::c_int as libc::c_uchar,
    30 as libc::c_int as libc::c_uchar,
    41 as libc::c_int as libc::c_uchar,
    43 as libc::c_int as libc::c_uchar,
    9 as libc::c_int as libc::c_uchar,
    11 as libc::c_int as libc::c_uchar,
    18 as libc::c_int as libc::c_uchar,
    24 as libc::c_int as libc::c_uchar,
    31 as libc::c_int as libc::c_uchar,
    40 as libc::c_int as libc::c_uchar,
    44 as libc::c_int as libc::c_uchar,
    53 as libc::c_int as libc::c_uchar,
    10 as libc::c_int as libc::c_uchar,
    19 as libc::c_int as libc::c_uchar,
    23 as libc::c_int as libc::c_uchar,
    32 as libc::c_int as libc::c_uchar,
    39 as libc::c_int as libc::c_uchar,
    45 as libc::c_int as libc::c_uchar,
    52 as libc::c_int as libc::c_uchar,
    54 as libc::c_int as libc::c_uchar,
    20 as libc::c_int as libc::c_uchar,
    22 as libc::c_int as libc::c_uchar,
    33 as libc::c_int as libc::c_uchar,
    38 as libc::c_int as libc::c_uchar,
    46 as libc::c_int as libc::c_uchar,
    51 as libc::c_int as libc::c_uchar,
    55 as libc::c_int as libc::c_uchar,
    60 as libc::c_int as libc::c_uchar,
    21 as libc::c_int as libc::c_uchar,
    34 as libc::c_int as libc::c_uchar,
    37 as libc::c_int as libc::c_uchar,
    47 as libc::c_int as libc::c_uchar,
    50 as libc::c_int as libc::c_uchar,
    56 as libc::c_int as libc::c_uchar,
    59 as libc::c_int as libc::c_uchar,
    61 as libc::c_int as libc::c_uchar,
    35 as libc::c_int as libc::c_uchar,
    36 as libc::c_int as libc::c_uchar,
    48 as libc::c_int as libc::c_uchar,
    49 as libc::c_int as libc::c_uchar,
    57 as libc::c_int as libc::c_uchar,
    58 as libc::c_int as libc::c_uchar,
    62 as libc::c_int as libc::c_uchar,
    63 as libc::c_int as libc::c_uchar,
];
unsafe extern "C" fn stbiw__jpg_DCT(
    mut d0p: *mut libc::c_float,
    mut d1p: *mut libc::c_float,
    mut d2p: *mut libc::c_float,
    mut d3p: *mut libc::c_float,
    mut d4p: *mut libc::c_float,
    mut d5p: *mut libc::c_float,
    mut d6p: *mut libc::c_float,
    mut d7p: *mut libc::c_float,
) {
    let mut d0: libc::c_float = *d0p;
    let mut d1: libc::c_float = *d1p;
    let mut d2: libc::c_float = *d2p;
    let mut d3: libc::c_float = *d3p;
    let mut d4: libc::c_float = *d4p;
    let mut d5: libc::c_float = *d5p;
    let mut d6: libc::c_float = *d6p;
    let mut d7: libc::c_float = *d7p;
    let mut z1: libc::c_float = 0.;
    let mut z2: libc::c_float = 0.;
    let mut z3: libc::c_float = 0.;
    let mut z4: libc::c_float = 0.;
    let mut z5: libc::c_float = 0.;
    let mut z11: libc::c_float = 0.;
    let mut z13: libc::c_float = 0.;
    let mut tmp0: libc::c_float = d0 + d7;
    let mut tmp7: libc::c_float = d0 - d7;
    let mut tmp1: libc::c_float = d1 + d6;
    let mut tmp6: libc::c_float = d1 - d6;
    let mut tmp2: libc::c_float = d2 + d5;
    let mut tmp5: libc::c_float = d2 - d5;
    let mut tmp3: libc::c_float = d3 + d4;
    let mut tmp4: libc::c_float = d3 - d4;
    let mut tmp10: libc::c_float = tmp0 + tmp3;
    let mut tmp13: libc::c_float = tmp0 - tmp3;
    let mut tmp11: libc::c_float = tmp1 + tmp2;
    let mut tmp12: libc::c_float = tmp1 - tmp2;
    d0 = tmp10 + tmp11;
    d4 = tmp10 - tmp11;
    z1 = (tmp12 + tmp13) * 0.707106781f32;
    d2 = tmp13 + z1;
    d6 = tmp13 - z1;
    tmp10 = tmp4 + tmp5;
    tmp11 = tmp5 + tmp6;
    tmp12 = tmp6 + tmp7;
    z5 = (tmp10 - tmp12) * 0.382683433f32;
    z2 = tmp10 * 0.541196100f32 + z5;
    z4 = tmp12 * 1.306562965f32 + z5;
    z3 = tmp11 * 0.707106781f32;
    z11 = tmp7 + z3;
    z13 = tmp7 - z3;
    *d5p = z13 + z2;
    *d3p = z13 - z2;
    *d1p = z11 + z4;
    *d7p = z11 - z4;
    *d0p = d0;
    *d2p = d2;
    *d4p = d4;
    *d6p = d6;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_write_png_to_func(
    mut func: Option::<stbi_write_func>,
    mut context: *mut libc::c_void,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut comp: libc::c_int,
    mut data: *const libc::c_void,
    mut stride_bytes: libc::c_int,
) -> libc::c_int {
    let mut len: libc::c_int = 0;
    let mut png: *mut libc::c_uchar = stbi_write_png_to_mem(
        data as *const libc::c_uchar,
        stride_bytes,
        x,
        y,
        comp,
        &mut len,
    );
    if png.is_null() {
        return 0 as libc::c_int;
    }
    func.expect("non-null function pointer")(context, png as *mut libc::c_void, len);
    free(png as *mut libc::c_void);
    return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_write_bmp_to_func(
    mut func: Option::<stbi_write_func>,
    mut context: *mut libc::c_void,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut comp: libc::c_int,
    mut data: *const libc::c_void,
) -> libc::c_int {
    let mut s: stbi__write_context = {
        let mut init = stbi__write_context {
            func: None,
            context: 0 as *mut libc::c_void,
            buffer: [0; 64],
            buf_used: 0,
        };
        init
    };
    stbi__start_write_callbacks(&mut s, func, context);
    return stbi_write_bmp_core(&mut s, x, y, comp, data);
}
#[no_mangle]
pub unsafe extern "C" fn stbi_write_tga_to_func(
    mut func: Option::<stbi_write_func>,
    mut context: *mut libc::c_void,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut comp: libc::c_int,
    mut data: *const libc::c_void,
) -> libc::c_int {
    let mut s: stbi__write_context = {
        let mut init = stbi__write_context {
            func: None,
            context: 0 as *mut libc::c_void,
            buffer: [0; 64],
            buf_used: 0,
        };
        init
    };
    stbi__start_write_callbacks(&mut s, func, context);
    return stbi_write_tga_core(&mut s, x, y, comp, data as *mut libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn stbi_write_hdr_to_func(
    mut func: Option::<stbi_write_func>,
    mut context: *mut libc::c_void,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut comp: libc::c_int,
    mut data: *const libc::c_float,
) -> libc::c_int {
    let mut s: stbi__write_context = {
        let mut init = stbi__write_context {
            func: None,
            context: 0 as *mut libc::c_void,
            buffer: [0; 64],
            buf_used: 0,
        };
        init
    };
    stbi__start_write_callbacks(&mut s, func, context);
    return stbi_write_hdr_core(&mut s, x, y, comp, data as *mut libc::c_float);
}
#[no_mangle]
pub unsafe extern "C" fn stbi_write_jpg_to_func(
    mut func: Option::<stbi_write_func>,
    mut context: *mut libc::c_void,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut comp: libc::c_int,
    mut data: *const libc::c_void,
    mut quality: libc::c_int,
) -> libc::c_int {
    let mut s: stbi__write_context = {
        let mut init = stbi__write_context {
            func: None,
            context: 0 as *mut libc::c_void,
            buffer: [0; 64],
            buf_used: 0,
        };
        init
    };
    stbi__start_write_callbacks(&mut s, func, context);
    return stbi_write_jpg_core(&mut s, x, y, comp, data as *mut libc::c_void, quality);
}
#[no_mangle]
pub unsafe extern "C" fn stbi_flip_vertically_on_write(mut flag: libc::c_int) {
    stbi__flip_vertically_on_write = flag;
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_Save_Png(
    mut path: cstr,
    mut sx: libc::c_int,
    mut sy: libc::c_int,
    mut components: libc::c_int,
    mut data: *mut uchar,
) -> bool {
    let mut stride: libc::c_int = components * sx;
    let mut result: libc::c_int = stbi_write_png(
        path,
        sx,
        sy,
        components,
        data as *const libc::c_void,
        stride,
    );
    return result != 0 as libc::c_int;
}
