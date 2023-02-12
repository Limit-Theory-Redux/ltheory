use ::libc;
extern "C" {
    pub type __sFILEX;
    fn memcpy(
        _: *mut libc::c_void,
        _: *const libc::c_void,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn memset(
        _: *mut libc::c_void,
        _: libc::c_int,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn strcmp(_: *const libc::c_char, _: *const libc::c_char) -> libc::c_int;
    fn strncmp(
        _: *const libc::c_char,
        _: *const libc::c_char,
        _: libc::c_ulong,
    ) -> libc::c_int;
    fn Fatal(_: cstr, _: ...);
    fn abs(_: libc::c_int) -> libc::c_int;
    fn strtol(
        _: *const libc::c_char,
        _: *mut *mut libc::c_char,
        _: libc::c_int,
    ) -> libc::c_long;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn realloc(_: *mut libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
    fn ungetc(_: libc::c_int, _: *mut FILE) -> libc::c_int;
    fn ftell(_: *mut FILE) -> libc::c_long;
    fn fseek(_: *mut FILE, _: libc::c_long, _: libc::c_int) -> libc::c_int;
    fn fread(
        _: *mut libc::c_void,
        _: libc::c_ulong,
        _: libc::c_ulong,
        _: *mut FILE,
    ) -> libc::c_ulong;
    fn fopen(_: *const libc::c_char, _: *const libc::c_char) -> *mut FILE;
    fn fgetc(_: *mut FILE) -> libc::c_int;
    fn ferror(_: *mut FILE) -> libc::c_int;
    fn feof(_: *mut FILE) -> libc::c_int;
    fn fclose(_: *mut FILE) -> libc::c_int;
    fn ldexp(_: libc::c_double, _: libc::c_int) -> libc::c_double;
    fn pow(_: libc::c_double, _: libc::c_double) -> libc::c_double;
}
pub type int16_t = libc::c_short;
pub type int32_t = libc::c_int;
pub type uint16_t = libc::c_ushort;
pub type uint32_t = libc::c_uint;
pub type __int64_t = libc::c_longlong;
pub type __darwin_size_t = libc::c_ulong;
pub type __darwin_off_t = __int64_t;
pub type size_t = __darwin_size_t;
pub type uchar = libc::c_uchar;
pub type cstr = *const libc::c_char;
pub type fpos_t = __darwin_off_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sbuf {
    pub _base: *mut libc::c_uchar,
    pub _size: libc::c_int,
}
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
pub type FILE = __sFILE;
pub type C2RustUnnamed = libc::c_uint;
pub const STBI_rgb_alpha: C2RustUnnamed = 4;
pub const STBI_rgb: C2RustUnnamed = 3;
pub const STBI_grey_alpha: C2RustUnnamed = 2;
pub const STBI_grey: C2RustUnnamed = 1;
pub const STBI_default: C2RustUnnamed = 0;
pub type stbi_uc = libc::c_uchar;
pub type stbi_us = libc::c_ushort;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi_io_callbacks {
    pub read: Option::<
        unsafe extern "C" fn(
            *mut libc::c_void,
            *mut libc::c_char,
            libc::c_int,
        ) -> libc::c_int,
    >,
    pub skip: Option::<unsafe extern "C" fn(*mut libc::c_void, libc::c_int) -> ()>,
    pub eof: Option::<unsafe extern "C" fn(*mut libc::c_void) -> libc::c_int>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__context {
    pub img_x: stbi__uint32,
    pub img_y: stbi__uint32,
    pub img_n: libc::c_int,
    pub img_out_n: libc::c_int,
    pub io: stbi_io_callbacks,
    pub io_user_data: *mut libc::c_void,
    pub read_from_callbacks: libc::c_int,
    pub buflen: libc::c_int,
    pub buffer_start: [stbi_uc; 128],
    pub callback_already_read: libc::c_int,
    pub img_buffer: *mut stbi_uc,
    pub img_buffer_end: *mut stbi_uc,
    pub img_buffer_original: *mut stbi_uc,
    pub img_buffer_original_end: *mut stbi_uc,
}
pub type stbi__uint32 = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__result_info {
    pub bits_per_channel: libc::c_int,
    pub num_channels: libc::c_int,
    pub channel_order: libc::c_int,
}
pub type stbi__uint16 = uint16_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__jpeg {
    pub s: *mut stbi__context,
    pub huff_dc: [stbi__huffman; 4],
    pub huff_ac: [stbi__huffman; 4],
    pub dequant: [[stbi__uint16; 64]; 4],
    pub fast_ac: [[stbi__int16; 512]; 4],
    pub img_h_max: libc::c_int,
    pub img_v_max: libc::c_int,
    pub img_mcu_x: libc::c_int,
    pub img_mcu_y: libc::c_int,
    pub img_mcu_w: libc::c_int,
    pub img_mcu_h: libc::c_int,
    pub img_comp: [C2RustUnnamed_0; 4],
    pub code_buffer: stbi__uint32,
    pub code_bits: libc::c_int,
    pub marker: libc::c_uchar,
    pub nomore: libc::c_int,
    pub progressive: libc::c_int,
    pub spec_start: libc::c_int,
    pub spec_end: libc::c_int,
    pub succ_high: libc::c_int,
    pub succ_low: libc::c_int,
    pub eob_run: libc::c_int,
    pub jfif: libc::c_int,
    pub app14_color_transform: libc::c_int,
    pub rgb: libc::c_int,
    pub scan_n: libc::c_int,
    pub order: [libc::c_int; 4],
    pub restart_interval: libc::c_int,
    pub todo: libc::c_int,
    pub idct_block_kernel: Option::<
        unsafe extern "C" fn(*mut stbi_uc, libc::c_int, *mut libc::c_short) -> (),
    >,
    pub YCbCr_to_RGB_kernel: Option::<
        unsafe extern "C" fn(
            *mut stbi_uc,
            *const stbi_uc,
            *const stbi_uc,
            *const stbi_uc,
            libc::c_int,
            libc::c_int,
        ) -> (),
    >,
    pub resample_row_hv_2_kernel: Option::<
        unsafe extern "C" fn(
            *mut stbi_uc,
            *mut stbi_uc,
            *mut stbi_uc,
            libc::c_int,
            libc::c_int,
        ) -> *mut stbi_uc,
    >,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0 {
    pub id: libc::c_int,
    pub h: libc::c_int,
    pub v: libc::c_int,
    pub tq: libc::c_int,
    pub hd: libc::c_int,
    pub ha: libc::c_int,
    pub dc_pred: libc::c_int,
    pub x: libc::c_int,
    pub y: libc::c_int,
    pub w2: libc::c_int,
    pub h2: libc::c_int,
    pub data: *mut stbi_uc,
    pub raw_data: *mut libc::c_void,
    pub raw_coeff: *mut libc::c_void,
    pub linebuf: *mut stbi_uc,
    pub coeff: *mut libc::c_short,
    pub coeff_w: libc::c_int,
    pub coeff_h: libc::c_int,
}
pub type stbi__int16 = int16_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__huffman {
    pub fast: [stbi_uc; 512],
    pub code: [stbi__uint16; 256],
    pub values: [stbi_uc; 256],
    pub size: [stbi_uc; 257],
    pub maxcode: [libc::c_uint; 18],
    pub delta: [libc::c_int; 17],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__resample {
    pub resample: resample_row_func,
    pub line0: *mut stbi_uc,
    pub line1: *mut stbi_uc,
    pub hs: libc::c_int,
    pub vs: libc::c_int,
    pub w_lores: libc::c_int,
    pub ystep: libc::c_int,
    pub ypos: libc::c_int,
}
pub type resample_row_func = Option::<
    unsafe extern "C" fn(
        *mut stbi_uc,
        *mut stbi_uc,
        *mut stbi_uc,
        libc::c_int,
        libc::c_int,
    ) -> *mut stbi_uc,
>;
pub const STBI__SCAN_load: C2RustUnnamed_2 = 0;
pub const STBI__SCAN_type: C2RustUnnamed_2 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__pic_packet {
    pub size: stbi_uc,
    pub type_0: stbi_uc,
    pub channel: stbi_uc,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__gif {
    pub w: libc::c_int,
    pub h: libc::c_int,
    pub out: *mut stbi_uc,
    pub background: *mut stbi_uc,
    pub history: *mut stbi_uc,
    pub flags: libc::c_int,
    pub bgindex: libc::c_int,
    pub ratio: libc::c_int,
    pub transparent: libc::c_int,
    pub eflags: libc::c_int,
    pub pal: [[stbi_uc; 4]; 256],
    pub lpal: [[stbi_uc; 4]; 256],
    pub codes: [stbi__gif_lzw; 8192],
    pub color_table: *mut stbi_uc,
    pub parse: libc::c_int,
    pub step: libc::c_int,
    pub lflags: libc::c_int,
    pub start_x: libc::c_int,
    pub start_y: libc::c_int,
    pub max_x: libc::c_int,
    pub max_y: libc::c_int,
    pub cur_x: libc::c_int,
    pub cur_y: libc::c_int,
    pub line_size: libc::c_int,
    pub delay: libc::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__gif_lzw {
    pub prefix: stbi__int16,
    pub first: stbi_uc,
    pub suffix: stbi_uc,
}
pub type stbi__int32 = int32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__bmp_data {
    pub bpp: libc::c_int,
    pub offset: libc::c_int,
    pub hsz: libc::c_int,
    pub mr: libc::c_uint,
    pub mg: libc::c_uint,
    pub mb: libc::c_uint,
    pub ma: libc::c_uint,
    pub all_a: libc::c_uint,
    pub extra_read: libc::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__png {
    pub s: *mut stbi__context,
    pub idata: *mut stbi_uc,
    pub expanded: *mut stbi_uc,
    pub out: *mut stbi_uc,
    pub depth: libc::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__pngchunk {
    pub length: stbi__uint32,
    pub type_0: stbi__uint32,
}
pub const STBI__F_paeth_first: C2RustUnnamed_3 = 6;
pub const STBI__F_avg_first: C2RustUnnamed_3 = 5;
pub const STBI__F_paeth: C2RustUnnamed_3 = 4;
pub const STBI__F_avg: C2RustUnnamed_3 = 3;
pub const STBI__F_up: C2RustUnnamed_3 = 2;
pub const STBI__F_sub: C2RustUnnamed_3 = 1;
pub const STBI__F_none: C2RustUnnamed_3 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__zbuf {
    pub zbuffer: *mut stbi_uc,
    pub zbuffer_end: *mut stbi_uc,
    pub num_bits: libc::c_int,
    pub code_buffer: stbi__uint32,
    pub zout: *mut libc::c_char,
    pub zout_start: *mut libc::c_char,
    pub zout_end: *mut libc::c_char,
    pub z_expandable: libc::c_int,
    pub z_length: stbi__zhuffman,
    pub z_distance: stbi__zhuffman,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__zhuffman {
    pub fast: [stbi__uint16; 512],
    pub firstcode: [stbi__uint16; 16],
    pub maxcode: [libc::c_int; 17],
    pub firstsymbol: [stbi__uint16; 16],
    pub size: [stbi_uc; 288],
    pub value: [stbi__uint16; 288],
}
pub const STBI__SCAN_header: C2RustUnnamed_2 = 2;
pub const STBI_ORDER_RGB: C2RustUnnamed_1 = 0;
pub type C2RustUnnamed_1 = libc::c_uint;
pub const STBI_ORDER_BGR: C2RustUnnamed_1 = 1;
pub type C2RustUnnamed_2 = libc::c_uint;
pub type C2RustUnnamed_3 = libc::c_uint;
#[no_mangle]
pub unsafe extern "C" fn stbi_info_from_callbacks(
    mut c: *const stbi_io_callbacks,
    mut user: *mut libc::c_void,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
) -> libc::c_int {
    let mut s: stbi__context = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: 0 as *mut libc::c_void,
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: 0 as *mut stbi_uc,
        img_buffer_end: 0 as *mut stbi_uc,
        img_buffer_original: 0 as *mut stbi_uc,
        img_buffer_original_end: 0 as *mut stbi_uc,
    };
    stbi__start_callbacks(&mut s, c as *mut stbi_io_callbacks, user);
    return stbi__info_main(&mut s, x, y, comp);
}
#[no_mangle]
pub unsafe extern "C" fn stbi_load_from_memory(
    mut buffer: *const stbi_uc,
    mut len: libc::c_int,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
) -> *mut stbi_uc {
    let mut s: stbi__context = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: 0 as *mut libc::c_void,
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: 0 as *mut stbi_uc,
        img_buffer_end: 0 as *mut stbi_uc,
        img_buffer_original: 0 as *mut stbi_uc,
        img_buffer_original_end: 0 as *mut stbi_uc,
    };
    stbi__start_mem(&mut s, buffer, len);
    return stbi__load_and_postprocess_8bit(&mut s, x, y, comp, req_comp);
}
unsafe extern "C" fn stbi__load_and_postprocess_8bit(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
) -> *mut libc::c_uchar {
    let mut ri: stbi__result_info = stbi__result_info {
        bits_per_channel: 0,
        num_channels: 0,
        channel_order: 0,
    };
    let mut result: *mut libc::c_void = stbi__load_main(
        s,
        x,
        y,
        comp,
        req_comp,
        &mut ri,
        8 as libc::c_int,
    );
    if result.is_null() {
        return 0 as *mut libc::c_uchar;
    }
    if ri.bits_per_channel != 8 as libc::c_int {
        result = stbi__convert_16_to_8(
            result as *mut stbi__uint16,
            *x,
            *y,
            if req_comp == 0 as libc::c_int { *comp } else { req_comp },
        ) as *mut libc::c_void;
        ri.bits_per_channel = 8 as libc::c_int;
    }
    if if stbi__vertically_flip_on_load_set != 0 {
        stbi__vertically_flip_on_load_local
    } else {
        stbi__vertically_flip_on_load_global
    } != 0
    {
        let mut channels: libc::c_int = if req_comp != 0 { req_comp } else { *comp };
        stbi__vertical_flip(
            result,
            *x,
            *y,
            (channels as libc::c_ulong)
                .wrapping_mul(::core::mem::size_of::<stbi_uc>() as libc::c_ulong)
                as libc::c_int,
        );
    }
    return result as *mut libc::c_uchar;
}
unsafe extern "C" fn stbi__load_main(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
    mut ri: *mut stbi__result_info,
    mut bpc: libc::c_int,
) -> *mut libc::c_void {
    memset(
        ri as *mut libc::c_void,
        0 as libc::c_int,
        ::core::mem::size_of::<stbi__result_info>() as libc::c_ulong,
    );
    (*ri).bits_per_channel = 8 as libc::c_int;
    (*ri).channel_order = STBI_ORDER_RGB as libc::c_int;
    (*ri).num_channels = 0 as libc::c_int;
    if stbi__png_test(s) != 0 {
        return stbi__png_load(s, x, y, comp, req_comp, ri);
    }
    if stbi__bmp_test(s) != 0 {
        return stbi__bmp_load(s, x, y, comp, req_comp, ri);
    }
    if stbi__gif_test(s) != 0 {
        return stbi__gif_load(s, x, y, comp, req_comp, ri);
    }
    if stbi__psd_test(s) != 0 {
        return stbi__psd_load(s, x, y, comp, req_comp, ri, bpc);
    }
    if stbi__pic_test(s) != 0 {
        return stbi__pic_load(s, x, y, comp, req_comp, ri);
    }
    if stbi__jpeg_test(s) != 0 {
        return stbi__jpeg_load(s, x, y, comp, req_comp, ri);
    }
    if stbi__pnm_test(s) != 0 {
        return stbi__pnm_load(s, x, y, comp, req_comp, ri);
    }
    if stbi__hdr_test(s) != 0 {
        let mut hdr: *mut libc::c_float = stbi__hdr_load(s, x, y, comp, req_comp, ri);
        return stbi__hdr_to_ldr(
            hdr,
            *x,
            *y,
            if req_comp != 0 { req_comp } else { *comp },
        ) as *mut libc::c_void;
    }
    if stbi__tga_test(s) != 0 {
        return stbi__tga_load(s, x, y, comp, req_comp, ri);
    }
    return (if stbi__err(b"unknown image type\0" as *const u8 as *const libc::c_char)
        != 0
    {
        0 as *mut libc::c_void
    } else {
        0 as *mut libc::c_void
    }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
}
unsafe extern "C" fn stbi__err(mut str: *const libc::c_char) -> libc::c_int {
    stbi__g_failure_reason = str;
    return 0 as libc::c_int;
}
#[thread_local]
static mut stbi__g_failure_reason: *const libc::c_char = 0 as *const libc::c_char;
unsafe extern "C" fn stbi__tga_load(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
    mut ri: *mut stbi__result_info,
) -> *mut libc::c_void {
    let mut tga_offset: libc::c_int = stbi__get8(s) as libc::c_int;
    let mut tga_indexed: libc::c_int = stbi__get8(s) as libc::c_int;
    let mut tga_image_type: libc::c_int = stbi__get8(s) as libc::c_int;
    let mut tga_is_RLE: libc::c_int = 0 as libc::c_int;
    let mut tga_palette_start: libc::c_int = stbi__get16le(s);
    let mut tga_palette_len: libc::c_int = stbi__get16le(s);
    let mut tga_palette_bits: libc::c_int = stbi__get8(s) as libc::c_int;
    let mut tga_x_origin: libc::c_int = stbi__get16le(s);
    let mut tga_y_origin: libc::c_int = stbi__get16le(s);
    let mut tga_width: libc::c_int = stbi__get16le(s);
    let mut tga_height: libc::c_int = stbi__get16le(s);
    let mut tga_bits_per_pixel: libc::c_int = stbi__get8(s) as libc::c_int;
    let mut tga_comp: libc::c_int = 0;
    let mut tga_rgb16: libc::c_int = 0 as libc::c_int;
    let mut tga_inverted: libc::c_int = stbi__get8(s) as libc::c_int;
    let mut tga_data: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
    let mut tga_palette: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
    let mut i: libc::c_int = 0;
    let mut j: libc::c_int = 0;
    let mut raw_data: [libc::c_uchar; 4] = [0 as libc::c_int as libc::c_uchar, 0, 0, 0];
    let mut RLE_count: libc::c_int = 0 as libc::c_int;
    let mut RLE_repeating: libc::c_int = 0 as libc::c_int;
    let mut read_next_pixel: libc::c_int = 1 as libc::c_int;
    if tga_height > (1 as libc::c_int) << 24 as libc::c_int {
        return (if stbi__err(b"too large\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    if tga_width > (1 as libc::c_int) << 24 as libc::c_int {
        return (if stbi__err(b"too large\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    if tga_image_type >= 8 as libc::c_int {
        tga_image_type -= 8 as libc::c_int;
        tga_is_RLE = 1 as libc::c_int;
    }
    tga_inverted = 1 as libc::c_int
        - (tga_inverted >> 5 as libc::c_int & 1 as libc::c_int);
    if tga_indexed != 0 {
        tga_comp = stbi__tga_get_comp(
            tga_palette_bits,
            0 as libc::c_int,
            &mut tga_rgb16,
        );
    } else {
        tga_comp = stbi__tga_get_comp(
            tga_bits_per_pixel,
            (tga_image_type == 3 as libc::c_int) as libc::c_int,
            &mut tga_rgb16,
        );
    }
    if tga_comp == 0 {
        return (if stbi__err(b"bad format\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    *x = tga_width;
    *y = tga_height;
    if !comp.is_null() {
        *comp = tga_comp;
    }
    if stbi__mad3sizes_valid(tga_width, tga_height, tga_comp, 0 as libc::c_int) == 0 {
        return (if stbi__err(b"too large\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    tga_data = stbi__malloc_mad3(tga_width, tga_height, tga_comp, 0 as libc::c_int)
        as *mut libc::c_uchar;
    if tga_data.is_null() {
        return (if stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    stbi__skip(s, tga_offset);
    if tga_indexed == 0 && tga_is_RLE == 0 && tga_rgb16 == 0 {
        i = 0 as libc::c_int;
        while i < tga_height {
            let mut row: libc::c_int = if tga_inverted != 0 {
                tga_height - i - 1 as libc::c_int
            } else {
                i
            };
            let mut tga_row: *mut stbi_uc = tga_data
                .offset((row * tga_width * tga_comp) as isize);
            stbi__getn(s, tga_row, tga_width * tga_comp);
            i += 1;
        }
    } else {
        if tga_indexed != 0 {
            if tga_palette_len == 0 as libc::c_int {
                free(tga_data as *mut libc::c_void);
                return (if stbi__err(
                    b"bad palette\0" as *const u8 as *const libc::c_char,
                ) != 0
                {
                    0 as *mut libc::c_void
                } else {
                    0 as *mut libc::c_void
                }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
            }
            stbi__skip(s, tga_palette_start);
            tga_palette = stbi__malloc_mad2(tga_palette_len, tga_comp, 0 as libc::c_int)
                as *mut libc::c_uchar;
            if tga_palette.is_null() {
                free(tga_data as *mut libc::c_void);
                return (if stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char)
                    != 0
                {
                    0 as *mut libc::c_void
                } else {
                    0 as *mut libc::c_void
                }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
            }
            if tga_rgb16 != 0 {
                let mut pal_entry: *mut stbi_uc = tga_palette;
                i = 0 as libc::c_int;
                while i < tga_palette_len {
                    stbi__tga_read_rgb16(s, pal_entry);
                    pal_entry = pal_entry.offset(tga_comp as isize);
                    i += 1;
                }
            } else if stbi__getn(s, tga_palette, tga_palette_len * tga_comp) == 0 {
                free(tga_data as *mut libc::c_void);
                free(tga_palette as *mut libc::c_void);
                return (if stbi__err(
                    b"bad palette\0" as *const u8 as *const libc::c_char,
                ) != 0
                {
                    0 as *mut libc::c_void
                } else {
                    0 as *mut libc::c_void
                }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
            }
        }
        i = 0 as libc::c_int;
        while i < tga_width * tga_height {
            if tga_is_RLE != 0 {
                if RLE_count == 0 as libc::c_int {
                    let mut RLE_cmd: libc::c_int = stbi__get8(s) as libc::c_int;
                    RLE_count = 1 as libc::c_int + (RLE_cmd & 127 as libc::c_int);
                    RLE_repeating = RLE_cmd >> 7 as libc::c_int;
                    read_next_pixel = 1 as libc::c_int;
                } else if RLE_repeating == 0 {
                    read_next_pixel = 1 as libc::c_int;
                }
            } else {
                read_next_pixel = 1 as libc::c_int;
            }
            if read_next_pixel != 0 {
                if tga_indexed != 0 {
                    let mut pal_idx: libc::c_int = if tga_bits_per_pixel
                        == 8 as libc::c_int
                    {
                        stbi__get8(s) as libc::c_int
                    } else {
                        stbi__get16le(s)
                    };
                    if pal_idx >= tga_palette_len {
                        pal_idx = 0 as libc::c_int;
                    }
                    pal_idx *= tga_comp;
                    j = 0 as libc::c_int;
                    while j < tga_comp {
                        raw_data[j
                            as usize] = *tga_palette.offset((pal_idx + j) as isize);
                        j += 1;
                    }
                } else if tga_rgb16 != 0 {
                    stbi__tga_read_rgb16(s, raw_data.as_mut_ptr());
                } else {
                    j = 0 as libc::c_int;
                    while j < tga_comp {
                        raw_data[j as usize] = stbi__get8(s);
                        j += 1;
                    }
                }
                read_next_pixel = 0 as libc::c_int;
            }
            j = 0 as libc::c_int;
            while j < tga_comp {
                *tga_data.offset((i * tga_comp + j) as isize) = raw_data[j as usize];
                j += 1;
            }
            RLE_count -= 1;
            i += 1;
        }
        if tga_inverted != 0 {
            j = 0 as libc::c_int;
            while (j * 2 as libc::c_int) < tga_height {
                let mut index1: libc::c_int = j * tga_width * tga_comp;
                let mut index2: libc::c_int = (tga_height - 1 as libc::c_int - j)
                    * tga_width * tga_comp;
                i = tga_width * tga_comp;
                while i > 0 as libc::c_int {
                    let mut temp: libc::c_uchar = *tga_data.offset(index1 as isize);
                    *tga_data
                        .offset(index1 as isize) = *tga_data.offset(index2 as isize);
                    *tga_data.offset(index2 as isize) = temp;
                    index1 += 1;
                    index2 += 1;
                    i -= 1;
                }
                j += 1;
            }
        }
        if !tga_palette.is_null() {
            free(tga_palette as *mut libc::c_void);
        }
    }
    if tga_comp >= 3 as libc::c_int && tga_rgb16 == 0 {
        let mut tga_pixel: *mut libc::c_uchar = tga_data;
        i = 0 as libc::c_int;
        while i < tga_width * tga_height {
            let mut temp_0: libc::c_uchar = *tga_pixel.offset(0 as libc::c_int as isize);
            *tga_pixel
                .offset(
                    0 as libc::c_int as isize,
                ) = *tga_pixel.offset(2 as libc::c_int as isize);
            *tga_pixel.offset(2 as libc::c_int as isize) = temp_0;
            tga_pixel = tga_pixel.offset(tga_comp as isize);
            i += 1;
        }
    }
    if req_comp != 0 && req_comp != tga_comp {
        tga_data = stbi__convert_format(
            tga_data,
            tga_comp,
            req_comp,
            tga_width as libc::c_uint,
            tga_height as libc::c_uint,
        );
    }
    tga_y_origin = 0 as libc::c_int;
    tga_x_origin = tga_y_origin;
    tga_palette_bits = tga_x_origin;
    tga_palette_len = tga_palette_bits;
    tga_palette_start = tga_palette_len;
    return tga_data as *mut libc::c_void;
}
unsafe extern "C" fn stbi__get16le(mut s: *mut stbi__context) -> libc::c_int {
    let mut z: libc::c_int = stbi__get8(s) as libc::c_int;
    return z + ((stbi__get8(s) as libc::c_int) << 8 as libc::c_int);
}
unsafe extern "C" fn stbi__get8(mut s: *mut stbi__context) -> stbi_uc {
    if (*s).img_buffer < (*s).img_buffer_end {
        let fresh0 = (*s).img_buffer;
        (*s).img_buffer = ((*s).img_buffer).offset(1);
        return *fresh0;
    }
    if (*s).read_from_callbacks != 0 {
        stbi__refill_buffer(s);
        let fresh1 = (*s).img_buffer;
        (*s).img_buffer = ((*s).img_buffer).offset(1);
        return *fresh1;
    }
    return 0 as libc::c_int as stbi_uc;
}
unsafe extern "C" fn stbi__refill_buffer(mut s: *mut stbi__context) {
    let mut n: libc::c_int = ((*s).io.read)
        .expect(
            "non-null function pointer",
        )(
        (*s).io_user_data,
        ((*s).buffer_start).as_mut_ptr() as *mut libc::c_char,
        (*s).buflen,
    );
    (*s).callback_already_read
        += ((*s).img_buffer).offset_from((*s).img_buffer_original) as libc::c_long
            as libc::c_int;
    if n == 0 as libc::c_int {
        (*s).read_from_callbacks = 0 as libc::c_int;
        (*s).img_buffer = ((*s).buffer_start).as_mut_ptr();
        (*s)
            .img_buffer_end = ((*s).buffer_start)
            .as_mut_ptr()
            .offset(1 as libc::c_int as isize);
        *(*s).img_buffer = 0 as libc::c_int as stbi_uc;
    } else {
        (*s).img_buffer = ((*s).buffer_start).as_mut_ptr();
        (*s).img_buffer_end = ((*s).buffer_start).as_mut_ptr().offset(n as isize);
    };
}
unsafe extern "C" fn stbi__convert_format(
    mut data: *mut libc::c_uchar,
    mut img_n: libc::c_int,
    mut req_comp: libc::c_int,
    mut x: libc::c_uint,
    mut y: libc::c_uint,
) -> *mut libc::c_uchar {
    let mut i: libc::c_int = 0;
    let mut j: libc::c_int = 0;
    let mut good: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
    if req_comp == img_n {
        return data;
    }
    good = stbi__malloc_mad3(
        req_comp,
        x as libc::c_int,
        y as libc::c_int,
        0 as libc::c_int,
    ) as *mut libc::c_uchar;
    if good.is_null() {
        free(data as *mut libc::c_void);
        return (if stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar;
    }
    j = 0 as libc::c_int;
    while j < y as libc::c_int {
        let mut src: *mut libc::c_uchar = data
            .offset(
                (j as libc::c_uint).wrapping_mul(x).wrapping_mul(img_n as libc::c_uint)
                    as isize,
            );
        let mut dest: *mut libc::c_uchar = good
            .offset(
                (j as libc::c_uint)
                    .wrapping_mul(x)
                    .wrapping_mul(req_comp as libc::c_uint) as isize,
            );
        match img_n * 8 as libc::c_int + req_comp {
            10 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    *dest
                        .offset(
                            0 as libc::c_int as isize,
                        ) = *src.offset(0 as libc::c_int as isize);
                    *dest
                        .offset(
                            1 as libc::c_int as isize,
                        ) = 255 as libc::c_int as libc::c_uchar;
                    i -= 1;
                    src = src.offset(1 as libc::c_int as isize);
                    dest = dest.offset(2 as libc::c_int as isize);
                }
            }
            11 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    let ref mut fresh2 = *dest.offset(2 as libc::c_int as isize);
                    *fresh2 = *src.offset(0 as libc::c_int as isize);
                    let ref mut fresh3 = *dest.offset(1 as libc::c_int as isize);
                    *fresh3 = *fresh2;
                    *dest.offset(0 as libc::c_int as isize) = *fresh3;
                    i -= 1;
                    src = src.offset(1 as libc::c_int as isize);
                    dest = dest.offset(3 as libc::c_int as isize);
                }
            }
            12 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    let ref mut fresh4 = *dest.offset(2 as libc::c_int as isize);
                    *fresh4 = *src.offset(0 as libc::c_int as isize);
                    let ref mut fresh5 = *dest.offset(1 as libc::c_int as isize);
                    *fresh5 = *fresh4;
                    *dest.offset(0 as libc::c_int as isize) = *fresh5;
                    *dest
                        .offset(
                            3 as libc::c_int as isize,
                        ) = 255 as libc::c_int as libc::c_uchar;
                    i -= 1;
                    src = src.offset(1 as libc::c_int as isize);
                    dest = dest.offset(4 as libc::c_int as isize);
                }
            }
            17 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    *dest
                        .offset(
                            0 as libc::c_int as isize,
                        ) = *src.offset(0 as libc::c_int as isize);
                    i -= 1;
                    src = src.offset(2 as libc::c_int as isize);
                    dest = dest.offset(1 as libc::c_int as isize);
                }
            }
            19 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    let ref mut fresh6 = *dest.offset(2 as libc::c_int as isize);
                    *fresh6 = *src.offset(0 as libc::c_int as isize);
                    let ref mut fresh7 = *dest.offset(1 as libc::c_int as isize);
                    *fresh7 = *fresh6;
                    *dest.offset(0 as libc::c_int as isize) = *fresh7;
                    i -= 1;
                    src = src.offset(2 as libc::c_int as isize);
                    dest = dest.offset(3 as libc::c_int as isize);
                }
            }
            20 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    let ref mut fresh8 = *dest.offset(2 as libc::c_int as isize);
                    *fresh8 = *src.offset(0 as libc::c_int as isize);
                    let ref mut fresh9 = *dest.offset(1 as libc::c_int as isize);
                    *fresh9 = *fresh8;
                    *dest.offset(0 as libc::c_int as isize) = *fresh9;
                    *dest
                        .offset(
                            3 as libc::c_int as isize,
                        ) = *src.offset(1 as libc::c_int as isize);
                    i -= 1;
                    src = src.offset(2 as libc::c_int as isize);
                    dest = dest.offset(4 as libc::c_int as isize);
                }
            }
            28 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    *dest
                        .offset(
                            0 as libc::c_int as isize,
                        ) = *src.offset(0 as libc::c_int as isize);
                    *dest
                        .offset(
                            1 as libc::c_int as isize,
                        ) = *src.offset(1 as libc::c_int as isize);
                    *dest
                        .offset(
                            2 as libc::c_int as isize,
                        ) = *src.offset(2 as libc::c_int as isize);
                    *dest
                        .offset(
                            3 as libc::c_int as isize,
                        ) = 255 as libc::c_int as libc::c_uchar;
                    i -= 1;
                    src = src.offset(3 as libc::c_int as isize);
                    dest = dest.offset(4 as libc::c_int as isize);
                }
            }
            25 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    *dest
                        .offset(
                            0 as libc::c_int as isize,
                        ) = stbi__compute_y(
                        *src.offset(0 as libc::c_int as isize) as libc::c_int,
                        *src.offset(1 as libc::c_int as isize) as libc::c_int,
                        *src.offset(2 as libc::c_int as isize) as libc::c_int,
                    );
                    i -= 1;
                    src = src.offset(3 as libc::c_int as isize);
                    dest = dest.offset(1 as libc::c_int as isize);
                }
            }
            26 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    *dest
                        .offset(
                            0 as libc::c_int as isize,
                        ) = stbi__compute_y(
                        *src.offset(0 as libc::c_int as isize) as libc::c_int,
                        *src.offset(1 as libc::c_int as isize) as libc::c_int,
                        *src.offset(2 as libc::c_int as isize) as libc::c_int,
                    );
                    *dest
                        .offset(
                            1 as libc::c_int as isize,
                        ) = 255 as libc::c_int as libc::c_uchar;
                    i -= 1;
                    src = src.offset(3 as libc::c_int as isize);
                    dest = dest.offset(2 as libc::c_int as isize);
                }
            }
            33 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    *dest
                        .offset(
                            0 as libc::c_int as isize,
                        ) = stbi__compute_y(
                        *src.offset(0 as libc::c_int as isize) as libc::c_int,
                        *src.offset(1 as libc::c_int as isize) as libc::c_int,
                        *src.offset(2 as libc::c_int as isize) as libc::c_int,
                    );
                    i -= 1;
                    src = src.offset(4 as libc::c_int as isize);
                    dest = dest.offset(1 as libc::c_int as isize);
                }
            }
            34 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    *dest
                        .offset(
                            0 as libc::c_int as isize,
                        ) = stbi__compute_y(
                        *src.offset(0 as libc::c_int as isize) as libc::c_int,
                        *src.offset(1 as libc::c_int as isize) as libc::c_int,
                        *src.offset(2 as libc::c_int as isize) as libc::c_int,
                    );
                    *dest
                        .offset(
                            1 as libc::c_int as isize,
                        ) = *src.offset(3 as libc::c_int as isize);
                    i -= 1;
                    src = src.offset(4 as libc::c_int as isize);
                    dest = dest.offset(2 as libc::c_int as isize);
                }
            }
            35 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    *dest
                        .offset(
                            0 as libc::c_int as isize,
                        ) = *src.offset(0 as libc::c_int as isize);
                    *dest
                        .offset(
                            1 as libc::c_int as isize,
                        ) = *src.offset(1 as libc::c_int as isize);
                    *dest
                        .offset(
                            2 as libc::c_int as isize,
                        ) = *src.offset(2 as libc::c_int as isize);
                    i -= 1;
                    src = src.offset(4 as libc::c_int as isize);
                    dest = dest.offset(3 as libc::c_int as isize);
                }
            }
            _ => {
                free(data as *mut libc::c_void);
                free(good as *mut libc::c_void);
                return (if stbi__err(
                    b"unsupported\0" as *const u8 as *const libc::c_char,
                ) != 0
                {
                    0 as *mut libc::c_void
                } else {
                    0 as *mut libc::c_void
                }) as size_t as *mut libc::c_uchar;
            }
        }
        j += 1;
    }
    free(data as *mut libc::c_void);
    return good;
}
unsafe extern "C" fn stbi__compute_y(
    mut r: libc::c_int,
    mut g: libc::c_int,
    mut b: libc::c_int,
) -> stbi_uc {
    return (r * 77 as libc::c_int + g * 150 as libc::c_int + 29 as libc::c_int * b
        >> 8 as libc::c_int) as stbi_uc;
}
unsafe extern "C" fn stbi__malloc_mad3(
    mut a: libc::c_int,
    mut b: libc::c_int,
    mut c: libc::c_int,
    mut add: libc::c_int,
) -> *mut libc::c_void {
    if stbi__mad3sizes_valid(a, b, c, add) == 0 {
        return 0 as *mut libc::c_void;
    }
    return stbi__malloc((a * b * c + add) as size_t);
}
unsafe extern "C" fn stbi__malloc(mut size: size_t) -> *mut libc::c_void {
    return malloc(size);
}
unsafe extern "C" fn stbi__mad3sizes_valid(
    mut a: libc::c_int,
    mut b: libc::c_int,
    mut c: libc::c_int,
    mut add: libc::c_int,
) -> libc::c_int {
    return (stbi__mul2sizes_valid(a, b) != 0 && stbi__mul2sizes_valid(a * b, c) != 0
        && stbi__addsizes_valid(a * b * c, add) != 0) as libc::c_int;
}
unsafe extern "C" fn stbi__addsizes_valid(
    mut a: libc::c_int,
    mut b: libc::c_int,
) -> libc::c_int {
    if b < 0 as libc::c_int {
        return 0 as libc::c_int;
    }
    return (a <= 2147483647 as libc::c_int - b) as libc::c_int;
}
unsafe extern "C" fn stbi__mul2sizes_valid(
    mut a: libc::c_int,
    mut b: libc::c_int,
) -> libc::c_int {
    if a < 0 as libc::c_int || b < 0 as libc::c_int {
        return 0 as libc::c_int;
    }
    if b == 0 as libc::c_int {
        return 1 as libc::c_int;
    }
    return (a <= 2147483647 as libc::c_int / b) as libc::c_int;
}
unsafe extern "C" fn stbi__tga_read_rgb16(
    mut s: *mut stbi__context,
    mut out: *mut stbi_uc,
) {
    let mut px: stbi__uint16 = stbi__get16le(s) as stbi__uint16;
    let mut fiveBitMask: stbi__uint16 = 31 as libc::c_int as stbi__uint16;
    let mut r: libc::c_int = px as libc::c_int >> 10 as libc::c_int
        & fiveBitMask as libc::c_int;
    let mut g: libc::c_int = px as libc::c_int >> 5 as libc::c_int
        & fiveBitMask as libc::c_int;
    let mut b: libc::c_int = px as libc::c_int & fiveBitMask as libc::c_int;
    *out
        .offset(
            0 as libc::c_int as isize,
        ) = (r * 255 as libc::c_int / 31 as libc::c_int) as stbi_uc;
    *out
        .offset(
            1 as libc::c_int as isize,
        ) = (g * 255 as libc::c_int / 31 as libc::c_int) as stbi_uc;
    *out
        .offset(
            2 as libc::c_int as isize,
        ) = (b * 255 as libc::c_int / 31 as libc::c_int) as stbi_uc;
}
unsafe extern "C" fn stbi__getn(
    mut s: *mut stbi__context,
    mut buffer: *mut stbi_uc,
    mut n: libc::c_int,
) -> libc::c_int {
    if ((*s).io.read).is_some() {
        let mut blen: libc::c_int = ((*s).img_buffer_end).offset_from((*s).img_buffer)
            as libc::c_long as libc::c_int;
        if blen < n {
            let mut res: libc::c_int = 0;
            let mut count: libc::c_int = 0;
            memcpy(
                buffer as *mut libc::c_void,
                (*s).img_buffer as *const libc::c_void,
                blen as libc::c_ulong,
            );
            count = ((*s).io.read)
                .expect(
                    "non-null function pointer",
                )(
                (*s).io_user_data,
                (buffer as *mut libc::c_char).offset(blen as isize),
                n - blen,
            );
            res = (count == n - blen) as libc::c_int;
            (*s).img_buffer = (*s).img_buffer_end;
            return res;
        }
    }
    if ((*s).img_buffer).offset(n as isize) <= (*s).img_buffer_end {
        memcpy(
            buffer as *mut libc::c_void,
            (*s).img_buffer as *const libc::c_void,
            n as libc::c_ulong,
        );
        (*s).img_buffer = ((*s).img_buffer).offset(n as isize);
        return 1 as libc::c_int;
    } else {
        return 0 as libc::c_int
    };
}
unsafe extern "C" fn stbi__malloc_mad2(
    mut a: libc::c_int,
    mut b: libc::c_int,
    mut add: libc::c_int,
) -> *mut libc::c_void {
    if stbi__mad2sizes_valid(a, b, add) == 0 {
        return 0 as *mut libc::c_void;
    }
    return stbi__malloc((a * b + add) as size_t);
}
unsafe extern "C" fn stbi__mad2sizes_valid(
    mut a: libc::c_int,
    mut b: libc::c_int,
    mut add: libc::c_int,
) -> libc::c_int {
    return (stbi__mul2sizes_valid(a, b) != 0 && stbi__addsizes_valid(a * b, add) != 0)
        as libc::c_int;
}
unsafe extern "C" fn stbi__skip(mut s: *mut stbi__context, mut n: libc::c_int) {
    if n == 0 as libc::c_int {
        return;
    }
    if n < 0 as libc::c_int {
        (*s).img_buffer = (*s).img_buffer_end;
        return;
    }
    if ((*s).io.read).is_some() {
        let mut blen: libc::c_int = ((*s).img_buffer_end).offset_from((*s).img_buffer)
            as libc::c_long as libc::c_int;
        if blen < n {
            (*s).img_buffer = (*s).img_buffer_end;
            ((*s).io.skip)
                .expect("non-null function pointer")((*s).io_user_data, n - blen);
            return;
        }
    }
    (*s).img_buffer = ((*s).img_buffer).offset(n as isize);
}
unsafe extern "C" fn stbi__tga_get_comp(
    mut bits_per_pixel: libc::c_int,
    mut is_grey: libc::c_int,
    mut is_rgb16: *mut libc::c_int,
) -> libc::c_int {
    if !is_rgb16.is_null() {
        *is_rgb16 = 0 as libc::c_int;
    }
    match bits_per_pixel {
        8 => return STBI_grey as libc::c_int,
        16 => {
            if is_grey != 0 {
                return STBI_grey_alpha as libc::c_int;
            }
        }
        15 => {}
        24 | 32 => return bits_per_pixel / 8 as libc::c_int,
        _ => return 0 as libc::c_int,
    }
    if !is_rgb16.is_null() {
        *is_rgb16 = 1 as libc::c_int;
    }
    return STBI_rgb as libc::c_int;
}
unsafe extern "C" fn stbi__tga_test(mut s: *mut stbi__context) -> libc::c_int {
    let mut current_block: u64;
    let mut res: libc::c_int = 0 as libc::c_int;
    let mut sz: libc::c_int = 0;
    let mut tga_color_type: libc::c_int = 0;
    stbi__get8(s);
    tga_color_type = stbi__get8(s) as libc::c_int;
    if !(tga_color_type > 1 as libc::c_int) {
        sz = stbi__get8(s) as libc::c_int;
        if tga_color_type == 1 as libc::c_int {
            if sz != 1 as libc::c_int && sz != 9 as libc::c_int {
                current_block = 11945806727881333379;
            } else {
                stbi__skip(s, 4 as libc::c_int);
                sz = stbi__get8(s) as libc::c_int;
                if sz != 8 as libc::c_int && sz != 15 as libc::c_int
                    && sz != 16 as libc::c_int && sz != 24 as libc::c_int
                    && sz != 32 as libc::c_int
                {
                    current_block = 11945806727881333379;
                } else {
                    stbi__skip(s, 4 as libc::c_int);
                    current_block = 13536709405535804910;
                }
            }
        } else if sz != 2 as libc::c_int && sz != 3 as libc::c_int
            && sz != 10 as libc::c_int && sz != 11 as libc::c_int
        {
            current_block = 11945806727881333379;
        } else {
            stbi__skip(s, 9 as libc::c_int);
            current_block = 13536709405535804910;
        }
        match current_block {
            11945806727881333379 => {}
            _ => {
                if !(stbi__get16le(s) < 1 as libc::c_int) {
                    if !(stbi__get16le(s) < 1 as libc::c_int) {
                        sz = stbi__get8(s) as libc::c_int;
                        if !(tga_color_type == 1 as libc::c_int && sz != 8 as libc::c_int
                            && sz != 16 as libc::c_int)
                        {
                            if !(sz != 8 as libc::c_int && sz != 15 as libc::c_int
                                && sz != 16 as libc::c_int && sz != 24 as libc::c_int
                                && sz != 32 as libc::c_int)
                            {
                                res = 1 as libc::c_int;
                            }
                        }
                    }
                }
            }
        }
    }
    stbi__rewind(s);
    return res;
}
unsafe extern "C" fn stbi__rewind(mut s: *mut stbi__context) {
    (*s).img_buffer = (*s).img_buffer_original;
    (*s).img_buffer_end = (*s).img_buffer_original_end;
}
unsafe extern "C" fn stbi__hdr_load(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
    mut ri: *mut stbi__result_info,
) -> *mut libc::c_float {
    let mut buffer: [libc::c_char; 1024] = [0; 1024];
    let mut token: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut valid: libc::c_int = 0 as libc::c_int;
    let mut width: libc::c_int = 0;
    let mut height: libc::c_int = 0;
    let mut scanline: *mut stbi_uc = 0 as *mut stbi_uc;
    let mut hdr_data: *mut libc::c_float = 0 as *mut libc::c_float;
    let mut len: libc::c_int = 0;
    let mut count: libc::c_uchar = 0;
    let mut value: libc::c_uchar = 0;
    let mut i: libc::c_int = 0;
    let mut j: libc::c_int = 0;
    let mut k: libc::c_int = 0;
    let mut c1: libc::c_int = 0;
    let mut c2: libc::c_int = 0;
    let mut z: libc::c_int = 0;
    let mut headerToken: *const libc::c_char = 0 as *const libc::c_char;
    headerToken = stbi__hdr_gettoken(s, buffer.as_mut_ptr());
    if strcmp(headerToken, b"#?RADIANCE\0" as *const u8 as *const libc::c_char)
        != 0 as libc::c_int
        && strcmp(headerToken, b"#?RGBE\0" as *const u8 as *const libc::c_char)
            != 0 as libc::c_int
    {
        return (if stbi__err(b"not HDR\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_float;
    }
    loop {
        token = stbi__hdr_gettoken(s, buffer.as_mut_ptr());
        if *token.offset(0 as libc::c_int as isize) as libc::c_int == 0 as libc::c_int {
            break;
        }
        if strcmp(token, b"FORMAT=32-bit_rle_rgbe\0" as *const u8 as *const libc::c_char)
            == 0 as libc::c_int
        {
            valid = 1 as libc::c_int;
        }
    }
    if valid == 0 {
        return (if stbi__err(b"unsupported format\0" as *const u8 as *const libc::c_char)
            != 0
        {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_float;
    }
    token = stbi__hdr_gettoken(s, buffer.as_mut_ptr());
    if strncmp(
        token,
        b"-Y \0" as *const u8 as *const libc::c_char,
        3 as libc::c_int as libc::c_ulong,
    ) != 0
    {
        return (if stbi__err(
            b"unsupported data layout\0" as *const u8 as *const libc::c_char,
        ) != 0
        {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_float;
    }
    token = token.offset(3 as libc::c_int as isize);
    height = strtol(token, &mut token, 10 as libc::c_int) as libc::c_int;
    while *token as libc::c_int == ' ' as i32 {
        token = token.offset(1);
    }
    if strncmp(
        token,
        b"+X \0" as *const u8 as *const libc::c_char,
        3 as libc::c_int as libc::c_ulong,
    ) != 0
    {
        return (if stbi__err(
            b"unsupported data layout\0" as *const u8 as *const libc::c_char,
        ) != 0
        {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_float;
    }
    token = token.offset(3 as libc::c_int as isize);
    width = strtol(token, 0 as *mut *mut libc::c_char, 10 as libc::c_int) as libc::c_int;
    if height > (1 as libc::c_int) << 24 as libc::c_int {
        return (if stbi__err(b"too large\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_float;
    }
    if width > (1 as libc::c_int) << 24 as libc::c_int {
        return (if stbi__err(b"too large\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_float;
    }
    *x = width;
    *y = height;
    if !comp.is_null() {
        *comp = 3 as libc::c_int;
    }
    if req_comp == 0 as libc::c_int {
        req_comp = 3 as libc::c_int;
    }
    if stbi__mad4sizes_valid(
        width,
        height,
        req_comp,
        ::core::mem::size_of::<libc::c_float>() as libc::c_ulong as libc::c_int,
        0 as libc::c_int,
    ) == 0
    {
        return (if stbi__err(b"too large\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_float;
    }
    hdr_data = stbi__malloc_mad4(
        width,
        height,
        req_comp,
        ::core::mem::size_of::<libc::c_float>() as libc::c_ulong as libc::c_int,
        0 as libc::c_int,
    ) as *mut libc::c_float;
    if hdr_data.is_null() {
        return (if stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_float;
    }
    's_445: {
        let mut rgbe: [stbi_uc; 4] = [0; 4];
        let mut current_block_99: u64;
        if width < 8 as libc::c_int || width >= 32768 as libc::c_int {
            j = 0 as libc::c_int;
            current_block_99 = 18377268871191777778;
        } else {
            scanline = 0 as *mut stbi_uc;
            j = 0 as libc::c_int;
            loop {
                if !(j < height) {
                    current_block_99 = 10213293998891106930;
                    break;
                }
                c1 = stbi__get8(s) as libc::c_int;
                c2 = stbi__get8(s) as libc::c_int;
                len = stbi__get8(s) as libc::c_int;
                if c1 != 2 as libc::c_int || c2 != 2 as libc::c_int
                    || len & 0x80 as libc::c_int != 0
                {
                    let mut rgbe_0: [stbi_uc; 4] = [0; 4];
                    rgbe_0[0 as libc::c_int as usize] = c1 as stbi_uc;
                    rgbe_0[1 as libc::c_int as usize] = c2 as stbi_uc;
                    rgbe_0[2 as libc::c_int as usize] = len as stbi_uc;
                    rgbe_0[3 as libc::c_int as usize] = stbi__get8(s);
                    stbi__hdr_convert(hdr_data, rgbe_0.as_mut_ptr(), req_comp);
                    i = 1 as libc::c_int;
                    j = 0 as libc::c_int;
                    free(scanline as *mut libc::c_void);
                    current_block_99 = 17668554241052760946;
                    break;
                } else {
                    len <<= 8 as libc::c_int;
                    len |= stbi__get8(s) as libc::c_int;
                    if len != width {
                        free(hdr_data as *mut libc::c_void);
                        free(scanline as *mut libc::c_void);
                        return (if stbi__err(
                            b"invalid decoded scanline length\0" as *const u8
                                as *const libc::c_char,
                        ) != 0
                        {
                            0 as *mut libc::c_void
                        } else {
                            0 as *mut libc::c_void
                        }) as size_t as *mut libc::c_float;
                    }
                    if scanline.is_null() {
                        scanline = stbi__malloc_mad2(
                            width,
                            4 as libc::c_int,
                            0 as libc::c_int,
                        ) as *mut stbi_uc;
                        if scanline.is_null() {
                            free(hdr_data as *mut libc::c_void);
                            return (if stbi__err(
                                b"outofmem\0" as *const u8 as *const libc::c_char,
                            ) != 0
                            {
                                0 as *mut libc::c_void
                            } else {
                                0 as *mut libc::c_void
                            }) as size_t as *mut libc::c_float;
                        }
                    }
                    k = 0 as libc::c_int;
                    while k < 4 as libc::c_int {
                        let mut nleft: libc::c_int = 0;
                        i = 0 as libc::c_int;
                        loop {
                            nleft = width - i;
                            if !(nleft > 0 as libc::c_int) {
                                break;
                            }
                            count = stbi__get8(s);
                            if count as libc::c_int > 128 as libc::c_int {
                                value = stbi__get8(s);
                                count = (count as libc::c_int - 128 as libc::c_int)
                                    as libc::c_uchar;
                                if count as libc::c_int > nleft {
                                    free(hdr_data as *mut libc::c_void);
                                    free(scanline as *mut libc::c_void);
                                    return (if stbi__err(
                                        b"corrupt\0" as *const u8 as *const libc::c_char,
                                    ) != 0
                                    {
                                        0 as *mut libc::c_void
                                    } else {
                                        0 as *mut libc::c_void
                                    }) as size_t as *mut libc::c_float;
                                }
                                z = 0 as libc::c_int;
                                while z < count as libc::c_int {
                                    let fresh10 = i;
                                    i = i + 1;
                                    *scanline
                                        .offset((fresh10 * 4 as libc::c_int + k) as isize) = value;
                                    z += 1;
                                }
                            } else {
                                if count as libc::c_int > nleft {
                                    free(hdr_data as *mut libc::c_void);
                                    free(scanline as *mut libc::c_void);
                                    return (if stbi__err(
                                        b"corrupt\0" as *const u8 as *const libc::c_char,
                                    ) != 0
                                    {
                                        0 as *mut libc::c_void
                                    } else {
                                        0 as *mut libc::c_void
                                    }) as size_t as *mut libc::c_float;
                                }
                                z = 0 as libc::c_int;
                                while z < count as libc::c_int {
                                    let fresh11 = i;
                                    i = i + 1;
                                    *scanline
                                        .offset(
                                            (fresh11 * 4 as libc::c_int + k) as isize,
                                        ) = stbi__get8(s);
                                    z += 1;
                                }
                            }
                        }
                        k += 1;
                    }
                    i = 0 as libc::c_int;
                    while i < width {
                        stbi__hdr_convert(
                            hdr_data.offset(((j * width + i) * req_comp) as isize),
                            scanline.offset((i * 4 as libc::c_int) as isize),
                            req_comp,
                        );
                        i += 1;
                    }
                    j += 1;
                }
            }
            match current_block_99 {
                17668554241052760946 => {}
                _ => {
                    if !scanline.is_null() {
                        free(scanline as *mut libc::c_void);
                    }
                    current_block_99 = 12705158477165241210;
                }
            }
        }
        loop {
            match current_block_99 {
                12705158477165241210 => {
                    break 's_445;
                }
                18377268871191777778 => {
                    if !(j < height) {
                        current_block_99 = 12705158477165241210;
                        continue;
                    }
                    i = 0 as libc::c_int;
                }
                _ => {
                    stbi__getn(s, rgbe.as_mut_ptr(), 4 as libc::c_int);
                    stbi__hdr_convert(
                        hdr_data
                            .offset((j * width * req_comp) as isize)
                            .offset((i * req_comp) as isize),
                        rgbe.as_mut_ptr(),
                        req_comp,
                    );
                    i += 1;
                }
            }
            if i < width {
                rgbe = [0; 4];
                current_block_99 = 17668554241052760946;
            } else {
                j += 1;
                current_block_99 = 18377268871191777778;
            }
        }
    }
    return hdr_data;
}
unsafe extern "C" fn stbi__hdr_convert(
    mut output: *mut libc::c_float,
    mut input: *mut stbi_uc,
    mut req_comp: libc::c_int,
) {
    if *input.offset(3 as libc::c_int as isize) as libc::c_int != 0 as libc::c_int {
        let mut f1: libc::c_float = 0.;
        f1 = ldexp(
            1.0f32 as libc::c_double,
            *input.offset(3 as libc::c_int as isize) as libc::c_int
                - (128 as libc::c_int + 8 as libc::c_int),
        ) as libc::c_float;
        if req_comp <= 2 as libc::c_int {
            *output
                .offset(
                    0 as libc::c_int as isize,
                ) = (*input.offset(0 as libc::c_int as isize) as libc::c_int
                + *input.offset(1 as libc::c_int as isize) as libc::c_int
                + *input.offset(2 as libc::c_int as isize) as libc::c_int)
                as libc::c_float * f1 / 3 as libc::c_int as libc::c_float;
        } else {
            *output
                .offset(
                    0 as libc::c_int as isize,
                ) = *input.offset(0 as libc::c_int as isize) as libc::c_int
                as libc::c_float * f1;
            *output
                .offset(
                    1 as libc::c_int as isize,
                ) = *input.offset(1 as libc::c_int as isize) as libc::c_int
                as libc::c_float * f1;
            *output
                .offset(
                    2 as libc::c_int as isize,
                ) = *input.offset(2 as libc::c_int as isize) as libc::c_int
                as libc::c_float * f1;
        }
        if req_comp == 2 as libc::c_int {
            *output
                .offset(1 as libc::c_int as isize) = 1 as libc::c_int as libc::c_float;
        }
        if req_comp == 4 as libc::c_int {
            *output
                .offset(3 as libc::c_int as isize) = 1 as libc::c_int as libc::c_float;
        }
    } else {
        let mut current_block_15: u64;
        match req_comp {
            4 => {
                *output
                    .offset(
                        3 as libc::c_int as isize,
                    ) = 1 as libc::c_int as libc::c_float;
                current_block_15 = 10400494003451230392;
            }
            3 => {
                current_block_15 = 10400494003451230392;
            }
            2 => {
                *output
                    .offset(
                        1 as libc::c_int as isize,
                    ) = 1 as libc::c_int as libc::c_float;
                current_block_15 = 17894757719231207738;
            }
            1 => {
                current_block_15 = 17894757719231207738;
            }
            _ => {
                current_block_15 = 17407779659766490442;
            }
        }
        match current_block_15 {
            10400494003451230392 => {
                let ref mut fresh12 = *output.offset(2 as libc::c_int as isize);
                *fresh12 = 0 as libc::c_int as libc::c_float;
                let ref mut fresh13 = *output.offset(1 as libc::c_int as isize);
                *fresh13 = *fresh12;
                *output.offset(0 as libc::c_int as isize) = *fresh13;
            }
            17894757719231207738 => {
                *output
                    .offset(
                        0 as libc::c_int as isize,
                    ) = 0 as libc::c_int as libc::c_float;
            }
            _ => {}
        }
    };
}
unsafe extern "C" fn stbi__malloc_mad4(
    mut a: libc::c_int,
    mut b: libc::c_int,
    mut c: libc::c_int,
    mut d: libc::c_int,
    mut add: libc::c_int,
) -> *mut libc::c_void {
    if stbi__mad4sizes_valid(a, b, c, d, add) == 0 {
        return 0 as *mut libc::c_void;
    }
    return stbi__malloc((a * b * c * d + add) as size_t);
}
unsafe extern "C" fn stbi__mad4sizes_valid(
    mut a: libc::c_int,
    mut b: libc::c_int,
    mut c: libc::c_int,
    mut d: libc::c_int,
    mut add: libc::c_int,
) -> libc::c_int {
    return (stbi__mul2sizes_valid(a, b) != 0 && stbi__mul2sizes_valid(a * b, c) != 0
        && stbi__mul2sizes_valid(a * b * c, d) != 0
        && stbi__addsizes_valid(a * b * c * d, add) != 0) as libc::c_int;
}
unsafe extern "C" fn stbi__hdr_gettoken(
    mut z: *mut stbi__context,
    mut buffer: *mut libc::c_char,
) -> *mut libc::c_char {
    let mut len: libc::c_int = 0 as libc::c_int;
    let mut c: libc::c_char = '\0' as i32 as libc::c_char;
    c = stbi__get8(z) as libc::c_char;
    while stbi__at_eof(z) == 0 && c as libc::c_int != '\n' as i32 {
        let fresh14 = len;
        len = len + 1;
        *buffer.offset(fresh14 as isize) = c;
        if len == 1024 as libc::c_int - 1 as libc::c_int {
            while stbi__at_eof(z) == 0 && stbi__get8(z) as libc::c_int != '\n' as i32 {}
            break;
        } else {
            c = stbi__get8(z) as libc::c_char;
        }
    }
    *buffer.offset(len as isize) = 0 as libc::c_int as libc::c_char;
    return buffer;
}
unsafe extern "C" fn stbi__at_eof(mut s: *mut stbi__context) -> libc::c_int {
    if ((*s).io.read).is_some() {
        if ((*s).io.eof).expect("non-null function pointer")((*s).io_user_data) == 0 {
            return 0 as libc::c_int;
        }
        if (*s).read_from_callbacks == 0 as libc::c_int {
            return 1 as libc::c_int;
        }
    }
    return ((*s).img_buffer >= (*s).img_buffer_end) as libc::c_int;
}
unsafe extern "C" fn stbi__hdr_to_ldr(
    mut data: *mut libc::c_float,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut comp: libc::c_int,
) -> *mut stbi_uc {
    let mut i: libc::c_int = 0;
    let mut k: libc::c_int = 0;
    let mut n: libc::c_int = 0;
    let mut output: *mut stbi_uc = 0 as *mut stbi_uc;
    if data.is_null() {
        return 0 as *mut stbi_uc;
    }
    output = stbi__malloc_mad3(x, y, comp, 0 as libc::c_int) as *mut stbi_uc;
    if output.is_null() {
        free(data as *mut libc::c_void);
        return (if stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar;
    }
    if comp & 1 as libc::c_int != 0 {
        n = comp;
    } else {
        n = comp - 1 as libc::c_int;
    }
    i = 0 as libc::c_int;
    while i < x * y {
        k = 0 as libc::c_int;
        while k < n {
            let mut z: libc::c_float = pow(
                (*data.offset((i * comp + k) as isize) * stbi__h2l_scale_i)
                    as libc::c_double,
                stbi__h2l_gamma_i as libc::c_double,
            ) as libc::c_float * 255 as libc::c_int as libc::c_float + 0.5f32;
            if z < 0 as libc::c_int as libc::c_float {
                z = 0 as libc::c_int as libc::c_float;
            }
            if z > 255 as libc::c_int as libc::c_float {
                z = 255 as libc::c_int as libc::c_float;
            }
            *output.offset((i * comp + k) as isize) = z as libc::c_int as stbi_uc;
            k += 1;
        }
        if k < comp {
            let mut z_0: libc::c_float = *data.offset((i * comp + k) as isize)
                * 255 as libc::c_int as libc::c_float + 0.5f32;
            if z_0 < 0 as libc::c_int as libc::c_float {
                z_0 = 0 as libc::c_int as libc::c_float;
            }
            if z_0 > 255 as libc::c_int as libc::c_float {
                z_0 = 255 as libc::c_int as libc::c_float;
            }
            *output.offset((i * comp + k) as isize) = z_0 as libc::c_int as stbi_uc;
        }
        i += 1;
    }
    free(data as *mut libc::c_void);
    return output;
}
static mut stbi__h2l_gamma_i: libc::c_float = 1.0f32 / 2.2f32;
static mut stbi__h2l_scale_i: libc::c_float = 1.0f32;
unsafe extern "C" fn stbi__hdr_test(mut s: *mut stbi__context) -> libc::c_int {
    let mut r: libc::c_int = stbi__hdr_test_core(
        s,
        b"#?RADIANCE\n\0" as *const u8 as *const libc::c_char,
    );
    stbi__rewind(s);
    if r == 0 {
        r = stbi__hdr_test_core(s, b"#?RGBE\n\0" as *const u8 as *const libc::c_char);
        stbi__rewind(s);
    }
    return r;
}
unsafe extern "C" fn stbi__hdr_test_core(
    mut s: *mut stbi__context,
    mut signature: *const libc::c_char,
) -> libc::c_int {
    let mut i: libc::c_int = 0;
    i = 0 as libc::c_int;
    while *signature.offset(i as isize) != 0 {
        if stbi__get8(s) as libc::c_int != *signature.offset(i as isize) as libc::c_int {
            return 0 as libc::c_int;
        }
        i += 1;
    }
    stbi__rewind(s);
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__pnm_load(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
    mut ri: *mut stbi__result_info,
) -> *mut libc::c_void {
    let mut out: *mut stbi_uc = 0 as *mut stbi_uc;
    (*ri)
        .bits_per_channel = stbi__pnm_info(
        s,
        &mut (*s).img_x as *mut stbi__uint32 as *mut libc::c_int,
        &mut (*s).img_y as *mut stbi__uint32 as *mut libc::c_int,
        &mut (*s).img_n as *mut libc::c_int,
    );
    if (*ri).bits_per_channel == 0 as libc::c_int {
        return 0 as *mut libc::c_void;
    }
    if (*s).img_y > ((1 as libc::c_int) << 24 as libc::c_int) as libc::c_uint {
        return (if stbi__err(b"too large\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    if (*s).img_x > ((1 as libc::c_int) << 24 as libc::c_int) as libc::c_uint {
        return (if stbi__err(b"too large\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    *x = (*s).img_x as libc::c_int;
    *y = (*s).img_y as libc::c_int;
    if !comp.is_null() {
        *comp = (*s).img_n;
    }
    if stbi__mad4sizes_valid(
        (*s).img_n,
        (*s).img_x as libc::c_int,
        (*s).img_y as libc::c_int,
        (*ri).bits_per_channel / 8 as libc::c_int,
        0 as libc::c_int,
    ) == 0
    {
        return (if stbi__err(b"too large\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    out = stbi__malloc_mad4(
        (*s).img_n,
        (*s).img_x as libc::c_int,
        (*s).img_y as libc::c_int,
        (*ri).bits_per_channel / 8 as libc::c_int,
        0 as libc::c_int,
    ) as *mut stbi_uc;
    if out.is_null() {
        return (if stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    stbi__getn(
        s,
        out,
        ((*s).img_n as libc::c_uint)
            .wrapping_mul((*s).img_x)
            .wrapping_mul((*s).img_y)
            .wrapping_mul(((*ri).bits_per_channel / 8 as libc::c_int) as libc::c_uint)
            as libc::c_int,
    );
    if req_comp != 0 && req_comp != (*s).img_n {
        out = stbi__convert_format(out, (*s).img_n, req_comp, (*s).img_x, (*s).img_y);
        if out.is_null() {
            return out as *mut libc::c_void;
        }
    }
    return out as *mut libc::c_void;
}
unsafe extern "C" fn stbi__pnm_info(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
) -> libc::c_int {
    let mut maxv: libc::c_int = 0;
    let mut dummy: libc::c_int = 0;
    let mut c: libc::c_char = 0;
    let mut p: libc::c_char = 0;
    let mut t: libc::c_char = 0;
    if x.is_null() {
        x = &mut dummy;
    }
    if y.is_null() {
        y = &mut dummy;
    }
    if comp.is_null() {
        comp = &mut dummy;
    }
    stbi__rewind(s);
    p = stbi__get8(s) as libc::c_char;
    t = stbi__get8(s) as libc::c_char;
    if p as libc::c_int != 'P' as i32
        || t as libc::c_int != '5' as i32 && t as libc::c_int != '6' as i32
    {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    *comp = if t as libc::c_int == '6' as i32 {
        3 as libc::c_int
    } else {
        1 as libc::c_int
    };
    c = stbi__get8(s) as libc::c_char;
    stbi__pnm_skip_whitespace(s, &mut c);
    *x = stbi__pnm_getinteger(s, &mut c);
    stbi__pnm_skip_whitespace(s, &mut c);
    *y = stbi__pnm_getinteger(s, &mut c);
    stbi__pnm_skip_whitespace(s, &mut c);
    maxv = stbi__pnm_getinteger(s, &mut c);
    if maxv > 65535 as libc::c_int {
        return stbi__err(b"max value > 65535\0" as *const u8 as *const libc::c_char)
    } else if maxv > 255 as libc::c_int {
        return 16 as libc::c_int
    } else {
        return 8 as libc::c_int
    };
}
unsafe extern "C" fn stbi__pnm_getinteger(
    mut s: *mut stbi__context,
    mut c: *mut libc::c_char,
) -> libc::c_int {
    let mut value: libc::c_int = 0 as libc::c_int;
    while stbi__at_eof(s) == 0 && stbi__pnm_isdigit(*c) != 0 {
        value = value * 10 as libc::c_int + (*c as libc::c_int - '0' as i32);
        *c = stbi__get8(s) as libc::c_char;
    }
    return value;
}
unsafe extern "C" fn stbi__pnm_isdigit(mut c: libc::c_char) -> libc::c_int {
    return (c as libc::c_int >= '0' as i32 && c as libc::c_int <= '9' as i32)
        as libc::c_int;
}
unsafe extern "C" fn stbi__pnm_skip_whitespace(
    mut s: *mut stbi__context,
    mut c: *mut libc::c_char,
) {
    loop {
        while stbi__at_eof(s) == 0 && stbi__pnm_isspace(*c) != 0 {
            *c = stbi__get8(s) as libc::c_char;
        }
        if stbi__at_eof(s) != 0 || *c as libc::c_int != '#' as i32 {
            break;
        }
        while stbi__at_eof(s) == 0 && *c as libc::c_int != '\n' as i32
            && *c as libc::c_int != '\r' as i32
        {
            *c = stbi__get8(s) as libc::c_char;
        }
    };
}
unsafe extern "C" fn stbi__pnm_isspace(mut c: libc::c_char) -> libc::c_int {
    return (c as libc::c_int == ' ' as i32 || c as libc::c_int == '\t' as i32
        || c as libc::c_int == '\n' as i32 || c as libc::c_int == '\u{b}' as i32
        || c as libc::c_int == '\u{c}' as i32 || c as libc::c_int == '\r' as i32)
        as libc::c_int;
}
unsafe extern "C" fn stbi__pnm_test(mut s: *mut stbi__context) -> libc::c_int {
    let mut p: libc::c_char = 0;
    let mut t: libc::c_char = 0;
    p = stbi__get8(s) as libc::c_char;
    t = stbi__get8(s) as libc::c_char;
    if p as libc::c_int != 'P' as i32
        || t as libc::c_int != '5' as i32 && t as libc::c_int != '6' as i32
    {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__jpeg_load(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
    mut ri: *mut stbi__result_info,
) -> *mut libc::c_void {
    let mut result: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
    let mut j: *mut stbi__jpeg = stbi__malloc(
        ::core::mem::size_of::<stbi__jpeg>() as libc::c_ulong,
    ) as *mut stbi__jpeg;
    if j.is_null() {
        return (if stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    (*j).s = s;
    stbi__setup_jpeg(j);
    result = load_jpeg_image(j, x, y, comp, req_comp);
    free(j as *mut libc::c_void);
    return result as *mut libc::c_void;
}
unsafe extern "C" fn load_jpeg_image(
    mut z: *mut stbi__jpeg,
    mut out_x: *mut libc::c_int,
    mut out_y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
) -> *mut stbi_uc {
    let mut n: libc::c_int = 0;
    let mut decode_n: libc::c_int = 0;
    let mut is_rgb: libc::c_int = 0;
    (*(*z).s).img_n = 0 as libc::c_int;
    if req_comp < 0 as libc::c_int || req_comp > 4 as libc::c_int {
        return (if stbi__err(b"bad req_comp\0" as *const u8 as *const libc::c_char) != 0
        {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar;
    }
    if stbi__decode_jpeg_image(z) == 0 {
        stbi__cleanup_jpeg(z);
        return 0 as *mut stbi_uc;
    }
    n = if req_comp != 0 {
        req_comp
    } else if (*(*z).s).img_n >= 3 as libc::c_int {
        3 as libc::c_int
    } else {
        1 as libc::c_int
    };
    is_rgb = ((*(*z).s).img_n == 3 as libc::c_int
        && ((*z).rgb == 3 as libc::c_int
            || (*z).app14_color_transform == 0 as libc::c_int && (*z).jfif == 0))
        as libc::c_int;
    if (*(*z).s).img_n == 3 as libc::c_int && n < 3 as libc::c_int && is_rgb == 0 {
        decode_n = 1 as libc::c_int;
    } else {
        decode_n = (*(*z).s).img_n;
    }
    if decode_n <= 0 as libc::c_int {
        stbi__cleanup_jpeg(z);
        return 0 as *mut stbi_uc;
    }
    let mut k: libc::c_int = 0;
    let mut i: libc::c_uint = 0;
    let mut j: libc::c_uint = 0;
    let mut output: *mut stbi_uc = 0 as *mut stbi_uc;
    let mut coutput: [*mut stbi_uc; 4] = [
        0 as *mut stbi_uc,
        0 as *mut stbi_uc,
        0 as *mut stbi_uc,
        0 as *mut stbi_uc,
    ];
    let mut res_comp: [stbi__resample; 4] = [stbi__resample {
        resample: None,
        line0: 0 as *mut stbi_uc,
        line1: 0 as *mut stbi_uc,
        hs: 0,
        vs: 0,
        w_lores: 0,
        ystep: 0,
        ypos: 0,
    }; 4];
    k = 0 as libc::c_int;
    while k < decode_n {
        let mut r: *mut stbi__resample = &mut *res_comp.as_mut_ptr().offset(k as isize)
            as *mut stbi__resample;
        (*z)
            .img_comp[k as usize]
            .linebuf = stbi__malloc(
            ((*(*z).s).img_x).wrapping_add(3 as libc::c_int as libc::c_uint) as size_t,
        ) as *mut stbi_uc;
        if ((*z).img_comp[k as usize].linebuf).is_null() {
            stbi__cleanup_jpeg(z);
            return (if stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char) != 0
            {
                0 as *mut libc::c_void
            } else {
                0 as *mut libc::c_void
            }) as size_t as *mut libc::c_uchar;
        }
        (*r).hs = (*z).img_h_max / (*z).img_comp[k as usize].h;
        (*r).vs = (*z).img_v_max / (*z).img_comp[k as usize].v;
        (*r).ystep = (*r).vs >> 1 as libc::c_int;
        (*r)
            .w_lores = ((*(*z).s).img_x)
            .wrapping_add((*r).hs as libc::c_uint)
            .wrapping_sub(1 as libc::c_int as libc::c_uint)
            .wrapping_div((*r).hs as libc::c_uint) as libc::c_int;
        (*r).ypos = 0 as libc::c_int;
        (*r).line1 = (*z).img_comp[k as usize].data;
        (*r).line0 = (*r).line1;
        if (*r).hs == 1 as libc::c_int && (*r).vs == 1 as libc::c_int {
            (*r)
                .resample = Some(
                resample_row_1
                    as unsafe extern "C" fn(
                        *mut stbi_uc,
                        *mut stbi_uc,
                        *mut stbi_uc,
                        libc::c_int,
                        libc::c_int,
                    ) -> *mut stbi_uc,
            );
        } else if (*r).hs == 1 as libc::c_int && (*r).vs == 2 as libc::c_int {
            (*r)
                .resample = Some(
                stbi__resample_row_v_2
                    as unsafe extern "C" fn(
                        *mut stbi_uc,
                        *mut stbi_uc,
                        *mut stbi_uc,
                        libc::c_int,
                        libc::c_int,
                    ) -> *mut stbi_uc,
            );
        } else if (*r).hs == 2 as libc::c_int && (*r).vs == 1 as libc::c_int {
            (*r)
                .resample = Some(
                stbi__resample_row_h_2
                    as unsafe extern "C" fn(
                        *mut stbi_uc,
                        *mut stbi_uc,
                        *mut stbi_uc,
                        libc::c_int,
                        libc::c_int,
                    ) -> *mut stbi_uc,
            );
        } else if (*r).hs == 2 as libc::c_int && (*r).vs == 2 as libc::c_int {
            (*r).resample = (*z).resample_row_hv_2_kernel;
        } else {
            (*r)
                .resample = Some(
                stbi__resample_row_generic
                    as unsafe extern "C" fn(
                        *mut stbi_uc,
                        *mut stbi_uc,
                        *mut stbi_uc,
                        libc::c_int,
                        libc::c_int,
                    ) -> *mut stbi_uc,
            );
        }
        k += 1;
    }
    output = stbi__malloc_mad3(
        n,
        (*(*z).s).img_x as libc::c_int,
        (*(*z).s).img_y as libc::c_int,
        1 as libc::c_int,
    ) as *mut stbi_uc;
    if output.is_null() {
        stbi__cleanup_jpeg(z);
        return (if stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar;
    }
    j = 0 as libc::c_int as libc::c_uint;
    while j < (*(*z).s).img_y {
        let mut out: *mut stbi_uc = output
            .offset(
                (n as libc::c_uint).wrapping_mul((*(*z).s).img_x).wrapping_mul(j)
                    as isize,
            );
        k = 0 as libc::c_int;
        while k < decode_n {
            let mut r_0: *mut stbi__resample = &mut *res_comp
                .as_mut_ptr()
                .offset(k as isize) as *mut stbi__resample;
            let mut y_bot: libc::c_int = ((*r_0).ystep >= (*r_0).vs >> 1 as libc::c_int)
                as libc::c_int;
            coutput[k
                as usize] = ((*r_0).resample)
                .expect(
                    "non-null function pointer",
                )(
                (*z).img_comp[k as usize].linebuf,
                if y_bot != 0 { (*r_0).line1 } else { (*r_0).line0 },
                if y_bot != 0 { (*r_0).line0 } else { (*r_0).line1 },
                (*r_0).w_lores,
                (*r_0).hs,
            );
            (*r_0).ystep += 1;
            if (*r_0).ystep >= (*r_0).vs {
                (*r_0).ystep = 0 as libc::c_int;
                (*r_0).line0 = (*r_0).line1;
                (*r_0).ypos += 1;
                if (*r_0).ypos < (*z).img_comp[k as usize].y {
                    (*r_0)
                        .line1 = ((*r_0).line1)
                        .offset((*z).img_comp[k as usize].w2 as isize);
                }
            }
            k += 1;
        }
        if n >= 3 as libc::c_int {
            let mut y: *mut stbi_uc = coutput[0 as libc::c_int as usize];
            if (*(*z).s).img_n == 3 as libc::c_int {
                if is_rgb != 0 {
                    i = 0 as libc::c_int as libc::c_uint;
                    while i < (*(*z).s).img_x {
                        *out.offset(0 as libc::c_int as isize) = *y.offset(i as isize);
                        *out
                            .offset(
                                1 as libc::c_int as isize,
                            ) = *(coutput[1 as libc::c_int as usize]).offset(i as isize);
                        *out
                            .offset(
                                2 as libc::c_int as isize,
                            ) = *(coutput[2 as libc::c_int as usize]).offset(i as isize);
                        *out
                            .offset(
                                3 as libc::c_int as isize,
                            ) = 255 as libc::c_int as stbi_uc;
                        out = out.offset(n as isize);
                        i = i.wrapping_add(1);
                    }
                } else {
                    ((*z).YCbCr_to_RGB_kernel)
                        .expect(
                            "non-null function pointer",
                        )(
                        out,
                        y,
                        coutput[1 as libc::c_int as usize],
                        coutput[2 as libc::c_int as usize],
                        (*(*z).s).img_x as libc::c_int,
                        n,
                    );
                }
            } else if (*(*z).s).img_n == 4 as libc::c_int {
                if (*z).app14_color_transform == 0 as libc::c_int {
                    i = 0 as libc::c_int as libc::c_uint;
                    while i < (*(*z).s).img_x {
                        let mut m: stbi_uc = *(coutput[3 as libc::c_int as usize])
                            .offset(i as isize);
                        *out
                            .offset(
                                0 as libc::c_int as isize,
                            ) = stbi__blinn_8x8(
                            *(coutput[0 as libc::c_int as usize]).offset(i as isize),
                            m,
                        );
                        *out
                            .offset(
                                1 as libc::c_int as isize,
                            ) = stbi__blinn_8x8(
                            *(coutput[1 as libc::c_int as usize]).offset(i as isize),
                            m,
                        );
                        *out
                            .offset(
                                2 as libc::c_int as isize,
                            ) = stbi__blinn_8x8(
                            *(coutput[2 as libc::c_int as usize]).offset(i as isize),
                            m,
                        );
                        *out
                            .offset(
                                3 as libc::c_int as isize,
                            ) = 255 as libc::c_int as stbi_uc;
                        out = out.offset(n as isize);
                        i = i.wrapping_add(1);
                    }
                } else if (*z).app14_color_transform == 2 as libc::c_int {
                    ((*z).YCbCr_to_RGB_kernel)
                        .expect(
                            "non-null function pointer",
                        )(
                        out,
                        y,
                        coutput[1 as libc::c_int as usize],
                        coutput[2 as libc::c_int as usize],
                        (*(*z).s).img_x as libc::c_int,
                        n,
                    );
                    i = 0 as libc::c_int as libc::c_uint;
                    while i < (*(*z).s).img_x {
                        let mut m_0: stbi_uc = *(coutput[3 as libc::c_int as usize])
                            .offset(i as isize);
                        *out
                            .offset(
                                0 as libc::c_int as isize,
                            ) = stbi__blinn_8x8(
                            (255 as libc::c_int
                                - *out.offset(0 as libc::c_int as isize) as libc::c_int)
                                as stbi_uc,
                            m_0,
                        );
                        *out
                            .offset(
                                1 as libc::c_int as isize,
                            ) = stbi__blinn_8x8(
                            (255 as libc::c_int
                                - *out.offset(1 as libc::c_int as isize) as libc::c_int)
                                as stbi_uc,
                            m_0,
                        );
                        *out
                            .offset(
                                2 as libc::c_int as isize,
                            ) = stbi__blinn_8x8(
                            (255 as libc::c_int
                                - *out.offset(2 as libc::c_int as isize) as libc::c_int)
                                as stbi_uc,
                            m_0,
                        );
                        out = out.offset(n as isize);
                        i = i.wrapping_add(1);
                    }
                } else {
                    ((*z).YCbCr_to_RGB_kernel)
                        .expect(
                            "non-null function pointer",
                        )(
                        out,
                        y,
                        coutput[1 as libc::c_int as usize],
                        coutput[2 as libc::c_int as usize],
                        (*(*z).s).img_x as libc::c_int,
                        n,
                    );
                }
            } else {
                i = 0 as libc::c_int as libc::c_uint;
                while i < (*(*z).s).img_x {
                    let ref mut fresh15 = *out.offset(2 as libc::c_int as isize);
                    *fresh15 = *y.offset(i as isize);
                    let ref mut fresh16 = *out.offset(1 as libc::c_int as isize);
                    *fresh16 = *fresh15;
                    *out.offset(0 as libc::c_int as isize) = *fresh16;
                    *out
                        .offset(
                            3 as libc::c_int as isize,
                        ) = 255 as libc::c_int as stbi_uc;
                    out = out.offset(n as isize);
                    i = i.wrapping_add(1);
                }
            }
        } else if is_rgb != 0 {
            if n == 1 as libc::c_int {
                i = 0 as libc::c_int as libc::c_uint;
                while i < (*(*z).s).img_x {
                    let fresh17 = out;
                    out = out.offset(1);
                    *fresh17 = stbi__compute_y(
                        *(coutput[0 as libc::c_int as usize]).offset(i as isize)
                            as libc::c_int,
                        *(coutput[1 as libc::c_int as usize]).offset(i as isize)
                            as libc::c_int,
                        *(coutput[2 as libc::c_int as usize]).offset(i as isize)
                            as libc::c_int,
                    );
                    i = i.wrapping_add(1);
                }
            } else {
                i = 0 as libc::c_int as libc::c_uint;
                while i < (*(*z).s).img_x {
                    *out
                        .offset(
                            0 as libc::c_int as isize,
                        ) = stbi__compute_y(
                        *(coutput[0 as libc::c_int as usize]).offset(i as isize)
                            as libc::c_int,
                        *(coutput[1 as libc::c_int as usize]).offset(i as isize)
                            as libc::c_int,
                        *(coutput[2 as libc::c_int as usize]).offset(i as isize)
                            as libc::c_int,
                    );
                    *out
                        .offset(
                            1 as libc::c_int as isize,
                        ) = 255 as libc::c_int as stbi_uc;
                    i = i.wrapping_add(1);
                    out = out.offset(2 as libc::c_int as isize);
                }
            }
        } else if (*(*z).s).img_n == 4 as libc::c_int
            && (*z).app14_color_transform == 0 as libc::c_int
        {
            i = 0 as libc::c_int as libc::c_uint;
            while i < (*(*z).s).img_x {
                let mut m_1: stbi_uc = *(coutput[3 as libc::c_int as usize])
                    .offset(i as isize);
                let mut r_1: stbi_uc = stbi__blinn_8x8(
                    *(coutput[0 as libc::c_int as usize]).offset(i as isize),
                    m_1,
                );
                let mut g: stbi_uc = stbi__blinn_8x8(
                    *(coutput[1 as libc::c_int as usize]).offset(i as isize),
                    m_1,
                );
                let mut b: stbi_uc = stbi__blinn_8x8(
                    *(coutput[2 as libc::c_int as usize]).offset(i as isize),
                    m_1,
                );
                *out
                    .offset(
                        0 as libc::c_int as isize,
                    ) = stbi__compute_y(
                    r_1 as libc::c_int,
                    g as libc::c_int,
                    b as libc::c_int,
                );
                *out.offset(1 as libc::c_int as isize) = 255 as libc::c_int as stbi_uc;
                out = out.offset(n as isize);
                i = i.wrapping_add(1);
            }
        } else if (*(*z).s).img_n == 4 as libc::c_int
            && (*z).app14_color_transform == 2 as libc::c_int
        {
            i = 0 as libc::c_int as libc::c_uint;
            while i < (*(*z).s).img_x {
                *out
                    .offset(
                        0 as libc::c_int as isize,
                    ) = stbi__blinn_8x8(
                    (255 as libc::c_int
                        - *(coutput[0 as libc::c_int as usize]).offset(i as isize)
                            as libc::c_int) as stbi_uc,
                    *(coutput[3 as libc::c_int as usize]).offset(i as isize),
                );
                *out.offset(1 as libc::c_int as isize) = 255 as libc::c_int as stbi_uc;
                out = out.offset(n as isize);
                i = i.wrapping_add(1);
            }
        } else {
            let mut y_0: *mut stbi_uc = coutput[0 as libc::c_int as usize];
            if n == 1 as libc::c_int {
                i = 0 as libc::c_int as libc::c_uint;
                while i < (*(*z).s).img_x {
                    *out.offset(i as isize) = *y_0.offset(i as isize);
                    i = i.wrapping_add(1);
                }
            } else {
                i = 0 as libc::c_int as libc::c_uint;
                while i < (*(*z).s).img_x {
                    let fresh18 = out;
                    out = out.offset(1);
                    *fresh18 = *y_0.offset(i as isize);
                    let fresh19 = out;
                    out = out.offset(1);
                    *fresh19 = 255 as libc::c_int as stbi_uc;
                    i = i.wrapping_add(1);
                }
            }
        }
        j = j.wrapping_add(1);
    }
    stbi__cleanup_jpeg(z);
    *out_x = (*(*z).s).img_x as libc::c_int;
    *out_y = (*(*z).s).img_y as libc::c_int;
    if !comp.is_null() {
        *comp = if (*(*z).s).img_n >= 3 as libc::c_int {
            3 as libc::c_int
        } else {
            1 as libc::c_int
        };
    }
    return output;
}
unsafe extern "C" fn stbi__cleanup_jpeg(mut j: *mut stbi__jpeg) {
    stbi__free_jpeg_components(j, (*(*j).s).img_n, 0 as libc::c_int);
}
unsafe extern "C" fn stbi__free_jpeg_components(
    mut z: *mut stbi__jpeg,
    mut ncomp: libc::c_int,
    mut why: libc::c_int,
) -> libc::c_int {
    let mut i: libc::c_int = 0;
    i = 0 as libc::c_int;
    while i < ncomp {
        if !((*z).img_comp[i as usize].raw_data).is_null() {
            free((*z).img_comp[i as usize].raw_data);
            (*z).img_comp[i as usize].raw_data = 0 as *mut libc::c_void;
            (*z).img_comp[i as usize].data = 0 as *mut stbi_uc;
        }
        if !((*z).img_comp[i as usize].raw_coeff).is_null() {
            free((*z).img_comp[i as usize].raw_coeff);
            (*z).img_comp[i as usize].raw_coeff = 0 as *mut libc::c_void;
            (*z).img_comp[i as usize].coeff = 0 as *mut libc::c_short;
        }
        if !((*z).img_comp[i as usize].linebuf).is_null() {
            free((*z).img_comp[i as usize].linebuf as *mut libc::c_void);
            (*z).img_comp[i as usize].linebuf = 0 as *mut stbi_uc;
        }
        i += 1;
    }
    return why;
}
unsafe extern "C" fn stbi__blinn_8x8(mut x: stbi_uc, mut y: stbi_uc) -> stbi_uc {
    let mut t: libc::c_uint = (x as libc::c_int * y as libc::c_int + 128 as libc::c_int)
        as libc::c_uint;
    return (t.wrapping_add(t >> 8 as libc::c_int) >> 8 as libc::c_int) as stbi_uc;
}
unsafe extern "C" fn stbi__resample_row_generic(
    mut out: *mut stbi_uc,
    mut in_near: *mut stbi_uc,
    mut in_far: *mut stbi_uc,
    mut w: libc::c_int,
    mut hs: libc::c_int,
) -> *mut stbi_uc {
    let mut i: libc::c_int = 0;
    let mut j: libc::c_int = 0;
    i = 0 as libc::c_int;
    while i < w {
        j = 0 as libc::c_int;
        while j < hs {
            *out.offset((i * hs + j) as isize) = *in_near.offset(i as isize);
            j += 1;
        }
        i += 1;
    }
    return out;
}
unsafe extern "C" fn stbi__resample_row_h_2(
    mut out: *mut stbi_uc,
    mut in_near: *mut stbi_uc,
    mut in_far: *mut stbi_uc,
    mut w: libc::c_int,
    mut hs: libc::c_int,
) -> *mut stbi_uc {
    let mut i: libc::c_int = 0;
    let mut input: *mut stbi_uc = in_near;
    if w == 1 as libc::c_int {
        let ref mut fresh20 = *out.offset(1 as libc::c_int as isize);
        *fresh20 = *input.offset(0 as libc::c_int as isize);
        *out.offset(0 as libc::c_int as isize) = *fresh20;
        return out;
    }
    *out.offset(0 as libc::c_int as isize) = *input.offset(0 as libc::c_int as isize);
    *out
        .offset(
            1 as libc::c_int as isize,
        ) = (*input.offset(0 as libc::c_int as isize) as libc::c_int * 3 as libc::c_int
        + *input.offset(1 as libc::c_int as isize) as libc::c_int + 2 as libc::c_int
        >> 2 as libc::c_int) as stbi_uc;
    i = 1 as libc::c_int;
    while i < w - 1 as libc::c_int {
        let mut n: libc::c_int = 3 as libc::c_int
            * *input.offset(i as isize) as libc::c_int + 2 as libc::c_int;
        *out
            .offset(
                (i * 2 as libc::c_int + 0 as libc::c_int) as isize,
            ) = (n + *input.offset((i - 1 as libc::c_int) as isize) as libc::c_int
            >> 2 as libc::c_int) as stbi_uc;
        *out
            .offset(
                (i * 2 as libc::c_int + 1 as libc::c_int) as isize,
            ) = (n + *input.offset((i + 1 as libc::c_int) as isize) as libc::c_int
            >> 2 as libc::c_int) as stbi_uc;
        i += 1;
    }
    *out
        .offset(
            (i * 2 as libc::c_int + 0 as libc::c_int) as isize,
        ) = (*input.offset((w - 2 as libc::c_int) as isize) as libc::c_int
        * 3 as libc::c_int
        + *input.offset((w - 1 as libc::c_int) as isize) as libc::c_int
        + 2 as libc::c_int >> 2 as libc::c_int) as stbi_uc;
    *out
        .offset(
            (i * 2 as libc::c_int + 1 as libc::c_int) as isize,
        ) = *input.offset((w - 1 as libc::c_int) as isize);
    return out;
}
unsafe extern "C" fn stbi__resample_row_v_2(
    mut out: *mut stbi_uc,
    mut in_near: *mut stbi_uc,
    mut in_far: *mut stbi_uc,
    mut w: libc::c_int,
    mut hs: libc::c_int,
) -> *mut stbi_uc {
    let mut i: libc::c_int = 0;
    i = 0 as libc::c_int;
    while i < w {
        *out
            .offset(
                i as isize,
            ) = (3 as libc::c_int * *in_near.offset(i as isize) as libc::c_int
            + *in_far.offset(i as isize) as libc::c_int + 2 as libc::c_int
            >> 2 as libc::c_int) as stbi_uc;
        i += 1;
    }
    return out;
}
unsafe extern "C" fn resample_row_1(
    mut out: *mut stbi_uc,
    mut in_near: *mut stbi_uc,
    mut in_far: *mut stbi_uc,
    mut w: libc::c_int,
    mut hs: libc::c_int,
) -> *mut stbi_uc {
    return in_near;
}
unsafe extern "C" fn stbi__decode_jpeg_image(mut j: *mut stbi__jpeg) -> libc::c_int {
    let mut m: libc::c_int = 0;
    m = 0 as libc::c_int;
    while m < 4 as libc::c_int {
        (*j).img_comp[m as usize].raw_data = 0 as *mut libc::c_void;
        (*j).img_comp[m as usize].raw_coeff = 0 as *mut libc::c_void;
        m += 1;
    }
    (*j).restart_interval = 0 as libc::c_int;
    if stbi__decode_jpeg_header(j, STBI__SCAN_load as libc::c_int) == 0 {
        return 0 as libc::c_int;
    }
    m = stbi__get_marker(j) as libc::c_int;
    while !(m == 0xd9 as libc::c_int) {
        if m == 0xda as libc::c_int {
            if stbi__process_scan_header(j) == 0 {
                return 0 as libc::c_int;
            }
            if stbi__parse_entropy_coded_data(j) == 0 {
                return 0 as libc::c_int;
            }
            if (*j).marker as libc::c_int == 0xff as libc::c_int {
                while stbi__at_eof((*j).s) == 0 {
                    let mut x: libc::c_int = stbi__get8((*j).s) as libc::c_int;
                    if !(x == 255 as libc::c_int) {
                        continue;
                    }
                    (*j).marker = stbi__get8((*j).s);
                    break;
                }
            }
        } else if m == 0xdc as libc::c_int {
            let mut Ld: libc::c_int = stbi__get16be((*j).s);
            let mut NL: stbi__uint32 = stbi__get16be((*j).s) as stbi__uint32;
            if Ld != 4 as libc::c_int {
                return stbi__err(b"bad DNL len\0" as *const u8 as *const libc::c_char);
            }
            if NL != (*(*j).s).img_y {
                return stbi__err(
                    b"bad DNL height\0" as *const u8 as *const libc::c_char,
                );
            }
        } else if stbi__process_marker(j, m) == 0 {
            return 0 as libc::c_int
        }
        m = stbi__get_marker(j) as libc::c_int;
    }
    if (*j).progressive != 0 {
        stbi__jpeg_finish(j);
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__jpeg_finish(mut z: *mut stbi__jpeg) {
    if (*z).progressive != 0 {
        let mut i: libc::c_int = 0;
        let mut j: libc::c_int = 0;
        let mut n: libc::c_int = 0;
        n = 0 as libc::c_int;
        while n < (*(*z).s).img_n {
            let mut w: libc::c_int = (*z).img_comp[n as usize].x + 7 as libc::c_int
                >> 3 as libc::c_int;
            let mut h: libc::c_int = (*z).img_comp[n as usize].y + 7 as libc::c_int
                >> 3 as libc::c_int;
            j = 0 as libc::c_int;
            while j < h {
                i = 0 as libc::c_int;
                while i < w {
                    let mut data: *mut libc::c_short = ((*z).img_comp[n as usize].coeff)
                        .offset(
                            (64 as libc::c_int
                                * (i + j * (*z).img_comp[n as usize].coeff_w)) as isize,
                        );
                    stbi__jpeg_dequantize(
                        data,
                        ((*z).dequant[(*z).img_comp[n as usize].tq as usize])
                            .as_mut_ptr(),
                    );
                    ((*z).idct_block_kernel)
                        .expect(
                            "non-null function pointer",
                        )(
                        ((*z).img_comp[n as usize].data)
                            .offset(
                                ((*z).img_comp[n as usize].w2 * j * 8 as libc::c_int)
                                    as isize,
                            )
                            .offset((i * 8 as libc::c_int) as isize),
                        (*z).img_comp[n as usize].w2,
                        data,
                    );
                    i += 1;
                }
                j += 1;
            }
            n += 1;
        }
    }
}
unsafe extern "C" fn stbi__jpeg_dequantize(
    mut data: *mut libc::c_short,
    mut dequant: *mut stbi__uint16,
) {
    let mut i: libc::c_int = 0;
    i = 0 as libc::c_int;
    while i < 64 as libc::c_int {
        let ref mut fresh21 = *data.offset(i as isize);
        *fresh21 = (*fresh21 as libc::c_int * *dequant.offset(i as isize) as libc::c_int)
            as libc::c_short;
        i += 1;
    }
}
unsafe extern "C" fn stbi__get_marker(mut j: *mut stbi__jpeg) -> stbi_uc {
    let mut x: stbi_uc = 0;
    if (*j).marker as libc::c_int != 0xff as libc::c_int {
        x = (*j).marker;
        (*j).marker = 0xff as libc::c_int as libc::c_uchar;
        return x;
    }
    x = stbi__get8((*j).s);
    if x as libc::c_int != 0xff as libc::c_int {
        return 0xff as libc::c_int as stbi_uc;
    }
    while x as libc::c_int == 0xff as libc::c_int {
        x = stbi__get8((*j).s);
    }
    return x;
}
unsafe extern "C" fn stbi__process_marker(
    mut z: *mut stbi__jpeg,
    mut m: libc::c_int,
) -> libc::c_int {
    let mut L: libc::c_int = 0;
    match m {
        255 => return stbi__err(b"expected marker\0" as *const u8 as *const libc::c_char),
        221 => {
            if stbi__get16be((*z).s) != 4 as libc::c_int {
                return stbi__err(b"bad DRI len\0" as *const u8 as *const libc::c_char);
            }
            (*z).restart_interval = stbi__get16be((*z).s);
            return 1 as libc::c_int;
        }
        219 => {
            L = stbi__get16be((*z).s) - 2 as libc::c_int;
            while L > 0 as libc::c_int {
                let mut q: libc::c_int = stbi__get8((*z).s) as libc::c_int;
                let mut p: libc::c_int = q >> 4 as libc::c_int;
                let mut sixteen: libc::c_int = (p != 0 as libc::c_int) as libc::c_int;
                let mut t: libc::c_int = q & 15 as libc::c_int;
                let mut i: libc::c_int = 0;
                if p != 0 as libc::c_int && p != 1 as libc::c_int {
                    return stbi__err(
                        b"bad DQT type\0" as *const u8 as *const libc::c_char,
                    );
                }
                if t > 3 as libc::c_int {
                    return stbi__err(
                        b"bad DQT table\0" as *const u8 as *const libc::c_char,
                    );
                }
                i = 0 as libc::c_int;
                while i < 64 as libc::c_int {
                    (*z)
                        .dequant[t
                        as usize][stbi__jpeg_dezigzag[i as usize]
                        as usize] = (if sixteen != 0 {
                        stbi__get16be((*z).s)
                    } else {
                        stbi__get8((*z).s) as libc::c_int
                    }) as stbi__uint16;
                    i += 1;
                }
                L -= if sixteen != 0 { 129 as libc::c_int } else { 65 as libc::c_int };
            }
            return (L == 0 as libc::c_int) as libc::c_int;
        }
        196 => {
            L = stbi__get16be((*z).s) - 2 as libc::c_int;
            while L > 0 as libc::c_int {
                let mut v: *mut stbi_uc = 0 as *mut stbi_uc;
                let mut sizes: [libc::c_int; 16] = [0; 16];
                let mut i_0: libc::c_int = 0;
                let mut n: libc::c_int = 0 as libc::c_int;
                let mut q_0: libc::c_int = stbi__get8((*z).s) as libc::c_int;
                let mut tc: libc::c_int = q_0 >> 4 as libc::c_int;
                let mut th: libc::c_int = q_0 & 15 as libc::c_int;
                if tc > 1 as libc::c_int || th > 3 as libc::c_int {
                    return stbi__err(
                        b"bad DHT header\0" as *const u8 as *const libc::c_char,
                    );
                }
                i_0 = 0 as libc::c_int;
                while i_0 < 16 as libc::c_int {
                    sizes[i_0 as usize] = stbi__get8((*z).s) as libc::c_int;
                    n += sizes[i_0 as usize];
                    i_0 += 1;
                }
                L -= 17 as libc::c_int;
                if tc == 0 as libc::c_int {
                    if stbi__build_huffman(
                        ((*z).huff_dc).as_mut_ptr().offset(th as isize),
                        sizes.as_mut_ptr(),
                    ) == 0
                    {
                        return 0 as libc::c_int;
                    }
                    v = ((*z).huff_dc[th as usize].values).as_mut_ptr();
                } else {
                    if stbi__build_huffman(
                        ((*z).huff_ac).as_mut_ptr().offset(th as isize),
                        sizes.as_mut_ptr(),
                    ) == 0
                    {
                        return 0 as libc::c_int;
                    }
                    v = ((*z).huff_ac[th as usize].values).as_mut_ptr();
                }
                i_0 = 0 as libc::c_int;
                while i_0 < n {
                    *v.offset(i_0 as isize) = stbi__get8((*z).s);
                    i_0 += 1;
                }
                if tc != 0 as libc::c_int {
                    stbi__build_fast_ac(
                        ((*z).fast_ac[th as usize]).as_mut_ptr(),
                        ((*z).huff_ac).as_mut_ptr().offset(th as isize),
                    );
                }
                L -= n;
            }
            return (L == 0 as libc::c_int) as libc::c_int;
        }
        _ => {}
    }
    if m >= 0xe0 as libc::c_int && m <= 0xef as libc::c_int || m == 0xfe as libc::c_int {
        L = stbi__get16be((*z).s);
        if L < 2 as libc::c_int {
            if m == 0xfe as libc::c_int {
                return stbi__err(b"bad COM len\0" as *const u8 as *const libc::c_char)
            } else {
                return stbi__err(b"bad APP len\0" as *const u8 as *const libc::c_char)
            }
        }
        L -= 2 as libc::c_int;
        if m == 0xe0 as libc::c_int && L >= 5 as libc::c_int {
            static mut tag: [libc::c_uchar; 5] = [
                'J' as i32 as libc::c_uchar,
                'F' as i32 as libc::c_uchar,
                'I' as i32 as libc::c_uchar,
                'F' as i32 as libc::c_uchar,
                '\0' as i32 as libc::c_uchar,
            ];
            let mut ok: libc::c_int = 1 as libc::c_int;
            let mut i_1: libc::c_int = 0;
            i_1 = 0 as libc::c_int;
            while i_1 < 5 as libc::c_int {
                if stbi__get8((*z).s) as libc::c_int != tag[i_1 as usize] as libc::c_int
                {
                    ok = 0 as libc::c_int;
                }
                i_1 += 1;
            }
            L -= 5 as libc::c_int;
            if ok != 0 {
                (*z).jfif = 1 as libc::c_int;
            }
        } else if m == 0xee as libc::c_int && L >= 12 as libc::c_int {
            static mut tag_0: [libc::c_uchar; 6] = [
                'A' as i32 as libc::c_uchar,
                'd' as i32 as libc::c_uchar,
                'o' as i32 as libc::c_uchar,
                'b' as i32 as libc::c_uchar,
                'e' as i32 as libc::c_uchar,
                '\0' as i32 as libc::c_uchar,
            ];
            let mut ok_0: libc::c_int = 1 as libc::c_int;
            let mut i_2: libc::c_int = 0;
            i_2 = 0 as libc::c_int;
            while i_2 < 6 as libc::c_int {
                if stbi__get8((*z).s) as libc::c_int
                    != tag_0[i_2 as usize] as libc::c_int
                {
                    ok_0 = 0 as libc::c_int;
                }
                i_2 += 1;
            }
            L -= 6 as libc::c_int;
            if ok_0 != 0 {
                stbi__get8((*z).s);
                stbi__get16be((*z).s);
                stbi__get16be((*z).s);
                (*z).app14_color_transform = stbi__get8((*z).s) as libc::c_int;
                L -= 6 as libc::c_int;
            }
        }
        stbi__skip((*z).s, L);
        return 1 as libc::c_int;
    }
    return stbi__err(b"unknown marker\0" as *const u8 as *const libc::c_char);
}
unsafe extern "C" fn stbi__get16be(mut s: *mut stbi__context) -> libc::c_int {
    let mut z: libc::c_int = stbi__get8(s) as libc::c_int;
    return (z << 8 as libc::c_int) + stbi__get8(s) as libc::c_int;
}
unsafe extern "C" fn stbi__build_fast_ac(
    mut fast_ac: *mut stbi__int16,
    mut h: *mut stbi__huffman,
) {
    let mut i: libc::c_int = 0;
    i = 0 as libc::c_int;
    while i < (1 as libc::c_int) << 9 as libc::c_int {
        let mut fast: stbi_uc = (*h).fast[i as usize];
        *fast_ac.offset(i as isize) = 0 as libc::c_int as stbi__int16;
        if (fast as libc::c_int) < 255 as libc::c_int {
            let mut rs: libc::c_int = (*h).values[fast as usize] as libc::c_int;
            let mut run: libc::c_int = rs >> 4 as libc::c_int & 15 as libc::c_int;
            let mut magbits: libc::c_int = rs & 15 as libc::c_int;
            let mut len: libc::c_int = (*h).size[fast as usize] as libc::c_int;
            if magbits != 0 && len + magbits <= 9 as libc::c_int {
                let mut k: libc::c_int = (i << len
                    & ((1 as libc::c_int) << 9 as libc::c_int) - 1 as libc::c_int)
                    >> 9 as libc::c_int - magbits;
                let mut m: libc::c_int = (1 as libc::c_int)
                    << magbits - 1 as libc::c_int;
                if k < m {
                    k = (k as libc::c_uint)
                        .wrapping_add(
                            (!(0 as libc::c_uint) << magbits)
                                .wrapping_add(1 as libc::c_int as libc::c_uint),
                        ) as libc::c_int as libc::c_int;
                }
                if k >= -(128 as libc::c_int) && k <= 127 as libc::c_int {
                    *fast_ac
                        .offset(
                            i as isize,
                        ) = (k * 256 as libc::c_int + run * 16 as libc::c_int
                        + (len + magbits)) as stbi__int16;
                }
            }
        }
        i += 1;
    }
}
unsafe extern "C" fn stbi__build_huffman(
    mut h: *mut stbi__huffman,
    mut count: *mut libc::c_int,
) -> libc::c_int {
    let mut i: libc::c_int = 0;
    let mut j: libc::c_int = 0;
    let mut k: libc::c_int = 0 as libc::c_int;
    let mut code: libc::c_uint = 0;
    i = 0 as libc::c_int;
    while i < 16 as libc::c_int {
        j = 0 as libc::c_int;
        while j < *count.offset(i as isize) {
            let fresh22 = k;
            k = k + 1;
            (*h).size[fresh22 as usize] = (i + 1 as libc::c_int) as stbi_uc;
            j += 1;
        }
        i += 1;
    }
    (*h).size[k as usize] = 0 as libc::c_int as stbi_uc;
    code = 0 as libc::c_int as libc::c_uint;
    k = 0 as libc::c_int;
    j = 1 as libc::c_int;
    while j <= 16 as libc::c_int {
        (*h).delta[j as usize] = (k as libc::c_uint).wrapping_sub(code) as libc::c_int;
        if (*h).size[k as usize] as libc::c_int == j {
            while (*h).size[k as usize] as libc::c_int == j {
                let fresh23 = code;
                code = code.wrapping_add(1);
                let fresh24 = k;
                k = k + 1;
                (*h).code[fresh24 as usize] = fresh23 as stbi__uint16;
            }
            if code.wrapping_sub(1 as libc::c_int as libc::c_uint)
                >= (1 as libc::c_uint) << j
            {
                return stbi__err(
                    b"bad code lengths\0" as *const u8 as *const libc::c_char,
                );
            }
        }
        (*h).maxcode[j as usize] = code << 16 as libc::c_int - j;
        code <<= 1 as libc::c_int;
        j += 1;
    }
    (*h).maxcode[j as usize] = 0xffffffff as libc::c_uint;
    memset(
        ((*h).fast).as_mut_ptr() as *mut libc::c_void,
        255 as libc::c_int,
        ((1 as libc::c_int) << 9 as libc::c_int) as libc::c_ulong,
    );
    i = 0 as libc::c_int;
    while i < k {
        let mut s: libc::c_int = (*h).size[i as usize] as libc::c_int;
        if s <= 9 as libc::c_int {
            let mut c: libc::c_int = ((*h).code[i as usize] as libc::c_int)
                << 9 as libc::c_int - s;
            let mut m: libc::c_int = (1 as libc::c_int) << 9 as libc::c_int - s;
            j = 0 as libc::c_int;
            while j < m {
                (*h).fast[(c + j) as usize] = i as stbi_uc;
                j += 1;
            }
        }
        i += 1;
    }
    return 1 as libc::c_int;
}
static mut stbi__jpeg_dezigzag: [stbi_uc; 79] = [
    0 as libc::c_int as stbi_uc,
    1 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    16 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    2 as libc::c_int as stbi_uc,
    3 as libc::c_int as stbi_uc,
    10 as libc::c_int as stbi_uc,
    17 as libc::c_int as stbi_uc,
    24 as libc::c_int as stbi_uc,
    32 as libc::c_int as stbi_uc,
    25 as libc::c_int as stbi_uc,
    18 as libc::c_int as stbi_uc,
    11 as libc::c_int as stbi_uc,
    4 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    12 as libc::c_int as stbi_uc,
    19 as libc::c_int as stbi_uc,
    26 as libc::c_int as stbi_uc,
    33 as libc::c_int as stbi_uc,
    40 as libc::c_int as stbi_uc,
    48 as libc::c_int as stbi_uc,
    41 as libc::c_int as stbi_uc,
    34 as libc::c_int as stbi_uc,
    27 as libc::c_int as stbi_uc,
    20 as libc::c_int as stbi_uc,
    13 as libc::c_int as stbi_uc,
    6 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    14 as libc::c_int as stbi_uc,
    21 as libc::c_int as stbi_uc,
    28 as libc::c_int as stbi_uc,
    35 as libc::c_int as stbi_uc,
    42 as libc::c_int as stbi_uc,
    49 as libc::c_int as stbi_uc,
    56 as libc::c_int as stbi_uc,
    57 as libc::c_int as stbi_uc,
    50 as libc::c_int as stbi_uc,
    43 as libc::c_int as stbi_uc,
    36 as libc::c_int as stbi_uc,
    29 as libc::c_int as stbi_uc,
    22 as libc::c_int as stbi_uc,
    15 as libc::c_int as stbi_uc,
    23 as libc::c_int as stbi_uc,
    30 as libc::c_int as stbi_uc,
    37 as libc::c_int as stbi_uc,
    44 as libc::c_int as stbi_uc,
    51 as libc::c_int as stbi_uc,
    58 as libc::c_int as stbi_uc,
    59 as libc::c_int as stbi_uc,
    52 as libc::c_int as stbi_uc,
    45 as libc::c_int as stbi_uc,
    38 as libc::c_int as stbi_uc,
    31 as libc::c_int as stbi_uc,
    39 as libc::c_int as stbi_uc,
    46 as libc::c_int as stbi_uc,
    53 as libc::c_int as stbi_uc,
    60 as libc::c_int as stbi_uc,
    61 as libc::c_int as stbi_uc,
    54 as libc::c_int as stbi_uc,
    47 as libc::c_int as stbi_uc,
    55 as libc::c_int as stbi_uc,
    62 as libc::c_int as stbi_uc,
    63 as libc::c_int as stbi_uc,
    63 as libc::c_int as stbi_uc,
    63 as libc::c_int as stbi_uc,
    63 as libc::c_int as stbi_uc,
    63 as libc::c_int as stbi_uc,
    63 as libc::c_int as stbi_uc,
    63 as libc::c_int as stbi_uc,
    63 as libc::c_int as stbi_uc,
    63 as libc::c_int as stbi_uc,
    63 as libc::c_int as stbi_uc,
    63 as libc::c_int as stbi_uc,
    63 as libc::c_int as stbi_uc,
    63 as libc::c_int as stbi_uc,
    63 as libc::c_int as stbi_uc,
    63 as libc::c_int as stbi_uc,
    63 as libc::c_int as stbi_uc,
];
unsafe extern "C" fn stbi__parse_entropy_coded_data(
    mut z: *mut stbi__jpeg,
) -> libc::c_int {
    stbi__jpeg_reset(z);
    if (*z).progressive == 0 {
        if (*z).scan_n == 1 as libc::c_int {
            let mut i: libc::c_int = 0;
            let mut j: libc::c_int = 0;
            let mut data: [libc::c_short; 64] = [0; 64];
            let mut n: libc::c_int = (*z).order[0 as libc::c_int as usize];
            let mut w: libc::c_int = (*z).img_comp[n as usize].x + 7 as libc::c_int
                >> 3 as libc::c_int;
            let mut h: libc::c_int = (*z).img_comp[n as usize].y + 7 as libc::c_int
                >> 3 as libc::c_int;
            j = 0 as libc::c_int;
            while j < h {
                i = 0 as libc::c_int;
                while i < w {
                    let mut ha: libc::c_int = (*z).img_comp[n as usize].ha;
                    if stbi__jpeg_decode_block(
                        z,
                        data.as_mut_ptr(),
                        ((*z).huff_dc)
                            .as_mut_ptr()
                            .offset((*z).img_comp[n as usize].hd as isize),
                        ((*z).huff_ac).as_mut_ptr().offset(ha as isize),
                        ((*z).fast_ac[ha as usize]).as_mut_ptr(),
                        n,
                        ((*z).dequant[(*z).img_comp[n as usize].tq as usize])
                            .as_mut_ptr(),
                    ) == 0
                    {
                        return 0 as libc::c_int;
                    }
                    ((*z).idct_block_kernel)
                        .expect(
                            "non-null function pointer",
                        )(
                        ((*z).img_comp[n as usize].data)
                            .offset(
                                ((*z).img_comp[n as usize].w2 * j * 8 as libc::c_int)
                                    as isize,
                            )
                            .offset((i * 8 as libc::c_int) as isize),
                        (*z).img_comp[n as usize].w2,
                        data.as_mut_ptr(),
                    );
                    (*z).todo -= 1;
                    if (*z).todo <= 0 as libc::c_int {
                        if (*z).code_bits < 24 as libc::c_int {
                            stbi__grow_buffer_unsafe(z);
                        }
                        if !((*z).marker as libc::c_int >= 0xd0 as libc::c_int
                            && (*z).marker as libc::c_int <= 0xd7 as libc::c_int)
                        {
                            return 1 as libc::c_int;
                        }
                        stbi__jpeg_reset(z);
                    }
                    i += 1;
                }
                j += 1;
            }
            return 1 as libc::c_int;
        } else {
            let mut i_0: libc::c_int = 0;
            let mut j_0: libc::c_int = 0;
            let mut k: libc::c_int = 0;
            let mut x: libc::c_int = 0;
            let mut y: libc::c_int = 0;
            let mut data_0: [libc::c_short; 64] = [0; 64];
            j_0 = 0 as libc::c_int;
            while j_0 < (*z).img_mcu_y {
                i_0 = 0 as libc::c_int;
                while i_0 < (*z).img_mcu_x {
                    k = 0 as libc::c_int;
                    while k < (*z).scan_n {
                        let mut n_0: libc::c_int = (*z).order[k as usize];
                        y = 0 as libc::c_int;
                        while y < (*z).img_comp[n_0 as usize].v {
                            x = 0 as libc::c_int;
                            while x < (*z).img_comp[n_0 as usize].h {
                                let mut x2: libc::c_int = (i_0
                                    * (*z).img_comp[n_0 as usize].h + x) * 8 as libc::c_int;
                                let mut y2: libc::c_int = (j_0
                                    * (*z).img_comp[n_0 as usize].v + y) * 8 as libc::c_int;
                                let mut ha_0: libc::c_int = (*z).img_comp[n_0 as usize].ha;
                                if stbi__jpeg_decode_block(
                                    z,
                                    data_0.as_mut_ptr(),
                                    ((*z).huff_dc)
                                        .as_mut_ptr()
                                        .offset((*z).img_comp[n_0 as usize].hd as isize),
                                    ((*z).huff_ac).as_mut_ptr().offset(ha_0 as isize),
                                    ((*z).fast_ac[ha_0 as usize]).as_mut_ptr(),
                                    n_0,
                                    ((*z).dequant[(*z).img_comp[n_0 as usize].tq as usize])
                                        .as_mut_ptr(),
                                ) == 0
                                {
                                    return 0 as libc::c_int;
                                }
                                ((*z).idct_block_kernel)
                                    .expect(
                                        "non-null function pointer",
                                    )(
                                    ((*z).img_comp[n_0 as usize].data)
                                        .offset(((*z).img_comp[n_0 as usize].w2 * y2) as isize)
                                        .offset(x2 as isize),
                                    (*z).img_comp[n_0 as usize].w2,
                                    data_0.as_mut_ptr(),
                                );
                                x += 1;
                            }
                            y += 1;
                        }
                        k += 1;
                    }
                    (*z).todo -= 1;
                    if (*z).todo <= 0 as libc::c_int {
                        if (*z).code_bits < 24 as libc::c_int {
                            stbi__grow_buffer_unsafe(z);
                        }
                        if !((*z).marker as libc::c_int >= 0xd0 as libc::c_int
                            && (*z).marker as libc::c_int <= 0xd7 as libc::c_int)
                        {
                            return 1 as libc::c_int;
                        }
                        stbi__jpeg_reset(z);
                    }
                    i_0 += 1;
                }
                j_0 += 1;
            }
            return 1 as libc::c_int;
        }
    } else if (*z).scan_n == 1 as libc::c_int {
        let mut i_1: libc::c_int = 0;
        let mut j_1: libc::c_int = 0;
        let mut n_1: libc::c_int = (*z).order[0 as libc::c_int as usize];
        let mut w_0: libc::c_int = (*z).img_comp[n_1 as usize].x + 7 as libc::c_int
            >> 3 as libc::c_int;
        let mut h_0: libc::c_int = (*z).img_comp[n_1 as usize].y + 7 as libc::c_int
            >> 3 as libc::c_int;
        j_1 = 0 as libc::c_int;
        while j_1 < h_0 {
            i_1 = 0 as libc::c_int;
            while i_1 < w_0 {
                let mut data_1: *mut libc::c_short = ((*z).img_comp[n_1 as usize].coeff)
                    .offset(
                        (64 as libc::c_int
                            * (i_1 + j_1 * (*z).img_comp[n_1 as usize].coeff_w)) as isize,
                    );
                if (*z).spec_start == 0 as libc::c_int {
                    if stbi__jpeg_decode_block_prog_dc(
                        z,
                        data_1,
                        &mut *((*z).huff_dc)
                            .as_mut_ptr()
                            .offset(
                                (*((*z).img_comp).as_mut_ptr().offset(n_1 as isize)).hd
                                    as isize,
                            ),
                        n_1,
                    ) == 0
                    {
                        return 0 as libc::c_int;
                    }
                } else {
                    let mut ha_1: libc::c_int = (*z).img_comp[n_1 as usize].ha;
                    if stbi__jpeg_decode_block_prog_ac(
                        z,
                        data_1,
                        &mut *((*z).huff_ac).as_mut_ptr().offset(ha_1 as isize),
                        ((*z).fast_ac[ha_1 as usize]).as_mut_ptr(),
                    ) == 0
                    {
                        return 0 as libc::c_int;
                    }
                }
                (*z).todo -= 1;
                if (*z).todo <= 0 as libc::c_int {
                    if (*z).code_bits < 24 as libc::c_int {
                        stbi__grow_buffer_unsafe(z);
                    }
                    if !((*z).marker as libc::c_int >= 0xd0 as libc::c_int
                        && (*z).marker as libc::c_int <= 0xd7 as libc::c_int)
                    {
                        return 1 as libc::c_int;
                    }
                    stbi__jpeg_reset(z);
                }
                i_1 += 1;
            }
            j_1 += 1;
        }
        return 1 as libc::c_int;
    } else {
        let mut i_2: libc::c_int = 0;
        let mut j_2: libc::c_int = 0;
        let mut k_0: libc::c_int = 0;
        let mut x_0: libc::c_int = 0;
        let mut y_0: libc::c_int = 0;
        j_2 = 0 as libc::c_int;
        while j_2 < (*z).img_mcu_y {
            i_2 = 0 as libc::c_int;
            while i_2 < (*z).img_mcu_x {
                k_0 = 0 as libc::c_int;
                while k_0 < (*z).scan_n {
                    let mut n_2: libc::c_int = (*z).order[k_0 as usize];
                    y_0 = 0 as libc::c_int;
                    while y_0 < (*z).img_comp[n_2 as usize].v {
                        x_0 = 0 as libc::c_int;
                        while x_0 < (*z).img_comp[n_2 as usize].h {
                            let mut x2_0: libc::c_int = i_2
                                * (*z).img_comp[n_2 as usize].h + x_0;
                            let mut y2_0: libc::c_int = j_2
                                * (*z).img_comp[n_2 as usize].v + y_0;
                            let mut data_2: *mut libc::c_short = ((*z)
                                .img_comp[n_2 as usize]
                                .coeff)
                                .offset(
                                    (64 as libc::c_int
                                        * (x2_0 + y2_0 * (*z).img_comp[n_2 as usize].coeff_w))
                                        as isize,
                                );
                            if stbi__jpeg_decode_block_prog_dc(
                                z,
                                data_2,
                                &mut *((*z).huff_dc)
                                    .as_mut_ptr()
                                    .offset(
                                        (*((*z).img_comp).as_mut_ptr().offset(n_2 as isize)).hd
                                            as isize,
                                    ),
                                n_2,
                            ) == 0
                            {
                                return 0 as libc::c_int;
                            }
                            x_0 += 1;
                        }
                        y_0 += 1;
                    }
                    k_0 += 1;
                }
                (*z).todo -= 1;
                if (*z).todo <= 0 as libc::c_int {
                    if (*z).code_bits < 24 as libc::c_int {
                        stbi__grow_buffer_unsafe(z);
                    }
                    if !((*z).marker as libc::c_int >= 0xd0 as libc::c_int
                        && (*z).marker as libc::c_int <= 0xd7 as libc::c_int)
                    {
                        return 1 as libc::c_int;
                    }
                    stbi__jpeg_reset(z);
                }
                i_2 += 1;
            }
            j_2 += 1;
        }
        return 1 as libc::c_int;
    };
}
unsafe extern "C" fn stbi__jpeg_reset(mut j: *mut stbi__jpeg) {
    (*j).code_bits = 0 as libc::c_int;
    (*j).code_buffer = 0 as libc::c_int as stbi__uint32;
    (*j).nomore = 0 as libc::c_int;
    (*j).img_comp[3 as libc::c_int as usize].dc_pred = 0 as libc::c_int;
    (*j)
        .img_comp[2 as libc::c_int as usize]
        .dc_pred = (*j).img_comp[3 as libc::c_int as usize].dc_pred;
    (*j)
        .img_comp[1 as libc::c_int as usize]
        .dc_pred = (*j).img_comp[2 as libc::c_int as usize].dc_pred;
    (*j)
        .img_comp[0 as libc::c_int as usize]
        .dc_pred = (*j).img_comp[1 as libc::c_int as usize].dc_pred;
    (*j).marker = 0xff as libc::c_int as libc::c_uchar;
    (*j)
        .todo = if (*j).restart_interval != 0 {
        (*j).restart_interval
    } else {
        0x7fffffff as libc::c_int
    };
    (*j).eob_run = 0 as libc::c_int;
}
unsafe extern "C" fn stbi__grow_buffer_unsafe(mut j: *mut stbi__jpeg) {
    loop {
        let mut b: libc::c_uint = (if (*j).nomore != 0 {
            0 as libc::c_int
        } else {
            stbi__get8((*j).s) as libc::c_int
        }) as libc::c_uint;
        if b == 0xff as libc::c_int as libc::c_uint {
            let mut c: libc::c_int = stbi__get8((*j).s) as libc::c_int;
            while c == 0xff as libc::c_int {
                c = stbi__get8((*j).s) as libc::c_int;
            }
            if c != 0 as libc::c_int {
                (*j).marker = c as libc::c_uchar;
                (*j).nomore = 1 as libc::c_int;
                return;
            }
        }
        (*j).code_buffer |= b << 24 as libc::c_int - (*j).code_bits;
        (*j).code_bits += 8 as libc::c_int;
        if !((*j).code_bits <= 24 as libc::c_int) {
            break;
        }
    };
}
unsafe extern "C" fn stbi__jpeg_decode_block_prog_dc(
    mut j: *mut stbi__jpeg,
    mut data: *mut libc::c_short,
    mut hdc: *mut stbi__huffman,
    mut b: libc::c_int,
) -> libc::c_int {
    let mut diff: libc::c_int = 0;
    let mut dc: libc::c_int = 0;
    let mut t: libc::c_int = 0;
    if (*j).spec_end != 0 as libc::c_int {
        return stbi__err(b"can't merge dc and ac\0" as *const u8 as *const libc::c_char);
    }
    if (*j).code_bits < 16 as libc::c_int {
        stbi__grow_buffer_unsafe(j);
    }
    if (*j).succ_high == 0 as libc::c_int {
        memset(
            data as *mut libc::c_void,
            0 as libc::c_int,
            (64 as libc::c_int as libc::c_ulong)
                .wrapping_mul(::core::mem::size_of::<libc::c_short>() as libc::c_ulong),
        );
        t = stbi__jpeg_huff_decode(j, hdc);
        if t < 0 as libc::c_int || t > 15 as libc::c_int {
            return stbi__err(
                b"can't merge dc and ac\0" as *const u8 as *const libc::c_char,
            );
        }
        diff = if t != 0 { stbi__extend_receive(j, t) } else { 0 as libc::c_int };
        dc = (*j).img_comp[b as usize].dc_pred + diff;
        (*j).img_comp[b as usize].dc_pred = dc;
        *data
            .offset(
                0 as libc::c_int as isize,
            ) = (dc * ((1 as libc::c_int) << (*j).succ_low)) as libc::c_short;
    } else if stbi__jpeg_get_bit(j) != 0 {
        let ref mut fresh25 = *data.offset(0 as libc::c_int as isize);
        *fresh25 = (*fresh25 as libc::c_int
            + ((1 as libc::c_int) << (*j).succ_low) as libc::c_short as libc::c_int)
            as libc::c_short;
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__jpeg_get_bit(mut j: *mut stbi__jpeg) -> libc::c_int {
    let mut k: libc::c_uint = 0;
    if (*j).code_bits < 1 as libc::c_int {
        stbi__grow_buffer_unsafe(j);
    }
    k = (*j).code_buffer;
    (*j).code_buffer <<= 1 as libc::c_int;
    (*j).code_bits -= 1;
    return (k & 0x80000000 as libc::c_uint) as libc::c_int;
}
unsafe extern "C" fn stbi__extend_receive(
    mut j: *mut stbi__jpeg,
    mut n: libc::c_int,
) -> libc::c_int {
    let mut k: libc::c_uint = 0;
    let mut sgn: libc::c_int = 0;
    if (*j).code_bits < n {
        stbi__grow_buffer_unsafe(j);
    }
    sgn = ((*j).code_buffer >> 31 as libc::c_int) as libc::c_int;
    k = (*j).code_buffer << n | (*j).code_buffer >> (-n & 31 as libc::c_int);
    (*j).code_buffer = k & !stbi__bmask[n as usize];
    k &= stbi__bmask[n as usize];
    (*j).code_bits -= n;
    return k
        .wrapping_add((stbi__jbias[n as usize] & sgn - 1 as libc::c_int) as libc::c_uint)
        as libc::c_int;
}
static mut stbi__jbias: [libc::c_int; 16] = [
    0 as libc::c_int,
    -(1 as libc::c_int),
    -(3 as libc::c_int),
    -(7 as libc::c_int),
    -(15 as libc::c_int),
    -(31 as libc::c_int),
    -(63 as libc::c_int),
    -(127 as libc::c_int),
    -(255 as libc::c_int),
    -(511 as libc::c_int),
    -(1023 as libc::c_int),
    -(2047 as libc::c_int),
    -(4095 as libc::c_int),
    -(8191 as libc::c_int),
    -(16383 as libc::c_int),
    -(32767 as libc::c_int),
];
static mut stbi__bmask: [stbi__uint32; 17] = [
    0 as libc::c_int as stbi__uint32,
    1 as libc::c_int as stbi__uint32,
    3 as libc::c_int as stbi__uint32,
    7 as libc::c_int as stbi__uint32,
    15 as libc::c_int as stbi__uint32,
    31 as libc::c_int as stbi__uint32,
    63 as libc::c_int as stbi__uint32,
    127 as libc::c_int as stbi__uint32,
    255 as libc::c_int as stbi__uint32,
    511 as libc::c_int as stbi__uint32,
    1023 as libc::c_int as stbi__uint32,
    2047 as libc::c_int as stbi__uint32,
    4095 as libc::c_int as stbi__uint32,
    8191 as libc::c_int as stbi__uint32,
    16383 as libc::c_int as stbi__uint32,
    32767 as libc::c_int as stbi__uint32,
    65535 as libc::c_int as stbi__uint32,
];
unsafe extern "C" fn stbi__jpeg_huff_decode(
    mut j: *mut stbi__jpeg,
    mut h: *mut stbi__huffman,
) -> libc::c_int {
    let mut temp: libc::c_uint = 0;
    let mut c: libc::c_int = 0;
    let mut k: libc::c_int = 0;
    if (*j).code_bits < 16 as libc::c_int {
        stbi__grow_buffer_unsafe(j);
    }
    c = ((*j).code_buffer >> 32 as libc::c_int - 9 as libc::c_int
        & (((1 as libc::c_int) << 9 as libc::c_int) - 1 as libc::c_int) as libc::c_uint)
        as libc::c_int;
    k = (*h).fast[c as usize] as libc::c_int;
    if k < 255 as libc::c_int {
        let mut s: libc::c_int = (*h).size[k as usize] as libc::c_int;
        if s > (*j).code_bits {
            return -(1 as libc::c_int);
        }
        (*j).code_buffer <<= s;
        (*j).code_bits -= s;
        return (*h).values[k as usize] as libc::c_int;
    }
    temp = (*j).code_buffer >> 16 as libc::c_int;
    k = 9 as libc::c_int + 1 as libc::c_int;
    while !(temp < (*h).maxcode[k as usize]) {
        k += 1;
    }
    if k == 17 as libc::c_int {
        (*j).code_bits -= 16 as libc::c_int;
        return -(1 as libc::c_int);
    }
    if k > (*j).code_bits {
        return -(1 as libc::c_int);
    }
    c = ((*j).code_buffer >> 32 as libc::c_int - k & stbi__bmask[k as usize])
        .wrapping_add((*h).delta[k as usize] as libc::c_uint) as libc::c_int;
    (*j).code_bits -= k;
    (*j).code_buffer <<= k;
    return (*h).values[c as usize] as libc::c_int;
}
unsafe extern "C" fn stbi__jpeg_decode_block_prog_ac(
    mut j: *mut stbi__jpeg,
    mut data: *mut libc::c_short,
    mut hac: *mut stbi__huffman,
    mut fac: *mut stbi__int16,
) -> libc::c_int {
    let mut k: libc::c_int = 0;
    if (*j).spec_start == 0 as libc::c_int {
        return stbi__err(b"can't merge dc and ac\0" as *const u8 as *const libc::c_char);
    }
    if (*j).succ_high == 0 as libc::c_int {
        let mut shift: libc::c_int = (*j).succ_low;
        if (*j).eob_run != 0 {
            (*j).eob_run -= 1;
            return 1 as libc::c_int;
        }
        k = (*j).spec_start;
        loop {
            let mut zig: libc::c_uint = 0;
            let mut c: libc::c_int = 0;
            let mut r: libc::c_int = 0;
            let mut s: libc::c_int = 0;
            if (*j).code_bits < 16 as libc::c_int {
                stbi__grow_buffer_unsafe(j);
            }
            c = ((*j).code_buffer >> 32 as libc::c_int - 9 as libc::c_int
                & (((1 as libc::c_int) << 9 as libc::c_int) - 1 as libc::c_int)
                    as libc::c_uint) as libc::c_int;
            r = *fac.offset(c as isize) as libc::c_int;
            if r != 0 {
                k += r >> 4 as libc::c_int & 15 as libc::c_int;
                s = r & 15 as libc::c_int;
                (*j).code_buffer <<= s;
                (*j).code_bits -= s;
                let fresh26 = k;
                k = k + 1;
                zig = stbi__jpeg_dezigzag[fresh26 as usize] as libc::c_uint;
                *data
                    .offset(
                        zig as isize,
                    ) = ((r >> 8 as libc::c_int) * ((1 as libc::c_int) << shift))
                    as libc::c_short;
            } else {
                let mut rs: libc::c_int = stbi__jpeg_huff_decode(j, hac);
                if rs < 0 as libc::c_int {
                    return stbi__err(
                        b"bad huffman code\0" as *const u8 as *const libc::c_char,
                    );
                }
                s = rs & 15 as libc::c_int;
                r = rs >> 4 as libc::c_int;
                if s == 0 as libc::c_int {
                    if r < 15 as libc::c_int {
                        (*j).eob_run = (1 as libc::c_int) << r;
                        if r != 0 {
                            (*j).eob_run += stbi__jpeg_get_bits(j, r);
                        }
                        (*j).eob_run -= 1;
                        break;
                    } else {
                        k += 16 as libc::c_int;
                    }
                } else {
                    k += r;
                    let fresh27 = k;
                    k = k + 1;
                    zig = stbi__jpeg_dezigzag[fresh27 as usize] as libc::c_uint;
                    *data
                        .offset(
                            zig as isize,
                        ) = (stbi__extend_receive(j, s) * ((1 as libc::c_int) << shift))
                        as libc::c_short;
                }
            }
            if !(k <= (*j).spec_end) {
                break;
            }
        }
    } else {
        let mut bit: libc::c_short = ((1 as libc::c_int) << (*j).succ_low)
            as libc::c_short;
        if (*j).eob_run != 0 {
            (*j).eob_run -= 1;
            k = (*j).spec_start;
            while k <= (*j).spec_end {
                let mut p: *mut libc::c_short = &mut *data
                    .offset(*stbi__jpeg_dezigzag.as_ptr().offset(k as isize) as isize)
                    as *mut libc::c_short;
                if *p as libc::c_int != 0 as libc::c_int {
                    if stbi__jpeg_get_bit(j) != 0 {
                        if *p as libc::c_int & bit as libc::c_int == 0 as libc::c_int {
                            if *p as libc::c_int > 0 as libc::c_int {
                                *p = (*p as libc::c_int + bit as libc::c_int)
                                    as libc::c_short;
                            } else {
                                *p = (*p as libc::c_int - bit as libc::c_int)
                                    as libc::c_short;
                            }
                        }
                    }
                }
                k += 1;
            }
        } else {
            k = (*j).spec_start;
            loop {
                let mut r_0: libc::c_int = 0;
                let mut s_0: libc::c_int = 0;
                let mut rs_0: libc::c_int = stbi__jpeg_huff_decode(j, hac);
                if rs_0 < 0 as libc::c_int {
                    return stbi__err(
                        b"bad huffman code\0" as *const u8 as *const libc::c_char,
                    );
                }
                s_0 = rs_0 & 15 as libc::c_int;
                r_0 = rs_0 >> 4 as libc::c_int;
                if s_0 == 0 as libc::c_int {
                    if r_0 < 15 as libc::c_int {
                        (*j).eob_run = ((1 as libc::c_int) << r_0) - 1 as libc::c_int;
                        if r_0 != 0 {
                            (*j).eob_run += stbi__jpeg_get_bits(j, r_0);
                        }
                        r_0 = 64 as libc::c_int;
                    }
                } else {
                    if s_0 != 1 as libc::c_int {
                        return stbi__err(
                            b"bad huffman code\0" as *const u8 as *const libc::c_char,
                        );
                    }
                    if stbi__jpeg_get_bit(j) != 0 {
                        s_0 = bit as libc::c_int;
                    } else {
                        s_0 = -(bit as libc::c_int);
                    }
                }
                while k <= (*j).spec_end {
                    let fresh28 = k;
                    k = k + 1;
                    let mut p_0: *mut libc::c_short = &mut *data
                        .offset(
                            *stbi__jpeg_dezigzag.as_ptr().offset(fresh28 as isize)
                                as isize,
                        ) as *mut libc::c_short;
                    if *p_0 as libc::c_int != 0 as libc::c_int {
                        if stbi__jpeg_get_bit(j) != 0 {
                            if *p_0 as libc::c_int & bit as libc::c_int
                                == 0 as libc::c_int
                            {
                                if *p_0 as libc::c_int > 0 as libc::c_int {
                                    *p_0 = (*p_0 as libc::c_int + bit as libc::c_int)
                                        as libc::c_short;
                                } else {
                                    *p_0 = (*p_0 as libc::c_int - bit as libc::c_int)
                                        as libc::c_short;
                                }
                            }
                        }
                    } else if r_0 == 0 as libc::c_int {
                        *p_0 = s_0 as libc::c_short;
                        break;
                    } else {
                        r_0 -= 1;
                    }
                }
                if !(k <= (*j).spec_end) {
                    break;
                }
            }
        }
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__jpeg_get_bits(
    mut j: *mut stbi__jpeg,
    mut n: libc::c_int,
) -> libc::c_int {
    let mut k: libc::c_uint = 0;
    if (*j).code_bits < n {
        stbi__grow_buffer_unsafe(j);
    }
    k = (*j).code_buffer << n | (*j).code_buffer >> (-n & 31 as libc::c_int);
    (*j).code_buffer = k & !stbi__bmask[n as usize];
    k &= stbi__bmask[n as usize];
    (*j).code_bits -= n;
    return k as libc::c_int;
}
unsafe extern "C" fn stbi__jpeg_decode_block(
    mut j: *mut stbi__jpeg,
    mut data: *mut libc::c_short,
    mut hdc: *mut stbi__huffman,
    mut hac: *mut stbi__huffman,
    mut fac: *mut stbi__int16,
    mut b: libc::c_int,
    mut dequant: *mut stbi__uint16,
) -> libc::c_int {
    let mut diff: libc::c_int = 0;
    let mut dc: libc::c_int = 0;
    let mut k: libc::c_int = 0;
    let mut t: libc::c_int = 0;
    if (*j).code_bits < 16 as libc::c_int {
        stbi__grow_buffer_unsafe(j);
    }
    t = stbi__jpeg_huff_decode(j, hdc);
    if t < 0 as libc::c_int || t > 15 as libc::c_int {
        return stbi__err(b"bad huffman code\0" as *const u8 as *const libc::c_char);
    }
    memset(
        data as *mut libc::c_void,
        0 as libc::c_int,
        (64 as libc::c_int as libc::c_ulong)
            .wrapping_mul(::core::mem::size_of::<libc::c_short>() as libc::c_ulong),
    );
    diff = if t != 0 { stbi__extend_receive(j, t) } else { 0 as libc::c_int };
    dc = (*j).img_comp[b as usize].dc_pred + diff;
    (*j).img_comp[b as usize].dc_pred = dc;
    *data
        .offset(
            0 as libc::c_int as isize,
        ) = (dc * *dequant.offset(0 as libc::c_int as isize) as libc::c_int)
        as libc::c_short;
    k = 1 as libc::c_int;
    loop {
        let mut zig: libc::c_uint = 0;
        let mut c: libc::c_int = 0;
        let mut r: libc::c_int = 0;
        let mut s: libc::c_int = 0;
        if (*j).code_bits < 16 as libc::c_int {
            stbi__grow_buffer_unsafe(j);
        }
        c = ((*j).code_buffer >> 32 as libc::c_int - 9 as libc::c_int
            & (((1 as libc::c_int) << 9 as libc::c_int) - 1 as libc::c_int)
                as libc::c_uint) as libc::c_int;
        r = *fac.offset(c as isize) as libc::c_int;
        if r != 0 {
            k += r >> 4 as libc::c_int & 15 as libc::c_int;
            s = r & 15 as libc::c_int;
            (*j).code_buffer <<= s;
            (*j).code_bits -= s;
            let fresh29 = k;
            k = k + 1;
            zig = stbi__jpeg_dezigzag[fresh29 as usize] as libc::c_uint;
            *data
                .offset(
                    zig as isize,
                ) = ((r >> 8 as libc::c_int)
                * *dequant.offset(zig as isize) as libc::c_int) as libc::c_short;
        } else {
            let mut rs: libc::c_int = stbi__jpeg_huff_decode(j, hac);
            if rs < 0 as libc::c_int {
                return stbi__err(
                    b"bad huffman code\0" as *const u8 as *const libc::c_char,
                );
            }
            s = rs & 15 as libc::c_int;
            r = rs >> 4 as libc::c_int;
            if s == 0 as libc::c_int {
                if rs != 0xf0 as libc::c_int {
                    break;
                }
                k += 16 as libc::c_int;
            } else {
                k += r;
                let fresh30 = k;
                k = k + 1;
                zig = stbi__jpeg_dezigzag[fresh30 as usize] as libc::c_uint;
                *data
                    .offset(
                        zig as isize,
                    ) = (stbi__extend_receive(j, s)
                    * *dequant.offset(zig as isize) as libc::c_int) as libc::c_short;
            }
        }
        if !(k < 64 as libc::c_int) {
            break;
        }
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__process_scan_header(mut z: *mut stbi__jpeg) -> libc::c_int {
    let mut i: libc::c_int = 0;
    let mut Ls: libc::c_int = stbi__get16be((*z).s);
    (*z).scan_n = stbi__get8((*z).s) as libc::c_int;
    if (*z).scan_n < 1 as libc::c_int || (*z).scan_n > 4 as libc::c_int
        || (*z).scan_n > (*(*z).s).img_n
    {
        return stbi__err(
            b"bad SOS component count\0" as *const u8 as *const libc::c_char,
        );
    }
    if Ls != 6 as libc::c_int + 2 as libc::c_int * (*z).scan_n {
        return stbi__err(b"bad SOS len\0" as *const u8 as *const libc::c_char);
    }
    i = 0 as libc::c_int;
    while i < (*z).scan_n {
        let mut id: libc::c_int = stbi__get8((*z).s) as libc::c_int;
        let mut which: libc::c_int = 0;
        let mut q: libc::c_int = stbi__get8((*z).s) as libc::c_int;
        which = 0 as libc::c_int;
        while which < (*(*z).s).img_n {
            if (*z).img_comp[which as usize].id == id {
                break;
            }
            which += 1;
        }
        if which == (*(*z).s).img_n {
            return 0 as libc::c_int;
        }
        (*z).img_comp[which as usize].hd = q >> 4 as libc::c_int;
        if (*z).img_comp[which as usize].hd > 3 as libc::c_int {
            return stbi__err(b"bad DC huff\0" as *const u8 as *const libc::c_char);
        }
        (*z).img_comp[which as usize].ha = q & 15 as libc::c_int;
        if (*z).img_comp[which as usize].ha > 3 as libc::c_int {
            return stbi__err(b"bad AC huff\0" as *const u8 as *const libc::c_char);
        }
        (*z).order[i as usize] = which;
        i += 1;
    }
    let mut aa: libc::c_int = 0;
    (*z).spec_start = stbi__get8((*z).s) as libc::c_int;
    (*z).spec_end = stbi__get8((*z).s) as libc::c_int;
    aa = stbi__get8((*z).s) as libc::c_int;
    (*z).succ_high = aa >> 4 as libc::c_int;
    (*z).succ_low = aa & 15 as libc::c_int;
    if (*z).progressive != 0 {
        if (*z).spec_start > 63 as libc::c_int || (*z).spec_end > 63 as libc::c_int
            || (*z).spec_start > (*z).spec_end || (*z).succ_high > 13 as libc::c_int
            || (*z).succ_low > 13 as libc::c_int
        {
            return stbi__err(b"bad SOS\0" as *const u8 as *const libc::c_char);
        }
    } else {
        if (*z).spec_start != 0 as libc::c_int {
            return stbi__err(b"bad SOS\0" as *const u8 as *const libc::c_char);
        }
        if (*z).succ_high != 0 as libc::c_int || (*z).succ_low != 0 as libc::c_int {
            return stbi__err(b"bad SOS\0" as *const u8 as *const libc::c_char);
        }
        (*z).spec_end = 63 as libc::c_int;
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__decode_jpeg_header(
    mut z: *mut stbi__jpeg,
    mut scan: libc::c_int,
) -> libc::c_int {
    let mut m: libc::c_int = 0;
    (*z).jfif = 0 as libc::c_int;
    (*z).app14_color_transform = -(1 as libc::c_int);
    (*z).marker = 0xff as libc::c_int as libc::c_uchar;
    m = stbi__get_marker(z) as libc::c_int;
    if !(m == 0xd8 as libc::c_int) {
        return stbi__err(b"no SOI\0" as *const u8 as *const libc::c_char);
    }
    if scan == STBI__SCAN_type as libc::c_int {
        return 1 as libc::c_int;
    }
    m = stbi__get_marker(z) as libc::c_int;
    while !(m == 0xc0 as libc::c_int || m == 0xc1 as libc::c_int
        || m == 0xc2 as libc::c_int)
    {
        if stbi__process_marker(z, m) == 0 {
            return 0 as libc::c_int;
        }
        m = stbi__get_marker(z) as libc::c_int;
        while m == 0xff as libc::c_int {
            if stbi__at_eof((*z).s) != 0 {
                return stbi__err(b"no SOF\0" as *const u8 as *const libc::c_char);
            }
            m = stbi__get_marker(z) as libc::c_int;
        }
    }
    (*z).progressive = (m == 0xc2 as libc::c_int) as libc::c_int;
    if stbi__process_frame_header(z, scan) == 0 {
        return 0 as libc::c_int;
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__process_frame_header(
    mut z: *mut stbi__jpeg,
    mut scan: libc::c_int,
) -> libc::c_int {
    let mut s: *mut stbi__context = (*z).s;
    let mut Lf: libc::c_int = 0;
    let mut p: libc::c_int = 0;
    let mut i: libc::c_int = 0;
    let mut q: libc::c_int = 0;
    let mut h_max: libc::c_int = 1 as libc::c_int;
    let mut v_max: libc::c_int = 1 as libc::c_int;
    let mut c: libc::c_int = 0;
    Lf = stbi__get16be(s);
    if Lf < 11 as libc::c_int {
        return stbi__err(b"bad SOF len\0" as *const u8 as *const libc::c_char);
    }
    p = stbi__get8(s) as libc::c_int;
    if p != 8 as libc::c_int {
        return stbi__err(b"only 8-bit\0" as *const u8 as *const libc::c_char);
    }
    (*s).img_y = stbi__get16be(s) as stbi__uint32;
    if (*s).img_y == 0 as libc::c_int as libc::c_uint {
        return stbi__err(b"no header height\0" as *const u8 as *const libc::c_char);
    }
    (*s).img_x = stbi__get16be(s) as stbi__uint32;
    if (*s).img_x == 0 as libc::c_int as libc::c_uint {
        return stbi__err(b"0 width\0" as *const u8 as *const libc::c_char);
    }
    if (*s).img_y > ((1 as libc::c_int) << 24 as libc::c_int) as libc::c_uint {
        return stbi__err(b"too large\0" as *const u8 as *const libc::c_char);
    }
    if (*s).img_x > ((1 as libc::c_int) << 24 as libc::c_int) as libc::c_uint {
        return stbi__err(b"too large\0" as *const u8 as *const libc::c_char);
    }
    c = stbi__get8(s) as libc::c_int;
    if c != 3 as libc::c_int && c != 1 as libc::c_int && c != 4 as libc::c_int {
        return stbi__err(b"bad component count\0" as *const u8 as *const libc::c_char);
    }
    (*s).img_n = c;
    i = 0 as libc::c_int;
    while i < c {
        (*z).img_comp[i as usize].data = 0 as *mut stbi_uc;
        (*z).img_comp[i as usize].linebuf = 0 as *mut stbi_uc;
        i += 1;
    }
    if Lf != 8 as libc::c_int + 3 as libc::c_int * (*s).img_n {
        return stbi__err(b"bad SOF len\0" as *const u8 as *const libc::c_char);
    }
    (*z).rgb = 0 as libc::c_int;
    i = 0 as libc::c_int;
    while i < (*s).img_n {
        static mut rgb: [libc::c_uchar; 3] = [
            'R' as i32 as libc::c_uchar,
            'G' as i32 as libc::c_uchar,
            'B' as i32 as libc::c_uchar,
        ];
        (*z).img_comp[i as usize].id = stbi__get8(s) as libc::c_int;
        if (*s).img_n == 3 as libc::c_int
            && (*z).img_comp[i as usize].id == rgb[i as usize] as libc::c_int
        {
            (*z).rgb += 1;
        }
        q = stbi__get8(s) as libc::c_int;
        (*z).img_comp[i as usize].h = q >> 4 as libc::c_int;
        if (*z).img_comp[i as usize].h == 0
            || (*z).img_comp[i as usize].h > 4 as libc::c_int
        {
            return stbi__err(b"bad H\0" as *const u8 as *const libc::c_char);
        }
        (*z).img_comp[i as usize].v = q & 15 as libc::c_int;
        if (*z).img_comp[i as usize].v == 0
            || (*z).img_comp[i as usize].v > 4 as libc::c_int
        {
            return stbi__err(b"bad V\0" as *const u8 as *const libc::c_char);
        }
        (*z).img_comp[i as usize].tq = stbi__get8(s) as libc::c_int;
        if (*z).img_comp[i as usize].tq > 3 as libc::c_int {
            return stbi__err(b"bad TQ\0" as *const u8 as *const libc::c_char);
        }
        i += 1;
    }
    if scan != STBI__SCAN_load as libc::c_int {
        return 1 as libc::c_int;
    }
    if stbi__mad3sizes_valid(
        (*s).img_x as libc::c_int,
        (*s).img_y as libc::c_int,
        (*s).img_n,
        0 as libc::c_int,
    ) == 0
    {
        return stbi__err(b"too large\0" as *const u8 as *const libc::c_char);
    }
    i = 0 as libc::c_int;
    while i < (*s).img_n {
        if (*z).img_comp[i as usize].h > h_max {
            h_max = (*z).img_comp[i as usize].h;
        }
        if (*z).img_comp[i as usize].v > v_max {
            v_max = (*z).img_comp[i as usize].v;
        }
        i += 1;
    }
    i = 0 as libc::c_int;
    while i < (*s).img_n {
        if h_max % (*z).img_comp[i as usize].h != 0 as libc::c_int {
            return stbi__err(b"bad H\0" as *const u8 as *const libc::c_char);
        }
        if v_max % (*z).img_comp[i as usize].v != 0 as libc::c_int {
            return stbi__err(b"bad V\0" as *const u8 as *const libc::c_char);
        }
        i += 1;
    }
    (*z).img_h_max = h_max;
    (*z).img_v_max = v_max;
    (*z).img_mcu_w = h_max * 8 as libc::c_int;
    (*z).img_mcu_h = v_max * 8 as libc::c_int;
    (*z)
        .img_mcu_x = ((*s).img_x)
        .wrapping_add((*z).img_mcu_w as libc::c_uint)
        .wrapping_sub(1 as libc::c_int as libc::c_uint)
        .wrapping_div((*z).img_mcu_w as libc::c_uint) as libc::c_int;
    (*z)
        .img_mcu_y = ((*s).img_y)
        .wrapping_add((*z).img_mcu_h as libc::c_uint)
        .wrapping_sub(1 as libc::c_int as libc::c_uint)
        .wrapping_div((*z).img_mcu_h as libc::c_uint) as libc::c_int;
    i = 0 as libc::c_int;
    while i < (*s).img_n {
        (*z)
            .img_comp[i as usize]
            .x = ((*s).img_x)
            .wrapping_mul((*z).img_comp[i as usize].h as libc::c_uint)
            .wrapping_add(h_max as libc::c_uint)
            .wrapping_sub(1 as libc::c_int as libc::c_uint)
            .wrapping_div(h_max as libc::c_uint) as libc::c_int;
        (*z)
            .img_comp[i as usize]
            .y = ((*s).img_y)
            .wrapping_mul((*z).img_comp[i as usize].v as libc::c_uint)
            .wrapping_add(v_max as libc::c_uint)
            .wrapping_sub(1 as libc::c_int as libc::c_uint)
            .wrapping_div(v_max as libc::c_uint) as libc::c_int;
        (*z)
            .img_comp[i as usize]
            .w2 = (*z).img_mcu_x * (*z).img_comp[i as usize].h * 8 as libc::c_int;
        (*z)
            .img_comp[i as usize]
            .h2 = (*z).img_mcu_y * (*z).img_comp[i as usize].v * 8 as libc::c_int;
        (*z).img_comp[i as usize].coeff = 0 as *mut libc::c_short;
        (*z).img_comp[i as usize].raw_coeff = 0 as *mut libc::c_void;
        (*z).img_comp[i as usize].linebuf = 0 as *mut stbi_uc;
        (*z)
            .img_comp[i as usize]
            .raw_data = stbi__malloc_mad2(
            (*z).img_comp[i as usize].w2,
            (*z).img_comp[i as usize].h2,
            15 as libc::c_int,
        );
        if ((*z).img_comp[i as usize].raw_data).is_null() {
            return stbi__free_jpeg_components(
                z,
                i + 1 as libc::c_int,
                stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char),
            );
        }
        (*z)
            .img_comp[i as usize]
            .data = (((*z).img_comp[i as usize].raw_data as size_t)
            .wrapping_add(15 as libc::c_int as libc::c_ulong)
            & !(15 as libc::c_int) as libc::c_ulong) as *mut stbi_uc;
        if (*z).progressive != 0 {
            (*z)
                .img_comp[i as usize]
                .coeff_w = (*z).img_comp[i as usize].w2 / 8 as libc::c_int;
            (*z)
                .img_comp[i as usize]
                .coeff_h = (*z).img_comp[i as usize].h2 / 8 as libc::c_int;
            (*z)
                .img_comp[i as usize]
                .raw_coeff = stbi__malloc_mad3(
                (*z).img_comp[i as usize].w2,
                (*z).img_comp[i as usize].h2,
                ::core::mem::size_of::<libc::c_short>() as libc::c_ulong as libc::c_int,
                15 as libc::c_int,
            );
            if ((*z).img_comp[i as usize].raw_coeff).is_null() {
                return stbi__free_jpeg_components(
                    z,
                    i + 1 as libc::c_int,
                    stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char),
                );
            }
            (*z)
                .img_comp[i as usize]
                .coeff = (((*z).img_comp[i as usize].raw_coeff as size_t)
                .wrapping_add(15 as libc::c_int as libc::c_ulong)
                & !(15 as libc::c_int) as libc::c_ulong) as *mut libc::c_short;
        }
        i += 1;
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__setup_jpeg(mut j: *mut stbi__jpeg) {
    (*j)
        .idct_block_kernel = Some(
        stbi__idct_block
            as unsafe extern "C" fn(*mut stbi_uc, libc::c_int, *mut libc::c_short) -> (),
    );
    (*j)
        .YCbCr_to_RGB_kernel = Some(
        stbi__YCbCr_to_RGB_row
            as unsafe extern "C" fn(
                *mut stbi_uc,
                *const stbi_uc,
                *const stbi_uc,
                *const stbi_uc,
                libc::c_int,
                libc::c_int,
            ) -> (),
    );
    (*j)
        .resample_row_hv_2_kernel = Some(
        stbi__resample_row_hv_2
            as unsafe extern "C" fn(
                *mut stbi_uc,
                *mut stbi_uc,
                *mut stbi_uc,
                libc::c_int,
                libc::c_int,
            ) -> *mut stbi_uc,
    );
}
unsafe extern "C" fn stbi__resample_row_hv_2(
    mut out: *mut stbi_uc,
    mut in_near: *mut stbi_uc,
    mut in_far: *mut stbi_uc,
    mut w: libc::c_int,
    mut hs: libc::c_int,
) -> *mut stbi_uc {
    let mut i: libc::c_int = 0;
    let mut t0: libc::c_int = 0;
    let mut t1: libc::c_int = 0;
    if w == 1 as libc::c_int {
        let ref mut fresh31 = *out.offset(1 as libc::c_int as isize);
        *fresh31 = (3 as libc::c_int
            * *in_near.offset(0 as libc::c_int as isize) as libc::c_int
            + *in_far.offset(0 as libc::c_int as isize) as libc::c_int + 2 as libc::c_int
            >> 2 as libc::c_int) as stbi_uc;
        *out.offset(0 as libc::c_int as isize) = *fresh31;
        return out;
    }
    t1 = 3 as libc::c_int * *in_near.offset(0 as libc::c_int as isize) as libc::c_int
        + *in_far.offset(0 as libc::c_int as isize) as libc::c_int;
    *out
        .offset(
            0 as libc::c_int as isize,
        ) = (t1 + 2 as libc::c_int >> 2 as libc::c_int) as stbi_uc;
    i = 1 as libc::c_int;
    while i < w {
        t0 = t1;
        t1 = 3 as libc::c_int * *in_near.offset(i as isize) as libc::c_int
            + *in_far.offset(i as isize) as libc::c_int;
        *out
            .offset(
                (i * 2 as libc::c_int - 1 as libc::c_int) as isize,
            ) = (3 as libc::c_int * t0 + t1 + 8 as libc::c_int >> 4 as libc::c_int)
            as stbi_uc;
        *out
            .offset(
                (i * 2 as libc::c_int) as isize,
            ) = (3 as libc::c_int * t1 + t0 + 8 as libc::c_int >> 4 as libc::c_int)
            as stbi_uc;
        i += 1;
    }
    *out
        .offset(
            (w * 2 as libc::c_int - 1 as libc::c_int) as isize,
        ) = (t1 + 2 as libc::c_int >> 2 as libc::c_int) as stbi_uc;
    return out;
}
unsafe extern "C" fn stbi__YCbCr_to_RGB_row(
    mut out: *mut stbi_uc,
    mut y: *const stbi_uc,
    mut pcb: *const stbi_uc,
    mut pcr: *const stbi_uc,
    mut count: libc::c_int,
    mut step: libc::c_int,
) {
    let mut i: libc::c_int = 0;
    i = 0 as libc::c_int;
    while i < count {
        let mut y_fixed: libc::c_int = ((*y.offset(i as isize) as libc::c_int)
            << 20 as libc::c_int) + ((1 as libc::c_int) << 19 as libc::c_int);
        let mut r: libc::c_int = 0;
        let mut g: libc::c_int = 0;
        let mut b: libc::c_int = 0;
        let mut cr: libc::c_int = *pcr.offset(i as isize) as libc::c_int
            - 128 as libc::c_int;
        let mut cb: libc::c_int = *pcb.offset(i as isize) as libc::c_int
            - 128 as libc::c_int;
        r = y_fixed
            + cr
                * (((1.40200f32 * 4096.0f32 + 0.5f32) as libc::c_int)
                    << 8 as libc::c_int);
        g = ((y_fixed
            + cr
                * -(((0.71414f32 * 4096.0f32 + 0.5f32) as libc::c_int)
                    << 8 as libc::c_int)) as libc::c_uint)
            .wrapping_add(
                (cb
                    * -(((0.34414f32 * 4096.0f32 + 0.5f32) as libc::c_int)
                        << 8 as libc::c_int)) as libc::c_uint
                    & 0xffff0000 as libc::c_uint,
            ) as libc::c_int;
        b = y_fixed
            + cb
                * (((1.77200f32 * 4096.0f32 + 0.5f32) as libc::c_int)
                    << 8 as libc::c_int);
        r >>= 20 as libc::c_int;
        g >>= 20 as libc::c_int;
        b >>= 20 as libc::c_int;
        if r as libc::c_uint > 255 as libc::c_int as libc::c_uint {
            if r < 0 as libc::c_int {
                r = 0 as libc::c_int;
            } else {
                r = 255 as libc::c_int;
            }
        }
        if g as libc::c_uint > 255 as libc::c_int as libc::c_uint {
            if g < 0 as libc::c_int {
                g = 0 as libc::c_int;
            } else {
                g = 255 as libc::c_int;
            }
        }
        if b as libc::c_uint > 255 as libc::c_int as libc::c_uint {
            if b < 0 as libc::c_int {
                b = 0 as libc::c_int;
            } else {
                b = 255 as libc::c_int;
            }
        }
        *out.offset(0 as libc::c_int as isize) = r as stbi_uc;
        *out.offset(1 as libc::c_int as isize) = g as stbi_uc;
        *out.offset(2 as libc::c_int as isize) = b as stbi_uc;
        *out.offset(3 as libc::c_int as isize) = 255 as libc::c_int as stbi_uc;
        out = out.offset(step as isize);
        i += 1;
    }
}
unsafe extern "C" fn stbi__idct_block(
    mut out: *mut stbi_uc,
    mut out_stride: libc::c_int,
    mut data: *mut libc::c_short,
) {
    let mut i: libc::c_int = 0;
    let mut val: [libc::c_int; 64] = [0; 64];
    let mut v: *mut libc::c_int = val.as_mut_ptr();
    let mut o: *mut stbi_uc = 0 as *mut stbi_uc;
    let mut d: *mut libc::c_short = data;
    i = 0 as libc::c_int;
    while i < 8 as libc::c_int {
        if *d.offset(8 as libc::c_int as isize) as libc::c_int == 0 as libc::c_int
            && *d.offset(16 as libc::c_int as isize) as libc::c_int == 0 as libc::c_int
            && *d.offset(24 as libc::c_int as isize) as libc::c_int == 0 as libc::c_int
            && *d.offset(32 as libc::c_int as isize) as libc::c_int == 0 as libc::c_int
            && *d.offset(40 as libc::c_int as isize) as libc::c_int == 0 as libc::c_int
            && *d.offset(48 as libc::c_int as isize) as libc::c_int == 0 as libc::c_int
            && *d.offset(56 as libc::c_int as isize) as libc::c_int == 0 as libc::c_int
        {
            let mut dcterm: libc::c_int = *d.offset(0 as libc::c_int as isize)
                as libc::c_int * 4 as libc::c_int;
            let ref mut fresh32 = *v.offset(56 as libc::c_int as isize);
            *fresh32 = dcterm;
            let ref mut fresh33 = *v.offset(48 as libc::c_int as isize);
            *fresh33 = *fresh32;
            let ref mut fresh34 = *v.offset(40 as libc::c_int as isize);
            *fresh34 = *fresh33;
            let ref mut fresh35 = *v.offset(32 as libc::c_int as isize);
            *fresh35 = *fresh34;
            let ref mut fresh36 = *v.offset(24 as libc::c_int as isize);
            *fresh36 = *fresh35;
            let ref mut fresh37 = *v.offset(16 as libc::c_int as isize);
            *fresh37 = *fresh36;
            let ref mut fresh38 = *v.offset(8 as libc::c_int as isize);
            *fresh38 = *fresh37;
            *v.offset(0 as libc::c_int as isize) = *fresh38;
        } else {
            let mut t0: libc::c_int = 0;
            let mut t1: libc::c_int = 0;
            let mut t2: libc::c_int = 0;
            let mut t3: libc::c_int = 0;
            let mut p1: libc::c_int = 0;
            let mut p2: libc::c_int = 0;
            let mut p3: libc::c_int = 0;
            let mut p4: libc::c_int = 0;
            let mut p5: libc::c_int = 0;
            let mut x0: libc::c_int = 0;
            let mut x1: libc::c_int = 0;
            let mut x2: libc::c_int = 0;
            let mut x3: libc::c_int = 0;
            p2 = *d.offset(16 as libc::c_int as isize) as libc::c_int;
            p3 = *d.offset(48 as libc::c_int as isize) as libc::c_int;
            p1 = (p2 + p3)
                * ((0.5411961f32 * 4096 as libc::c_int as libc::c_float)
                    as libc::c_double + 0.5f64) as libc::c_int;
            t2 = p1
                + p3
                    * ((-1.847759065f32 * 4096 as libc::c_int as libc::c_float)
                        as libc::c_double + 0.5f64) as libc::c_int;
            t3 = p1
                + p2
                    * ((0.765366865f32 * 4096 as libc::c_int as libc::c_float)
                        as libc::c_double + 0.5f64) as libc::c_int;
            p2 = *d.offset(0 as libc::c_int as isize) as libc::c_int;
            p3 = *d.offset(32 as libc::c_int as isize) as libc::c_int;
            t0 = (p2 + p3) * 4096 as libc::c_int;
            t1 = (p2 - p3) * 4096 as libc::c_int;
            x0 = t0 + t3;
            x3 = t0 - t3;
            x1 = t1 + t2;
            x2 = t1 - t2;
            t0 = *d.offset(56 as libc::c_int as isize) as libc::c_int;
            t1 = *d.offset(40 as libc::c_int as isize) as libc::c_int;
            t2 = *d.offset(24 as libc::c_int as isize) as libc::c_int;
            t3 = *d.offset(8 as libc::c_int as isize) as libc::c_int;
            p3 = t0 + t2;
            p4 = t1 + t3;
            p1 = t0 + t3;
            p2 = t1 + t2;
            p5 = (p3 + p4)
                * ((1.175875602f32 * 4096 as libc::c_int as libc::c_float)
                    as libc::c_double + 0.5f64) as libc::c_int;
            t0 = t0
                * ((0.298631336f32 * 4096 as libc::c_int as libc::c_float)
                    as libc::c_double + 0.5f64) as libc::c_int;
            t1 = t1
                * ((2.053119869f32 * 4096 as libc::c_int as libc::c_float)
                    as libc::c_double + 0.5f64) as libc::c_int;
            t2 = t2
                * ((3.072711026f32 * 4096 as libc::c_int as libc::c_float)
                    as libc::c_double + 0.5f64) as libc::c_int;
            t3 = t3
                * ((1.501321110f32 * 4096 as libc::c_int as libc::c_float)
                    as libc::c_double + 0.5f64) as libc::c_int;
            p1 = p5
                + p1
                    * ((-0.899976223f32 * 4096 as libc::c_int as libc::c_float)
                        as libc::c_double + 0.5f64) as libc::c_int;
            p2 = p5
                + p2
                    * ((-2.562915447f32 * 4096 as libc::c_int as libc::c_float)
                        as libc::c_double + 0.5f64) as libc::c_int;
            p3 = p3
                * ((-1.961570560f32 * 4096 as libc::c_int as libc::c_float)
                    as libc::c_double + 0.5f64) as libc::c_int;
            p4 = p4
                * ((-0.390180644f32 * 4096 as libc::c_int as libc::c_float)
                    as libc::c_double + 0.5f64) as libc::c_int;
            t3 += p1 + p4;
            t2 += p2 + p3;
            t1 += p2 + p4;
            t0 += p1 + p3;
            x0 += 512 as libc::c_int;
            x1 += 512 as libc::c_int;
            x2 += 512 as libc::c_int;
            x3 += 512 as libc::c_int;
            *v.offset(0 as libc::c_int as isize) = x0 + t3 >> 10 as libc::c_int;
            *v.offset(56 as libc::c_int as isize) = x0 - t3 >> 10 as libc::c_int;
            *v.offset(8 as libc::c_int as isize) = x1 + t2 >> 10 as libc::c_int;
            *v.offset(48 as libc::c_int as isize) = x1 - t2 >> 10 as libc::c_int;
            *v.offset(16 as libc::c_int as isize) = x2 + t1 >> 10 as libc::c_int;
            *v.offset(40 as libc::c_int as isize) = x2 - t1 >> 10 as libc::c_int;
            *v.offset(24 as libc::c_int as isize) = x3 + t0 >> 10 as libc::c_int;
            *v.offset(32 as libc::c_int as isize) = x3 - t0 >> 10 as libc::c_int;
        }
        i += 1;
        d = d.offset(1);
        v = v.offset(1);
    }
    i = 0 as libc::c_int;
    v = val.as_mut_ptr();
    o = out;
    while i < 8 as libc::c_int {
        let mut t0_0: libc::c_int = 0;
        let mut t1_0: libc::c_int = 0;
        let mut t2_0: libc::c_int = 0;
        let mut t3_0: libc::c_int = 0;
        let mut p1_0: libc::c_int = 0;
        let mut p2_0: libc::c_int = 0;
        let mut p3_0: libc::c_int = 0;
        let mut p4_0: libc::c_int = 0;
        let mut p5_0: libc::c_int = 0;
        let mut x0_0: libc::c_int = 0;
        let mut x1_0: libc::c_int = 0;
        let mut x2_0: libc::c_int = 0;
        let mut x3_0: libc::c_int = 0;
        p2_0 = *v.offset(2 as libc::c_int as isize);
        p3_0 = *v.offset(6 as libc::c_int as isize);
        p1_0 = (p2_0 + p3_0)
            * ((0.5411961f32 * 4096 as libc::c_int as libc::c_float) as libc::c_double
                + 0.5f64) as libc::c_int;
        t2_0 = p1_0
            + p3_0
                * ((-1.847759065f32 * 4096 as libc::c_int as libc::c_float)
                    as libc::c_double + 0.5f64) as libc::c_int;
        t3_0 = p1_0
            + p2_0
                * ((0.765366865f32 * 4096 as libc::c_int as libc::c_float)
                    as libc::c_double + 0.5f64) as libc::c_int;
        p2_0 = *v.offset(0 as libc::c_int as isize);
        p3_0 = *v.offset(4 as libc::c_int as isize);
        t0_0 = (p2_0 + p3_0) * 4096 as libc::c_int;
        t1_0 = (p2_0 - p3_0) * 4096 as libc::c_int;
        x0_0 = t0_0 + t3_0;
        x3_0 = t0_0 - t3_0;
        x1_0 = t1_0 + t2_0;
        x2_0 = t1_0 - t2_0;
        t0_0 = *v.offset(7 as libc::c_int as isize);
        t1_0 = *v.offset(5 as libc::c_int as isize);
        t2_0 = *v.offset(3 as libc::c_int as isize);
        t3_0 = *v.offset(1 as libc::c_int as isize);
        p3_0 = t0_0 + t2_0;
        p4_0 = t1_0 + t3_0;
        p1_0 = t0_0 + t3_0;
        p2_0 = t1_0 + t2_0;
        p5_0 = (p3_0 + p4_0)
            * ((1.175875602f32 * 4096 as libc::c_int as libc::c_float) as libc::c_double
                + 0.5f64) as libc::c_int;
        t0_0 = t0_0
            * ((0.298631336f32 * 4096 as libc::c_int as libc::c_float) as libc::c_double
                + 0.5f64) as libc::c_int;
        t1_0 = t1_0
            * ((2.053119869f32 * 4096 as libc::c_int as libc::c_float) as libc::c_double
                + 0.5f64) as libc::c_int;
        t2_0 = t2_0
            * ((3.072711026f32 * 4096 as libc::c_int as libc::c_float) as libc::c_double
                + 0.5f64) as libc::c_int;
        t3_0 = t3_0
            * ((1.501321110f32 * 4096 as libc::c_int as libc::c_float) as libc::c_double
                + 0.5f64) as libc::c_int;
        p1_0 = p5_0
            + p1_0
                * ((-0.899976223f32 * 4096 as libc::c_int as libc::c_float)
                    as libc::c_double + 0.5f64) as libc::c_int;
        p2_0 = p5_0
            + p2_0
                * ((-2.562915447f32 * 4096 as libc::c_int as libc::c_float)
                    as libc::c_double + 0.5f64) as libc::c_int;
        p3_0 = p3_0
            * ((-1.961570560f32 * 4096 as libc::c_int as libc::c_float) as libc::c_double
                + 0.5f64) as libc::c_int;
        p4_0 = p4_0
            * ((-0.390180644f32 * 4096 as libc::c_int as libc::c_float) as libc::c_double
                + 0.5f64) as libc::c_int;
        t3_0 += p1_0 + p4_0;
        t2_0 += p2_0 + p3_0;
        t1_0 += p2_0 + p4_0;
        t0_0 += p1_0 + p3_0;
        x0_0 += 65536 as libc::c_int + ((128 as libc::c_int) << 17 as libc::c_int);
        x1_0 += 65536 as libc::c_int + ((128 as libc::c_int) << 17 as libc::c_int);
        x2_0 += 65536 as libc::c_int + ((128 as libc::c_int) << 17 as libc::c_int);
        x3_0 += 65536 as libc::c_int + ((128 as libc::c_int) << 17 as libc::c_int);
        *o
            .offset(
                0 as libc::c_int as isize,
            ) = stbi__clamp(x0_0 + t3_0 >> 17 as libc::c_int);
        *o
            .offset(
                7 as libc::c_int as isize,
            ) = stbi__clamp(x0_0 - t3_0 >> 17 as libc::c_int);
        *o
            .offset(
                1 as libc::c_int as isize,
            ) = stbi__clamp(x1_0 + t2_0 >> 17 as libc::c_int);
        *o
            .offset(
                6 as libc::c_int as isize,
            ) = stbi__clamp(x1_0 - t2_0 >> 17 as libc::c_int);
        *o
            .offset(
                2 as libc::c_int as isize,
            ) = stbi__clamp(x2_0 + t1_0 >> 17 as libc::c_int);
        *o
            .offset(
                5 as libc::c_int as isize,
            ) = stbi__clamp(x2_0 - t1_0 >> 17 as libc::c_int);
        *o
            .offset(
                3 as libc::c_int as isize,
            ) = stbi__clamp(x3_0 + t0_0 >> 17 as libc::c_int);
        *o
            .offset(
                4 as libc::c_int as isize,
            ) = stbi__clamp(x3_0 - t0_0 >> 17 as libc::c_int);
        i += 1;
        v = v.offset(8 as libc::c_int as isize);
        o = o.offset(out_stride as isize);
    }
}
unsafe extern "C" fn stbi__clamp(mut x: libc::c_int) -> stbi_uc {
    if x as libc::c_uint > 255 as libc::c_int as libc::c_uint {
        if x < 0 as libc::c_int {
            return 0 as libc::c_int as stbi_uc;
        }
        if x > 255 as libc::c_int {
            return 255 as libc::c_int as stbi_uc;
        }
    }
    return x as stbi_uc;
}
unsafe extern "C" fn stbi__jpeg_test(mut s: *mut stbi__context) -> libc::c_int {
    let mut r: libc::c_int = 0;
    let mut j: *mut stbi__jpeg = stbi__malloc(
        ::core::mem::size_of::<stbi__jpeg>() as libc::c_ulong,
    ) as *mut stbi__jpeg;
    if j.is_null() {
        return stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char);
    }
    (*j).s = s;
    stbi__setup_jpeg(j);
    r = stbi__decode_jpeg_header(j, STBI__SCAN_type as libc::c_int);
    stbi__rewind(s);
    free(j as *mut libc::c_void);
    return r;
}
unsafe extern "C" fn stbi__pic_load(
    mut s: *mut stbi__context,
    mut px: *mut libc::c_int,
    mut py: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
    mut ri: *mut stbi__result_info,
) -> *mut libc::c_void {
    let mut result: *mut stbi_uc = 0 as *mut stbi_uc;
    let mut i: libc::c_int = 0;
    let mut x: libc::c_int = 0;
    let mut y: libc::c_int = 0;
    let mut internal_comp: libc::c_int = 0;
    if comp.is_null() {
        comp = &mut internal_comp;
    }
    i = 0 as libc::c_int;
    while i < 92 as libc::c_int {
        stbi__get8(s);
        i += 1;
    }
    x = stbi__get16be(s);
    y = stbi__get16be(s);
    if y > (1 as libc::c_int) << 24 as libc::c_int {
        return (if stbi__err(b"too large\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    if x > (1 as libc::c_int) << 24 as libc::c_int {
        return (if stbi__err(b"too large\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    if stbi__at_eof(s) != 0 {
        return (if stbi__err(b"bad file\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    if stbi__mad3sizes_valid(x, y, 4 as libc::c_int, 0 as libc::c_int) == 0 {
        return (if stbi__err(b"too large\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    stbi__get32be(s);
    stbi__get16be(s);
    stbi__get16be(s);
    result = stbi__malloc_mad3(x, y, 4 as libc::c_int, 0 as libc::c_int) as *mut stbi_uc;
    if result.is_null() {
        return (if stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    memset(
        result as *mut libc::c_void,
        0xff as libc::c_int,
        (x * y * 4 as libc::c_int) as libc::c_ulong,
    );
    if (stbi__pic_load_core(s, x, y, comp, result)).is_null() {
        free(result as *mut libc::c_void);
        result = 0 as *mut stbi_uc;
    }
    *px = x;
    *py = y;
    if req_comp == 0 as libc::c_int {
        req_comp = *comp;
    }
    result = stbi__convert_format(
        result,
        4 as libc::c_int,
        req_comp,
        x as libc::c_uint,
        y as libc::c_uint,
    );
    return result as *mut libc::c_void;
}
unsafe extern "C" fn stbi__pic_load_core(
    mut s: *mut stbi__context,
    mut width: libc::c_int,
    mut height: libc::c_int,
    mut comp: *mut libc::c_int,
    mut result: *mut stbi_uc,
) -> *mut stbi_uc {
    let mut act_comp: libc::c_int = 0 as libc::c_int;
    let mut num_packets: libc::c_int = 0 as libc::c_int;
    let mut y: libc::c_int = 0;
    let mut chained: libc::c_int = 0;
    let mut packets: [stbi__pic_packet; 10] = [stbi__pic_packet {
        size: 0,
        type_0: 0,
        channel: 0,
    }; 10];
    loop {
        let mut packet: *mut stbi__pic_packet = 0 as *mut stbi__pic_packet;
        if num_packets as libc::c_ulong
            == (::core::mem::size_of::<[stbi__pic_packet; 10]>() as libc::c_ulong)
                .wrapping_div(
                    ::core::mem::size_of::<stbi__pic_packet>() as libc::c_ulong,
                )
        {
            return (if stbi__err(b"bad format\0" as *const u8 as *const libc::c_char)
                != 0
            {
                0 as *mut libc::c_void
            } else {
                0 as *mut libc::c_void
            }) as size_t as *mut libc::c_uchar;
        }
        let fresh39 = num_packets;
        num_packets = num_packets + 1;
        packet = &mut *packets.as_mut_ptr().offset(fresh39 as isize)
            as *mut stbi__pic_packet;
        chained = stbi__get8(s) as libc::c_int;
        (*packet).size = stbi__get8(s);
        (*packet).type_0 = stbi__get8(s);
        (*packet).channel = stbi__get8(s);
        act_comp |= (*packet).channel as libc::c_int;
        if stbi__at_eof(s) != 0 {
            return (if stbi__err(b"bad file\0" as *const u8 as *const libc::c_char) != 0
            {
                0 as *mut libc::c_void
            } else {
                0 as *mut libc::c_void
            }) as size_t as *mut libc::c_uchar;
        }
        if (*packet).size as libc::c_int != 8 as libc::c_int {
            return (if stbi__err(b"bad format\0" as *const u8 as *const libc::c_char)
                != 0
            {
                0 as *mut libc::c_void
            } else {
                0 as *mut libc::c_void
            }) as size_t as *mut libc::c_uchar;
        }
        if !(chained != 0) {
            break;
        }
    }
    *comp = if act_comp & 0x10 as libc::c_int != 0 {
        4 as libc::c_int
    } else {
        3 as libc::c_int
    };
    y = 0 as libc::c_int;
    while y < height {
        let mut packet_idx: libc::c_int = 0;
        packet_idx = 0 as libc::c_int;
        while packet_idx < num_packets {
            let mut packet_0: *mut stbi__pic_packet = &mut *packets
                .as_mut_ptr()
                .offset(packet_idx as isize) as *mut stbi__pic_packet;
            let mut dest: *mut stbi_uc = result
                .offset((y * width * 4 as libc::c_int) as isize);
            match (*packet_0).type_0 as libc::c_int {
                0 => {
                    let mut x: libc::c_int = 0;
                    x = 0 as libc::c_int;
                    while x < width {
                        if (stbi__readval(s, (*packet_0).channel as libc::c_int, dest))
                            .is_null()
                        {
                            return 0 as *mut stbi_uc;
                        }
                        x += 1;
                        dest = dest.offset(4 as libc::c_int as isize);
                    }
                }
                1 => {
                    let mut left: libc::c_int = width;
                    let mut i: libc::c_int = 0;
                    while left > 0 as libc::c_int {
                        let mut count: stbi_uc = 0;
                        let mut value: [stbi_uc; 4] = [0; 4];
                        count = stbi__get8(s);
                        if stbi__at_eof(s) != 0 {
                            return (if stbi__err(
                                b"bad file\0" as *const u8 as *const libc::c_char,
                            ) != 0
                            {
                                0 as *mut libc::c_void
                            } else {
                                0 as *mut libc::c_void
                            }) as size_t as *mut libc::c_uchar;
                        }
                        if count as libc::c_int > left {
                            count = left as stbi_uc;
                        }
                        if (stbi__readval(
                            s,
                            (*packet_0).channel as libc::c_int,
                            value.as_mut_ptr(),
                        ))
                            .is_null()
                        {
                            return 0 as *mut stbi_uc;
                        }
                        i = 0 as libc::c_int;
                        while i < count as libc::c_int {
                            stbi__copyval(
                                (*packet_0).channel as libc::c_int,
                                dest,
                                value.as_mut_ptr(),
                            );
                            i += 1;
                            dest = dest.offset(4 as libc::c_int as isize);
                        }
                        left -= count as libc::c_int;
                    }
                }
                2 => {
                    let mut left_0: libc::c_int = width;
                    while left_0 > 0 as libc::c_int {
                        let mut count_0: libc::c_int = stbi__get8(s) as libc::c_int;
                        let mut i_0: libc::c_int = 0;
                        if stbi__at_eof(s) != 0 {
                            return (if stbi__err(
                                b"bad file\0" as *const u8 as *const libc::c_char,
                            ) != 0
                            {
                                0 as *mut libc::c_void
                            } else {
                                0 as *mut libc::c_void
                            }) as size_t as *mut libc::c_uchar;
                        }
                        if count_0 >= 128 as libc::c_int {
                            let mut value_0: [stbi_uc; 4] = [0; 4];
                            if count_0 == 128 as libc::c_int {
                                count_0 = stbi__get16be(s);
                            } else {
                                count_0 -= 127 as libc::c_int;
                            }
                            if count_0 > left_0 {
                                return (if stbi__err(
                                    b"bad file\0" as *const u8 as *const libc::c_char,
                                ) != 0
                                {
                                    0 as *mut libc::c_void
                                } else {
                                    0 as *mut libc::c_void
                                }) as size_t as *mut libc::c_uchar;
                            }
                            if (stbi__readval(
                                s,
                                (*packet_0).channel as libc::c_int,
                                value_0.as_mut_ptr(),
                            ))
                                .is_null()
                            {
                                return 0 as *mut stbi_uc;
                            }
                            i_0 = 0 as libc::c_int;
                            while i_0 < count_0 {
                                stbi__copyval(
                                    (*packet_0).channel as libc::c_int,
                                    dest,
                                    value_0.as_mut_ptr(),
                                );
                                i_0 += 1;
                                dest = dest.offset(4 as libc::c_int as isize);
                            }
                        } else {
                            count_0 += 1;
                            if count_0 > left_0 {
                                return (if stbi__err(
                                    b"bad file\0" as *const u8 as *const libc::c_char,
                                ) != 0
                                {
                                    0 as *mut libc::c_void
                                } else {
                                    0 as *mut libc::c_void
                                }) as size_t as *mut libc::c_uchar;
                            }
                            i_0 = 0 as libc::c_int;
                            while i_0 < count_0 {
                                if (stbi__readval(
                                    s,
                                    (*packet_0).channel as libc::c_int,
                                    dest,
                                ))
                                    .is_null()
                                {
                                    return 0 as *mut stbi_uc;
                                }
                                i_0 += 1;
                                dest = dest.offset(4 as libc::c_int as isize);
                            }
                        }
                        left_0 -= count_0;
                    }
                }
                _ => {
                    return (if stbi__err(
                        b"bad format\0" as *const u8 as *const libc::c_char,
                    ) != 0
                    {
                        0 as *mut libc::c_void
                    } else {
                        0 as *mut libc::c_void
                    }) as size_t as *mut libc::c_uchar;
                }
            }
            packet_idx += 1;
        }
        y += 1;
    }
    return result;
}
unsafe extern "C" fn stbi__readval(
    mut s: *mut stbi__context,
    mut channel: libc::c_int,
    mut dest: *mut stbi_uc,
) -> *mut stbi_uc {
    let mut mask: libc::c_int = 0x80 as libc::c_int;
    let mut i: libc::c_int = 0;
    i = 0 as libc::c_int;
    while i < 4 as libc::c_int {
        if channel & mask != 0 {
            if stbi__at_eof(s) != 0 {
                return (if stbi__err(b"bad file\0" as *const u8 as *const libc::c_char)
                    != 0
                {
                    0 as *mut libc::c_void
                } else {
                    0 as *mut libc::c_void
                }) as size_t as *mut libc::c_uchar;
            }
            *dest.offset(i as isize) = stbi__get8(s);
        }
        i += 1;
        mask >>= 1 as libc::c_int;
    }
    return dest;
}
unsafe extern "C" fn stbi__copyval(
    mut channel: libc::c_int,
    mut dest: *mut stbi_uc,
    mut src: *const stbi_uc,
) {
    let mut mask: libc::c_int = 0x80 as libc::c_int;
    let mut i: libc::c_int = 0;
    i = 0 as libc::c_int;
    while i < 4 as libc::c_int {
        if channel & mask != 0 {
            *dest.offset(i as isize) = *src.offset(i as isize);
        }
        i += 1;
        mask >>= 1 as libc::c_int;
    }
}
unsafe extern "C" fn stbi__get32be(mut s: *mut stbi__context) -> stbi__uint32 {
    let mut z: stbi__uint32 = stbi__get16be(s) as stbi__uint32;
    return (z << 16 as libc::c_int).wrapping_add(stbi__get16be(s) as libc::c_uint);
}
unsafe extern "C" fn stbi__pic_test(mut s: *mut stbi__context) -> libc::c_int {
    let mut r: libc::c_int = stbi__pic_test_core(s);
    stbi__rewind(s);
    return r;
}
unsafe extern "C" fn stbi__pic_test_core(mut s: *mut stbi__context) -> libc::c_int {
    let mut i: libc::c_int = 0;
    if stbi__pic_is4(s, b"S\x80\xF64\0" as *const u8 as *const libc::c_char) == 0 {
        return 0 as libc::c_int;
    }
    i = 0 as libc::c_int;
    while i < 84 as libc::c_int {
        stbi__get8(s);
        i += 1;
    }
    if stbi__pic_is4(s, b"PICT\0" as *const u8 as *const libc::c_char) == 0 {
        return 0 as libc::c_int;
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__pic_is4(
    mut s: *mut stbi__context,
    mut str: *const libc::c_char,
) -> libc::c_int {
    let mut i: libc::c_int = 0;
    i = 0 as libc::c_int;
    while i < 4 as libc::c_int {
        if stbi__get8(s) as libc::c_int
            != *str.offset(i as isize) as stbi_uc as libc::c_int
        {
            return 0 as libc::c_int;
        }
        i += 1;
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__psd_load(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
    mut ri: *mut stbi__result_info,
    mut bpc: libc::c_int,
) -> *mut libc::c_void {
    let mut pixelCount: libc::c_int = 0;
    let mut channelCount: libc::c_int = 0;
    let mut compression: libc::c_int = 0;
    let mut channel: libc::c_int = 0;
    let mut i: libc::c_int = 0;
    let mut bitdepth: libc::c_int = 0;
    let mut w: libc::c_int = 0;
    let mut h: libc::c_int = 0;
    let mut out: *mut stbi_uc = 0 as *mut stbi_uc;
    if stbi__get32be(s) != 0x38425053 as libc::c_int as libc::c_uint {
        return (if stbi__err(b"not PSD\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    if stbi__get16be(s) != 1 as libc::c_int {
        return (if stbi__err(b"wrong version\0" as *const u8 as *const libc::c_char) != 0
        {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    stbi__skip(s, 6 as libc::c_int);
    channelCount = stbi__get16be(s);
    if channelCount < 0 as libc::c_int || channelCount > 16 as libc::c_int {
        return (if stbi__err(
            b"wrong channel count\0" as *const u8 as *const libc::c_char,
        ) != 0
        {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    h = stbi__get32be(s) as libc::c_int;
    w = stbi__get32be(s) as libc::c_int;
    if h > (1 as libc::c_int) << 24 as libc::c_int {
        return (if stbi__err(b"too large\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    if w > (1 as libc::c_int) << 24 as libc::c_int {
        return (if stbi__err(b"too large\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    bitdepth = stbi__get16be(s);
    if bitdepth != 8 as libc::c_int && bitdepth != 16 as libc::c_int {
        return (if stbi__err(
            b"unsupported bit depth\0" as *const u8 as *const libc::c_char,
        ) != 0
        {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    if stbi__get16be(s) != 3 as libc::c_int {
        return (if stbi__err(b"wrong color format\0" as *const u8 as *const libc::c_char)
            != 0
        {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    stbi__skip(s, stbi__get32be(s) as libc::c_int);
    stbi__skip(s, stbi__get32be(s) as libc::c_int);
    stbi__skip(s, stbi__get32be(s) as libc::c_int);
    compression = stbi__get16be(s);
    if compression > 1 as libc::c_int {
        return (if stbi__err(b"bad compression\0" as *const u8 as *const libc::c_char)
            != 0
        {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    if stbi__mad3sizes_valid(4 as libc::c_int, w, h, 0 as libc::c_int) == 0 {
        return (if stbi__err(b"too large\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    if compression == 0 && bitdepth == 16 as libc::c_int && bpc == 16 as libc::c_int {
        out = stbi__malloc_mad3(8 as libc::c_int, w, h, 0 as libc::c_int)
            as *mut stbi_uc;
        (*ri).bits_per_channel = 16 as libc::c_int;
    } else {
        out = stbi__malloc((4 as libc::c_int * w * h) as size_t) as *mut stbi_uc;
    }
    if out.is_null() {
        return (if stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    pixelCount = w * h;
    if compression != 0 {
        stbi__skip(s, h * channelCount * 2 as libc::c_int);
        channel = 0 as libc::c_int;
        while channel < 4 as libc::c_int {
            let mut p: *mut stbi_uc = 0 as *mut stbi_uc;
            p = out.offset(channel as isize);
            if channel >= channelCount {
                i = 0 as libc::c_int;
                while i < pixelCount {
                    *p = (if channel == 3 as libc::c_int {
                        255 as libc::c_int
                    } else {
                        0 as libc::c_int
                    }) as stbi_uc;
                    i += 1;
                    p = p.offset(4 as libc::c_int as isize);
                }
            } else if stbi__psd_decode_rle(s, p, pixelCount) == 0 {
                free(out as *mut libc::c_void);
                return (if stbi__err(b"corrupt\0" as *const u8 as *const libc::c_char)
                    != 0
                {
                    0 as *mut libc::c_void
                } else {
                    0 as *mut libc::c_void
                }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
            }
            channel += 1;
        }
    } else {
        channel = 0 as libc::c_int;
        while channel < 4 as libc::c_int {
            if channel >= channelCount {
                if bitdepth == 16 as libc::c_int && bpc == 16 as libc::c_int {
                    let mut q: *mut stbi__uint16 = (out as *mut stbi__uint16)
                        .offset(channel as isize);
                    let mut val: stbi__uint16 = (if channel == 3 as libc::c_int {
                        65535 as libc::c_int
                    } else {
                        0 as libc::c_int
                    }) as stbi__uint16;
                    i = 0 as libc::c_int;
                    while i < pixelCount {
                        *q = val;
                        i += 1;
                        q = q.offset(4 as libc::c_int as isize);
                    }
                } else {
                    let mut p_0: *mut stbi_uc = out.offset(channel as isize);
                    let mut val_0: stbi_uc = (if channel == 3 as libc::c_int {
                        255 as libc::c_int
                    } else {
                        0 as libc::c_int
                    }) as stbi_uc;
                    i = 0 as libc::c_int;
                    while i < pixelCount {
                        *p_0 = val_0;
                        i += 1;
                        p_0 = p_0.offset(4 as libc::c_int as isize);
                    }
                }
            } else if (*ri).bits_per_channel == 16 as libc::c_int {
                let mut q_0: *mut stbi__uint16 = (out as *mut stbi__uint16)
                    .offset(channel as isize);
                i = 0 as libc::c_int;
                while i < pixelCount {
                    *q_0 = stbi__get16be(s) as stbi__uint16;
                    i += 1;
                    q_0 = q_0.offset(4 as libc::c_int as isize);
                }
            } else {
                let mut p_1: *mut stbi_uc = out.offset(channel as isize);
                if bitdepth == 16 as libc::c_int {
                    i = 0 as libc::c_int;
                    while i < pixelCount {
                        *p_1 = (stbi__get16be(s) >> 8 as libc::c_int) as stbi_uc;
                        i += 1;
                        p_1 = p_1.offset(4 as libc::c_int as isize);
                    }
                } else {
                    i = 0 as libc::c_int;
                    while i < pixelCount {
                        *p_1 = stbi__get8(s);
                        i += 1;
                        p_1 = p_1.offset(4 as libc::c_int as isize);
                    }
                }
            }
            channel += 1;
        }
    }
    if channelCount >= 4 as libc::c_int {
        if (*ri).bits_per_channel == 16 as libc::c_int {
            i = 0 as libc::c_int;
            while i < w * h {
                let mut pixel: *mut stbi__uint16 = (out as *mut stbi__uint16)
                    .offset((4 as libc::c_int * i) as isize);
                if *pixel.offset(3 as libc::c_int as isize) as libc::c_int
                    != 0 as libc::c_int
                    && *pixel.offset(3 as libc::c_int as isize) as libc::c_int
                        != 65535 as libc::c_int
                {
                    let mut a: libc::c_float = *pixel.offset(3 as libc::c_int as isize)
                        as libc::c_int as libc::c_float / 65535.0f32;
                    let mut ra: libc::c_float = 1.0f32 / a;
                    let mut inv_a: libc::c_float = 65535.0f32
                        * (1 as libc::c_int as libc::c_float - ra);
                    *pixel
                        .offset(
                            0 as libc::c_int as isize,
                        ) = (*pixel.offset(0 as libc::c_int as isize) as libc::c_int
                        as libc::c_float * ra + inv_a) as stbi__uint16;
                    *pixel
                        .offset(
                            1 as libc::c_int as isize,
                        ) = (*pixel.offset(1 as libc::c_int as isize) as libc::c_int
                        as libc::c_float * ra + inv_a) as stbi__uint16;
                    *pixel
                        .offset(
                            2 as libc::c_int as isize,
                        ) = (*pixel.offset(2 as libc::c_int as isize) as libc::c_int
                        as libc::c_float * ra + inv_a) as stbi__uint16;
                }
                i += 1;
            }
        } else {
            i = 0 as libc::c_int;
            while i < w * h {
                let mut pixel_0: *mut libc::c_uchar = out
                    .offset((4 as libc::c_int * i) as isize);
                if *pixel_0.offset(3 as libc::c_int as isize) as libc::c_int
                    != 0 as libc::c_int
                    && *pixel_0.offset(3 as libc::c_int as isize) as libc::c_int
                        != 255 as libc::c_int
                {
                    let mut a_0: libc::c_float = *pixel_0
                        .offset(3 as libc::c_int as isize) as libc::c_int
                        as libc::c_float / 255.0f32;
                    let mut ra_0: libc::c_float = 1.0f32 / a_0;
                    let mut inv_a_0: libc::c_float = 255.0f32
                        * (1 as libc::c_int as libc::c_float - ra_0);
                    *pixel_0
                        .offset(
                            0 as libc::c_int as isize,
                        ) = (*pixel_0.offset(0 as libc::c_int as isize) as libc::c_int
                        as libc::c_float * ra_0 + inv_a_0) as libc::c_uchar;
                    *pixel_0
                        .offset(
                            1 as libc::c_int as isize,
                        ) = (*pixel_0.offset(1 as libc::c_int as isize) as libc::c_int
                        as libc::c_float * ra_0 + inv_a_0) as libc::c_uchar;
                    *pixel_0
                        .offset(
                            2 as libc::c_int as isize,
                        ) = (*pixel_0.offset(2 as libc::c_int as isize) as libc::c_int
                        as libc::c_float * ra_0 + inv_a_0) as libc::c_uchar;
                }
                i += 1;
            }
        }
    }
    if req_comp != 0 && req_comp != 4 as libc::c_int {
        if (*ri).bits_per_channel == 16 as libc::c_int {
            out = stbi__convert_format16(
                out as *mut stbi__uint16,
                4 as libc::c_int,
                req_comp,
                w as libc::c_uint,
                h as libc::c_uint,
            ) as *mut stbi_uc;
        } else {
            out = stbi__convert_format(
                out,
                4 as libc::c_int,
                req_comp,
                w as libc::c_uint,
                h as libc::c_uint,
            );
        }
        if out.is_null() {
            return out as *mut libc::c_void;
        }
    }
    if !comp.is_null() {
        *comp = 4 as libc::c_int;
    }
    *y = h;
    *x = w;
    return out as *mut libc::c_void;
}
unsafe extern "C" fn stbi__convert_format16(
    mut data: *mut stbi__uint16,
    mut img_n: libc::c_int,
    mut req_comp: libc::c_int,
    mut x: libc::c_uint,
    mut y: libc::c_uint,
) -> *mut stbi__uint16 {
    let mut i: libc::c_int = 0;
    let mut j: libc::c_int = 0;
    let mut good: *mut stbi__uint16 = 0 as *mut stbi__uint16;
    if req_comp == img_n {
        return data;
    }
    good = stbi__malloc(
        (req_comp as libc::c_uint)
            .wrapping_mul(x)
            .wrapping_mul(y)
            .wrapping_mul(2 as libc::c_int as libc::c_uint) as size_t,
    ) as *mut stbi__uint16;
    if good.is_null() {
        free(data as *mut libc::c_void);
        return (if stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut stbi__uint16;
    }
    j = 0 as libc::c_int;
    while j < y as libc::c_int {
        let mut src: *mut stbi__uint16 = data
            .offset(
                (j as libc::c_uint).wrapping_mul(x).wrapping_mul(img_n as libc::c_uint)
                    as isize,
            );
        let mut dest: *mut stbi__uint16 = good
            .offset(
                (j as libc::c_uint)
                    .wrapping_mul(x)
                    .wrapping_mul(req_comp as libc::c_uint) as isize,
            );
        match img_n * 8 as libc::c_int + req_comp {
            10 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    *dest
                        .offset(
                            0 as libc::c_int as isize,
                        ) = *src.offset(0 as libc::c_int as isize);
                    *dest
                        .offset(
                            1 as libc::c_int as isize,
                        ) = 0xffff as libc::c_int as stbi__uint16;
                    i -= 1;
                    src = src.offset(1 as libc::c_int as isize);
                    dest = dest.offset(2 as libc::c_int as isize);
                }
            }
            11 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    let ref mut fresh40 = *dest.offset(2 as libc::c_int as isize);
                    *fresh40 = *src.offset(0 as libc::c_int as isize);
                    let ref mut fresh41 = *dest.offset(1 as libc::c_int as isize);
                    *fresh41 = *fresh40;
                    *dest.offset(0 as libc::c_int as isize) = *fresh41;
                    i -= 1;
                    src = src.offset(1 as libc::c_int as isize);
                    dest = dest.offset(3 as libc::c_int as isize);
                }
            }
            12 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    let ref mut fresh42 = *dest.offset(2 as libc::c_int as isize);
                    *fresh42 = *src.offset(0 as libc::c_int as isize);
                    let ref mut fresh43 = *dest.offset(1 as libc::c_int as isize);
                    *fresh43 = *fresh42;
                    *dest.offset(0 as libc::c_int as isize) = *fresh43;
                    *dest
                        .offset(
                            3 as libc::c_int as isize,
                        ) = 0xffff as libc::c_int as stbi__uint16;
                    i -= 1;
                    src = src.offset(1 as libc::c_int as isize);
                    dest = dest.offset(4 as libc::c_int as isize);
                }
            }
            17 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    *dest
                        .offset(
                            0 as libc::c_int as isize,
                        ) = *src.offset(0 as libc::c_int as isize);
                    i -= 1;
                    src = src.offset(2 as libc::c_int as isize);
                    dest = dest.offset(1 as libc::c_int as isize);
                }
            }
            19 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    let ref mut fresh44 = *dest.offset(2 as libc::c_int as isize);
                    *fresh44 = *src.offset(0 as libc::c_int as isize);
                    let ref mut fresh45 = *dest.offset(1 as libc::c_int as isize);
                    *fresh45 = *fresh44;
                    *dest.offset(0 as libc::c_int as isize) = *fresh45;
                    i -= 1;
                    src = src.offset(2 as libc::c_int as isize);
                    dest = dest.offset(3 as libc::c_int as isize);
                }
            }
            20 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    let ref mut fresh46 = *dest.offset(2 as libc::c_int as isize);
                    *fresh46 = *src.offset(0 as libc::c_int as isize);
                    let ref mut fresh47 = *dest.offset(1 as libc::c_int as isize);
                    *fresh47 = *fresh46;
                    *dest.offset(0 as libc::c_int as isize) = *fresh47;
                    *dest
                        .offset(
                            3 as libc::c_int as isize,
                        ) = *src.offset(1 as libc::c_int as isize);
                    i -= 1;
                    src = src.offset(2 as libc::c_int as isize);
                    dest = dest.offset(4 as libc::c_int as isize);
                }
            }
            28 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    *dest
                        .offset(
                            0 as libc::c_int as isize,
                        ) = *src.offset(0 as libc::c_int as isize);
                    *dest
                        .offset(
                            1 as libc::c_int as isize,
                        ) = *src.offset(1 as libc::c_int as isize);
                    *dest
                        .offset(
                            2 as libc::c_int as isize,
                        ) = *src.offset(2 as libc::c_int as isize);
                    *dest
                        .offset(
                            3 as libc::c_int as isize,
                        ) = 0xffff as libc::c_int as stbi__uint16;
                    i -= 1;
                    src = src.offset(3 as libc::c_int as isize);
                    dest = dest.offset(4 as libc::c_int as isize);
                }
            }
            25 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    *dest
                        .offset(
                            0 as libc::c_int as isize,
                        ) = stbi__compute_y_16(
                        *src.offset(0 as libc::c_int as isize) as libc::c_int,
                        *src.offset(1 as libc::c_int as isize) as libc::c_int,
                        *src.offset(2 as libc::c_int as isize) as libc::c_int,
                    );
                    i -= 1;
                    src = src.offset(3 as libc::c_int as isize);
                    dest = dest.offset(1 as libc::c_int as isize);
                }
            }
            26 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    *dest
                        .offset(
                            0 as libc::c_int as isize,
                        ) = stbi__compute_y_16(
                        *src.offset(0 as libc::c_int as isize) as libc::c_int,
                        *src.offset(1 as libc::c_int as isize) as libc::c_int,
                        *src.offset(2 as libc::c_int as isize) as libc::c_int,
                    );
                    *dest
                        .offset(
                            1 as libc::c_int as isize,
                        ) = 0xffff as libc::c_int as stbi__uint16;
                    i -= 1;
                    src = src.offset(3 as libc::c_int as isize);
                    dest = dest.offset(2 as libc::c_int as isize);
                }
            }
            33 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    *dest
                        .offset(
                            0 as libc::c_int as isize,
                        ) = stbi__compute_y_16(
                        *src.offset(0 as libc::c_int as isize) as libc::c_int,
                        *src.offset(1 as libc::c_int as isize) as libc::c_int,
                        *src.offset(2 as libc::c_int as isize) as libc::c_int,
                    );
                    i -= 1;
                    src = src.offset(4 as libc::c_int as isize);
                    dest = dest.offset(1 as libc::c_int as isize);
                }
            }
            34 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    *dest
                        .offset(
                            0 as libc::c_int as isize,
                        ) = stbi__compute_y_16(
                        *src.offset(0 as libc::c_int as isize) as libc::c_int,
                        *src.offset(1 as libc::c_int as isize) as libc::c_int,
                        *src.offset(2 as libc::c_int as isize) as libc::c_int,
                    );
                    *dest
                        .offset(
                            1 as libc::c_int as isize,
                        ) = *src.offset(3 as libc::c_int as isize);
                    i -= 1;
                    src = src.offset(4 as libc::c_int as isize);
                    dest = dest.offset(2 as libc::c_int as isize);
                }
            }
            35 => {
                i = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                while i >= 0 as libc::c_int {
                    *dest
                        .offset(
                            0 as libc::c_int as isize,
                        ) = *src.offset(0 as libc::c_int as isize);
                    *dest
                        .offset(
                            1 as libc::c_int as isize,
                        ) = *src.offset(1 as libc::c_int as isize);
                    *dest
                        .offset(
                            2 as libc::c_int as isize,
                        ) = *src.offset(2 as libc::c_int as isize);
                    i -= 1;
                    src = src.offset(4 as libc::c_int as isize);
                    dest = dest.offset(3 as libc::c_int as isize);
                }
            }
            _ => {
                free(data as *mut libc::c_void);
                free(good as *mut libc::c_void);
                return (if stbi__err(
                    b"unsupported\0" as *const u8 as *const libc::c_char,
                ) != 0
                {
                    0 as *mut libc::c_void
                } else {
                    0 as *mut libc::c_void
                }) as size_t as *mut libc::c_uchar as *mut stbi__uint16;
            }
        }
        j += 1;
    }
    free(data as *mut libc::c_void);
    return good;
}
unsafe extern "C" fn stbi__compute_y_16(
    mut r: libc::c_int,
    mut g: libc::c_int,
    mut b: libc::c_int,
) -> stbi__uint16 {
    return (r * 77 as libc::c_int + g * 150 as libc::c_int + 29 as libc::c_int * b
        >> 8 as libc::c_int) as stbi__uint16;
}
unsafe extern "C" fn stbi__psd_decode_rle(
    mut s: *mut stbi__context,
    mut p: *mut stbi_uc,
    mut pixelCount: libc::c_int,
) -> libc::c_int {
    let mut count: libc::c_int = 0;
    let mut nleft: libc::c_int = 0;
    let mut len: libc::c_int = 0;
    count = 0 as libc::c_int;
    loop {
        nleft = pixelCount - count;
        if !(nleft > 0 as libc::c_int) {
            break;
        }
        len = stbi__get8(s) as libc::c_int;
        if !(len == 128 as libc::c_int) {
            if len < 128 as libc::c_int {
                len += 1;
                if len > nleft {
                    return 0 as libc::c_int;
                }
                count += len;
                while len != 0 {
                    *p = stbi__get8(s);
                    p = p.offset(4 as libc::c_int as isize);
                    len -= 1;
                }
            } else if len > 128 as libc::c_int {
                let mut val: stbi_uc = 0;
                len = 257 as libc::c_int - len;
                if len > nleft {
                    return 0 as libc::c_int;
                }
                val = stbi__get8(s);
                count += len;
                while len != 0 {
                    *p = val;
                    p = p.offset(4 as libc::c_int as isize);
                    len -= 1;
                }
            }
        }
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__psd_test(mut s: *mut stbi__context) -> libc::c_int {
    let mut r: libc::c_int = (stbi__get32be(s)
        == 0x38425053 as libc::c_int as libc::c_uint) as libc::c_int;
    stbi__rewind(s);
    return r;
}
unsafe extern "C" fn stbi__gif_load(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
    mut ri: *mut stbi__result_info,
) -> *mut libc::c_void {
    let mut u: *mut stbi_uc = 0 as *mut stbi_uc;
    let mut g: stbi__gif = stbi__gif {
        w: 0,
        h: 0,
        out: 0 as *mut stbi_uc,
        background: 0 as *mut stbi_uc,
        history: 0 as *mut stbi_uc,
        flags: 0,
        bgindex: 0,
        ratio: 0,
        transparent: 0,
        eflags: 0,
        pal: [[0; 4]; 256],
        lpal: [[0; 4]; 256],
        codes: [stbi__gif_lzw {
            prefix: 0,
            first: 0,
            suffix: 0,
        }; 8192],
        color_table: 0 as *mut stbi_uc,
        parse: 0,
        step: 0,
        lflags: 0,
        start_x: 0,
        start_y: 0,
        max_x: 0,
        max_y: 0,
        cur_x: 0,
        cur_y: 0,
        line_size: 0,
        delay: 0,
    };
    memset(
        &mut g as *mut stbi__gif as *mut libc::c_void,
        0 as libc::c_int,
        ::core::mem::size_of::<stbi__gif>() as libc::c_ulong,
    );
    u = stbi__gif_load_next(s, &mut g, comp, req_comp, 0 as *mut stbi_uc);
    if u == s as *mut stbi_uc {
        u = 0 as *mut stbi_uc;
    }
    if !u.is_null() {
        *x = g.w;
        *y = g.h;
        if req_comp != 0 && req_comp != 4 as libc::c_int {
            u = stbi__convert_format(
                u,
                4 as libc::c_int,
                req_comp,
                g.w as libc::c_uint,
                g.h as libc::c_uint,
            );
        }
    } else if !(g.out).is_null() {
        free(g.out as *mut libc::c_void);
    }
    free(g.history as *mut libc::c_void);
    free(g.background as *mut libc::c_void);
    return u as *mut libc::c_void;
}
unsafe extern "C" fn stbi__gif_load_next(
    mut s: *mut stbi__context,
    mut g: *mut stbi__gif,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
    mut two_back: *mut stbi_uc,
) -> *mut stbi_uc {
    let mut dispose: libc::c_int = 0;
    let mut first_frame: libc::c_int = 0;
    let mut pi: libc::c_int = 0;
    let mut pcount: libc::c_int = 0;
    first_frame = 0 as libc::c_int;
    if ((*g).out).is_null() {
        if stbi__gif_header(s, g, comp, 0 as libc::c_int) == 0 {
            return 0 as *mut stbi_uc;
        }
        if stbi__mad3sizes_valid(4 as libc::c_int, (*g).w, (*g).h, 0 as libc::c_int) == 0
        {
            return (if stbi__err(b"too large\0" as *const u8 as *const libc::c_char) != 0
            {
                0 as *mut libc::c_void
            } else {
                0 as *mut libc::c_void
            }) as size_t as *mut libc::c_uchar;
        }
        pcount = (*g).w * (*g).h;
        (*g).out = stbi__malloc((4 as libc::c_int * pcount) as size_t) as *mut stbi_uc;
        (*g)
            .background = stbi__malloc((4 as libc::c_int * pcount) as size_t)
            as *mut stbi_uc;
        (*g).history = stbi__malloc(pcount as size_t) as *mut stbi_uc;
        if ((*g).out).is_null() || ((*g).background).is_null()
            || ((*g).history).is_null()
        {
            return (if stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char) != 0
            {
                0 as *mut libc::c_void
            } else {
                0 as *mut libc::c_void
            }) as size_t as *mut libc::c_uchar;
        }
        memset(
            (*g).out as *mut libc::c_void,
            0 as libc::c_int,
            (4 as libc::c_int * pcount) as libc::c_ulong,
        );
        memset(
            (*g).background as *mut libc::c_void,
            0 as libc::c_int,
            (4 as libc::c_int * pcount) as libc::c_ulong,
        );
        memset(
            (*g).history as *mut libc::c_void,
            0 as libc::c_int,
            pcount as libc::c_ulong,
        );
        first_frame = 1 as libc::c_int;
    } else {
        dispose = ((*g).eflags & 0x1c as libc::c_int) >> 2 as libc::c_int;
        pcount = (*g).w * (*g).h;
        if dispose == 3 as libc::c_int && two_back.is_null() {
            dispose = 2 as libc::c_int;
        }
        if dispose == 3 as libc::c_int {
            pi = 0 as libc::c_int;
            while pi < pcount {
                if *((*g).history).offset(pi as isize) != 0 {
                    memcpy(
                        &mut *((*g).out).offset((pi * 4 as libc::c_int) as isize)
                            as *mut stbi_uc as *mut libc::c_void,
                        &mut *two_back.offset((pi * 4 as libc::c_int) as isize)
                            as *mut stbi_uc as *const libc::c_void,
                        4 as libc::c_int as libc::c_ulong,
                    );
                }
                pi += 1;
            }
        } else if dispose == 2 as libc::c_int {
            pi = 0 as libc::c_int;
            while pi < pcount {
                if *((*g).history).offset(pi as isize) != 0 {
                    memcpy(
                        &mut *((*g).out).offset((pi * 4 as libc::c_int) as isize)
                            as *mut stbi_uc as *mut libc::c_void,
                        &mut *((*g).background).offset((pi * 4 as libc::c_int) as isize)
                            as *mut stbi_uc as *const libc::c_void,
                        4 as libc::c_int as libc::c_ulong,
                    );
                }
                pi += 1;
            }
        }
        memcpy(
            (*g).background as *mut libc::c_void,
            (*g).out as *const libc::c_void,
            (4 as libc::c_int * (*g).w * (*g).h) as libc::c_ulong,
        );
    }
    memset(
        (*g).history as *mut libc::c_void,
        0 as libc::c_int,
        ((*g).w * (*g).h) as libc::c_ulong,
    );
    loop {
        let mut tag: libc::c_int = stbi__get8(s) as libc::c_int;
        let mut current_block_110: u64;
        match tag {
            44 => {
                let mut x: stbi__int32 = 0;
                let mut y: stbi__int32 = 0;
                let mut w: stbi__int32 = 0;
                let mut h: stbi__int32 = 0;
                let mut o: *mut stbi_uc = 0 as *mut stbi_uc;
                x = stbi__get16le(s);
                y = stbi__get16le(s);
                w = stbi__get16le(s);
                h = stbi__get16le(s);
                if x + w > (*g).w || y + h > (*g).h {
                    return (if stbi__err(
                        b"bad Image Descriptor\0" as *const u8 as *const libc::c_char,
                    ) != 0
                    {
                        0 as *mut libc::c_void
                    } else {
                        0 as *mut libc::c_void
                    }) as size_t as *mut libc::c_uchar;
                }
                (*g).line_size = (*g).w * 4 as libc::c_int;
                (*g).start_x = x * 4 as libc::c_int;
                (*g).start_y = y * (*g).line_size;
                (*g).max_x = (*g).start_x + w * 4 as libc::c_int;
                (*g).max_y = (*g).start_y + h * (*g).line_size;
                (*g).cur_x = (*g).start_x;
                (*g).cur_y = (*g).start_y;
                if w == 0 as libc::c_int {
                    (*g).cur_y = (*g).max_y;
                }
                (*g).lflags = stbi__get8(s) as libc::c_int;
                if (*g).lflags & 0x40 as libc::c_int != 0 {
                    (*g).step = 8 as libc::c_int * (*g).line_size;
                    (*g).parse = 3 as libc::c_int;
                } else {
                    (*g).step = (*g).line_size;
                    (*g).parse = 0 as libc::c_int;
                }
                if (*g).lflags & 0x80 as libc::c_int != 0 {
                    stbi__gif_parse_colortable(
                        s,
                        ((*g).lpal).as_mut_ptr(),
                        (2 as libc::c_int) << ((*g).lflags & 7 as libc::c_int),
                        if (*g).eflags & 0x1 as libc::c_int != 0 {
                            (*g).transparent
                        } else {
                            -(1 as libc::c_int)
                        },
                    );
                    (*g).color_table = ((*g).lpal).as_mut_ptr() as *mut stbi_uc;
                } else if (*g).flags & 0x80 as libc::c_int != 0 {
                    (*g).color_table = ((*g).pal).as_mut_ptr() as *mut stbi_uc;
                } else {
                    return (if stbi__err(
                        b"missing color table\0" as *const u8 as *const libc::c_char,
                    ) != 0
                    {
                        0 as *mut libc::c_void
                    } else {
                        0 as *mut libc::c_void
                    }) as size_t as *mut libc::c_uchar
                }
                o = stbi__process_gif_raster(s, g);
                if o.is_null() {
                    return 0 as *mut stbi_uc;
                }
                pcount = (*g).w * (*g).h;
                if first_frame != 0 && (*g).bgindex > 0 as libc::c_int {
                    pi = 0 as libc::c_int;
                    while pi < pcount {
                        if *((*g).history).offset(pi as isize) as libc::c_int
                            == 0 as libc::c_int
                        {
                            (*g)
                                .pal[(*g).bgindex
                                as usize][3 as libc::c_int
                                as usize] = 255 as libc::c_int as stbi_uc;
                            memcpy(
                                &mut *((*g).out).offset((pi * 4 as libc::c_int) as isize)
                                    as *mut stbi_uc as *mut libc::c_void,
                                &mut *((*g).pal).as_mut_ptr().offset((*g).bgindex as isize)
                                    as *mut [stbi_uc; 4] as *const libc::c_void,
                                4 as libc::c_int as libc::c_ulong,
                            );
                        }
                        pi += 1;
                    }
                }
                return o;
            }
            33 => {
                let mut len: libc::c_int = 0;
                let mut ext: libc::c_int = stbi__get8(s) as libc::c_int;
                if ext == 0xf9 as libc::c_int {
                    len = stbi__get8(s) as libc::c_int;
                    if len == 4 as libc::c_int {
                        (*g).eflags = stbi__get8(s) as libc::c_int;
                        (*g).delay = 10 as libc::c_int * stbi__get16le(s);
                        if (*g).transparent >= 0 as libc::c_int {
                            (*g)
                                .pal[(*g).transparent
                                as usize][3 as libc::c_int
                                as usize] = 255 as libc::c_int as stbi_uc;
                        }
                        if (*g).eflags & 0x1 as libc::c_int != 0 {
                            (*g).transparent = stbi__get8(s) as libc::c_int;
                            if (*g).transparent >= 0 as libc::c_int {
                                (*g)
                                    .pal[(*g).transparent
                                    as usize][3 as libc::c_int
                                    as usize] = 0 as libc::c_int as stbi_uc;
                            }
                        } else {
                            stbi__skip(s, 1 as libc::c_int);
                            (*g).transparent = -(1 as libc::c_int);
                        }
                        current_block_110 = 8102658916883067714;
                    } else {
                        stbi__skip(s, len);
                        current_block_110 = 11071260907632769126;
                    }
                } else {
                    current_block_110 = 8102658916883067714;
                }
                match current_block_110 {
                    8102658916883067714 => {
                        loop {
                            len = stbi__get8(s) as libc::c_int;
                            if !(len != 0 as libc::c_int) {
                                break;
                            }
                            stbi__skip(s, len);
                        }
                    }
                    _ => {}
                }
            }
            59 => return s as *mut stbi_uc,
            _ => {
                return (if stbi__err(
                    b"unknown code\0" as *const u8 as *const libc::c_char,
                ) != 0
                {
                    0 as *mut libc::c_void
                } else {
                    0 as *mut libc::c_void
                }) as size_t as *mut libc::c_uchar;
            }
        }
    };
}
unsafe extern "C" fn stbi__process_gif_raster(
    mut s: *mut stbi__context,
    mut g: *mut stbi__gif,
) -> *mut stbi_uc {
    let mut lzw_cs: stbi_uc = 0;
    let mut len: stbi__int32 = 0;
    let mut init_code: stbi__int32 = 0;
    let mut first: stbi__uint32 = 0;
    let mut codesize: stbi__int32 = 0;
    let mut codemask: stbi__int32 = 0;
    let mut avail: stbi__int32 = 0;
    let mut oldcode: stbi__int32 = 0;
    let mut bits: stbi__int32 = 0;
    let mut valid_bits: stbi__int32 = 0;
    let mut clear: stbi__int32 = 0;
    let mut p: *mut stbi__gif_lzw = 0 as *mut stbi__gif_lzw;
    lzw_cs = stbi__get8(s);
    if lzw_cs as libc::c_int > 12 as libc::c_int {
        return 0 as *mut stbi_uc;
    }
    clear = (1 as libc::c_int) << lzw_cs as libc::c_int;
    first = 1 as libc::c_int as stbi__uint32;
    codesize = lzw_cs as libc::c_int + 1 as libc::c_int;
    codemask = ((1 as libc::c_int) << codesize) - 1 as libc::c_int;
    bits = 0 as libc::c_int;
    valid_bits = 0 as libc::c_int;
    init_code = 0 as libc::c_int;
    while init_code < clear {
        (*g).codes[init_code as usize].prefix = -(1 as libc::c_int) as stbi__int16;
        (*g).codes[init_code as usize].first = init_code as stbi_uc;
        (*g).codes[init_code as usize].suffix = init_code as stbi_uc;
        init_code += 1;
    }
    avail = clear + 2 as libc::c_int;
    oldcode = -(1 as libc::c_int);
    len = 0 as libc::c_int;
    loop {
        if valid_bits < codesize {
            if len == 0 as libc::c_int {
                len = stbi__get8(s) as stbi__int32;
                if len == 0 as libc::c_int {
                    return (*g).out;
                }
            }
            len -= 1;
            bits |= (stbi__get8(s) as stbi__int32) << valid_bits;
            valid_bits += 8 as libc::c_int;
        } else {
            let mut code: stbi__int32 = bits & codemask;
            bits >>= codesize;
            valid_bits -= codesize;
            if code == clear {
                codesize = lzw_cs as libc::c_int + 1 as libc::c_int;
                codemask = ((1 as libc::c_int) << codesize) - 1 as libc::c_int;
                avail = clear + 2 as libc::c_int;
                oldcode = -(1 as libc::c_int);
                first = 0 as libc::c_int as stbi__uint32;
            } else if code == clear + 1 as libc::c_int {
                stbi__skip(s, len);
                loop {
                    len = stbi__get8(s) as stbi__int32;
                    if !(len > 0 as libc::c_int) {
                        break;
                    }
                    stbi__skip(s, len);
                }
                return (*g).out;
            } else {
                if code <= avail {
                    if first != 0 {
                        return (if stbi__err(
                            b"no clear code\0" as *const u8 as *const libc::c_char,
                        ) != 0
                        {
                            0 as *mut libc::c_void
                        } else {
                            0 as *mut libc::c_void
                        }) as size_t as *mut libc::c_uchar;
                    }
                    if oldcode >= 0 as libc::c_int {
                        let fresh48 = avail;
                        avail = avail + 1;
                        p = &mut *((*g).codes).as_mut_ptr().offset(fresh48 as isize)
                            as *mut stbi__gif_lzw;
                        if avail > 8192 as libc::c_int {
                            return (if stbi__err(
                                b"too many codes\0" as *const u8 as *const libc::c_char,
                            ) != 0
                            {
                                0 as *mut libc::c_void
                            } else {
                                0 as *mut libc::c_void
                            }) as size_t as *mut libc::c_uchar;
                        }
                        (*p).prefix = oldcode as stbi__int16;
                        (*p).first = (*g).codes[oldcode as usize].first;
                        (*p)
                            .suffix = (if code == avail {
                            (*p).first as libc::c_int
                        } else {
                            (*g).codes[code as usize].first as libc::c_int
                        }) as stbi_uc;
                    } else if code == avail {
                        return (if stbi__err(
                            b"illegal code in raster\0" as *const u8
                                as *const libc::c_char,
                        ) != 0
                        {
                            0 as *mut libc::c_void
                        } else {
                            0 as *mut libc::c_void
                        }) as size_t as *mut libc::c_uchar
                    }
                    stbi__out_gif_code(g, code as stbi__uint16);
                    if avail & codemask == 0 as libc::c_int
                        && avail <= 0xfff as libc::c_int
                    {
                        codesize += 1;
                        codemask = ((1 as libc::c_int) << codesize) - 1 as libc::c_int;
                    }
                    oldcode = code;
                } else {
                    return (if stbi__err(
                        b"illegal code in raster\0" as *const u8 as *const libc::c_char,
                    ) != 0
                    {
                        0 as *mut libc::c_void
                    } else {
                        0 as *mut libc::c_void
                    }) as size_t as *mut libc::c_uchar
                }
            }
        }
    };
}
unsafe extern "C" fn stbi__out_gif_code(mut g: *mut stbi__gif, mut code: stbi__uint16) {
    let mut p: *mut stbi_uc = 0 as *mut stbi_uc;
    let mut c: *mut stbi_uc = 0 as *mut stbi_uc;
    let mut idx: libc::c_int = 0;
    if (*g).codes[code as usize].prefix as libc::c_int >= 0 as libc::c_int {
        stbi__out_gif_code(g, (*g).codes[code as usize].prefix as stbi__uint16);
    }
    if (*g).cur_y >= (*g).max_y {
        return;
    }
    idx = (*g).cur_x + (*g).cur_y;
    p = &mut *((*g).out).offset(idx as isize) as *mut stbi_uc;
    *((*g).history)
        .offset((idx / 4 as libc::c_int) as isize) = 1 as libc::c_int as stbi_uc;
    c = &mut *((*g).color_table)
        .offset(
            ((*((*g).codes).as_mut_ptr().offset(code as isize)).suffix as libc::c_int
                * 4 as libc::c_int) as isize,
        ) as *mut stbi_uc;
    if *c.offset(3 as libc::c_int as isize) as libc::c_int > 128 as libc::c_int {
        *p.offset(0 as libc::c_int as isize) = *c.offset(2 as libc::c_int as isize);
        *p.offset(1 as libc::c_int as isize) = *c.offset(1 as libc::c_int as isize);
        *p.offset(2 as libc::c_int as isize) = *c.offset(0 as libc::c_int as isize);
        *p.offset(3 as libc::c_int as isize) = *c.offset(3 as libc::c_int as isize);
    }
    (*g).cur_x += 4 as libc::c_int;
    if (*g).cur_x >= (*g).max_x {
        (*g).cur_x = (*g).start_x;
        (*g).cur_y += (*g).step;
        while (*g).cur_y >= (*g).max_y && (*g).parse > 0 as libc::c_int {
            (*g).step = ((1 as libc::c_int) << (*g).parse) * (*g).line_size;
            (*g).cur_y = (*g).start_y + ((*g).step >> 1 as libc::c_int);
            (*g).parse -= 1;
        }
    }
}
unsafe extern "C" fn stbi__gif_parse_colortable(
    mut s: *mut stbi__context,
    mut pal: *mut [stbi_uc; 4],
    mut num_entries: libc::c_int,
    mut transp: libc::c_int,
) {
    let mut i: libc::c_int = 0;
    i = 0 as libc::c_int;
    while i < num_entries {
        (*pal.offset(i as isize))[2 as libc::c_int as usize] = stbi__get8(s);
        (*pal.offset(i as isize))[1 as libc::c_int as usize] = stbi__get8(s);
        (*pal.offset(i as isize))[0 as libc::c_int as usize] = stbi__get8(s);
        (*pal
            .offset(
                i as isize,
            ))[3 as libc::c_int
            as usize] = (if transp == i { 0 as libc::c_int } else { 255 as libc::c_int })
            as stbi_uc;
        i += 1;
    }
}
unsafe extern "C" fn stbi__gif_header(
    mut s: *mut stbi__context,
    mut g: *mut stbi__gif,
    mut comp: *mut libc::c_int,
    mut is_info: libc::c_int,
) -> libc::c_int {
    let mut version: stbi_uc = 0;
    if stbi__get8(s) as libc::c_int != 'G' as i32
        || stbi__get8(s) as libc::c_int != 'I' as i32
        || stbi__get8(s) as libc::c_int != 'F' as i32
        || stbi__get8(s) as libc::c_int != '8' as i32
    {
        return stbi__err(b"not GIF\0" as *const u8 as *const libc::c_char);
    }
    version = stbi__get8(s);
    if version as libc::c_int != '7' as i32 && version as libc::c_int != '9' as i32 {
        return stbi__err(b"not GIF\0" as *const u8 as *const libc::c_char);
    }
    if stbi__get8(s) as libc::c_int != 'a' as i32 {
        return stbi__err(b"not GIF\0" as *const u8 as *const libc::c_char);
    }
    stbi__g_failure_reason = b"\0" as *const u8 as *const libc::c_char;
    (*g).w = stbi__get16le(s);
    (*g).h = stbi__get16le(s);
    (*g).flags = stbi__get8(s) as libc::c_int;
    (*g).bgindex = stbi__get8(s) as libc::c_int;
    (*g).ratio = stbi__get8(s) as libc::c_int;
    (*g).transparent = -(1 as libc::c_int);
    if (*g).w > (1 as libc::c_int) << 24 as libc::c_int {
        return stbi__err(b"too large\0" as *const u8 as *const libc::c_char);
    }
    if (*g).h > (1 as libc::c_int) << 24 as libc::c_int {
        return stbi__err(b"too large\0" as *const u8 as *const libc::c_char);
    }
    if !comp.is_null() {
        *comp = 4 as libc::c_int;
    }
    if is_info != 0 {
        return 1 as libc::c_int;
    }
    if (*g).flags & 0x80 as libc::c_int != 0 {
        stbi__gif_parse_colortable(
            s,
            ((*g).pal).as_mut_ptr(),
            (2 as libc::c_int) << ((*g).flags & 7 as libc::c_int),
            -(1 as libc::c_int),
        );
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__gif_test(mut s: *mut stbi__context) -> libc::c_int {
    let mut r: libc::c_int = stbi__gif_test_raw(s);
    stbi__rewind(s);
    return r;
}
unsafe extern "C" fn stbi__gif_test_raw(mut s: *mut stbi__context) -> libc::c_int {
    let mut sz: libc::c_int = 0;
    if stbi__get8(s) as libc::c_int != 'G' as i32
        || stbi__get8(s) as libc::c_int != 'I' as i32
        || stbi__get8(s) as libc::c_int != 'F' as i32
        || stbi__get8(s) as libc::c_int != '8' as i32
    {
        return 0 as libc::c_int;
    }
    sz = stbi__get8(s) as libc::c_int;
    if sz != '9' as i32 && sz != '7' as i32 {
        return 0 as libc::c_int;
    }
    if stbi__get8(s) as libc::c_int != 'a' as i32 {
        return 0 as libc::c_int;
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__bmp_load(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
    mut ri: *mut stbi__result_info,
) -> *mut libc::c_void {
    let mut out: *mut stbi_uc = 0 as *mut stbi_uc;
    let mut mr: libc::c_uint = 0 as libc::c_int as libc::c_uint;
    let mut mg: libc::c_uint = 0 as libc::c_int as libc::c_uint;
    let mut mb: libc::c_uint = 0 as libc::c_int as libc::c_uint;
    let mut ma: libc::c_uint = 0 as libc::c_int as libc::c_uint;
    let mut all_a: libc::c_uint = 0;
    let mut pal: [[stbi_uc; 4]; 256] = [[0; 4]; 256];
    let mut psize: libc::c_int = 0 as libc::c_int;
    let mut i: libc::c_int = 0;
    let mut j: libc::c_int = 0;
    let mut width: libc::c_int = 0;
    let mut flip_vertically: libc::c_int = 0;
    let mut pad: libc::c_int = 0;
    let mut target: libc::c_int = 0;
    let mut info: stbi__bmp_data = stbi__bmp_data {
        bpp: 0,
        offset: 0,
        hsz: 0,
        mr: 0,
        mg: 0,
        mb: 0,
        ma: 0,
        all_a: 0,
        extra_read: 0,
    };
    info.all_a = 255 as libc::c_int as libc::c_uint;
    if (stbi__bmp_parse_header(s, &mut info)).is_null() {
        return 0 as *mut libc::c_void;
    }
    flip_vertically = ((*s).img_y as libc::c_int > 0 as libc::c_int) as libc::c_int;
    (*s).img_y = abs((*s).img_y as libc::c_int) as stbi__uint32;
    if (*s).img_y > ((1 as libc::c_int) << 24 as libc::c_int) as libc::c_uint {
        return (if stbi__err(b"too large\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    if (*s).img_x > ((1 as libc::c_int) << 24 as libc::c_int) as libc::c_uint {
        return (if stbi__err(b"too large\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    mr = info.mr;
    mg = info.mg;
    mb = info.mb;
    ma = info.ma;
    all_a = info.all_a;
    if info.hsz == 12 as libc::c_int {
        if info.bpp < 24 as libc::c_int {
            psize = (info.offset - info.extra_read - 24 as libc::c_int)
                / 3 as libc::c_int;
        }
    } else if info.bpp < 16 as libc::c_int {
        psize = info.offset - info.extra_read - info.hsz >> 2 as libc::c_int;
    }
    if psize == 0 as libc::c_int {
        if info.offset as libc::c_long
            != (*s).callback_already_read as libc::c_long
                + ((*s).img_buffer).offset_from((*s).img_buffer_original) as libc::c_long
        {
            return (if stbi__err(b"bad offset\0" as *const u8 as *const libc::c_char)
                != 0
            {
                0 as *mut libc::c_void
            } else {
                0 as *mut libc::c_void
            }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
        }
    }
    if info.bpp == 24 as libc::c_int && ma == 0xff000000 as libc::c_uint {
        (*s).img_n = 3 as libc::c_int;
    } else {
        (*s).img_n = if ma != 0 { 4 as libc::c_int } else { 3 as libc::c_int };
    }
    if req_comp != 0 && req_comp >= 3 as libc::c_int {
        target = req_comp;
    } else {
        target = (*s).img_n;
    }
    if stbi__mad3sizes_valid(
        target,
        (*s).img_x as libc::c_int,
        (*s).img_y as libc::c_int,
        0 as libc::c_int,
    ) == 0
    {
        return (if stbi__err(b"too large\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    out = stbi__malloc_mad3(
        target,
        (*s).img_x as libc::c_int,
        (*s).img_y as libc::c_int,
        0 as libc::c_int,
    ) as *mut stbi_uc;
    if out.is_null() {
        return (if stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    if info.bpp < 16 as libc::c_int {
        let mut z: libc::c_int = 0 as libc::c_int;
        if psize == 0 as libc::c_int || psize > 256 as libc::c_int {
            free(out as *mut libc::c_void);
            return (if stbi__err(b"invalid\0" as *const u8 as *const libc::c_char) != 0 {
                0 as *mut libc::c_void
            } else {
                0 as *mut libc::c_void
            }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
        }
        i = 0 as libc::c_int;
        while i < psize {
            pal[i as usize][2 as libc::c_int as usize] = stbi__get8(s);
            pal[i as usize][1 as libc::c_int as usize] = stbi__get8(s);
            pal[i as usize][0 as libc::c_int as usize] = stbi__get8(s);
            if info.hsz != 12 as libc::c_int {
                stbi__get8(s);
            }
            pal[i as usize][3 as libc::c_int as usize] = 255 as libc::c_int as stbi_uc;
            i += 1;
        }
        stbi__skip(
            s,
            info.offset - info.extra_read - info.hsz
                - psize
                    * (if info.hsz == 12 as libc::c_int {
                        3 as libc::c_int
                    } else {
                        4 as libc::c_int
                    }),
        );
        if info.bpp == 1 as libc::c_int {
            width = (((*s).img_x).wrapping_add(7 as libc::c_int as libc::c_uint)
                >> 3 as libc::c_int) as libc::c_int;
        } else if info.bpp == 4 as libc::c_int {
            width = (((*s).img_x).wrapping_add(1 as libc::c_int as libc::c_uint)
                >> 1 as libc::c_int) as libc::c_int;
        } else if info.bpp == 8 as libc::c_int {
            width = (*s).img_x as libc::c_int;
        } else {
            free(out as *mut libc::c_void);
            return (if stbi__err(b"bad bpp\0" as *const u8 as *const libc::c_char) != 0 {
                0 as *mut libc::c_void
            } else {
                0 as *mut libc::c_void
            }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
        }
        pad = -width & 3 as libc::c_int;
        if info.bpp == 1 as libc::c_int {
            j = 0 as libc::c_int;
            while j < (*s).img_y as libc::c_int {
                let mut bit_offset: libc::c_int = 7 as libc::c_int;
                let mut v: libc::c_int = stbi__get8(s) as libc::c_int;
                i = 0 as libc::c_int;
                while i < (*s).img_x as libc::c_int {
                    let mut color: libc::c_int = v >> bit_offset & 0x1 as libc::c_int;
                    let fresh49 = z;
                    z = z + 1;
                    *out
                        .offset(
                            fresh49 as isize,
                        ) = pal[color as usize][0 as libc::c_int as usize];
                    let fresh50 = z;
                    z = z + 1;
                    *out
                        .offset(
                            fresh50 as isize,
                        ) = pal[color as usize][1 as libc::c_int as usize];
                    let fresh51 = z;
                    z = z + 1;
                    *out
                        .offset(
                            fresh51 as isize,
                        ) = pal[color as usize][2 as libc::c_int as usize];
                    if target == 4 as libc::c_int {
                        let fresh52 = z;
                        z = z + 1;
                        *out.offset(fresh52 as isize) = 255 as libc::c_int as stbi_uc;
                    }
                    if i + 1 as libc::c_int == (*s).img_x as libc::c_int {
                        break;
                    }
                    bit_offset -= 1;
                    if bit_offset < 0 as libc::c_int {
                        bit_offset = 7 as libc::c_int;
                        v = stbi__get8(s) as libc::c_int;
                    }
                    i += 1;
                }
                stbi__skip(s, pad);
                j += 1;
            }
        } else {
            j = 0 as libc::c_int;
            while j < (*s).img_y as libc::c_int {
                i = 0 as libc::c_int;
                while i < (*s).img_x as libc::c_int {
                    let mut v_0: libc::c_int = stbi__get8(s) as libc::c_int;
                    let mut v2: libc::c_int = 0 as libc::c_int;
                    if info.bpp == 4 as libc::c_int {
                        v2 = v_0 & 15 as libc::c_int;
                        v_0 >>= 4 as libc::c_int;
                    }
                    let fresh53 = z;
                    z = z + 1;
                    *out
                        .offset(
                            fresh53 as isize,
                        ) = pal[v_0 as usize][0 as libc::c_int as usize];
                    let fresh54 = z;
                    z = z + 1;
                    *out
                        .offset(
                            fresh54 as isize,
                        ) = pal[v_0 as usize][1 as libc::c_int as usize];
                    let fresh55 = z;
                    z = z + 1;
                    *out
                        .offset(
                            fresh55 as isize,
                        ) = pal[v_0 as usize][2 as libc::c_int as usize];
                    if target == 4 as libc::c_int {
                        let fresh56 = z;
                        z = z + 1;
                        *out.offset(fresh56 as isize) = 255 as libc::c_int as stbi_uc;
                    }
                    if i + 1 as libc::c_int == (*s).img_x as libc::c_int {
                        break;
                    }
                    v_0 = if info.bpp == 8 as libc::c_int {
                        stbi__get8(s) as libc::c_int
                    } else {
                        v2
                    };
                    let fresh57 = z;
                    z = z + 1;
                    *out
                        .offset(
                            fresh57 as isize,
                        ) = pal[v_0 as usize][0 as libc::c_int as usize];
                    let fresh58 = z;
                    z = z + 1;
                    *out
                        .offset(
                            fresh58 as isize,
                        ) = pal[v_0 as usize][1 as libc::c_int as usize];
                    let fresh59 = z;
                    z = z + 1;
                    *out
                        .offset(
                            fresh59 as isize,
                        ) = pal[v_0 as usize][2 as libc::c_int as usize];
                    if target == 4 as libc::c_int {
                        let fresh60 = z;
                        z = z + 1;
                        *out.offset(fresh60 as isize) = 255 as libc::c_int as stbi_uc;
                    }
                    i += 2 as libc::c_int;
                }
                stbi__skip(s, pad);
                j += 1;
            }
        }
    } else {
        let mut rshift: libc::c_int = 0 as libc::c_int;
        let mut gshift: libc::c_int = 0 as libc::c_int;
        let mut bshift: libc::c_int = 0 as libc::c_int;
        let mut ashift: libc::c_int = 0 as libc::c_int;
        let mut rcount: libc::c_int = 0 as libc::c_int;
        let mut gcount: libc::c_int = 0 as libc::c_int;
        let mut bcount: libc::c_int = 0 as libc::c_int;
        let mut acount: libc::c_int = 0 as libc::c_int;
        let mut z_0: libc::c_int = 0 as libc::c_int;
        let mut easy: libc::c_int = 0 as libc::c_int;
        stbi__skip(s, info.offset - info.extra_read - info.hsz);
        if info.bpp == 24 as libc::c_int {
            width = (3 as libc::c_int as libc::c_uint).wrapping_mul((*s).img_x)
                as libc::c_int;
        } else if info.bpp == 16 as libc::c_int {
            width = (2 as libc::c_int as libc::c_uint).wrapping_mul((*s).img_x)
                as libc::c_int;
        } else {
            width = 0 as libc::c_int;
        }
        pad = -width & 3 as libc::c_int;
        if info.bpp == 24 as libc::c_int {
            easy = 1 as libc::c_int;
        } else if info.bpp == 32 as libc::c_int {
            if mb == 0xff as libc::c_int as libc::c_uint
                && mg == 0xff00 as libc::c_int as libc::c_uint
                && mr == 0xff0000 as libc::c_int as libc::c_uint
                && ma == 0xff000000 as libc::c_uint
            {
                easy = 2 as libc::c_int;
            }
        }
        if easy == 0 {
            if mr == 0 || mg == 0 || mb == 0 {
                free(out as *mut libc::c_void);
                return (if stbi__err(b"bad masks\0" as *const u8 as *const libc::c_char)
                    != 0
                {
                    0 as *mut libc::c_void
                } else {
                    0 as *mut libc::c_void
                }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
            }
            rshift = stbi__high_bit(mr) - 7 as libc::c_int;
            rcount = stbi__bitcount(mr);
            gshift = stbi__high_bit(mg) - 7 as libc::c_int;
            gcount = stbi__bitcount(mg);
            bshift = stbi__high_bit(mb) - 7 as libc::c_int;
            bcount = stbi__bitcount(mb);
            ashift = stbi__high_bit(ma) - 7 as libc::c_int;
            acount = stbi__bitcount(ma);
            if rcount > 8 as libc::c_int || gcount > 8 as libc::c_int
                || bcount > 8 as libc::c_int || acount > 8 as libc::c_int
            {
                free(out as *mut libc::c_void);
                return (if stbi__err(b"bad masks\0" as *const u8 as *const libc::c_char)
                    != 0
                {
                    0 as *mut libc::c_void
                } else {
                    0 as *mut libc::c_void
                }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
            }
        }
        j = 0 as libc::c_int;
        while j < (*s).img_y as libc::c_int {
            if easy != 0 {
                i = 0 as libc::c_int;
                while i < (*s).img_x as libc::c_int {
                    let mut a: libc::c_uchar = 0;
                    *out.offset((z_0 + 2 as libc::c_int) as isize) = stbi__get8(s);
                    *out.offset((z_0 + 1 as libc::c_int) as isize) = stbi__get8(s);
                    *out.offset((z_0 + 0 as libc::c_int) as isize) = stbi__get8(s);
                    z_0 += 3 as libc::c_int;
                    a = (if easy == 2 as libc::c_int {
                        stbi__get8(s) as libc::c_int
                    } else {
                        255 as libc::c_int
                    }) as libc::c_uchar;
                    all_a |= a as libc::c_uint;
                    if target == 4 as libc::c_int {
                        let fresh61 = z_0;
                        z_0 = z_0 + 1;
                        *out.offset(fresh61 as isize) = a;
                    }
                    i += 1;
                }
            } else {
                let mut bpp: libc::c_int = info.bpp;
                i = 0 as libc::c_int;
                while i < (*s).img_x as libc::c_int {
                    let mut v_1: stbi__uint32 = if bpp == 16 as libc::c_int {
                        stbi__get16le(s) as stbi__uint32
                    } else {
                        stbi__get32le(s)
                    };
                    let mut a_0: libc::c_uint = 0;
                    let fresh62 = z_0;
                    z_0 = z_0 + 1;
                    *out
                        .offset(
                            fresh62 as isize,
                        ) = (stbi__shiftsigned(v_1 & mr, rshift, rcount)
                        & 255 as libc::c_int) as stbi_uc;
                    let fresh63 = z_0;
                    z_0 = z_0 + 1;
                    *out
                        .offset(
                            fresh63 as isize,
                        ) = (stbi__shiftsigned(v_1 & mg, gshift, gcount)
                        & 255 as libc::c_int) as stbi_uc;
                    let fresh64 = z_0;
                    z_0 = z_0 + 1;
                    *out
                        .offset(
                            fresh64 as isize,
                        ) = (stbi__shiftsigned(v_1 & mb, bshift, bcount)
                        & 255 as libc::c_int) as stbi_uc;
                    a_0 = (if ma != 0 {
                        stbi__shiftsigned(v_1 & ma, ashift, acount)
                    } else {
                        255 as libc::c_int
                    }) as libc::c_uint;
                    all_a |= a_0;
                    if target == 4 as libc::c_int {
                        let fresh65 = z_0;
                        z_0 = z_0 + 1;
                        *out
                            .offset(
                                fresh65 as isize,
                            ) = (a_0 & 255 as libc::c_int as libc::c_uint) as stbi_uc;
                    }
                    i += 1;
                }
            }
            stbi__skip(s, pad);
            j += 1;
        }
    }
    if target == 4 as libc::c_int && all_a == 0 as libc::c_int as libc::c_uint {
        i = (4 as libc::c_int as libc::c_uint)
            .wrapping_mul((*s).img_x)
            .wrapping_mul((*s).img_y)
            .wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
        while i >= 0 as libc::c_int {
            *out.offset(i as isize) = 255 as libc::c_int as stbi_uc;
            i -= 4 as libc::c_int;
        }
    }
    if flip_vertically != 0 {
        let mut t: stbi_uc = 0;
        j = 0 as libc::c_int;
        while j < (*s).img_y as libc::c_int >> 1 as libc::c_int {
            let mut p1: *mut stbi_uc = out
                .offset(
                    (j as libc::c_uint)
                        .wrapping_mul((*s).img_x)
                        .wrapping_mul(target as libc::c_uint) as isize,
                );
            let mut p2: *mut stbi_uc = out
                .offset(
                    ((*s).img_y)
                        .wrapping_sub(1 as libc::c_int as libc::c_uint)
                        .wrapping_sub(j as libc::c_uint)
                        .wrapping_mul((*s).img_x)
                        .wrapping_mul(target as libc::c_uint) as isize,
                );
            i = 0 as libc::c_int;
            while i < (*s).img_x as libc::c_int * target {
                t = *p1.offset(i as isize);
                *p1.offset(i as isize) = *p2.offset(i as isize);
                *p2.offset(i as isize) = t;
                i += 1;
            }
            j += 1;
        }
    }
    if req_comp != 0 && req_comp != target {
        out = stbi__convert_format(out, target, req_comp, (*s).img_x, (*s).img_y);
        if out.is_null() {
            return out as *mut libc::c_void;
        }
    }
    *x = (*s).img_x as libc::c_int;
    *y = (*s).img_y as libc::c_int;
    if !comp.is_null() {
        *comp = (*s).img_n;
    }
    return out as *mut libc::c_void;
}
unsafe extern "C" fn stbi__get32le(mut s: *mut stbi__context) -> stbi__uint32 {
    let mut z: stbi__uint32 = stbi__get16le(s) as stbi__uint32;
    z = (z as libc::c_uint)
        .wrapping_add((stbi__get16le(s) as stbi__uint32) << 16 as libc::c_int)
        as stbi__uint32 as stbi__uint32;
    return z;
}
unsafe extern "C" fn stbi__shiftsigned(
    mut v: libc::c_uint,
    mut shift: libc::c_int,
    mut bits: libc::c_int,
) -> libc::c_int {
    static mut mul_table: [libc::c_uint; 9] = [
        0 as libc::c_int as libc::c_uint,
        0xff as libc::c_int as libc::c_uint,
        0x55 as libc::c_int as libc::c_uint,
        0x49 as libc::c_int as libc::c_uint,
        0x11 as libc::c_int as libc::c_uint,
        0x21 as libc::c_int as libc::c_uint,
        0x41 as libc::c_int as libc::c_uint,
        0x81 as libc::c_int as libc::c_uint,
        0x1 as libc::c_int as libc::c_uint,
    ];
    static mut shift_table: [libc::c_uint; 9] = [
        0 as libc::c_int as libc::c_uint,
        0 as libc::c_int as libc::c_uint,
        0 as libc::c_int as libc::c_uint,
        1 as libc::c_int as libc::c_uint,
        0 as libc::c_int as libc::c_uint,
        2 as libc::c_int as libc::c_uint,
        4 as libc::c_int as libc::c_uint,
        6 as libc::c_int as libc::c_uint,
        0 as libc::c_int as libc::c_uint,
    ];
    if shift < 0 as libc::c_int {
        v <<= -shift;
    } else {
        v >>= shift;
    }
    v >>= 8 as libc::c_int - bits;
    return v.wrapping_mul(mul_table[bits as usize]) as libc::c_int
        >> shift_table[bits as usize];
}
unsafe extern "C" fn stbi__bitcount(mut a: libc::c_uint) -> libc::c_int {
    a = (a & 0x55555555 as libc::c_int as libc::c_uint)
        .wrapping_add(a >> 1 as libc::c_int & 0x55555555 as libc::c_int as libc::c_uint);
    a = (a & 0x33333333 as libc::c_int as libc::c_uint)
        .wrapping_add(a >> 2 as libc::c_int & 0x33333333 as libc::c_int as libc::c_uint);
    a = a.wrapping_add(a >> 4 as libc::c_int) & 0xf0f0f0f as libc::c_int as libc::c_uint;
    a = a.wrapping_add(a >> 8 as libc::c_int);
    a = a.wrapping_add(a >> 16 as libc::c_int);
    return (a & 0xff as libc::c_int as libc::c_uint) as libc::c_int;
}
unsafe extern "C" fn stbi__high_bit(mut z: libc::c_uint) -> libc::c_int {
    let mut n: libc::c_int = 0 as libc::c_int;
    if z == 0 as libc::c_int as libc::c_uint {
        return -(1 as libc::c_int);
    }
    if z >= 0x10000 as libc::c_int as libc::c_uint {
        n += 16 as libc::c_int;
        z >>= 16 as libc::c_int;
    }
    if z >= 0x100 as libc::c_int as libc::c_uint {
        n += 8 as libc::c_int;
        z >>= 8 as libc::c_int;
    }
    if z >= 0x10 as libc::c_int as libc::c_uint {
        n += 4 as libc::c_int;
        z >>= 4 as libc::c_int;
    }
    if z >= 0x4 as libc::c_int as libc::c_uint {
        n += 2 as libc::c_int;
        z >>= 2 as libc::c_int;
    }
    if z >= 0x2 as libc::c_int as libc::c_uint {
        n += 1 as libc::c_int;
    }
    return n;
}
unsafe extern "C" fn stbi__bmp_parse_header(
    mut s: *mut stbi__context,
    mut info: *mut stbi__bmp_data,
) -> *mut libc::c_void {
    let mut hsz: libc::c_int = 0;
    if stbi__get8(s) as libc::c_int != 'B' as i32
        || stbi__get8(s) as libc::c_int != 'M' as i32
    {
        return (if stbi__err(b"not BMP\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    stbi__get32le(s);
    stbi__get16le(s);
    stbi__get16le(s);
    (*info).offset = stbi__get32le(s) as libc::c_int;
    hsz = stbi__get32le(s) as libc::c_int;
    (*info).hsz = hsz;
    (*info).ma = 0 as libc::c_int as libc::c_uint;
    (*info).mb = (*info).ma;
    (*info).mg = (*info).mb;
    (*info).mr = (*info).mg;
    (*info).extra_read = 14 as libc::c_int;
    if (*info).offset < 0 as libc::c_int {
        return (if stbi__err(b"bad BMP\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    if hsz != 12 as libc::c_int && hsz != 40 as libc::c_int && hsz != 56 as libc::c_int
        && hsz != 108 as libc::c_int && hsz != 124 as libc::c_int
    {
        return (if stbi__err(b"unknown BMP\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    if hsz == 12 as libc::c_int {
        (*s).img_x = stbi__get16le(s) as stbi__uint32;
        (*s).img_y = stbi__get16le(s) as stbi__uint32;
    } else {
        (*s).img_x = stbi__get32le(s);
        (*s).img_y = stbi__get32le(s);
    }
    if stbi__get16le(s) != 1 as libc::c_int {
        return (if stbi__err(b"bad BMP\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    (*info).bpp = stbi__get16le(s);
    if hsz != 12 as libc::c_int {
        let mut compress: libc::c_int = stbi__get32le(s) as libc::c_int;
        if compress == 1 as libc::c_int || compress == 2 as libc::c_int {
            return (if stbi__err(b"BMP RLE\0" as *const u8 as *const libc::c_char) != 0 {
                0 as *mut libc::c_void
            } else {
                0 as *mut libc::c_void
            }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
        }
        if compress >= 4 as libc::c_int {
            return (if stbi__err(b"BMP JPEG/PNG\0" as *const u8 as *const libc::c_char)
                != 0
            {
                0 as *mut libc::c_void
            } else {
                0 as *mut libc::c_void
            }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
        }
        if compress == 3 as libc::c_int && (*info).bpp != 16 as libc::c_int
            && (*info).bpp != 32 as libc::c_int
        {
            return (if stbi__err(b"bad BMP\0" as *const u8 as *const libc::c_char) != 0 {
                0 as *mut libc::c_void
            } else {
                0 as *mut libc::c_void
            }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
        }
        stbi__get32le(s);
        stbi__get32le(s);
        stbi__get32le(s);
        stbi__get32le(s);
        stbi__get32le(s);
        if hsz == 40 as libc::c_int || hsz == 56 as libc::c_int {
            if hsz == 56 as libc::c_int {
                stbi__get32le(s);
                stbi__get32le(s);
                stbi__get32le(s);
                stbi__get32le(s);
            }
            if (*info).bpp == 16 as libc::c_int || (*info).bpp == 32 as libc::c_int {
                if compress == 0 as libc::c_int {
                    stbi__bmp_set_mask_defaults(info, compress);
                } else if compress == 3 as libc::c_int {
                    (*info).mr = stbi__get32le(s);
                    (*info).mg = stbi__get32le(s);
                    (*info).mb = stbi__get32le(s);
                    (*info).extra_read += 12 as libc::c_int;
                    if (*info).mr == (*info).mg && (*info).mg == (*info).mb {
                        return (if stbi__err(
                            b"bad BMP\0" as *const u8 as *const libc::c_char,
                        ) != 0
                        {
                            0 as *mut libc::c_void
                        } else {
                            0 as *mut libc::c_void
                        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
                    }
                } else {
                    return (if stbi__err(
                        b"bad BMP\0" as *const u8 as *const libc::c_char,
                    ) != 0
                    {
                        0 as *mut libc::c_void
                    } else {
                        0 as *mut libc::c_void
                    }) as size_t as *mut libc::c_uchar as *mut libc::c_void
                }
            }
        } else {
            let mut i: libc::c_int = 0;
            if hsz != 108 as libc::c_int && hsz != 124 as libc::c_int {
                return (if stbi__err(b"bad BMP\0" as *const u8 as *const libc::c_char)
                    != 0
                {
                    0 as *mut libc::c_void
                } else {
                    0 as *mut libc::c_void
                }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
            }
            (*info).mr = stbi__get32le(s);
            (*info).mg = stbi__get32le(s);
            (*info).mb = stbi__get32le(s);
            (*info).ma = stbi__get32le(s);
            if compress != 3 as libc::c_int {
                stbi__bmp_set_mask_defaults(info, compress);
            }
            stbi__get32le(s);
            i = 0 as libc::c_int;
            while i < 12 as libc::c_int {
                stbi__get32le(s);
                i += 1;
            }
            if hsz == 124 as libc::c_int {
                stbi__get32le(s);
                stbi__get32le(s);
                stbi__get32le(s);
                stbi__get32le(s);
            }
        }
    }
    return 1 as libc::c_int as *mut libc::c_void;
}
unsafe extern "C" fn stbi__bmp_set_mask_defaults(
    mut info: *mut stbi__bmp_data,
    mut compress: libc::c_int,
) -> libc::c_int {
    if compress == 3 as libc::c_int {
        return 1 as libc::c_int;
    }
    if compress == 0 as libc::c_int {
        if (*info).bpp == 16 as libc::c_int {
            (*info).mr = (31 as libc::c_uint) << 10 as libc::c_int;
            (*info).mg = (31 as libc::c_uint) << 5 as libc::c_int;
            (*info).mb = (31 as libc::c_uint) << 0 as libc::c_int;
        } else if (*info).bpp == 32 as libc::c_int {
            (*info).mr = (0xff as libc::c_uint) << 16 as libc::c_int;
            (*info).mg = (0xff as libc::c_uint) << 8 as libc::c_int;
            (*info).mb = (0xff as libc::c_uint) << 0 as libc::c_int;
            (*info).ma = (0xff as libc::c_uint) << 24 as libc::c_int;
            (*info).all_a = 0 as libc::c_int as libc::c_uint;
        } else {
            (*info).ma = 0 as libc::c_int as libc::c_uint;
            (*info).mb = (*info).ma;
            (*info).mg = (*info).mb;
            (*info).mr = (*info).mg;
        }
        return 1 as libc::c_int;
    }
    return 0 as libc::c_int;
}
unsafe extern "C" fn stbi__bmp_test(mut s: *mut stbi__context) -> libc::c_int {
    let mut r: libc::c_int = stbi__bmp_test_raw(s);
    stbi__rewind(s);
    return r;
}
unsafe extern "C" fn stbi__bmp_test_raw(mut s: *mut stbi__context) -> libc::c_int {
    let mut r: libc::c_int = 0;
    let mut sz: libc::c_int = 0;
    if stbi__get8(s) as libc::c_int != 'B' as i32 {
        return 0 as libc::c_int;
    }
    if stbi__get8(s) as libc::c_int != 'M' as i32 {
        return 0 as libc::c_int;
    }
    stbi__get32le(s);
    stbi__get16le(s);
    stbi__get16le(s);
    stbi__get32le(s);
    sz = stbi__get32le(s) as libc::c_int;
    r = (sz == 12 as libc::c_int || sz == 40 as libc::c_int || sz == 56 as libc::c_int
        || sz == 108 as libc::c_int || sz == 124 as libc::c_int) as libc::c_int;
    return r;
}
unsafe extern "C" fn stbi__png_load(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
    mut ri: *mut stbi__result_info,
) -> *mut libc::c_void {
    let mut p: stbi__png = stbi__png {
        s: 0 as *mut stbi__context,
        idata: 0 as *mut stbi_uc,
        expanded: 0 as *mut stbi_uc,
        out: 0 as *mut stbi_uc,
        depth: 0,
    };
    p.s = s;
    return stbi__do_png(&mut p, x, y, comp, req_comp, ri);
}
unsafe extern "C" fn stbi__do_png(
    mut p: *mut stbi__png,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut n: *mut libc::c_int,
    mut req_comp: libc::c_int,
    mut ri: *mut stbi__result_info,
) -> *mut libc::c_void {
    let mut result: *mut libc::c_void = 0 as *mut libc::c_void;
    if req_comp < 0 as libc::c_int || req_comp > 4 as libc::c_int {
        return (if stbi__err(b"bad req_comp\0" as *const u8 as *const libc::c_char) != 0
        {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
    }
    if stbi__parse_png_file(p, STBI__SCAN_load as libc::c_int, req_comp) != 0 {
        if (*p).depth <= 8 as libc::c_int {
            (*ri).bits_per_channel = 8 as libc::c_int;
        } else if (*p).depth == 16 as libc::c_int {
            (*ri).bits_per_channel = 16 as libc::c_int;
        } else {
            return (if stbi__err(
                b"bad bits_per_channel\0" as *const u8 as *const libc::c_char,
            ) != 0
            {
                0 as *mut libc::c_void
            } else {
                0 as *mut libc::c_void
            }) as size_t as *mut libc::c_uchar as *mut libc::c_void
        }
        result = (*p).out as *mut libc::c_void;
        (*p).out = 0 as *mut stbi_uc;
        if req_comp != 0 && req_comp != (*(*p).s).img_out_n {
            if (*ri).bits_per_channel == 8 as libc::c_int {
                result = stbi__convert_format(
                    result as *mut libc::c_uchar,
                    (*(*p).s).img_out_n,
                    req_comp,
                    (*(*p).s).img_x,
                    (*(*p).s).img_y,
                ) as *mut libc::c_void;
            } else {
                result = stbi__convert_format16(
                    result as *mut stbi__uint16,
                    (*(*p).s).img_out_n,
                    req_comp,
                    (*(*p).s).img_x,
                    (*(*p).s).img_y,
                ) as *mut libc::c_void;
            }
            (*(*p).s).img_out_n = req_comp;
            if result.is_null() {
                return result;
            }
        }
        *x = (*(*p).s).img_x as libc::c_int;
        *y = (*(*p).s).img_y as libc::c_int;
        if !n.is_null() {
            *n = (*(*p).s).img_n;
        }
    }
    free((*p).out as *mut libc::c_void);
    (*p).out = 0 as *mut stbi_uc;
    free((*p).expanded as *mut libc::c_void);
    (*p).expanded = 0 as *mut stbi_uc;
    free((*p).idata as *mut libc::c_void);
    (*p).idata = 0 as *mut stbi_uc;
    return result;
}
unsafe extern "C" fn stbi__parse_png_file(
    mut z: *mut stbi__png,
    mut scan: libc::c_int,
    mut req_comp: libc::c_int,
) -> libc::c_int {
    let mut palette: [stbi_uc; 1024] = [0; 1024];
    let mut pal_img_n: stbi_uc = 0 as libc::c_int as stbi_uc;
    let mut has_trans: stbi_uc = 0 as libc::c_int as stbi_uc;
    let mut tc: [stbi_uc; 3] = [0 as libc::c_int as stbi_uc, 0, 0];
    let mut tc16: [stbi__uint16; 3] = [0; 3];
    let mut ioff: stbi__uint32 = 0 as libc::c_int as stbi__uint32;
    let mut idata_limit: stbi__uint32 = 0 as libc::c_int as stbi__uint32;
    let mut i: stbi__uint32 = 0;
    let mut pal_len: stbi__uint32 = 0 as libc::c_int as stbi__uint32;
    let mut first: libc::c_int = 1 as libc::c_int;
    let mut k: libc::c_int = 0;
    let mut interlace: libc::c_int = 0 as libc::c_int;
    let mut color: libc::c_int = 0 as libc::c_int;
    let mut is_iphone: libc::c_int = 0 as libc::c_int;
    let mut s: *mut stbi__context = (*z).s;
    (*z).expanded = 0 as *mut stbi_uc;
    (*z).idata = 0 as *mut stbi_uc;
    (*z).out = 0 as *mut stbi_uc;
    if stbi__check_png_header(s) == 0 {
        return 0 as libc::c_int;
    }
    if scan == STBI__SCAN_type as libc::c_int {
        return 1 as libc::c_int;
    }
    loop {
        let mut c: stbi__pngchunk = stbi__get_chunk_header(s);
        match c.type_0 {
            1130840649 => {
                is_iphone = 1 as libc::c_int;
                stbi__skip(s, c.length as libc::c_int);
            }
            1229472850 => {
                let mut comp: libc::c_int = 0;
                let mut filter: libc::c_int = 0;
                if first == 0 {
                    return stbi__err(
                        b"multiple IHDR\0" as *const u8 as *const libc::c_char,
                    );
                }
                first = 0 as libc::c_int;
                if c.length != 13 as libc::c_int as libc::c_uint {
                    return stbi__err(
                        b"bad IHDR len\0" as *const u8 as *const libc::c_char,
                    );
                }
                (*s).img_x = stbi__get32be(s);
                (*s).img_y = stbi__get32be(s);
                if (*s).img_y > ((1 as libc::c_int) << 24 as libc::c_int) as libc::c_uint
                {
                    return stbi__err(b"too large\0" as *const u8 as *const libc::c_char);
                }
                if (*s).img_x > ((1 as libc::c_int) << 24 as libc::c_int) as libc::c_uint
                {
                    return stbi__err(b"too large\0" as *const u8 as *const libc::c_char);
                }
                (*z).depth = stbi__get8(s) as libc::c_int;
                if (*z).depth != 1 as libc::c_int && (*z).depth != 2 as libc::c_int
                    && (*z).depth != 4 as libc::c_int && (*z).depth != 8 as libc::c_int
                    && (*z).depth != 16 as libc::c_int
                {
                    return stbi__err(
                        b"1/2/4/8/16-bit only\0" as *const u8 as *const libc::c_char,
                    );
                }
                color = stbi__get8(s) as libc::c_int;
                if color > 6 as libc::c_int {
                    return stbi__err(b"bad ctype\0" as *const u8 as *const libc::c_char);
                }
                if color == 3 as libc::c_int && (*z).depth == 16 as libc::c_int {
                    return stbi__err(b"bad ctype\0" as *const u8 as *const libc::c_char);
                }
                if color == 3 as libc::c_int {
                    pal_img_n = 3 as libc::c_int as stbi_uc;
                } else if color & 1 as libc::c_int != 0 {
                    return stbi__err(b"bad ctype\0" as *const u8 as *const libc::c_char)
                }
                comp = stbi__get8(s) as libc::c_int;
                if comp != 0 {
                    return stbi__err(
                        b"bad comp method\0" as *const u8 as *const libc::c_char,
                    );
                }
                filter = stbi__get8(s) as libc::c_int;
                if filter != 0 {
                    return stbi__err(
                        b"bad filter method\0" as *const u8 as *const libc::c_char,
                    );
                }
                interlace = stbi__get8(s) as libc::c_int;
                if interlace > 1 as libc::c_int {
                    return stbi__err(
                        b"bad interlace method\0" as *const u8 as *const libc::c_char,
                    );
                }
                if (*s).img_x == 0 || (*s).img_y == 0 {
                    return stbi__err(
                        b"0-pixel image\0" as *const u8 as *const libc::c_char,
                    );
                }
                if pal_img_n == 0 {
                    (*s)
                        .img_n = (if color & 2 as libc::c_int != 0 {
                        3 as libc::c_int
                    } else {
                        1 as libc::c_int
                    })
                        + (if color & 4 as libc::c_int != 0 {
                            1 as libc::c_int
                        } else {
                            0 as libc::c_int
                        });
                    if (((1 as libc::c_int) << 30 as libc::c_int) as libc::c_uint)
                        .wrapping_div((*s).img_x)
                        .wrapping_div((*s).img_n as libc::c_uint) < (*s).img_y
                    {
                        return stbi__err(
                            b"too large\0" as *const u8 as *const libc::c_char,
                        );
                    }
                    if scan == STBI__SCAN_header as libc::c_int {
                        return 1 as libc::c_int;
                    }
                } else {
                    (*s).img_n = 1 as libc::c_int;
                    if (((1 as libc::c_int) << 30 as libc::c_int) as libc::c_uint)
                        .wrapping_div((*s).img_x)
                        .wrapping_div(4 as libc::c_int as libc::c_uint) < (*s).img_y
                    {
                        return stbi__err(
                            b"too large\0" as *const u8 as *const libc::c_char,
                        );
                    }
                }
            }
            1347179589 => {
                if first != 0 {
                    return stbi__err(
                        b"first not IHDR\0" as *const u8 as *const libc::c_char,
                    );
                }
                if c.length > (256 as libc::c_int * 3 as libc::c_int) as libc::c_uint {
                    return stbi__err(
                        b"invalid PLTE\0" as *const u8 as *const libc::c_char,
                    );
                }
                pal_len = (c.length).wrapping_div(3 as libc::c_int as libc::c_uint);
                if pal_len.wrapping_mul(3 as libc::c_int as libc::c_uint) != c.length {
                    return stbi__err(
                        b"invalid PLTE\0" as *const u8 as *const libc::c_char,
                    );
                }
                i = 0 as libc::c_int as stbi__uint32;
                while i < pal_len {
                    palette[i
                        .wrapping_mul(4 as libc::c_int as libc::c_uint)
                        .wrapping_add(0 as libc::c_int as libc::c_uint)
                        as usize] = stbi__get8(s);
                    palette[i
                        .wrapping_mul(4 as libc::c_int as libc::c_uint)
                        .wrapping_add(1 as libc::c_int as libc::c_uint)
                        as usize] = stbi__get8(s);
                    palette[i
                        .wrapping_mul(4 as libc::c_int as libc::c_uint)
                        .wrapping_add(2 as libc::c_int as libc::c_uint)
                        as usize] = stbi__get8(s);
                    palette[i
                        .wrapping_mul(4 as libc::c_int as libc::c_uint)
                        .wrapping_add(3 as libc::c_int as libc::c_uint)
                        as usize] = 255 as libc::c_int as stbi_uc;
                    i = i.wrapping_add(1);
                }
            }
            1951551059 => {
                if first != 0 {
                    return stbi__err(
                        b"first not IHDR\0" as *const u8 as *const libc::c_char,
                    );
                }
                if !((*z).idata).is_null() {
                    return stbi__err(
                        b"tRNS after IDAT\0" as *const u8 as *const libc::c_char,
                    );
                }
                if pal_img_n != 0 {
                    if scan == STBI__SCAN_header as libc::c_int {
                        (*s).img_n = 4 as libc::c_int;
                        return 1 as libc::c_int;
                    }
                    if pal_len == 0 as libc::c_int as libc::c_uint {
                        return stbi__err(
                            b"tRNS before PLTE\0" as *const u8 as *const libc::c_char,
                        );
                    }
                    if c.length > pal_len {
                        return stbi__err(
                            b"bad tRNS len\0" as *const u8 as *const libc::c_char,
                        );
                    }
                    pal_img_n = 4 as libc::c_int as stbi_uc;
                    i = 0 as libc::c_int as stbi__uint32;
                    while i < c.length {
                        palette[i
                            .wrapping_mul(4 as libc::c_int as libc::c_uint)
                            .wrapping_add(3 as libc::c_int as libc::c_uint)
                            as usize] = stbi__get8(s);
                        i = i.wrapping_add(1);
                    }
                } else {
                    if (*s).img_n & 1 as libc::c_int == 0 {
                        return stbi__err(
                            b"tRNS with alpha\0" as *const u8 as *const libc::c_char,
                        );
                    }
                    if c.length
                        != ((*s).img_n as stbi__uint32)
                            .wrapping_mul(2 as libc::c_int as libc::c_uint)
                    {
                        return stbi__err(
                            b"bad tRNS len\0" as *const u8 as *const libc::c_char,
                        );
                    }
                    has_trans = 1 as libc::c_int as stbi_uc;
                    if (*z).depth == 16 as libc::c_int {
                        k = 0 as libc::c_int;
                        while k < (*s).img_n {
                            tc16[k as usize] = stbi__get16be(s) as stbi__uint16;
                            k += 1;
                        }
                    } else {
                        k = 0 as libc::c_int;
                        while k < (*s).img_n {
                            tc[k
                                as usize] = ((stbi__get16be(s) & 255 as libc::c_int)
                                as stbi_uc as libc::c_int
                                * stbi__depth_scale_table[(*z).depth as usize]
                                    as libc::c_int) as stbi_uc;
                            k += 1;
                        }
                    }
                }
            }
            1229209940 => {
                if first != 0 {
                    return stbi__err(
                        b"first not IHDR\0" as *const u8 as *const libc::c_char,
                    );
                }
                if pal_img_n as libc::c_int != 0 && pal_len == 0 {
                    return stbi__err(b"no PLTE\0" as *const u8 as *const libc::c_char);
                }
                if scan == STBI__SCAN_header as libc::c_int {
                    (*s).img_n = pal_img_n as libc::c_int;
                    return 1 as libc::c_int;
                }
                if (ioff.wrapping_add(c.length) as libc::c_int) < ioff as libc::c_int {
                    return 0 as libc::c_int;
                }
                if ioff.wrapping_add(c.length) > idata_limit {
                    let mut idata_limit_old: stbi__uint32 = idata_limit;
                    let mut p: *mut stbi_uc = 0 as *mut stbi_uc;
                    if idata_limit == 0 as libc::c_int as libc::c_uint {
                        idata_limit = if c.length > 4096 as libc::c_int as libc::c_uint {
                            c.length
                        } else {
                            4096 as libc::c_int as libc::c_uint
                        };
                    }
                    while ioff.wrapping_add(c.length) > idata_limit {
                        idata_limit = (idata_limit as libc::c_uint)
                            .wrapping_mul(2 as libc::c_int as libc::c_uint)
                            as stbi__uint32 as stbi__uint32;
                    }
                    p = realloc(
                        (*z).idata as *mut libc::c_void,
                        idata_limit as libc::c_ulong,
                    ) as *mut stbi_uc;
                    if p.is_null() {
                        return stbi__err(
                            b"outofmem\0" as *const u8 as *const libc::c_char,
                        );
                    }
                    (*z).idata = p;
                }
                if stbi__getn(
                    s,
                    ((*z).idata).offset(ioff as isize),
                    c.length as libc::c_int,
                ) == 0
                {
                    return stbi__err(b"outofdata\0" as *const u8 as *const libc::c_char);
                }
                ioff = (ioff as libc::c_uint).wrapping_add(c.length) as stbi__uint32
                    as stbi__uint32;
            }
            1229278788 => {
                let mut raw_len: stbi__uint32 = 0;
                let mut bpl: stbi__uint32 = 0;
                if first != 0 {
                    return stbi__err(
                        b"first not IHDR\0" as *const u8 as *const libc::c_char,
                    );
                }
                if scan != STBI__SCAN_load as libc::c_int {
                    return 1 as libc::c_int;
                }
                if ((*z).idata).is_null() {
                    return stbi__err(b"no IDAT\0" as *const u8 as *const libc::c_char);
                }
                bpl = ((*s).img_x)
                    .wrapping_mul((*z).depth as libc::c_uint)
                    .wrapping_add(7 as libc::c_int as libc::c_uint)
                    .wrapping_div(8 as libc::c_int as libc::c_uint);
                raw_len = bpl
                    .wrapping_mul((*s).img_y)
                    .wrapping_mul((*s).img_n as libc::c_uint)
                    .wrapping_add((*s).img_y);
                (*z)
                    .expanded = stbi_zlib_decode_malloc_guesssize_headerflag(
                    (*z).idata as *mut libc::c_char,
                    ioff as libc::c_int,
                    raw_len as libc::c_int,
                    &mut raw_len as *mut stbi__uint32 as *mut libc::c_int,
                    (is_iphone == 0) as libc::c_int,
                ) as *mut stbi_uc;
                if ((*z).expanded).is_null() {
                    return 0 as libc::c_int;
                }
                free((*z).idata as *mut libc::c_void);
                (*z).idata = 0 as *mut stbi_uc;
                if req_comp == (*s).img_n + 1 as libc::c_int
                    && req_comp != 3 as libc::c_int && pal_img_n == 0
                    || has_trans as libc::c_int != 0
                {
                    (*s).img_out_n = (*s).img_n + 1 as libc::c_int;
                } else {
                    (*s).img_out_n = (*s).img_n;
                }
                if stbi__create_png_image(
                    z,
                    (*z).expanded,
                    raw_len,
                    (*s).img_out_n,
                    (*z).depth,
                    color,
                    interlace,
                ) == 0
                {
                    return 0 as libc::c_int;
                }
                if has_trans != 0 {
                    if (*z).depth == 16 as libc::c_int {
                        if stbi__compute_transparency16(
                            z,
                            tc16.as_mut_ptr(),
                            (*s).img_out_n,
                        ) == 0
                        {
                            return 0 as libc::c_int;
                        }
                    } else if stbi__compute_transparency(
                        z,
                        tc.as_mut_ptr(),
                        (*s).img_out_n,
                    ) == 0
                    {
                        return 0 as libc::c_int
                    }
                }
                if is_iphone != 0
                    && (if stbi__de_iphone_flag_set != 0 {
                        stbi__de_iphone_flag_local
                    } else {
                        stbi__de_iphone_flag_global
                    }) != 0 && (*s).img_out_n > 2 as libc::c_int
                {
                    stbi__de_iphone(z);
                }
                if pal_img_n != 0 {
                    (*s).img_n = pal_img_n as libc::c_int;
                    (*s).img_out_n = pal_img_n as libc::c_int;
                    if req_comp >= 3 as libc::c_int {
                        (*s).img_out_n = req_comp;
                    }
                    if stbi__expand_png_palette(
                        z,
                        palette.as_mut_ptr(),
                        pal_len as libc::c_int,
                        (*s).img_out_n,
                    ) == 0
                    {
                        return 0 as libc::c_int;
                    }
                } else if has_trans != 0 {
                    (*s).img_n += 1;
                }
                free((*z).expanded as *mut libc::c_void);
                (*z).expanded = 0 as *mut stbi_uc;
                stbi__get32be(s);
                return 1 as libc::c_int;
            }
            _ => {
                if first != 0 {
                    return stbi__err(
                        b"first not IHDR\0" as *const u8 as *const libc::c_char,
                    );
                }
                if c.type_0 & ((1 as libc::c_int) << 29 as libc::c_int) as libc::c_uint
                    == 0 as libc::c_int as libc::c_uint
                {
                    static mut invalid_chunk: [libc::c_char; 25] = unsafe {
                        *::core::mem::transmute::<
                            &[u8; 25],
                            &mut [libc::c_char; 25],
                        >(b"XXXX PNG chunk not known\0")
                    };
                    invalid_chunk[0 as libc::c_int
                        as usize] = (c.type_0 >> 24 as libc::c_int
                        & 255 as libc::c_int as libc::c_uint) as stbi_uc as libc::c_char;
                    invalid_chunk[1 as libc::c_int
                        as usize] = (c.type_0 >> 16 as libc::c_int
                        & 255 as libc::c_int as libc::c_uint) as stbi_uc as libc::c_char;
                    invalid_chunk[2 as libc::c_int
                        as usize] = (c.type_0 >> 8 as libc::c_int
                        & 255 as libc::c_int as libc::c_uint) as stbi_uc as libc::c_char;
                    invalid_chunk[3 as libc::c_int
                        as usize] = (c.type_0 >> 0 as libc::c_int
                        & 255 as libc::c_int as libc::c_uint) as stbi_uc as libc::c_char;
                    return stbi__err(invalid_chunk.as_mut_ptr());
                }
                stbi__skip(s, c.length as libc::c_int);
            }
        }
        stbi__get32be(s);
    };
}
unsafe extern "C" fn stbi__get_chunk_header(
    mut s: *mut stbi__context,
) -> stbi__pngchunk {
    let mut c: stbi__pngchunk = stbi__pngchunk {
        length: 0,
        type_0: 0,
    };
    c.length = stbi__get32be(s);
    c.type_0 = stbi__get32be(s);
    return c;
}
unsafe extern "C" fn stbi__expand_png_palette(
    mut a: *mut stbi__png,
    mut palette: *mut stbi_uc,
    mut len: libc::c_int,
    mut pal_img_n: libc::c_int,
) -> libc::c_int {
    let mut i: stbi__uint32 = 0;
    let mut pixel_count: stbi__uint32 = ((*(*a).s).img_x).wrapping_mul((*(*a).s).img_y);
    let mut p: *mut stbi_uc = 0 as *mut stbi_uc;
    let mut temp_out: *mut stbi_uc = 0 as *mut stbi_uc;
    let mut orig: *mut stbi_uc = (*a).out;
    p = stbi__malloc_mad2(pixel_count as libc::c_int, pal_img_n, 0 as libc::c_int)
        as *mut stbi_uc;
    if p.is_null() {
        return stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char);
    }
    temp_out = p;
    if pal_img_n == 3 as libc::c_int {
        i = 0 as libc::c_int as stbi__uint32;
        while i < pixel_count {
            let mut n: libc::c_int = *orig.offset(i as isize) as libc::c_int
                * 4 as libc::c_int;
            *p.offset(0 as libc::c_int as isize) = *palette.offset(n as isize);
            *p
                .offset(
                    1 as libc::c_int as isize,
                ) = *palette.offset((n + 1 as libc::c_int) as isize);
            *p
                .offset(
                    2 as libc::c_int as isize,
                ) = *palette.offset((n + 2 as libc::c_int) as isize);
            p = p.offset(3 as libc::c_int as isize);
            i = i.wrapping_add(1);
        }
    } else {
        i = 0 as libc::c_int as stbi__uint32;
        while i < pixel_count {
            let mut n_0: libc::c_int = *orig.offset(i as isize) as libc::c_int
                * 4 as libc::c_int;
            *p.offset(0 as libc::c_int as isize) = *palette.offset(n_0 as isize);
            *p
                .offset(
                    1 as libc::c_int as isize,
                ) = *palette.offset((n_0 + 1 as libc::c_int) as isize);
            *p
                .offset(
                    2 as libc::c_int as isize,
                ) = *palette.offset((n_0 + 2 as libc::c_int) as isize);
            *p
                .offset(
                    3 as libc::c_int as isize,
                ) = *palette.offset((n_0 + 3 as libc::c_int) as isize);
            p = p.offset(4 as libc::c_int as isize);
            i = i.wrapping_add(1);
        }
    }
    free((*a).out as *mut libc::c_void);
    (*a).out = temp_out;
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__de_iphone(mut z: *mut stbi__png) {
    let mut s: *mut stbi__context = (*z).s;
    let mut i: stbi__uint32 = 0;
    let mut pixel_count: stbi__uint32 = ((*s).img_x).wrapping_mul((*s).img_y);
    let mut p: *mut stbi_uc = (*z).out;
    if (*s).img_out_n == 3 as libc::c_int {
        i = 0 as libc::c_int as stbi__uint32;
        while i < pixel_count {
            let mut t: stbi_uc = *p.offset(0 as libc::c_int as isize);
            *p.offset(0 as libc::c_int as isize) = *p.offset(2 as libc::c_int as isize);
            *p.offset(2 as libc::c_int as isize) = t;
            p = p.offset(3 as libc::c_int as isize);
            i = i.wrapping_add(1);
        }
    } else if if stbi__unpremultiply_on_load_set != 0 {
        stbi__unpremultiply_on_load_local
    } else {
        stbi__unpremultiply_on_load_global
    } != 0
    {
        i = 0 as libc::c_int as stbi__uint32;
        while i < pixel_count {
            let mut a: stbi_uc = *p.offset(3 as libc::c_int as isize);
            let mut t_0: stbi_uc = *p.offset(0 as libc::c_int as isize);
            if a != 0 {
                let mut half: stbi_uc = (a as libc::c_int / 2 as libc::c_int) as stbi_uc;
                *p
                    .offset(
                        0 as libc::c_int as isize,
                    ) = ((*p.offset(2 as libc::c_int as isize) as libc::c_int
                    * 255 as libc::c_int + half as libc::c_int) / a as libc::c_int)
                    as stbi_uc;
                *p
                    .offset(
                        1 as libc::c_int as isize,
                    ) = ((*p.offset(1 as libc::c_int as isize) as libc::c_int
                    * 255 as libc::c_int + half as libc::c_int) / a as libc::c_int)
                    as stbi_uc;
                *p
                    .offset(
                        2 as libc::c_int as isize,
                    ) = ((t_0 as libc::c_int * 255 as libc::c_int + half as libc::c_int)
                    / a as libc::c_int) as stbi_uc;
            } else {
                *p
                    .offset(
                        0 as libc::c_int as isize,
                    ) = *p.offset(2 as libc::c_int as isize);
                *p.offset(2 as libc::c_int as isize) = t_0;
            }
            p = p.offset(4 as libc::c_int as isize);
            i = i.wrapping_add(1);
        }
    } else {
        i = 0 as libc::c_int as stbi__uint32;
        while i < pixel_count {
            let mut t_1: stbi_uc = *p.offset(0 as libc::c_int as isize);
            *p.offset(0 as libc::c_int as isize) = *p.offset(2 as libc::c_int as isize);
            *p.offset(2 as libc::c_int as isize) = t_1;
            p = p.offset(4 as libc::c_int as isize);
            i = i.wrapping_add(1);
        }
    };
}
static mut stbi__unpremultiply_on_load_global: libc::c_int = 0 as libc::c_int;
#[thread_local]
static mut stbi__unpremultiply_on_load_local: libc::c_int = 0;
#[thread_local]
static mut stbi__unpremultiply_on_load_set: libc::c_int = 0;
static mut stbi__de_iphone_flag_global: libc::c_int = 0 as libc::c_int;
#[thread_local]
static mut stbi__de_iphone_flag_local: libc::c_int = 0;
#[thread_local]
static mut stbi__de_iphone_flag_set: libc::c_int = 0;
unsafe extern "C" fn stbi__compute_transparency(
    mut z: *mut stbi__png,
    mut tc: *mut stbi_uc,
    mut out_n: libc::c_int,
) -> libc::c_int {
    let mut s: *mut stbi__context = (*z).s;
    let mut i: stbi__uint32 = 0;
    let mut pixel_count: stbi__uint32 = ((*s).img_x).wrapping_mul((*s).img_y);
    let mut p: *mut stbi_uc = (*z).out;
    if out_n == 2 as libc::c_int {
        i = 0 as libc::c_int as stbi__uint32;
        while i < pixel_count {
            *p
                .offset(
                    1 as libc::c_int as isize,
                ) = (if *p.offset(0 as libc::c_int as isize) as libc::c_int
                == *tc.offset(0 as libc::c_int as isize) as libc::c_int
            {
                0 as libc::c_int
            } else {
                255 as libc::c_int
            }) as stbi_uc;
            p = p.offset(2 as libc::c_int as isize);
            i = i.wrapping_add(1);
        }
    } else {
        i = 0 as libc::c_int as stbi__uint32;
        while i < pixel_count {
            if *p.offset(0 as libc::c_int as isize) as libc::c_int
                == *tc.offset(0 as libc::c_int as isize) as libc::c_int
                && *p.offset(1 as libc::c_int as isize) as libc::c_int
                    == *tc.offset(1 as libc::c_int as isize) as libc::c_int
                && *p.offset(2 as libc::c_int as isize) as libc::c_int
                    == *tc.offset(2 as libc::c_int as isize) as libc::c_int
            {
                *p.offset(3 as libc::c_int as isize) = 0 as libc::c_int as stbi_uc;
            }
            p = p.offset(4 as libc::c_int as isize);
            i = i.wrapping_add(1);
        }
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__compute_transparency16(
    mut z: *mut stbi__png,
    mut tc: *mut stbi__uint16,
    mut out_n: libc::c_int,
) -> libc::c_int {
    let mut s: *mut stbi__context = (*z).s;
    let mut i: stbi__uint32 = 0;
    let mut pixel_count: stbi__uint32 = ((*s).img_x).wrapping_mul((*s).img_y);
    let mut p: *mut stbi__uint16 = (*z).out as *mut stbi__uint16;
    if out_n == 2 as libc::c_int {
        i = 0 as libc::c_int as stbi__uint32;
        while i < pixel_count {
            *p
                .offset(
                    1 as libc::c_int as isize,
                ) = (if *p.offset(0 as libc::c_int as isize) as libc::c_int
                == *tc.offset(0 as libc::c_int as isize) as libc::c_int
            {
                0 as libc::c_int
            } else {
                65535 as libc::c_int
            }) as stbi__uint16;
            p = p.offset(2 as libc::c_int as isize);
            i = i.wrapping_add(1);
        }
    } else {
        i = 0 as libc::c_int as stbi__uint32;
        while i < pixel_count {
            if *p.offset(0 as libc::c_int as isize) as libc::c_int
                == *tc.offset(0 as libc::c_int as isize) as libc::c_int
                && *p.offset(1 as libc::c_int as isize) as libc::c_int
                    == *tc.offset(1 as libc::c_int as isize) as libc::c_int
                && *p.offset(2 as libc::c_int as isize) as libc::c_int
                    == *tc.offset(2 as libc::c_int as isize) as libc::c_int
            {
                *p.offset(3 as libc::c_int as isize) = 0 as libc::c_int as stbi__uint16;
            }
            p = p.offset(4 as libc::c_int as isize);
            i = i.wrapping_add(1);
        }
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__create_png_image(
    mut a: *mut stbi__png,
    mut image_data: *mut stbi_uc,
    mut image_data_len: stbi__uint32,
    mut out_n: libc::c_int,
    mut depth: libc::c_int,
    mut color: libc::c_int,
    mut interlaced: libc::c_int,
) -> libc::c_int {
    let mut bytes: libc::c_int = if depth == 16 as libc::c_int {
        2 as libc::c_int
    } else {
        1 as libc::c_int
    };
    let mut out_bytes: libc::c_int = out_n * bytes;
    let mut final_0: *mut stbi_uc = 0 as *mut stbi_uc;
    let mut p: libc::c_int = 0;
    if interlaced == 0 {
        return stbi__create_png_image_raw(
            a,
            image_data,
            image_data_len,
            out_n,
            (*(*a).s).img_x,
            (*(*a).s).img_y,
            depth,
            color,
        );
    }
    final_0 = stbi__malloc_mad3(
        (*(*a).s).img_x as libc::c_int,
        (*(*a).s).img_y as libc::c_int,
        out_bytes,
        0 as libc::c_int,
    ) as *mut stbi_uc;
    if final_0.is_null() {
        return stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char);
    }
    p = 0 as libc::c_int;
    while p < 7 as libc::c_int {
        let mut xorig: [libc::c_int; 7] = [
            0 as libc::c_int,
            4 as libc::c_int,
            0 as libc::c_int,
            2 as libc::c_int,
            0 as libc::c_int,
            1 as libc::c_int,
            0 as libc::c_int,
        ];
        let mut yorig: [libc::c_int; 7] = [
            0 as libc::c_int,
            0 as libc::c_int,
            4 as libc::c_int,
            0 as libc::c_int,
            2 as libc::c_int,
            0 as libc::c_int,
            1 as libc::c_int,
        ];
        let mut xspc: [libc::c_int; 7] = [
            8 as libc::c_int,
            8 as libc::c_int,
            4 as libc::c_int,
            4 as libc::c_int,
            2 as libc::c_int,
            2 as libc::c_int,
            1 as libc::c_int,
        ];
        let mut yspc: [libc::c_int; 7] = [
            8 as libc::c_int,
            8 as libc::c_int,
            8 as libc::c_int,
            4 as libc::c_int,
            4 as libc::c_int,
            2 as libc::c_int,
            2 as libc::c_int,
        ];
        let mut i: libc::c_int = 0;
        let mut j: libc::c_int = 0;
        let mut x: libc::c_int = 0;
        let mut y: libc::c_int = 0;
        x = ((*(*a).s).img_x)
            .wrapping_sub(xorig[p as usize] as libc::c_uint)
            .wrapping_add(xspc[p as usize] as libc::c_uint)
            .wrapping_sub(1 as libc::c_int as libc::c_uint)
            .wrapping_div(xspc[p as usize] as libc::c_uint) as libc::c_int;
        y = ((*(*a).s).img_y)
            .wrapping_sub(yorig[p as usize] as libc::c_uint)
            .wrapping_add(yspc[p as usize] as libc::c_uint)
            .wrapping_sub(1 as libc::c_int as libc::c_uint)
            .wrapping_div(yspc[p as usize] as libc::c_uint) as libc::c_int;
        if x != 0 && y != 0 {
            let mut img_len: stbi__uint32 = ((((*(*a).s).img_n * x * depth
                + 7 as libc::c_int >> 3 as libc::c_int) + 1 as libc::c_int) * y)
                as stbi__uint32;
            if stbi__create_png_image_raw(
                a,
                image_data,
                image_data_len,
                out_n,
                x as stbi__uint32,
                y as stbi__uint32,
                depth,
                color,
            ) == 0
            {
                free(final_0 as *mut libc::c_void);
                return 0 as libc::c_int;
            }
            j = 0 as libc::c_int;
            while j < y {
                i = 0 as libc::c_int;
                while i < x {
                    let mut out_y: libc::c_int = j * yspc[p as usize]
                        + yorig[p as usize];
                    let mut out_x: libc::c_int = i * xspc[p as usize]
                        + xorig[p as usize];
                    memcpy(
                        final_0
                            .offset(
                                (out_y as libc::c_uint)
                                    .wrapping_mul((*(*a).s).img_x)
                                    .wrapping_mul(out_bytes as libc::c_uint) as isize,
                            )
                            .offset((out_x * out_bytes) as isize) as *mut libc::c_void,
                        ((*a).out).offset(((j * x + i) * out_bytes) as isize)
                            as *const libc::c_void,
                        out_bytes as libc::c_ulong,
                    );
                    i += 1;
                }
                j += 1;
            }
            free((*a).out as *mut libc::c_void);
            image_data = image_data.offset(img_len as isize);
            image_data_len = (image_data_len as libc::c_uint).wrapping_sub(img_len)
                as stbi__uint32 as stbi__uint32;
        }
        p += 1;
    }
    (*a).out = final_0;
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__create_png_image_raw(
    mut a: *mut stbi__png,
    mut raw: *mut stbi_uc,
    mut raw_len: stbi__uint32,
    mut out_n: libc::c_int,
    mut x: stbi__uint32,
    mut y: stbi__uint32,
    mut depth: libc::c_int,
    mut color: libc::c_int,
) -> libc::c_int {
    let mut bytes: libc::c_int = if depth == 16 as libc::c_int {
        2 as libc::c_int
    } else {
        1 as libc::c_int
    };
    let mut s: *mut stbi__context = (*a).s;
    let mut i: stbi__uint32 = 0;
    let mut j: stbi__uint32 = 0;
    let mut stride: stbi__uint32 = x
        .wrapping_mul(out_n as libc::c_uint)
        .wrapping_mul(bytes as libc::c_uint);
    let mut img_len: stbi__uint32 = 0;
    let mut img_width_bytes: stbi__uint32 = 0;
    let mut k: libc::c_int = 0;
    let mut img_n: libc::c_int = (*s).img_n;
    let mut output_bytes: libc::c_int = out_n * bytes;
    let mut filter_bytes: libc::c_int = img_n * bytes;
    let mut width: libc::c_int = x as libc::c_int;
    (*a)
        .out = stbi__malloc_mad3(
        x as libc::c_int,
        y as libc::c_int,
        output_bytes,
        0 as libc::c_int,
    ) as *mut stbi_uc;
    if ((*a).out).is_null() {
        return stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char);
    }
    if stbi__mad3sizes_valid(img_n, x as libc::c_int, depth, 7 as libc::c_int) == 0 {
        return stbi__err(b"too large\0" as *const u8 as *const libc::c_char);
    }
    img_width_bytes = (img_n as libc::c_uint)
        .wrapping_mul(x)
        .wrapping_mul(depth as libc::c_uint)
        .wrapping_add(7 as libc::c_int as libc::c_uint) >> 3 as libc::c_int;
    img_len = img_width_bytes
        .wrapping_add(1 as libc::c_int as libc::c_uint)
        .wrapping_mul(y);
    if raw_len < img_len {
        return stbi__err(b"not enough pixels\0" as *const u8 as *const libc::c_char);
    }
    j = 0 as libc::c_int as stbi__uint32;
    while j < y {
        let mut cur: *mut stbi_uc = ((*a).out).offset(stride.wrapping_mul(j) as isize);
        let mut prior: *mut stbi_uc = 0 as *mut stbi_uc;
        let fresh66 = raw;
        raw = raw.offset(1);
        let mut filter: libc::c_int = *fresh66 as libc::c_int;
        if filter > 4 as libc::c_int {
            return stbi__err(b"invalid filter\0" as *const u8 as *const libc::c_char);
        }
        if depth < 8 as libc::c_int {
            if img_width_bytes > x {
                return stbi__err(b"invalid width\0" as *const u8 as *const libc::c_char);
            }
            cur = cur
                .offset(
                    x.wrapping_mul(out_n as libc::c_uint).wrapping_sub(img_width_bytes)
                        as isize,
                );
            filter_bytes = 1 as libc::c_int;
            width = img_width_bytes as libc::c_int;
        }
        prior = cur.offset(-(stride as isize));
        if j == 0 as libc::c_int as libc::c_uint {
            filter = first_row_filter[filter as usize] as libc::c_int;
        }
        k = 0 as libc::c_int;
        while k < filter_bytes {
            match filter {
                0 => {
                    *cur.offset(k as isize) = *raw.offset(k as isize);
                }
                1 => {
                    *cur.offset(k as isize) = *raw.offset(k as isize);
                }
                2 => {
                    *cur
                        .offset(
                            k as isize,
                        ) = (*raw.offset(k as isize) as libc::c_int
                        + *prior.offset(k as isize) as libc::c_int & 255 as libc::c_int)
                        as stbi_uc;
                }
                3 => {
                    *cur
                        .offset(
                            k as isize,
                        ) = (*raw.offset(k as isize) as libc::c_int
                        + (*prior.offset(k as isize) as libc::c_int >> 1 as libc::c_int)
                        & 255 as libc::c_int) as stbi_uc;
                }
                4 => {
                    *cur
                        .offset(
                            k as isize,
                        ) = (*raw.offset(k as isize) as libc::c_int
                        + stbi__paeth(
                            0 as libc::c_int,
                            *prior.offset(k as isize) as libc::c_int,
                            0 as libc::c_int,
                        ) & 255 as libc::c_int) as stbi_uc;
                }
                5 => {
                    *cur.offset(k as isize) = *raw.offset(k as isize);
                }
                6 => {
                    *cur.offset(k as isize) = *raw.offset(k as isize);
                }
                _ => {}
            }
            k += 1;
        }
        if depth == 8 as libc::c_int {
            if img_n != out_n {
                *cur.offset(img_n as isize) = 255 as libc::c_int as stbi_uc;
            }
            raw = raw.offset(img_n as isize);
            cur = cur.offset(out_n as isize);
            prior = prior.offset(out_n as isize);
        } else if depth == 16 as libc::c_int {
            if img_n != out_n {
                *cur.offset(filter_bytes as isize) = 255 as libc::c_int as stbi_uc;
                *cur
                    .offset(
                        (filter_bytes + 1 as libc::c_int) as isize,
                    ) = 255 as libc::c_int as stbi_uc;
            }
            raw = raw.offset(filter_bytes as isize);
            cur = cur.offset(output_bytes as isize);
            prior = prior.offset(output_bytes as isize);
        } else {
            raw = raw.offset(1 as libc::c_int as isize);
            cur = cur.offset(1 as libc::c_int as isize);
            prior = prior.offset(1 as libc::c_int as isize);
        }
        if depth < 8 as libc::c_int || img_n == out_n {
            let mut nk: libc::c_int = (width - 1 as libc::c_int) * filter_bytes;
            match filter {
                0 => {
                    memcpy(
                        cur as *mut libc::c_void,
                        raw as *const libc::c_void,
                        nk as libc::c_ulong,
                    );
                }
                1 => {
                    k = 0 as libc::c_int;
                    while k < nk {
                        *cur
                            .offset(
                                k as isize,
                            ) = (*raw.offset(k as isize) as libc::c_int
                            + *cur.offset((k - filter_bytes) as isize) as libc::c_int
                            & 255 as libc::c_int) as stbi_uc;
                        k += 1;
                    }
                }
                2 => {
                    k = 0 as libc::c_int;
                    while k < nk {
                        *cur
                            .offset(
                                k as isize,
                            ) = (*raw.offset(k as isize) as libc::c_int
                            + *prior.offset(k as isize) as libc::c_int
                            & 255 as libc::c_int) as stbi_uc;
                        k += 1;
                    }
                }
                3 => {
                    k = 0 as libc::c_int;
                    while k < nk {
                        *cur
                            .offset(
                                k as isize,
                            ) = (*raw.offset(k as isize) as libc::c_int
                            + (*prior.offset(k as isize) as libc::c_int
                                + *cur.offset((k - filter_bytes) as isize) as libc::c_int
                                >> 1 as libc::c_int) & 255 as libc::c_int) as stbi_uc;
                        k += 1;
                    }
                }
                4 => {
                    k = 0 as libc::c_int;
                    while k < nk {
                        *cur
                            .offset(
                                k as isize,
                            ) = (*raw.offset(k as isize) as libc::c_int
                            + stbi__paeth(
                                *cur.offset((k - filter_bytes) as isize) as libc::c_int,
                                *prior.offset(k as isize) as libc::c_int,
                                *prior.offset((k - filter_bytes) as isize) as libc::c_int,
                            ) & 255 as libc::c_int) as stbi_uc;
                        k += 1;
                    }
                }
                5 => {
                    k = 0 as libc::c_int;
                    while k < nk {
                        *cur
                            .offset(
                                k as isize,
                            ) = (*raw.offset(k as isize) as libc::c_int
                            + (*cur.offset((k - filter_bytes) as isize) as libc::c_int
                                >> 1 as libc::c_int) & 255 as libc::c_int) as stbi_uc;
                        k += 1;
                    }
                }
                6 => {
                    k = 0 as libc::c_int;
                    while k < nk {
                        *cur
                            .offset(
                                k as isize,
                            ) = (*raw.offset(k as isize) as libc::c_int
                            + stbi__paeth(
                                *cur.offset((k - filter_bytes) as isize) as libc::c_int,
                                0 as libc::c_int,
                                0 as libc::c_int,
                            ) & 255 as libc::c_int) as stbi_uc;
                        k += 1;
                    }
                }
                _ => {}
            }
            raw = raw.offset(nk as isize);
        } else {
            match filter {
                0 => {
                    i = x.wrapping_sub(1 as libc::c_int as libc::c_uint);
                    while i >= 1 as libc::c_int as libc::c_uint {
                        k = 0 as libc::c_int;
                        while k < filter_bytes {
                            *cur.offset(k as isize) = *raw.offset(k as isize);
                            k += 1;
                        }
                        i = i.wrapping_sub(1);
                        *cur
                            .offset(
                                filter_bytes as isize,
                            ) = 255 as libc::c_int as stbi_uc;
                        raw = raw.offset(filter_bytes as isize);
                        cur = cur.offset(output_bytes as isize);
                        prior = prior.offset(output_bytes as isize);
                    }
                }
                1 => {
                    i = x.wrapping_sub(1 as libc::c_int as libc::c_uint);
                    while i >= 1 as libc::c_int as libc::c_uint {
                        k = 0 as libc::c_int;
                        while k < filter_bytes {
                            *cur
                                .offset(
                                    k as isize,
                                ) = (*raw.offset(k as isize) as libc::c_int
                                + *cur.offset((k - output_bytes) as isize) as libc::c_int
                                & 255 as libc::c_int) as stbi_uc;
                            k += 1;
                        }
                        i = i.wrapping_sub(1);
                        *cur
                            .offset(
                                filter_bytes as isize,
                            ) = 255 as libc::c_int as stbi_uc;
                        raw = raw.offset(filter_bytes as isize);
                        cur = cur.offset(output_bytes as isize);
                        prior = prior.offset(output_bytes as isize);
                    }
                }
                2 => {
                    i = x.wrapping_sub(1 as libc::c_int as libc::c_uint);
                    while i >= 1 as libc::c_int as libc::c_uint {
                        k = 0 as libc::c_int;
                        while k < filter_bytes {
                            *cur
                                .offset(
                                    k as isize,
                                ) = (*raw.offset(k as isize) as libc::c_int
                                + *prior.offset(k as isize) as libc::c_int
                                & 255 as libc::c_int) as stbi_uc;
                            k += 1;
                        }
                        i = i.wrapping_sub(1);
                        *cur
                            .offset(
                                filter_bytes as isize,
                            ) = 255 as libc::c_int as stbi_uc;
                        raw = raw.offset(filter_bytes as isize);
                        cur = cur.offset(output_bytes as isize);
                        prior = prior.offset(output_bytes as isize);
                    }
                }
                3 => {
                    i = x.wrapping_sub(1 as libc::c_int as libc::c_uint);
                    while i >= 1 as libc::c_int as libc::c_uint {
                        k = 0 as libc::c_int;
                        while k < filter_bytes {
                            *cur
                                .offset(
                                    k as isize,
                                ) = (*raw.offset(k as isize) as libc::c_int
                                + (*prior.offset(k as isize) as libc::c_int
                                    + *cur.offset((k - output_bytes) as isize) as libc::c_int
                                    >> 1 as libc::c_int) & 255 as libc::c_int) as stbi_uc;
                            k += 1;
                        }
                        i = i.wrapping_sub(1);
                        *cur
                            .offset(
                                filter_bytes as isize,
                            ) = 255 as libc::c_int as stbi_uc;
                        raw = raw.offset(filter_bytes as isize);
                        cur = cur.offset(output_bytes as isize);
                        prior = prior.offset(output_bytes as isize);
                    }
                }
                4 => {
                    i = x.wrapping_sub(1 as libc::c_int as libc::c_uint);
                    while i >= 1 as libc::c_int as libc::c_uint {
                        k = 0 as libc::c_int;
                        while k < filter_bytes {
                            *cur
                                .offset(
                                    k as isize,
                                ) = (*raw.offset(k as isize) as libc::c_int
                                + stbi__paeth(
                                    *cur.offset((k - output_bytes) as isize) as libc::c_int,
                                    *prior.offset(k as isize) as libc::c_int,
                                    *prior.offset((k - output_bytes) as isize) as libc::c_int,
                                ) & 255 as libc::c_int) as stbi_uc;
                            k += 1;
                        }
                        i = i.wrapping_sub(1);
                        *cur
                            .offset(
                                filter_bytes as isize,
                            ) = 255 as libc::c_int as stbi_uc;
                        raw = raw.offset(filter_bytes as isize);
                        cur = cur.offset(output_bytes as isize);
                        prior = prior.offset(output_bytes as isize);
                    }
                }
                5 => {
                    i = x.wrapping_sub(1 as libc::c_int as libc::c_uint);
                    while i >= 1 as libc::c_int as libc::c_uint {
                        k = 0 as libc::c_int;
                        while k < filter_bytes {
                            *cur
                                .offset(
                                    k as isize,
                                ) = (*raw.offset(k as isize) as libc::c_int
                                + (*cur.offset((k - output_bytes) as isize) as libc::c_int
                                    >> 1 as libc::c_int) & 255 as libc::c_int) as stbi_uc;
                            k += 1;
                        }
                        i = i.wrapping_sub(1);
                        *cur
                            .offset(
                                filter_bytes as isize,
                            ) = 255 as libc::c_int as stbi_uc;
                        raw = raw.offset(filter_bytes as isize);
                        cur = cur.offset(output_bytes as isize);
                        prior = prior.offset(output_bytes as isize);
                    }
                }
                6 => {
                    i = x.wrapping_sub(1 as libc::c_int as libc::c_uint);
                    while i >= 1 as libc::c_int as libc::c_uint {
                        k = 0 as libc::c_int;
                        while k < filter_bytes {
                            *cur
                                .offset(
                                    k as isize,
                                ) = (*raw.offset(k as isize) as libc::c_int
                                + stbi__paeth(
                                    *cur.offset((k - output_bytes) as isize) as libc::c_int,
                                    0 as libc::c_int,
                                    0 as libc::c_int,
                                ) & 255 as libc::c_int) as stbi_uc;
                            k += 1;
                        }
                        i = i.wrapping_sub(1);
                        *cur
                            .offset(
                                filter_bytes as isize,
                            ) = 255 as libc::c_int as stbi_uc;
                        raw = raw.offset(filter_bytes as isize);
                        cur = cur.offset(output_bytes as isize);
                        prior = prior.offset(output_bytes as isize);
                    }
                }
                _ => {}
            }
            if depth == 16 as libc::c_int {
                cur = ((*a).out).offset(stride.wrapping_mul(j) as isize);
                i = 0 as libc::c_int as stbi__uint32;
                while i < x {
                    *cur
                        .offset(
                            (filter_bytes + 1 as libc::c_int) as isize,
                        ) = 255 as libc::c_int as stbi_uc;
                    i = i.wrapping_add(1);
                    cur = cur.offset(output_bytes as isize);
                }
            }
        }
        j = j.wrapping_add(1);
    }
    if depth < 8 as libc::c_int {
        j = 0 as libc::c_int as stbi__uint32;
        while j < y {
            let mut cur_0: *mut stbi_uc = ((*a).out)
                .offset(stride.wrapping_mul(j) as isize);
            let mut in_0: *mut stbi_uc = ((*a).out)
                .offset(stride.wrapping_mul(j) as isize)
                .offset(x.wrapping_mul(out_n as libc::c_uint) as isize)
                .offset(-(img_width_bytes as isize));
            let mut scale: stbi_uc = (if color == 0 as libc::c_int {
                stbi__depth_scale_table[depth as usize] as libc::c_int
            } else {
                1 as libc::c_int
            }) as stbi_uc;
            if depth == 4 as libc::c_int {
                k = x.wrapping_mul(img_n as libc::c_uint) as libc::c_int;
                while k >= 2 as libc::c_int {
                    let fresh67 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh67 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 4 as libc::c_int)) as stbi_uc;
                    let fresh68 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh68 = (scale as libc::c_int
                        * (*in_0 as libc::c_int & 0xf as libc::c_int)) as stbi_uc;
                    k -= 2 as libc::c_int;
                    in_0 = in_0.offset(1);
                }
                if k > 0 as libc::c_int {
                    let fresh69 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh69 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 4 as libc::c_int)) as stbi_uc;
                }
            } else if depth == 2 as libc::c_int {
                k = x.wrapping_mul(img_n as libc::c_uint) as libc::c_int;
                while k >= 4 as libc::c_int {
                    let fresh70 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh70 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 6 as libc::c_int)) as stbi_uc;
                    let fresh71 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh71 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 4 as libc::c_int
                            & 0x3 as libc::c_int)) as stbi_uc;
                    let fresh72 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh72 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 2 as libc::c_int
                            & 0x3 as libc::c_int)) as stbi_uc;
                    let fresh73 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh73 = (scale as libc::c_int
                        * (*in_0 as libc::c_int & 0x3 as libc::c_int)) as stbi_uc;
                    k -= 4 as libc::c_int;
                    in_0 = in_0.offset(1);
                }
                if k > 0 as libc::c_int {
                    let fresh74 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh74 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 6 as libc::c_int)) as stbi_uc;
                }
                if k > 1 as libc::c_int {
                    let fresh75 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh75 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 4 as libc::c_int
                            & 0x3 as libc::c_int)) as stbi_uc;
                }
                if k > 2 as libc::c_int {
                    let fresh76 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh76 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 2 as libc::c_int
                            & 0x3 as libc::c_int)) as stbi_uc;
                }
            } else if depth == 1 as libc::c_int {
                k = x.wrapping_mul(img_n as libc::c_uint) as libc::c_int;
                while k >= 8 as libc::c_int {
                    let fresh77 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh77 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 7 as libc::c_int)) as stbi_uc;
                    let fresh78 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh78 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 6 as libc::c_int
                            & 0x1 as libc::c_int)) as stbi_uc;
                    let fresh79 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh79 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 5 as libc::c_int
                            & 0x1 as libc::c_int)) as stbi_uc;
                    let fresh80 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh80 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 4 as libc::c_int
                            & 0x1 as libc::c_int)) as stbi_uc;
                    let fresh81 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh81 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 3 as libc::c_int
                            & 0x1 as libc::c_int)) as stbi_uc;
                    let fresh82 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh82 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 2 as libc::c_int
                            & 0x1 as libc::c_int)) as stbi_uc;
                    let fresh83 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh83 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 1 as libc::c_int
                            & 0x1 as libc::c_int)) as stbi_uc;
                    let fresh84 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh84 = (scale as libc::c_int
                        * (*in_0 as libc::c_int & 0x1 as libc::c_int)) as stbi_uc;
                    k -= 8 as libc::c_int;
                    in_0 = in_0.offset(1);
                }
                if k > 0 as libc::c_int {
                    let fresh85 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh85 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 7 as libc::c_int)) as stbi_uc;
                }
                if k > 1 as libc::c_int {
                    let fresh86 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh86 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 6 as libc::c_int
                            & 0x1 as libc::c_int)) as stbi_uc;
                }
                if k > 2 as libc::c_int {
                    let fresh87 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh87 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 5 as libc::c_int
                            & 0x1 as libc::c_int)) as stbi_uc;
                }
                if k > 3 as libc::c_int {
                    let fresh88 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh88 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 4 as libc::c_int
                            & 0x1 as libc::c_int)) as stbi_uc;
                }
                if k > 4 as libc::c_int {
                    let fresh89 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh89 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 3 as libc::c_int
                            & 0x1 as libc::c_int)) as stbi_uc;
                }
                if k > 5 as libc::c_int {
                    let fresh90 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh90 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 2 as libc::c_int
                            & 0x1 as libc::c_int)) as stbi_uc;
                }
                if k > 6 as libc::c_int {
                    let fresh91 = cur_0;
                    cur_0 = cur_0.offset(1);
                    *fresh91 = (scale as libc::c_int
                        * (*in_0 as libc::c_int >> 1 as libc::c_int
                            & 0x1 as libc::c_int)) as stbi_uc;
                }
            }
            if img_n != out_n {
                let mut q: libc::c_int = 0;
                cur_0 = ((*a).out).offset(stride.wrapping_mul(j) as isize);
                if img_n == 1 as libc::c_int {
                    q = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                    while q >= 0 as libc::c_int {
                        *cur_0
                            .offset(
                                (q * 2 as libc::c_int + 1 as libc::c_int) as isize,
                            ) = 255 as libc::c_int as stbi_uc;
                        *cur_0
                            .offset(
                                (q * 2 as libc::c_int + 0 as libc::c_int) as isize,
                            ) = *cur_0.offset(q as isize);
                        q -= 1;
                    }
                } else {
                    q = x.wrapping_sub(1 as libc::c_int as libc::c_uint) as libc::c_int;
                    while q >= 0 as libc::c_int {
                        *cur_0
                            .offset(
                                (q * 4 as libc::c_int + 3 as libc::c_int) as isize,
                            ) = 255 as libc::c_int as stbi_uc;
                        *cur_0
                            .offset(
                                (q * 4 as libc::c_int + 2 as libc::c_int) as isize,
                            ) = *cur_0
                            .offset((q * 3 as libc::c_int + 2 as libc::c_int) as isize);
                        *cur_0
                            .offset(
                                (q * 4 as libc::c_int + 1 as libc::c_int) as isize,
                            ) = *cur_0
                            .offset((q * 3 as libc::c_int + 1 as libc::c_int) as isize);
                        *cur_0
                            .offset(
                                (q * 4 as libc::c_int + 0 as libc::c_int) as isize,
                            ) = *cur_0
                            .offset((q * 3 as libc::c_int + 0 as libc::c_int) as isize);
                        q -= 1;
                    }
                }
            }
            j = j.wrapping_add(1);
        }
    } else if depth == 16 as libc::c_int {
        let mut cur_1: *mut stbi_uc = (*a).out;
        let mut cur16: *mut stbi__uint16 = cur_1 as *mut stbi__uint16;
        i = 0 as libc::c_int as stbi__uint32;
        while i < x.wrapping_mul(y).wrapping_mul(out_n as libc::c_uint) {
            *cur16 = ((*cur_1.offset(0 as libc::c_int as isize) as libc::c_int)
                << 8 as libc::c_int
                | *cur_1.offset(1 as libc::c_int as isize) as libc::c_int)
                as stbi__uint16;
            i = i.wrapping_add(1);
            cur16 = cur16.offset(1);
            cur_1 = cur_1.offset(2 as libc::c_int as isize);
        }
    }
    return 1 as libc::c_int;
}
static mut stbi__depth_scale_table: [stbi_uc; 9] = [
    0 as libc::c_int as stbi_uc,
    0xff as libc::c_int as stbi_uc,
    0x55 as libc::c_int as stbi_uc,
    0 as libc::c_int as stbi_uc,
    0x11 as libc::c_int as stbi_uc,
    0 as libc::c_int as stbi_uc,
    0 as libc::c_int as stbi_uc,
    0 as libc::c_int as stbi_uc,
    0x1 as libc::c_int as stbi_uc,
];
unsafe extern "C" fn stbi__paeth(
    mut a: libc::c_int,
    mut b: libc::c_int,
    mut c: libc::c_int,
) -> libc::c_int {
    let mut p: libc::c_int = a + b - c;
    let mut pa: libc::c_int = abs(p - a);
    let mut pb: libc::c_int = abs(p - b);
    let mut pc: libc::c_int = abs(p - c);
    if pa <= pb && pa <= pc {
        return a;
    }
    if pb <= pc {
        return b;
    }
    return c;
}
static mut first_row_filter: [stbi_uc; 5] = [
    STBI__F_none as libc::c_int as stbi_uc,
    STBI__F_sub as libc::c_int as stbi_uc,
    STBI__F_none as libc::c_int as stbi_uc,
    STBI__F_avg_first as libc::c_int as stbi_uc,
    STBI__F_paeth_first as libc::c_int as stbi_uc,
];
#[no_mangle]
pub unsafe extern "C" fn stbi_zlib_decode_malloc_guesssize_headerflag(
    mut buffer: *const libc::c_char,
    mut len: libc::c_int,
    mut initial_size: libc::c_int,
    mut outlen: *mut libc::c_int,
    mut parse_header: libc::c_int,
) -> *mut libc::c_char {
    let mut a: stbi__zbuf = stbi__zbuf {
        zbuffer: 0 as *mut stbi_uc,
        zbuffer_end: 0 as *mut stbi_uc,
        num_bits: 0,
        code_buffer: 0,
        zout: 0 as *mut libc::c_char,
        zout_start: 0 as *mut libc::c_char,
        zout_end: 0 as *mut libc::c_char,
        z_expandable: 0,
        z_length: stbi__zhuffman {
            fast: [0; 512],
            firstcode: [0; 16],
            maxcode: [0; 17],
            firstsymbol: [0; 16],
            size: [0; 288],
            value: [0; 288],
        },
        z_distance: stbi__zhuffman {
            fast: [0; 512],
            firstcode: [0; 16],
            maxcode: [0; 17],
            firstsymbol: [0; 16],
            size: [0; 288],
            value: [0; 288],
        },
    };
    let mut p: *mut libc::c_char = stbi__malloc(initial_size as size_t)
        as *mut libc::c_char;
    if p.is_null() {
        return 0 as *mut libc::c_char;
    }
    a.zbuffer = buffer as *mut stbi_uc;
    a.zbuffer_end = (buffer as *mut stbi_uc).offset(len as isize);
    if stbi__do_zlib(&mut a, p, initial_size, 1 as libc::c_int, parse_header) != 0 {
        if !outlen.is_null() {
            *outlen = (a.zout).offset_from(a.zout_start) as libc::c_long as libc::c_int;
        }
        return a.zout_start;
    } else {
        free(a.zout_start as *mut libc::c_void);
        return 0 as *mut libc::c_char;
    };
}
unsafe extern "C" fn stbi__do_zlib(
    mut a: *mut stbi__zbuf,
    mut obuf: *mut libc::c_char,
    mut olen: libc::c_int,
    mut exp: libc::c_int,
    mut parse_header: libc::c_int,
) -> libc::c_int {
    (*a).zout_start = obuf;
    (*a).zout = obuf;
    (*a).zout_end = obuf.offset(olen as isize);
    (*a).z_expandable = exp;
    return stbi__parse_zlib(a, parse_header);
}
unsafe extern "C" fn stbi__parse_zlib(
    mut a: *mut stbi__zbuf,
    mut parse_header: libc::c_int,
) -> libc::c_int {
    let mut final_0: libc::c_int = 0;
    let mut type_0: libc::c_int = 0;
    if parse_header != 0 {
        if stbi__parse_zlib_header(a) == 0 {
            return 0 as libc::c_int;
        }
    }
    (*a).num_bits = 0 as libc::c_int;
    (*a).code_buffer = 0 as libc::c_int as stbi__uint32;
    loop {
        final_0 = stbi__zreceive(a, 1 as libc::c_int) as libc::c_int;
        type_0 = stbi__zreceive(a, 2 as libc::c_int) as libc::c_int;
        if type_0 == 0 as libc::c_int {
            if stbi__parse_uncompressed_block(a) == 0 {
                return 0 as libc::c_int;
            }
        } else if type_0 == 3 as libc::c_int {
            return 0 as libc::c_int
        } else {
            if type_0 == 1 as libc::c_int {
                if stbi__zbuild_huffman(
                    &mut (*a).z_length,
                    stbi__zdefault_length.as_ptr(),
                    288 as libc::c_int,
                ) == 0
                {
                    return 0 as libc::c_int;
                }
                if stbi__zbuild_huffman(
                    &mut (*a).z_distance,
                    stbi__zdefault_distance.as_ptr(),
                    32 as libc::c_int,
                ) == 0
                {
                    return 0 as libc::c_int;
                }
            } else if stbi__compute_huffman_codes(a) == 0 {
                return 0 as libc::c_int
            }
            if stbi__parse_huffman_block(a) == 0 {
                return 0 as libc::c_int;
            }
        }
        if !(final_0 == 0) {
            break;
        }
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__parse_huffman_block(mut a: *mut stbi__zbuf) -> libc::c_int {
    let mut zout: *mut libc::c_char = (*a).zout;
    loop {
        let mut z: libc::c_int = stbi__zhuffman_decode(a, &mut (*a).z_length);
        if z < 256 as libc::c_int {
            if z < 0 as libc::c_int {
                return stbi__err(
                    b"bad huffman code\0" as *const u8 as *const libc::c_char,
                );
            }
            if zout >= (*a).zout_end {
                if stbi__zexpand(a, zout, 1 as libc::c_int) == 0 {
                    return 0 as libc::c_int;
                }
                zout = (*a).zout;
            }
            let fresh92 = zout;
            zout = zout.offset(1);
            *fresh92 = z as libc::c_char;
        } else {
            let mut p: *mut stbi_uc = 0 as *mut stbi_uc;
            let mut len: libc::c_int = 0;
            let mut dist: libc::c_int = 0;
            if z == 256 as libc::c_int {
                (*a).zout = zout;
                return 1 as libc::c_int;
            }
            z -= 257 as libc::c_int;
            len = stbi__zlength_base[z as usize];
            if stbi__zlength_extra[z as usize] != 0 {
                len = (len as libc::c_uint)
                    .wrapping_add(stbi__zreceive(a, stbi__zlength_extra[z as usize]))
                    as libc::c_int as libc::c_int;
            }
            z = stbi__zhuffman_decode(a, &mut (*a).z_distance);
            if z < 0 as libc::c_int {
                return stbi__err(
                    b"bad huffman code\0" as *const u8 as *const libc::c_char,
                );
            }
            dist = stbi__zdist_base[z as usize];
            if stbi__zdist_extra[z as usize] != 0 {
                dist = (dist as libc::c_uint)
                    .wrapping_add(stbi__zreceive(a, stbi__zdist_extra[z as usize]))
                    as libc::c_int as libc::c_int;
            }
            if (zout.offset_from((*a).zout_start) as libc::c_long) < dist as libc::c_long
            {
                return stbi__err(b"bad dist\0" as *const u8 as *const libc::c_char);
            }
            if zout.offset(len as isize) > (*a).zout_end {
                if stbi__zexpand(a, zout, len) == 0 {
                    return 0 as libc::c_int;
                }
                zout = (*a).zout;
            }
            p = zout.offset(-(dist as isize)) as *mut stbi_uc;
            if dist == 1 as libc::c_int {
                let mut v: stbi_uc = *p;
                if len != 0 {
                    loop {
                        let fresh93 = zout;
                        zout = zout.offset(1);
                        *fresh93 = v as libc::c_char;
                        len -= 1;
                        if !(len != 0) {
                            break;
                        }
                    }
                }
            } else if len != 0 {
                loop {
                    let fresh94 = p;
                    p = p.offset(1);
                    let fresh95 = zout;
                    zout = zout.offset(1);
                    *fresh95 = *fresh94 as libc::c_char;
                    len -= 1;
                    if !(len != 0) {
                        break;
                    }
                }
            }
        }
    };
}
unsafe extern "C" fn stbi__zexpand(
    mut z: *mut stbi__zbuf,
    mut zout: *mut libc::c_char,
    mut n: libc::c_int,
) -> libc::c_int {
    let mut q: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut cur: libc::c_uint = 0;
    let mut limit: libc::c_uint = 0;
    let mut old_limit: libc::c_uint = 0;
    (*z).zout = zout;
    if (*z).z_expandable == 0 {
        return stbi__err(b"output buffer limit\0" as *const u8 as *const libc::c_char);
    }
    cur = ((*z).zout).offset_from((*z).zout_start) as libc::c_long as libc::c_uint;
    old_limit = ((*z).zout_end).offset_from((*z).zout_start) as libc::c_long
        as libc::c_uint;
    limit = old_limit;
    if (0xffffffff as libc::c_uint).wrapping_sub(cur) < n as libc::c_uint {
        return stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char);
    }
    while cur.wrapping_add(n as libc::c_uint) > limit {
        if limit
            > (0xffffffff as libc::c_uint).wrapping_div(2 as libc::c_int as libc::c_uint)
        {
            return stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char);
        }
        limit = limit.wrapping_mul(2 as libc::c_int as libc::c_uint);
    }
    q = realloc((*z).zout_start as *mut libc::c_void, limit as libc::c_ulong)
        as *mut libc::c_char;
    if q.is_null() {
        return stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char);
    }
    (*z).zout_start = q;
    (*z).zout = q.offset(cur as isize);
    (*z).zout_end = q.offset(limit as isize);
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__zhuffman_decode(
    mut a: *mut stbi__zbuf,
    mut z: *mut stbi__zhuffman,
) -> libc::c_int {
    let mut b: libc::c_int = 0;
    let mut s: libc::c_int = 0;
    if (*a).num_bits < 16 as libc::c_int {
        if stbi__zeof(a) != 0 {
            return -(1 as libc::c_int);
        }
        stbi__fill_bits(a);
    }
    b = (*z)
        .fast[((*a).code_buffer
        & (((1 as libc::c_int) << 9 as libc::c_int) - 1 as libc::c_int) as libc::c_uint)
        as usize] as libc::c_int;
    if b != 0 {
        s = b >> 9 as libc::c_int;
        (*a).code_buffer >>= s;
        (*a).num_bits -= s;
        return b & 511 as libc::c_int;
    }
    return stbi__zhuffman_decode_slowpath(a, z);
}
unsafe extern "C" fn stbi__zhuffman_decode_slowpath(
    mut a: *mut stbi__zbuf,
    mut z: *mut stbi__zhuffman,
) -> libc::c_int {
    let mut b: libc::c_int = 0;
    let mut s: libc::c_int = 0;
    let mut k: libc::c_int = 0;
    k = stbi__bit_reverse((*a).code_buffer as libc::c_int, 16 as libc::c_int);
    s = 9 as libc::c_int + 1 as libc::c_int;
    while !(k < (*z).maxcode[s as usize]) {
        s += 1;
    }
    if s >= 16 as libc::c_int {
        return -(1 as libc::c_int);
    }
    b = (k >> 16 as libc::c_int - s) - (*z).firstcode[s as usize] as libc::c_int
        + (*z).firstsymbol[s as usize] as libc::c_int;
    if b >= 288 as libc::c_int {
        return -(1 as libc::c_int);
    }
    if (*z).size[b as usize] as libc::c_int != s {
        return -(1 as libc::c_int);
    }
    (*a).code_buffer >>= s;
    (*a).num_bits -= s;
    return (*z).value[b as usize] as libc::c_int;
}
unsafe extern "C" fn stbi__bit_reverse(
    mut v: libc::c_int,
    mut bits: libc::c_int,
) -> libc::c_int {
    return stbi__bitreverse16(v) >> 16 as libc::c_int - bits;
}
unsafe extern "C" fn stbi__bitreverse16(mut n: libc::c_int) -> libc::c_int {
    n = (n & 0xaaaa as libc::c_int) >> 1 as libc::c_int
        | (n & 0x5555 as libc::c_int) << 1 as libc::c_int;
    n = (n & 0xcccc as libc::c_int) >> 2 as libc::c_int
        | (n & 0x3333 as libc::c_int) << 2 as libc::c_int;
    n = (n & 0xf0f0 as libc::c_int) >> 4 as libc::c_int
        | (n & 0xf0f as libc::c_int) << 4 as libc::c_int;
    n = (n & 0xff00 as libc::c_int) >> 8 as libc::c_int
        | (n & 0xff as libc::c_int) << 8 as libc::c_int;
    return n;
}
unsafe extern "C" fn stbi__fill_bits(mut z: *mut stbi__zbuf) {
    loop {
        if (*z).code_buffer >= (1 as libc::c_uint) << (*z).num_bits {
            (*z).zbuffer = (*z).zbuffer_end;
            return;
        }
        (*z).code_buffer |= (stbi__zget8(z) as libc::c_uint) << (*z).num_bits;
        (*z).num_bits += 8 as libc::c_int;
        if !((*z).num_bits <= 24 as libc::c_int) {
            break;
        }
    };
}
unsafe extern "C" fn stbi__zget8(mut z: *mut stbi__zbuf) -> stbi_uc {
    return (if stbi__zeof(z) != 0 {
        0 as libc::c_int
    } else {
        let fresh96 = (*z).zbuffer;
        (*z).zbuffer = ((*z).zbuffer).offset(1);
        *fresh96 as libc::c_int
    }) as stbi_uc;
}
unsafe extern "C" fn stbi__zeof(mut z: *mut stbi__zbuf) -> libc::c_int {
    return ((*z).zbuffer >= (*z).zbuffer_end) as libc::c_int;
}
static mut stbi__zdist_extra: [libc::c_int; 32] = [
    0 as libc::c_int,
    0 as libc::c_int,
    0 as libc::c_int,
    0 as libc::c_int,
    1 as libc::c_int,
    1 as libc::c_int,
    2 as libc::c_int,
    2 as libc::c_int,
    3 as libc::c_int,
    3 as libc::c_int,
    4 as libc::c_int,
    4 as libc::c_int,
    5 as libc::c_int,
    5 as libc::c_int,
    6 as libc::c_int,
    6 as libc::c_int,
    7 as libc::c_int,
    7 as libc::c_int,
    8 as libc::c_int,
    8 as libc::c_int,
    9 as libc::c_int,
    9 as libc::c_int,
    10 as libc::c_int,
    10 as libc::c_int,
    11 as libc::c_int,
    11 as libc::c_int,
    12 as libc::c_int,
    12 as libc::c_int,
    13 as libc::c_int,
    13 as libc::c_int,
    0,
    0,
];
unsafe extern "C" fn stbi__zreceive(
    mut z: *mut stbi__zbuf,
    mut n: libc::c_int,
) -> libc::c_uint {
    let mut k: libc::c_uint = 0;
    if (*z).num_bits < n {
        stbi__fill_bits(z);
    }
    k = (*z).code_buffer
        & (((1 as libc::c_int) << n) - 1 as libc::c_int) as libc::c_uint;
    (*z).code_buffer >>= n;
    (*z).num_bits -= n;
    return k;
}
static mut stbi__zdist_base: [libc::c_int; 32] = [
    1 as libc::c_int,
    2 as libc::c_int,
    3 as libc::c_int,
    4 as libc::c_int,
    5 as libc::c_int,
    7 as libc::c_int,
    9 as libc::c_int,
    13 as libc::c_int,
    17 as libc::c_int,
    25 as libc::c_int,
    33 as libc::c_int,
    49 as libc::c_int,
    65 as libc::c_int,
    97 as libc::c_int,
    129 as libc::c_int,
    193 as libc::c_int,
    257 as libc::c_int,
    385 as libc::c_int,
    513 as libc::c_int,
    769 as libc::c_int,
    1025 as libc::c_int,
    1537 as libc::c_int,
    2049 as libc::c_int,
    3073 as libc::c_int,
    4097 as libc::c_int,
    6145 as libc::c_int,
    8193 as libc::c_int,
    12289 as libc::c_int,
    16385 as libc::c_int,
    24577 as libc::c_int,
    0 as libc::c_int,
    0 as libc::c_int,
];
static mut stbi__zlength_extra: [libc::c_int; 31] = [
    0 as libc::c_int,
    0 as libc::c_int,
    0 as libc::c_int,
    0 as libc::c_int,
    0 as libc::c_int,
    0 as libc::c_int,
    0 as libc::c_int,
    0 as libc::c_int,
    1 as libc::c_int,
    1 as libc::c_int,
    1 as libc::c_int,
    1 as libc::c_int,
    2 as libc::c_int,
    2 as libc::c_int,
    2 as libc::c_int,
    2 as libc::c_int,
    3 as libc::c_int,
    3 as libc::c_int,
    3 as libc::c_int,
    3 as libc::c_int,
    4 as libc::c_int,
    4 as libc::c_int,
    4 as libc::c_int,
    4 as libc::c_int,
    5 as libc::c_int,
    5 as libc::c_int,
    5 as libc::c_int,
    5 as libc::c_int,
    0 as libc::c_int,
    0 as libc::c_int,
    0 as libc::c_int,
];
static mut stbi__zlength_base: [libc::c_int; 31] = [
    3 as libc::c_int,
    4 as libc::c_int,
    5 as libc::c_int,
    6 as libc::c_int,
    7 as libc::c_int,
    8 as libc::c_int,
    9 as libc::c_int,
    10 as libc::c_int,
    11 as libc::c_int,
    13 as libc::c_int,
    15 as libc::c_int,
    17 as libc::c_int,
    19 as libc::c_int,
    23 as libc::c_int,
    27 as libc::c_int,
    31 as libc::c_int,
    35 as libc::c_int,
    43 as libc::c_int,
    51 as libc::c_int,
    59 as libc::c_int,
    67 as libc::c_int,
    83 as libc::c_int,
    99 as libc::c_int,
    115 as libc::c_int,
    131 as libc::c_int,
    163 as libc::c_int,
    195 as libc::c_int,
    227 as libc::c_int,
    258 as libc::c_int,
    0 as libc::c_int,
    0 as libc::c_int,
];
unsafe extern "C" fn stbi__compute_huffman_codes(mut a: *mut stbi__zbuf) -> libc::c_int {
    static mut length_dezigzag: [stbi_uc; 19] = [
        16 as libc::c_int as stbi_uc,
        17 as libc::c_int as stbi_uc,
        18 as libc::c_int as stbi_uc,
        0 as libc::c_int as stbi_uc,
        8 as libc::c_int as stbi_uc,
        7 as libc::c_int as stbi_uc,
        9 as libc::c_int as stbi_uc,
        6 as libc::c_int as stbi_uc,
        10 as libc::c_int as stbi_uc,
        5 as libc::c_int as stbi_uc,
        11 as libc::c_int as stbi_uc,
        4 as libc::c_int as stbi_uc,
        12 as libc::c_int as stbi_uc,
        3 as libc::c_int as stbi_uc,
        13 as libc::c_int as stbi_uc,
        2 as libc::c_int as stbi_uc,
        14 as libc::c_int as stbi_uc,
        1 as libc::c_int as stbi_uc,
        15 as libc::c_int as stbi_uc,
    ];
    let mut z_codelength: stbi__zhuffman = stbi__zhuffman {
        fast: [0; 512],
        firstcode: [0; 16],
        maxcode: [0; 17],
        firstsymbol: [0; 16],
        size: [0; 288],
        value: [0; 288],
    };
    let mut lencodes: [stbi_uc; 455] = [0; 455];
    let mut codelength_sizes: [stbi_uc; 19] = [0; 19];
    let mut i: libc::c_int = 0;
    let mut n: libc::c_int = 0;
    let mut hlit: libc::c_int = (stbi__zreceive(a, 5 as libc::c_int))
        .wrapping_add(257 as libc::c_int as libc::c_uint) as libc::c_int;
    let mut hdist: libc::c_int = (stbi__zreceive(a, 5 as libc::c_int))
        .wrapping_add(1 as libc::c_int as libc::c_uint) as libc::c_int;
    let mut hclen: libc::c_int = (stbi__zreceive(a, 4 as libc::c_int))
        .wrapping_add(4 as libc::c_int as libc::c_uint) as libc::c_int;
    let mut ntot: libc::c_int = hlit + hdist;
    memset(
        codelength_sizes.as_mut_ptr() as *mut libc::c_void,
        0 as libc::c_int,
        ::core::mem::size_of::<[stbi_uc; 19]>() as libc::c_ulong,
    );
    i = 0 as libc::c_int;
    while i < hclen {
        let mut s: libc::c_int = stbi__zreceive(a, 3 as libc::c_int) as libc::c_int;
        codelength_sizes[length_dezigzag[i as usize] as usize] = s as stbi_uc;
        i += 1;
    }
    if stbi__zbuild_huffman(
        &mut z_codelength,
        codelength_sizes.as_mut_ptr(),
        19 as libc::c_int,
    ) == 0
    {
        return 0 as libc::c_int;
    }
    n = 0 as libc::c_int;
    while n < ntot {
        let mut c: libc::c_int = stbi__zhuffman_decode(a, &mut z_codelength);
        if c < 0 as libc::c_int || c >= 19 as libc::c_int {
            return stbi__err(b"bad codelengths\0" as *const u8 as *const libc::c_char);
        }
        if c < 16 as libc::c_int {
            let fresh97 = n;
            n = n + 1;
            lencodes[fresh97 as usize] = c as stbi_uc;
        } else {
            let mut fill: stbi_uc = 0 as libc::c_int as stbi_uc;
            if c == 16 as libc::c_int {
                c = (stbi__zreceive(a, 2 as libc::c_int))
                    .wrapping_add(3 as libc::c_int as libc::c_uint) as libc::c_int;
                if n == 0 as libc::c_int {
                    return stbi__err(
                        b"bad codelengths\0" as *const u8 as *const libc::c_char,
                    );
                }
                fill = lencodes[(n - 1 as libc::c_int) as usize];
            } else if c == 17 as libc::c_int {
                c = (stbi__zreceive(a, 3 as libc::c_int))
                    .wrapping_add(3 as libc::c_int as libc::c_uint) as libc::c_int;
            } else if c == 18 as libc::c_int {
                c = (stbi__zreceive(a, 7 as libc::c_int))
                    .wrapping_add(11 as libc::c_int as libc::c_uint) as libc::c_int;
            } else {
                return stbi__err(
                    b"bad codelengths\0" as *const u8 as *const libc::c_char,
                )
            }
            if ntot - n < c {
                return stbi__err(
                    b"bad codelengths\0" as *const u8 as *const libc::c_char,
                );
            }
            memset(
                lencodes.as_mut_ptr().offset(n as isize) as *mut libc::c_void,
                fill as libc::c_int,
                c as libc::c_ulong,
            );
            n += c;
        }
    }
    if n != ntot {
        return stbi__err(b"bad codelengths\0" as *const u8 as *const libc::c_char);
    }
    if stbi__zbuild_huffman(&mut (*a).z_length, lencodes.as_mut_ptr(), hlit) == 0 {
        return 0 as libc::c_int;
    }
    if stbi__zbuild_huffman(
        &mut (*a).z_distance,
        lencodes.as_mut_ptr().offset(hlit as isize),
        hdist,
    ) == 0
    {
        return 0 as libc::c_int;
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__zbuild_huffman(
    mut z: *mut stbi__zhuffman,
    mut sizelist: *const stbi_uc,
    mut num: libc::c_int,
) -> libc::c_int {
    let mut i: libc::c_int = 0;
    let mut k: libc::c_int = 0 as libc::c_int;
    let mut code: libc::c_int = 0;
    let mut next_code: [libc::c_int; 16] = [0; 16];
    let mut sizes: [libc::c_int; 17] = [0; 17];
    memset(
        sizes.as_mut_ptr() as *mut libc::c_void,
        0 as libc::c_int,
        ::core::mem::size_of::<[libc::c_int; 17]>() as libc::c_ulong,
    );
    memset(
        ((*z).fast).as_mut_ptr() as *mut libc::c_void,
        0 as libc::c_int,
        ::core::mem::size_of::<[stbi__uint16; 512]>() as libc::c_ulong,
    );
    i = 0 as libc::c_int;
    while i < num {
        sizes[*sizelist.offset(i as isize) as usize] += 1;
        i += 1;
    }
    sizes[0 as libc::c_int as usize] = 0 as libc::c_int;
    i = 1 as libc::c_int;
    while i < 16 as libc::c_int {
        if sizes[i as usize] > (1 as libc::c_int) << i {
            return stbi__err(b"bad sizes\0" as *const u8 as *const libc::c_char);
        }
        i += 1;
    }
    code = 0 as libc::c_int;
    i = 1 as libc::c_int;
    while i < 16 as libc::c_int {
        next_code[i as usize] = code;
        (*z).firstcode[i as usize] = code as stbi__uint16;
        (*z).firstsymbol[i as usize] = k as stbi__uint16;
        code = code + sizes[i as usize];
        if sizes[i as usize] != 0 {
            if code - 1 as libc::c_int >= (1 as libc::c_int) << i {
                return stbi__err(
                    b"bad codelengths\0" as *const u8 as *const libc::c_char,
                );
            }
        }
        (*z).maxcode[i as usize] = code << 16 as libc::c_int - i;
        code <<= 1 as libc::c_int;
        k += sizes[i as usize];
        i += 1;
    }
    (*z).maxcode[16 as libc::c_int as usize] = 0x10000 as libc::c_int;
    i = 0 as libc::c_int;
    while i < num {
        let mut s: libc::c_int = *sizelist.offset(i as isize) as libc::c_int;
        if s != 0 {
            let mut c: libc::c_int = next_code[s as usize]
                - (*z).firstcode[s as usize] as libc::c_int
                + (*z).firstsymbol[s as usize] as libc::c_int;
            let mut fastv: stbi__uint16 = (s << 9 as libc::c_int | i) as stbi__uint16;
            (*z).size[c as usize] = s as stbi_uc;
            (*z).value[c as usize] = i as stbi__uint16;
            if s <= 9 as libc::c_int {
                let mut j: libc::c_int = stbi__bit_reverse(next_code[s as usize], s);
                while j < (1 as libc::c_int) << 9 as libc::c_int {
                    (*z).fast[j as usize] = fastv;
                    j += (1 as libc::c_int) << s;
                }
            }
            next_code[s as usize] += 1;
        }
        i += 1;
    }
    return 1 as libc::c_int;
}
static mut stbi__zdefault_distance: [stbi_uc; 32] = [
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
    5 as libc::c_int as stbi_uc,
];
static mut stbi__zdefault_length: [stbi_uc; 288] = [
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    9 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    7 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
    8 as libc::c_int as stbi_uc,
];
unsafe extern "C" fn stbi__parse_uncompressed_block(
    mut a: *mut stbi__zbuf,
) -> libc::c_int {
    let mut header: [stbi_uc; 4] = [0; 4];
    let mut len: libc::c_int = 0;
    let mut nlen: libc::c_int = 0;
    let mut k: libc::c_int = 0;
    if (*a).num_bits & 7 as libc::c_int != 0 {
        stbi__zreceive(a, (*a).num_bits & 7 as libc::c_int);
    }
    k = 0 as libc::c_int;
    while (*a).num_bits > 0 as libc::c_int {
        let fresh98 = k;
        k = k + 1;
        header[fresh98
            as usize] = ((*a).code_buffer & 255 as libc::c_int as libc::c_uint)
            as stbi_uc;
        (*a).code_buffer >>= 8 as libc::c_int;
        (*a).num_bits -= 8 as libc::c_int;
    }
    if (*a).num_bits < 0 as libc::c_int {
        return stbi__err(b"zlib corrupt\0" as *const u8 as *const libc::c_char);
    }
    while k < 4 as libc::c_int {
        let fresh99 = k;
        k = k + 1;
        header[fresh99 as usize] = stbi__zget8(a);
    }
    len = header[1 as libc::c_int as usize] as libc::c_int * 256 as libc::c_int
        + header[0 as libc::c_int as usize] as libc::c_int;
    nlen = header[3 as libc::c_int as usize] as libc::c_int * 256 as libc::c_int
        + header[2 as libc::c_int as usize] as libc::c_int;
    if nlen != len ^ 0xffff as libc::c_int {
        return stbi__err(b"zlib corrupt\0" as *const u8 as *const libc::c_char);
    }
    if ((*a).zbuffer).offset(len as isize) > (*a).zbuffer_end {
        return stbi__err(b"read past buffer\0" as *const u8 as *const libc::c_char);
    }
    if ((*a).zout).offset(len as isize) > (*a).zout_end {
        if stbi__zexpand(a, (*a).zout, len) == 0 {
            return 0 as libc::c_int;
        }
    }
    memcpy(
        (*a).zout as *mut libc::c_void,
        (*a).zbuffer as *const libc::c_void,
        len as libc::c_ulong,
    );
    (*a).zbuffer = ((*a).zbuffer).offset(len as isize);
    (*a).zout = ((*a).zout).offset(len as isize);
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__parse_zlib_header(mut a: *mut stbi__zbuf) -> libc::c_int {
    let mut cmf: libc::c_int = stbi__zget8(a) as libc::c_int;
    let mut cm: libc::c_int = cmf & 15 as libc::c_int;
    let mut flg: libc::c_int = stbi__zget8(a) as libc::c_int;
    if stbi__zeof(a) != 0 {
        return stbi__err(b"bad zlib header\0" as *const u8 as *const libc::c_char);
    }
    if (cmf * 256 as libc::c_int + flg) % 31 as libc::c_int != 0 as libc::c_int {
        return stbi__err(b"bad zlib header\0" as *const u8 as *const libc::c_char);
    }
    if flg & 32 as libc::c_int != 0 {
        return stbi__err(b"no preset dict\0" as *const u8 as *const libc::c_char);
    }
    if cm != 8 as libc::c_int {
        return stbi__err(b"bad compression\0" as *const u8 as *const libc::c_char);
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__check_png_header(mut s: *mut stbi__context) -> libc::c_int {
    static mut png_sig: [stbi_uc; 8] = [
        137 as libc::c_int as stbi_uc,
        80 as libc::c_int as stbi_uc,
        78 as libc::c_int as stbi_uc,
        71 as libc::c_int as stbi_uc,
        13 as libc::c_int as stbi_uc,
        10 as libc::c_int as stbi_uc,
        26 as libc::c_int as stbi_uc,
        10 as libc::c_int as stbi_uc,
    ];
    let mut i: libc::c_int = 0;
    i = 0 as libc::c_int;
    while i < 8 as libc::c_int {
        if stbi__get8(s) as libc::c_int != png_sig[i as usize] as libc::c_int {
            return stbi__err(b"bad png sig\0" as *const u8 as *const libc::c_char);
        }
        i += 1;
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__png_test(mut s: *mut stbi__context) -> libc::c_int {
    let mut r: libc::c_int = 0;
    r = stbi__check_png_header(s);
    stbi__rewind(s);
    return r;
}
unsafe extern "C" fn stbi__vertical_flip(
    mut image: *mut libc::c_void,
    mut w: libc::c_int,
    mut h: libc::c_int,
    mut bytes_per_pixel: libc::c_int,
) {
    let mut row: libc::c_int = 0;
    let mut bytes_per_row: size_t = (w as size_t)
        .wrapping_mul(bytes_per_pixel as libc::c_ulong);
    let mut temp: [stbi_uc; 2048] = [0; 2048];
    let mut bytes: *mut stbi_uc = image as *mut stbi_uc;
    row = 0 as libc::c_int;
    while row < h >> 1 as libc::c_int {
        let mut row0: *mut stbi_uc = bytes
            .offset((row as libc::c_ulong).wrapping_mul(bytes_per_row) as isize);
        let mut row1: *mut stbi_uc = bytes
            .offset(
                ((h - row - 1 as libc::c_int) as libc::c_ulong)
                    .wrapping_mul(bytes_per_row) as isize,
            );
        let mut bytes_left: size_t = bytes_per_row;
        while bytes_left != 0 {
            let mut bytes_copy: size_t = if bytes_left
                < ::core::mem::size_of::<[stbi_uc; 2048]>() as libc::c_ulong
            {
                bytes_left
            } else {
                ::core::mem::size_of::<[stbi_uc; 2048]>() as libc::c_ulong
            };
            memcpy(
                temp.as_mut_ptr() as *mut libc::c_void,
                row0 as *const libc::c_void,
                bytes_copy,
            );
            memcpy(row0 as *mut libc::c_void, row1 as *const libc::c_void, bytes_copy);
            memcpy(
                row1 as *mut libc::c_void,
                temp.as_mut_ptr() as *const libc::c_void,
                bytes_copy,
            );
            row0 = row0.offset(bytes_copy as isize);
            row1 = row1.offset(bytes_copy as isize);
            bytes_left = (bytes_left as libc::c_ulong).wrapping_sub(bytes_copy) as size_t
                as size_t;
        }
        row += 1;
    }
}
static mut stbi__vertically_flip_on_load_global: libc::c_int = 0 as libc::c_int;
#[thread_local]
static mut stbi__vertically_flip_on_load_local: libc::c_int = 0;
#[thread_local]
static mut stbi__vertically_flip_on_load_set: libc::c_int = 0;
unsafe extern "C" fn stbi__convert_16_to_8(
    mut orig: *mut stbi__uint16,
    mut w: libc::c_int,
    mut h: libc::c_int,
    mut channels: libc::c_int,
) -> *mut stbi_uc {
    let mut i: libc::c_int = 0;
    let mut img_len: libc::c_int = w * h * channels;
    let mut reduced: *mut stbi_uc = 0 as *mut stbi_uc;
    reduced = stbi__malloc(img_len as size_t) as *mut stbi_uc;
    if reduced.is_null() {
        return (if stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar;
    }
    i = 0 as libc::c_int;
    while i < img_len {
        *reduced
            .offset(
                i as isize,
            ) = (*orig.offset(i as isize) as libc::c_int >> 8 as libc::c_int
            & 0xff as libc::c_int) as stbi_uc;
        i += 1;
    }
    free(orig as *mut libc::c_void);
    return reduced;
}
unsafe extern "C" fn stbi__start_mem(
    mut s: *mut stbi__context,
    mut buffer: *const stbi_uc,
    mut len: libc::c_int,
) {
    (*s).io.read = None;
    (*s).read_from_callbacks = 0 as libc::c_int;
    (*s).callback_already_read = 0 as libc::c_int;
    (*s).img_buffer_original = buffer as *mut stbi_uc;
    (*s).img_buffer = (*s).img_buffer_original;
    (*s).img_buffer_original_end = (buffer as *mut stbi_uc).offset(len as isize);
    (*s).img_buffer_end = (*s).img_buffer_original_end;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_load_from_callbacks(
    mut clbk: *const stbi_io_callbacks,
    mut user: *mut libc::c_void,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
) -> *mut stbi_uc {
    let mut s: stbi__context = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: 0 as *mut libc::c_void,
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: 0 as *mut stbi_uc,
        img_buffer_end: 0 as *mut stbi_uc,
        img_buffer_original: 0 as *mut stbi_uc,
        img_buffer_original_end: 0 as *mut stbi_uc,
    };
    stbi__start_callbacks(&mut s, clbk as *mut stbi_io_callbacks, user);
    return stbi__load_and_postprocess_8bit(&mut s, x, y, comp, req_comp);
}
unsafe extern "C" fn stbi__start_callbacks(
    mut s: *mut stbi__context,
    mut c: *mut stbi_io_callbacks,
    mut user: *mut libc::c_void,
) {
    (*s).io = *c;
    (*s).io_user_data = user;
    (*s)
        .buflen = ::core::mem::size_of::<[stbi_uc; 128]>() as libc::c_ulong
        as libc::c_int;
    (*s).read_from_callbacks = 1 as libc::c_int;
    (*s).callback_already_read = 0 as libc::c_int;
    (*s).img_buffer_original = ((*s).buffer_start).as_mut_ptr();
    (*s).img_buffer = (*s).img_buffer_original;
    stbi__refill_buffer(s);
    (*s).img_buffer_original_end = (*s).img_buffer_end;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_load(
    mut filename: *const libc::c_char,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
) -> *mut stbi_uc {
    let mut f: *mut FILE = stbi__fopen(
        filename,
        b"rb\0" as *const u8 as *const libc::c_char,
    );
    let mut result: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
    if f.is_null() {
        return (if stbi__err(b"can't fopen\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar;
    }
    result = stbi_load_from_file(f, x, y, comp, req_comp);
    fclose(f);
    return result;
}
unsafe extern "C" fn stbi__fopen(
    mut filename: *const libc::c_char,
    mut mode: *const libc::c_char,
) -> *mut FILE {
    let mut f: *mut FILE = 0 as *mut FILE;
    f = fopen(filename, mode);
    return f;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_load_from_file(
    mut f: *mut FILE,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
) -> *mut stbi_uc {
    let mut result: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
    let mut s: stbi__context = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: 0 as *mut libc::c_void,
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: 0 as *mut stbi_uc,
        img_buffer_end: 0 as *mut stbi_uc,
        img_buffer_original: 0 as *mut stbi_uc,
        img_buffer_original_end: 0 as *mut stbi_uc,
    };
    stbi__start_file(&mut s, f);
    result = stbi__load_and_postprocess_8bit(&mut s, x, y, comp, req_comp);
    if !result.is_null() {
        fseek(
            f,
            -((s.img_buffer_end).offset_from(s.img_buffer) as libc::c_long
                as libc::c_int) as libc::c_long,
            1 as libc::c_int,
        );
    }
    return result;
}
unsafe extern "C" fn stbi__start_file(mut s: *mut stbi__context, mut f: *mut FILE) {
    stbi__start_callbacks(s, &mut stbi__stdio_callbacks, f as *mut libc::c_void);
}
static mut stbi__stdio_callbacks: stbi_io_callbacks = unsafe {
    {
        let mut init = stbi_io_callbacks {
            read: Some(
                stbi__stdio_read
                    as unsafe extern "C" fn(
                        *mut libc::c_void,
                        *mut libc::c_char,
                        libc::c_int,
                    ) -> libc::c_int,
            ),
            skip: Some(
                stbi__stdio_skip
                    as unsafe extern "C" fn(*mut libc::c_void, libc::c_int) -> (),
            ),
            eof: Some(
                stbi__stdio_eof as unsafe extern "C" fn(*mut libc::c_void) -> libc::c_int,
            ),
        };
        init
    }
};
unsafe extern "C" fn stbi__stdio_eof(mut user: *mut libc::c_void) -> libc::c_int {
    return (feof(user as *mut FILE) != 0 || ferror(user as *mut FILE) != 0)
        as libc::c_int;
}
unsafe extern "C" fn stbi__stdio_skip(mut user: *mut libc::c_void, mut n: libc::c_int) {
    let mut ch: libc::c_int = 0;
    fseek(user as *mut FILE, n as libc::c_long, 1 as libc::c_int);
    ch = fgetc(user as *mut FILE);
    if ch != -(1 as libc::c_int) {
        ungetc(ch, user as *mut FILE);
    }
}
unsafe extern "C" fn stbi__stdio_read(
    mut user: *mut libc::c_void,
    mut data: *mut libc::c_char,
    mut size: libc::c_int,
) -> libc::c_int {
    return fread(
        data as *mut libc::c_void,
        1 as libc::c_int as libc::c_ulong,
        size as libc::c_ulong,
        user as *mut FILE,
    ) as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_load_gif_from_memory(
    mut buffer: *const stbi_uc,
    mut len: libc::c_int,
    mut delays: *mut *mut libc::c_int,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut z: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
) -> *mut stbi_uc {
    let mut result: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
    let mut s: stbi__context = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: 0 as *mut libc::c_void,
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: 0 as *mut stbi_uc,
        img_buffer_end: 0 as *mut stbi_uc,
        img_buffer_original: 0 as *mut stbi_uc,
        img_buffer_original_end: 0 as *mut stbi_uc,
    };
    stbi__start_mem(&mut s, buffer, len);
    result = stbi__load_gif_main(&mut s, delays, x, y, z, comp, req_comp)
        as *mut libc::c_uchar;
    if if stbi__vertically_flip_on_load_set != 0 {
        stbi__vertically_flip_on_load_local
    } else {
        stbi__vertically_flip_on_load_global
    } != 0
    {
        stbi__vertical_flip_slices(result as *mut libc::c_void, *x, *y, *z, *comp);
    }
    return result;
}
unsafe extern "C" fn stbi__vertical_flip_slices(
    mut image: *mut libc::c_void,
    mut w: libc::c_int,
    mut h: libc::c_int,
    mut z: libc::c_int,
    mut bytes_per_pixel: libc::c_int,
) {
    let mut slice: libc::c_int = 0;
    let mut slice_size: libc::c_int = w * h * bytes_per_pixel;
    let mut bytes: *mut stbi_uc = image as *mut stbi_uc;
    slice = 0 as libc::c_int;
    while slice < z {
        stbi__vertical_flip(bytes as *mut libc::c_void, w, h, bytes_per_pixel);
        bytes = bytes.offset(slice_size as isize);
        slice += 1;
    }
}
unsafe extern "C" fn stbi__load_gif_main(
    mut s: *mut stbi__context,
    mut delays: *mut *mut libc::c_int,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut z: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
) -> *mut libc::c_void {
    if stbi__gif_test(s) != 0 {
        let mut layers: libc::c_int = 0 as libc::c_int;
        let mut u: *mut stbi_uc = 0 as *mut stbi_uc;
        let mut out: *mut stbi_uc = 0 as *mut stbi_uc;
        let mut two_back: *mut stbi_uc = 0 as *mut stbi_uc;
        let mut g: stbi__gif = stbi__gif {
            w: 0,
            h: 0,
            out: 0 as *mut stbi_uc,
            background: 0 as *mut stbi_uc,
            history: 0 as *mut stbi_uc,
            flags: 0,
            bgindex: 0,
            ratio: 0,
            transparent: 0,
            eflags: 0,
            pal: [[0; 4]; 256],
            lpal: [[0; 4]; 256],
            codes: [stbi__gif_lzw {
                prefix: 0,
                first: 0,
                suffix: 0,
            }; 8192],
            color_table: 0 as *mut stbi_uc,
            parse: 0,
            step: 0,
            lflags: 0,
            start_x: 0,
            start_y: 0,
            max_x: 0,
            max_y: 0,
            cur_x: 0,
            cur_y: 0,
            line_size: 0,
            delay: 0,
        };
        let mut stride: libc::c_int = 0;
        let mut out_size: libc::c_int = 0 as libc::c_int;
        let mut delays_size: libc::c_int = 0 as libc::c_int;
        memset(
            &mut g as *mut stbi__gif as *mut libc::c_void,
            0 as libc::c_int,
            ::core::mem::size_of::<stbi__gif>() as libc::c_ulong,
        );
        if !delays.is_null() {
            *delays = 0 as *mut libc::c_int;
        }
        loop {
            u = stbi__gif_load_next(s, &mut g, comp, req_comp, two_back);
            if u == s as *mut stbi_uc {
                u = 0 as *mut stbi_uc;
            }
            if !u.is_null() {
                *x = g.w;
                *y = g.h;
                layers += 1;
                stride = g.w * g.h * 4 as libc::c_int;
                if !out.is_null() {
                    let mut tmp: *mut libc::c_void = realloc(
                        out as *mut libc::c_void,
                        (layers * stride) as libc::c_ulong,
                    ) as *mut stbi_uc as *mut libc::c_void;
                    if tmp.is_null() {
                        return stbi__load_gif_main_outofmem(&mut g, out, delays)
                    } else {
                        out = tmp as *mut stbi_uc;
                        out_size = layers * stride;
                    }
                    if !delays.is_null() {
                        let mut new_delays: *mut libc::c_int = realloc(
                            *delays as *mut libc::c_void,
                            (::core::mem::size_of::<libc::c_int>() as libc::c_ulong)
                                .wrapping_mul(layers as libc::c_ulong),
                        ) as *mut libc::c_int;
                        if new_delays.is_null() {
                            return stbi__load_gif_main_outofmem(&mut g, out, delays);
                        }
                        *delays = new_delays;
                        delays_size = (layers as libc::c_ulong)
                            .wrapping_mul(
                                ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
                            ) as libc::c_int;
                    }
                } else {
                    out = stbi__malloc((layers * stride) as size_t) as *mut stbi_uc;
                    if out.is_null() {
                        return stbi__load_gif_main_outofmem(&mut g, out, delays);
                    }
                    out_size = layers * stride;
                    if !delays.is_null() {
                        *delays = stbi__malloc(
                            (layers as libc::c_ulong)
                                .wrapping_mul(
                                    ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
                                ),
                        ) as *mut libc::c_int;
                        if (*delays).is_null() {
                            return stbi__load_gif_main_outofmem(&mut g, out, delays);
                        }
                        delays_size = (layers as libc::c_ulong)
                            .wrapping_mul(
                                ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
                            ) as libc::c_int;
                    }
                }
                memcpy(
                    out.offset(((layers - 1 as libc::c_int) * stride) as isize)
                        as *mut libc::c_void,
                    u as *const libc::c_void,
                    stride as libc::c_ulong,
                );
                if layers >= 2 as libc::c_int {
                    two_back = out.offset(-((2 as libc::c_int * stride) as isize));
                }
                if !delays.is_null() {
                    *(*delays)
                        .offset(
                            (layers as libc::c_uint).wrapping_sub(1 as libc::c_uint)
                                as isize,
                        ) = g.delay;
                }
            }
            if u.is_null() {
                break;
            }
        }
        free(g.out as *mut libc::c_void);
        free(g.history as *mut libc::c_void);
        free(g.background as *mut libc::c_void);
        if req_comp != 0 && req_comp != 4 as libc::c_int {
            out = stbi__convert_format(
                out,
                4 as libc::c_int,
                req_comp,
                (layers * g.w) as libc::c_uint,
                g.h as libc::c_uint,
            );
        }
        *z = layers;
        return out as *mut libc::c_void;
    } else {
        return (if stbi__err(b"not GIF\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut libc::c_void
    };
}
unsafe extern "C" fn stbi__load_gif_main_outofmem(
    mut g: *mut stbi__gif,
    mut out: *mut stbi_uc,
    mut delays: *mut *mut libc::c_int,
) -> *mut libc::c_void {
    free((*g).out as *mut libc::c_void);
    free((*g).history as *mut libc::c_void);
    free((*g).background as *mut libc::c_void);
    if !out.is_null() {
        free(out as *mut libc::c_void);
    }
    if !delays.is_null() && !(*delays).is_null() {
        free(*delays as *mut libc::c_void);
    }
    return (if stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char) != 0 {
        0 as *mut libc::c_void
    } else {
        0 as *mut libc::c_void
    }) as size_t as *mut libc::c_uchar as *mut libc::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_load_16_from_memory(
    mut buffer: *const stbi_uc,
    mut len: libc::c_int,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut channels_in_file: *mut libc::c_int,
    mut desired_channels: libc::c_int,
) -> *mut stbi_us {
    let mut s: stbi__context = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: 0 as *mut libc::c_void,
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: 0 as *mut stbi_uc,
        img_buffer_end: 0 as *mut stbi_uc,
        img_buffer_original: 0 as *mut stbi_uc,
        img_buffer_original_end: 0 as *mut stbi_uc,
    };
    stbi__start_mem(&mut s, buffer, len);
    return stbi__load_and_postprocess_16bit(
        &mut s,
        x,
        y,
        channels_in_file,
        desired_channels,
    );
}
unsafe extern "C" fn stbi__load_and_postprocess_16bit(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
) -> *mut stbi__uint16 {
    let mut ri: stbi__result_info = stbi__result_info {
        bits_per_channel: 0,
        num_channels: 0,
        channel_order: 0,
    };
    let mut result: *mut libc::c_void = stbi__load_main(
        s,
        x,
        y,
        comp,
        req_comp,
        &mut ri,
        16 as libc::c_int,
    );
    if result.is_null() {
        return 0 as *mut stbi__uint16;
    }
    if ri.bits_per_channel != 16 as libc::c_int {
        result = stbi__convert_8_to_16(
            result as *mut stbi_uc,
            *x,
            *y,
            if req_comp == 0 as libc::c_int { *comp } else { req_comp },
        ) as *mut libc::c_void;
        ri.bits_per_channel = 16 as libc::c_int;
    }
    if if stbi__vertically_flip_on_load_set != 0 {
        stbi__vertically_flip_on_load_local
    } else {
        stbi__vertically_flip_on_load_global
    } != 0
    {
        let mut channels: libc::c_int = if req_comp != 0 { req_comp } else { *comp };
        stbi__vertical_flip(
            result,
            *x,
            *y,
            (channels as libc::c_ulong)
                .wrapping_mul(::core::mem::size_of::<stbi__uint16>() as libc::c_ulong)
                as libc::c_int,
        );
    }
    return result as *mut stbi__uint16;
}
unsafe extern "C" fn stbi__convert_8_to_16(
    mut orig: *mut stbi_uc,
    mut w: libc::c_int,
    mut h: libc::c_int,
    mut channels: libc::c_int,
) -> *mut stbi__uint16 {
    let mut i: libc::c_int = 0;
    let mut img_len: libc::c_int = w * h * channels;
    let mut enlarged: *mut stbi__uint16 = 0 as *mut stbi__uint16;
    enlarged = stbi__malloc((img_len * 2 as libc::c_int) as size_t) as *mut stbi__uint16;
    if enlarged.is_null() {
        return (if stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut stbi__uint16;
    }
    i = 0 as libc::c_int;
    while i < img_len {
        *enlarged
            .offset(
                i as isize,
            ) = (((*orig.offset(i as isize) as libc::c_int) << 8 as libc::c_int)
            + *orig.offset(i as isize) as libc::c_int) as stbi__uint16;
        i += 1;
    }
    free(orig as *mut libc::c_void);
    return enlarged;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_load_16_from_callbacks(
    mut clbk: *const stbi_io_callbacks,
    mut user: *mut libc::c_void,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut channels_in_file: *mut libc::c_int,
    mut desired_channels: libc::c_int,
) -> *mut stbi_us {
    let mut s: stbi__context = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: 0 as *mut libc::c_void,
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: 0 as *mut stbi_uc,
        img_buffer_end: 0 as *mut stbi_uc,
        img_buffer_original: 0 as *mut stbi_uc,
        img_buffer_original_end: 0 as *mut stbi_uc,
    };
    stbi__start_callbacks(&mut s, clbk as *mut stbi_io_callbacks, user);
    return stbi__load_and_postprocess_16bit(
        &mut s,
        x,
        y,
        channels_in_file,
        desired_channels,
    );
}
#[no_mangle]
pub unsafe extern "C" fn stbi_load_16(
    mut filename: *const libc::c_char,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
) -> *mut stbi_us {
    let mut f: *mut FILE = stbi__fopen(
        filename,
        b"rb\0" as *const u8 as *const libc::c_char,
    );
    let mut result: *mut stbi__uint16 = 0 as *mut stbi__uint16;
    if f.is_null() {
        return (if stbi__err(b"can't fopen\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_uchar as *mut stbi_us;
    }
    result = stbi_load_from_file_16(f, x, y, comp, req_comp);
    fclose(f);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_load_from_file_16(
    mut f: *mut FILE,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
) -> *mut stbi_us {
    let mut result: *mut stbi__uint16 = 0 as *mut stbi__uint16;
    let mut s: stbi__context = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: 0 as *mut libc::c_void,
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: 0 as *mut stbi_uc,
        img_buffer_end: 0 as *mut stbi_uc,
        img_buffer_original: 0 as *mut stbi_uc,
        img_buffer_original_end: 0 as *mut stbi_uc,
    };
    stbi__start_file(&mut s, f);
    result = stbi__load_and_postprocess_16bit(&mut s, x, y, comp, req_comp);
    if !result.is_null() {
        fseek(
            f,
            -((s.img_buffer_end).offset_from(s.img_buffer) as libc::c_long
                as libc::c_int) as libc::c_long,
            1 as libc::c_int,
        );
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_loadf_from_memory(
    mut buffer: *const stbi_uc,
    mut len: libc::c_int,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
) -> *mut libc::c_float {
    let mut s: stbi__context = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: 0 as *mut libc::c_void,
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: 0 as *mut stbi_uc,
        img_buffer_end: 0 as *mut stbi_uc,
        img_buffer_original: 0 as *mut stbi_uc,
        img_buffer_original_end: 0 as *mut stbi_uc,
    };
    stbi__start_mem(&mut s, buffer, len);
    return stbi__loadf_main(&mut s, x, y, comp, req_comp);
}
unsafe extern "C" fn stbi__loadf_main(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
) -> *mut libc::c_float {
    let mut data: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
    if stbi__hdr_test(s) != 0 {
        let mut ri: stbi__result_info = stbi__result_info {
            bits_per_channel: 0,
            num_channels: 0,
            channel_order: 0,
        };
        let mut hdr_data: *mut libc::c_float = stbi__hdr_load(
            s,
            x,
            y,
            comp,
            req_comp,
            &mut ri,
        );
        if !hdr_data.is_null() {
            stbi__float_postprocess(hdr_data, x, y, comp, req_comp);
        }
        return hdr_data;
    }
    data = stbi__load_and_postprocess_8bit(s, x, y, comp, req_comp);
    if !data.is_null() {
        return stbi__ldr_to_hdr(
            data,
            *x,
            *y,
            if req_comp != 0 { req_comp } else { *comp },
        );
    }
    return (if stbi__err(b"unknown image type\0" as *const u8 as *const libc::c_char)
        != 0
    {
        0 as *mut libc::c_void
    } else {
        0 as *mut libc::c_void
    }) as size_t as *mut libc::c_float;
}
unsafe extern "C" fn stbi__ldr_to_hdr(
    mut data: *mut stbi_uc,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut comp: libc::c_int,
) -> *mut libc::c_float {
    let mut i: libc::c_int = 0;
    let mut k: libc::c_int = 0;
    let mut n: libc::c_int = 0;
    let mut output: *mut libc::c_float = 0 as *mut libc::c_float;
    if data.is_null() {
        return 0 as *mut libc::c_float;
    }
    output = stbi__malloc_mad4(
        x,
        y,
        comp,
        ::core::mem::size_of::<libc::c_float>() as libc::c_ulong as libc::c_int,
        0 as libc::c_int,
    ) as *mut libc::c_float;
    if output.is_null() {
        free(data as *mut libc::c_void);
        return (if stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_float;
    }
    if comp & 1 as libc::c_int != 0 {
        n = comp;
    } else {
        n = comp - 1 as libc::c_int;
    }
    i = 0 as libc::c_int;
    while i < x * y {
        k = 0 as libc::c_int;
        while k < n {
            *output
                .offset(
                    (i * comp + k) as isize,
                ) = (pow(
                (*data.offset((i * comp + k) as isize) as libc::c_int as libc::c_float
                    / 255.0f32) as libc::c_double,
                stbi__l2h_gamma as libc::c_double,
            ) * stbi__l2h_scale as libc::c_double) as libc::c_float;
            k += 1;
        }
        i += 1;
    }
    if n < comp {
        i = 0 as libc::c_int;
        while i < x * y {
            *output
                .offset(
                    (i * comp + n) as isize,
                ) = *data.offset((i * comp + n) as isize) as libc::c_int as libc::c_float
                / 255.0f32;
            i += 1;
        }
    }
    free(data as *mut libc::c_void);
    return output;
}
static mut stbi__l2h_scale: libc::c_float = 1.0f32;
static mut stbi__l2h_gamma: libc::c_float = 2.2f32;
unsafe extern "C" fn stbi__float_postprocess(
    mut result: *mut libc::c_float,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
) {
    if (if stbi__vertically_flip_on_load_set != 0 {
        stbi__vertically_flip_on_load_local
    } else {
        stbi__vertically_flip_on_load_global
    }) != 0 && !result.is_null()
    {
        let mut channels: libc::c_int = if req_comp != 0 { req_comp } else { *comp };
        stbi__vertical_flip(
            result as *mut libc::c_void,
            *x,
            *y,
            (channels as libc::c_ulong)
                .wrapping_mul(::core::mem::size_of::<libc::c_float>() as libc::c_ulong)
                as libc::c_int,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn stbi_loadf_from_callbacks(
    mut clbk: *const stbi_io_callbacks,
    mut user: *mut libc::c_void,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
) -> *mut libc::c_float {
    let mut s: stbi__context = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: 0 as *mut libc::c_void,
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: 0 as *mut stbi_uc,
        img_buffer_end: 0 as *mut stbi_uc,
        img_buffer_original: 0 as *mut stbi_uc,
        img_buffer_original_end: 0 as *mut stbi_uc,
    };
    stbi__start_callbacks(&mut s, clbk as *mut stbi_io_callbacks, user);
    return stbi__loadf_main(&mut s, x, y, comp, req_comp);
}
#[no_mangle]
pub unsafe extern "C" fn stbi_loadf(
    mut filename: *const libc::c_char,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
) -> *mut libc::c_float {
    let mut result: *mut libc::c_float = 0 as *mut libc::c_float;
    let mut f: *mut FILE = stbi__fopen(
        filename,
        b"rb\0" as *const u8 as *const libc::c_char,
    );
    if f.is_null() {
        return (if stbi__err(b"can't fopen\0" as *const u8 as *const libc::c_char) != 0 {
            0 as *mut libc::c_void
        } else {
            0 as *mut libc::c_void
        }) as size_t as *mut libc::c_float;
    }
    result = stbi_loadf_from_file(f, x, y, comp, req_comp);
    fclose(f);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_loadf_from_file(
    mut f: *mut FILE,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
    mut req_comp: libc::c_int,
) -> *mut libc::c_float {
    let mut s: stbi__context = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: 0 as *mut libc::c_void,
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: 0 as *mut stbi_uc,
        img_buffer_end: 0 as *mut stbi_uc,
        img_buffer_original: 0 as *mut stbi_uc,
        img_buffer_original_end: 0 as *mut stbi_uc,
    };
    stbi__start_file(&mut s, f);
    return stbi__loadf_main(&mut s, x, y, comp, req_comp);
}
#[no_mangle]
pub unsafe extern "C" fn stbi_hdr_to_ldr_gamma(mut gamma: libc::c_float) {
    stbi__h2l_gamma_i = 1 as libc::c_int as libc::c_float / gamma;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_hdr_to_ldr_scale(mut scale: libc::c_float) {
    stbi__h2l_scale_i = 1 as libc::c_int as libc::c_float / scale;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_ldr_to_hdr_gamma(mut gamma: libc::c_float) {
    stbi__l2h_gamma = gamma;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_ldr_to_hdr_scale(mut scale: libc::c_float) {
    stbi__l2h_scale = scale;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_is_hdr_from_callbacks(
    mut clbk: *const stbi_io_callbacks,
    mut user: *mut libc::c_void,
) -> libc::c_int {
    let mut s: stbi__context = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: 0 as *mut libc::c_void,
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: 0 as *mut stbi_uc,
        img_buffer_end: 0 as *mut stbi_uc,
        img_buffer_original: 0 as *mut stbi_uc,
        img_buffer_original_end: 0 as *mut stbi_uc,
    };
    stbi__start_callbacks(&mut s, clbk as *mut stbi_io_callbacks, user);
    return stbi__hdr_test(&mut s);
}
#[no_mangle]
pub unsafe extern "C" fn stbi_is_hdr_from_memory(
    mut buffer: *const stbi_uc,
    mut len: libc::c_int,
) -> libc::c_int {
    let mut s: stbi__context = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: 0 as *mut libc::c_void,
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: 0 as *mut stbi_uc,
        img_buffer_end: 0 as *mut stbi_uc,
        img_buffer_original: 0 as *mut stbi_uc,
        img_buffer_original_end: 0 as *mut stbi_uc,
    };
    stbi__start_mem(&mut s, buffer, len);
    return stbi__hdr_test(&mut s);
}
#[no_mangle]
pub unsafe extern "C" fn stbi_is_hdr(mut filename: *const libc::c_char) -> libc::c_int {
    let mut f: *mut FILE = stbi__fopen(
        filename,
        b"rb\0" as *const u8 as *const libc::c_char,
    );
    let mut result: libc::c_int = 0 as libc::c_int;
    if !f.is_null() {
        result = stbi_is_hdr_from_file(f);
        fclose(f);
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_is_hdr_from_file(mut f: *mut FILE) -> libc::c_int {
    let mut pos: libc::c_long = ftell(f);
    let mut res: libc::c_int = 0;
    let mut s: stbi__context = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: 0 as *mut libc::c_void,
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: 0 as *mut stbi_uc,
        img_buffer_end: 0 as *mut stbi_uc,
        img_buffer_original: 0 as *mut stbi_uc,
        img_buffer_original_end: 0 as *mut stbi_uc,
    };
    stbi__start_file(&mut s, f);
    res = stbi__hdr_test(&mut s);
    fseek(f, pos, 0 as libc::c_int);
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_failure_reason() -> *const libc::c_char {
    return stbi__g_failure_reason;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_image_free(mut retval_from_stbi_load: *mut libc::c_void) {
    free(retval_from_stbi_load);
}
#[no_mangle]
pub unsafe extern "C" fn stbi_info_from_memory(
    mut buffer: *const stbi_uc,
    mut len: libc::c_int,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
) -> libc::c_int {
    let mut s: stbi__context = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: 0 as *mut libc::c_void,
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: 0 as *mut stbi_uc,
        img_buffer_end: 0 as *mut stbi_uc,
        img_buffer_original: 0 as *mut stbi_uc,
        img_buffer_original_end: 0 as *mut stbi_uc,
    };
    stbi__start_mem(&mut s, buffer, len);
    return stbi__info_main(&mut s, x, y, comp);
}
unsafe extern "C" fn stbi__info_main(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
) -> libc::c_int {
    if stbi__jpeg_info(s, x, y, comp) != 0 {
        return 1 as libc::c_int;
    }
    if stbi__png_info(s, x, y, comp) != 0 {
        return 1 as libc::c_int;
    }
    if stbi__gif_info(s, x, y, comp) != 0 {
        return 1 as libc::c_int;
    }
    if stbi__bmp_info(s, x, y, comp) != 0 {
        return 1 as libc::c_int;
    }
    if stbi__psd_info(s, x, y, comp) != 0 {
        return 1 as libc::c_int;
    }
    if stbi__pic_info(s, x, y, comp) != 0 {
        return 1 as libc::c_int;
    }
    if stbi__pnm_info(s, x, y, comp) != 0 {
        return 1 as libc::c_int;
    }
    if stbi__hdr_info(s, x, y, comp) != 0 {
        return 1 as libc::c_int;
    }
    if stbi__tga_info(s, x, y, comp) != 0 {
        return 1 as libc::c_int;
    }
    return stbi__err(b"unknown image type\0" as *const u8 as *const libc::c_char);
}
unsafe extern "C" fn stbi__tga_info(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
) -> libc::c_int {
    let mut tga_w: libc::c_int = 0;
    let mut tga_h: libc::c_int = 0;
    let mut tga_comp: libc::c_int = 0;
    let mut tga_image_type: libc::c_int = 0;
    let mut tga_bits_per_pixel: libc::c_int = 0;
    let mut tga_colormap_bpp: libc::c_int = 0;
    let mut sz: libc::c_int = 0;
    let mut tga_colormap_type: libc::c_int = 0;
    stbi__get8(s);
    tga_colormap_type = stbi__get8(s) as libc::c_int;
    if tga_colormap_type > 1 as libc::c_int {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    tga_image_type = stbi__get8(s) as libc::c_int;
    if tga_colormap_type == 1 as libc::c_int {
        if tga_image_type != 1 as libc::c_int && tga_image_type != 9 as libc::c_int {
            stbi__rewind(s);
            return 0 as libc::c_int;
        }
        stbi__skip(s, 4 as libc::c_int);
        sz = stbi__get8(s) as libc::c_int;
        if sz != 8 as libc::c_int && sz != 15 as libc::c_int && sz != 16 as libc::c_int
            && sz != 24 as libc::c_int && sz != 32 as libc::c_int
        {
            stbi__rewind(s);
            return 0 as libc::c_int;
        }
        stbi__skip(s, 4 as libc::c_int);
        tga_colormap_bpp = sz;
    } else {
        if tga_image_type != 2 as libc::c_int && tga_image_type != 3 as libc::c_int
            && tga_image_type != 10 as libc::c_int && tga_image_type != 11 as libc::c_int
        {
            stbi__rewind(s);
            return 0 as libc::c_int;
        }
        stbi__skip(s, 9 as libc::c_int);
        tga_colormap_bpp = 0 as libc::c_int;
    }
    tga_w = stbi__get16le(s);
    if tga_w < 1 as libc::c_int {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    tga_h = stbi__get16le(s);
    if tga_h < 1 as libc::c_int {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    tga_bits_per_pixel = stbi__get8(s) as libc::c_int;
    stbi__get8(s);
    if tga_colormap_bpp != 0 as libc::c_int {
        if tga_bits_per_pixel != 8 as libc::c_int
            && tga_bits_per_pixel != 16 as libc::c_int
        {
            stbi__rewind(s);
            return 0 as libc::c_int;
        }
        tga_comp = stbi__tga_get_comp(
            tga_colormap_bpp,
            0 as libc::c_int,
            0 as *mut libc::c_int,
        );
    } else {
        tga_comp = stbi__tga_get_comp(
            tga_bits_per_pixel,
            (tga_image_type == 3 as libc::c_int || tga_image_type == 11 as libc::c_int)
                as libc::c_int,
            0 as *mut libc::c_int,
        );
    }
    if tga_comp == 0 {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    if !x.is_null() {
        *x = tga_w;
    }
    if !y.is_null() {
        *y = tga_h;
    }
    if !comp.is_null() {
        *comp = tga_comp;
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__hdr_info(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
) -> libc::c_int {
    let mut buffer: [libc::c_char; 1024] = [0; 1024];
    let mut token: *mut libc::c_char = 0 as *mut libc::c_char;
    let mut valid: libc::c_int = 0 as libc::c_int;
    let mut dummy: libc::c_int = 0;
    if x.is_null() {
        x = &mut dummy;
    }
    if y.is_null() {
        y = &mut dummy;
    }
    if comp.is_null() {
        comp = &mut dummy;
    }
    if stbi__hdr_test(s) == 0 as libc::c_int {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    loop {
        token = stbi__hdr_gettoken(s, buffer.as_mut_ptr());
        if *token.offset(0 as libc::c_int as isize) as libc::c_int == 0 as libc::c_int {
            break;
        }
        if strcmp(token, b"FORMAT=32-bit_rle_rgbe\0" as *const u8 as *const libc::c_char)
            == 0 as libc::c_int
        {
            valid = 1 as libc::c_int;
        }
    }
    if valid == 0 {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    token = stbi__hdr_gettoken(s, buffer.as_mut_ptr());
    if strncmp(
        token,
        b"-Y \0" as *const u8 as *const libc::c_char,
        3 as libc::c_int as libc::c_ulong,
    ) != 0
    {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    token = token.offset(3 as libc::c_int as isize);
    *y = strtol(token, &mut token, 10 as libc::c_int) as libc::c_int;
    while *token as libc::c_int == ' ' as i32 {
        token = token.offset(1);
    }
    if strncmp(
        token,
        b"+X \0" as *const u8 as *const libc::c_char,
        3 as libc::c_int as libc::c_ulong,
    ) != 0
    {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    token = token.offset(3 as libc::c_int as isize);
    *x = strtol(token, 0 as *mut *mut libc::c_char, 10 as libc::c_int) as libc::c_int;
    *comp = 3 as libc::c_int;
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__pic_info(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
) -> libc::c_int {
    let mut act_comp: libc::c_int = 0 as libc::c_int;
    let mut num_packets: libc::c_int = 0 as libc::c_int;
    let mut chained: libc::c_int = 0;
    let mut dummy: libc::c_int = 0;
    let mut packets: [stbi__pic_packet; 10] = [stbi__pic_packet {
        size: 0,
        type_0: 0,
        channel: 0,
    }; 10];
    if x.is_null() {
        x = &mut dummy;
    }
    if y.is_null() {
        y = &mut dummy;
    }
    if comp.is_null() {
        comp = &mut dummy;
    }
    if stbi__pic_is4(s, b"S\x80\xF64\0" as *const u8 as *const libc::c_char) == 0 {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    stbi__skip(s, 88 as libc::c_int);
    *x = stbi__get16be(s);
    *y = stbi__get16be(s);
    if stbi__at_eof(s) != 0 {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    if *x != 0 as libc::c_int && ((1 as libc::c_int) << 28 as libc::c_int) / *x < *y {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    stbi__skip(s, 8 as libc::c_int);
    loop {
        let mut packet: *mut stbi__pic_packet = 0 as *mut stbi__pic_packet;
        if num_packets as libc::c_ulong
            == (::core::mem::size_of::<[stbi__pic_packet; 10]>() as libc::c_ulong)
                .wrapping_div(
                    ::core::mem::size_of::<stbi__pic_packet>() as libc::c_ulong,
                )
        {
            return 0 as libc::c_int;
        }
        let fresh100 = num_packets;
        num_packets = num_packets + 1;
        packet = &mut *packets.as_mut_ptr().offset(fresh100 as isize)
            as *mut stbi__pic_packet;
        chained = stbi__get8(s) as libc::c_int;
        (*packet).size = stbi__get8(s);
        (*packet).type_0 = stbi__get8(s);
        (*packet).channel = stbi__get8(s);
        act_comp |= (*packet).channel as libc::c_int;
        if stbi__at_eof(s) != 0 {
            stbi__rewind(s);
            return 0 as libc::c_int;
        }
        if (*packet).size as libc::c_int != 8 as libc::c_int {
            stbi__rewind(s);
            return 0 as libc::c_int;
        }
        if !(chained != 0) {
            break;
        }
    }
    *comp = if act_comp & 0x10 as libc::c_int != 0 {
        4 as libc::c_int
    } else {
        3 as libc::c_int
    };
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__psd_info(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
) -> libc::c_int {
    let mut channelCount: libc::c_int = 0;
    let mut dummy: libc::c_int = 0;
    let mut depth: libc::c_int = 0;
    if x.is_null() {
        x = &mut dummy;
    }
    if y.is_null() {
        y = &mut dummy;
    }
    if comp.is_null() {
        comp = &mut dummy;
    }
    if stbi__get32be(s) != 0x38425053 as libc::c_int as libc::c_uint {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    if stbi__get16be(s) != 1 as libc::c_int {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    stbi__skip(s, 6 as libc::c_int);
    channelCount = stbi__get16be(s);
    if channelCount < 0 as libc::c_int || channelCount > 16 as libc::c_int {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    *y = stbi__get32be(s) as libc::c_int;
    *x = stbi__get32be(s) as libc::c_int;
    depth = stbi__get16be(s);
    if depth != 8 as libc::c_int && depth != 16 as libc::c_int {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    if stbi__get16be(s) != 3 as libc::c_int {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    *comp = 4 as libc::c_int;
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__bmp_info(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
) -> libc::c_int {
    let mut p: *mut libc::c_void = 0 as *mut libc::c_void;
    let mut info: stbi__bmp_data = stbi__bmp_data {
        bpp: 0,
        offset: 0,
        hsz: 0,
        mr: 0,
        mg: 0,
        mb: 0,
        ma: 0,
        all_a: 0,
        extra_read: 0,
    };
    info.all_a = 255 as libc::c_int as libc::c_uint;
    p = stbi__bmp_parse_header(s, &mut info);
    if p.is_null() {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    if !x.is_null() {
        *x = (*s).img_x as libc::c_int;
    }
    if !y.is_null() {
        *y = (*s).img_y as libc::c_int;
    }
    if !comp.is_null() {
        if info.bpp == 24 as libc::c_int && info.ma == 0xff000000 as libc::c_uint {
            *comp = 3 as libc::c_int;
        } else {
            *comp = if info.ma != 0 { 4 as libc::c_int } else { 3 as libc::c_int };
        }
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__gif_info(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
) -> libc::c_int {
    return stbi__gif_info_raw(s, x, y, comp);
}
unsafe extern "C" fn stbi__gif_info_raw(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
) -> libc::c_int {
    let mut g: *mut stbi__gif = stbi__malloc(
        ::core::mem::size_of::<stbi__gif>() as libc::c_ulong,
    ) as *mut stbi__gif;
    if g.is_null() {
        return stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char);
    }
    if stbi__gif_header(s, g, comp, 1 as libc::c_int) == 0 {
        free(g as *mut libc::c_void);
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    if !x.is_null() {
        *x = (*g).w;
    }
    if !y.is_null() {
        *y = (*g).h;
    }
    free(g as *mut libc::c_void);
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__png_info(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
) -> libc::c_int {
    let mut p: stbi__png = stbi__png {
        s: 0 as *mut stbi__context,
        idata: 0 as *mut stbi_uc,
        expanded: 0 as *mut stbi_uc,
        out: 0 as *mut stbi_uc,
        depth: 0,
    };
    p.s = s;
    return stbi__png_info_raw(&mut p, x, y, comp);
}
unsafe extern "C" fn stbi__png_info_raw(
    mut p: *mut stbi__png,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
) -> libc::c_int {
    if stbi__parse_png_file(p, STBI__SCAN_header as libc::c_int, 0 as libc::c_int) == 0 {
        stbi__rewind((*p).s);
        return 0 as libc::c_int;
    }
    if !x.is_null() {
        *x = (*(*p).s).img_x as libc::c_int;
    }
    if !y.is_null() {
        *y = (*(*p).s).img_y as libc::c_int;
    }
    if !comp.is_null() {
        *comp = (*(*p).s).img_n;
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__jpeg_info(
    mut s: *mut stbi__context,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
) -> libc::c_int {
    let mut result: libc::c_int = 0;
    let mut j: *mut stbi__jpeg = stbi__malloc(
        ::core::mem::size_of::<stbi__jpeg>() as libc::c_ulong,
    ) as *mut stbi__jpeg;
    if j.is_null() {
        return stbi__err(b"outofmem\0" as *const u8 as *const libc::c_char);
    }
    (*j).s = s;
    result = stbi__jpeg_info_raw(j, x, y, comp);
    free(j as *mut libc::c_void);
    return result;
}
unsafe extern "C" fn stbi__jpeg_info_raw(
    mut j: *mut stbi__jpeg,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
) -> libc::c_int {
    if stbi__decode_jpeg_header(j, STBI__SCAN_header as libc::c_int) == 0 {
        stbi__rewind((*j).s);
        return 0 as libc::c_int;
    }
    if !x.is_null() {
        *x = (*(*j).s).img_x as libc::c_int;
    }
    if !y.is_null() {
        *y = (*(*j).s).img_y as libc::c_int;
    }
    if !comp.is_null() {
        *comp = if (*(*j).s).img_n >= 3 as libc::c_int {
            3 as libc::c_int
        } else {
            1 as libc::c_int
        };
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__is_16_main(mut s: *mut stbi__context) -> libc::c_int {
    if stbi__png_is16(s) != 0 {
        return 1 as libc::c_int;
    }
    if stbi__psd_is16(s) != 0 {
        return 1 as libc::c_int;
    }
    if stbi__pnm_is16(s) != 0 {
        return 1 as libc::c_int;
    }
    return 0 as libc::c_int;
}
unsafe extern "C" fn stbi__pnm_is16(mut s: *mut stbi__context) -> libc::c_int {
    if stbi__pnm_info(
        s,
        0 as *mut libc::c_int,
        0 as *mut libc::c_int,
        0 as *mut libc::c_int,
    ) == 16 as libc::c_int
    {
        return 1 as libc::c_int;
    }
    return 0 as libc::c_int;
}
unsafe extern "C" fn stbi__psd_is16(mut s: *mut stbi__context) -> libc::c_int {
    let mut channelCount: libc::c_int = 0;
    let mut depth: libc::c_int = 0;
    if stbi__get32be(s) != 0x38425053 as libc::c_int as libc::c_uint {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    if stbi__get16be(s) != 1 as libc::c_int {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    stbi__skip(s, 6 as libc::c_int);
    channelCount = stbi__get16be(s);
    if channelCount < 0 as libc::c_int || channelCount > 16 as libc::c_int {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    depth = stbi__get16be(s);
    if depth != 16 as libc::c_int {
        stbi__rewind(s);
        return 0 as libc::c_int;
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn stbi__png_is16(mut s: *mut stbi__context) -> libc::c_int {
    let mut p: stbi__png = stbi__png {
        s: 0 as *mut stbi__context,
        idata: 0 as *mut stbi_uc,
        expanded: 0 as *mut stbi_uc,
        out: 0 as *mut stbi_uc,
        depth: 0,
    };
    p.s = s;
    if stbi__png_info_raw(
        &mut p,
        0 as *mut libc::c_int,
        0 as *mut libc::c_int,
        0 as *mut libc::c_int,
    ) == 0
    {
        return 0 as libc::c_int;
    }
    if p.depth != 16 as libc::c_int {
        stbi__rewind(p.s);
        return 0 as libc::c_int;
    }
    return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_info(
    mut filename: *const libc::c_char,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
) -> libc::c_int {
    let mut f: *mut FILE = stbi__fopen(
        filename,
        b"rb\0" as *const u8 as *const libc::c_char,
    );
    let mut result: libc::c_int = 0;
    if f.is_null() {
        return stbi__err(b"can't fopen\0" as *const u8 as *const libc::c_char);
    }
    result = stbi_info_from_file(f, x, y, comp);
    fclose(f);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_info_from_file(
    mut f: *mut FILE,
    mut x: *mut libc::c_int,
    mut y: *mut libc::c_int,
    mut comp: *mut libc::c_int,
) -> libc::c_int {
    let mut r: libc::c_int = 0;
    let mut s: stbi__context = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: 0 as *mut libc::c_void,
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: 0 as *mut stbi_uc,
        img_buffer_end: 0 as *mut stbi_uc,
        img_buffer_original: 0 as *mut stbi_uc,
        img_buffer_original_end: 0 as *mut stbi_uc,
    };
    let mut pos: libc::c_long = ftell(f);
    stbi__start_file(&mut s, f);
    r = stbi__info_main(&mut s, x, y, comp);
    fseek(f, pos, 0 as libc::c_int);
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_is_16_bit(
    mut filename: *const libc::c_char,
) -> libc::c_int {
    let mut f: *mut FILE = stbi__fopen(
        filename,
        b"rb\0" as *const u8 as *const libc::c_char,
    );
    let mut result: libc::c_int = 0;
    if f.is_null() {
        return stbi__err(b"can't fopen\0" as *const u8 as *const libc::c_char);
    }
    result = stbi_is_16_bit_from_file(f);
    fclose(f);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_is_16_bit_from_file(mut f: *mut FILE) -> libc::c_int {
    let mut r: libc::c_int = 0;
    let mut s: stbi__context = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: 0 as *mut libc::c_void,
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: 0 as *mut stbi_uc,
        img_buffer_end: 0 as *mut stbi_uc,
        img_buffer_original: 0 as *mut stbi_uc,
        img_buffer_original_end: 0 as *mut stbi_uc,
    };
    let mut pos: libc::c_long = ftell(f);
    stbi__start_file(&mut s, f);
    r = stbi__is_16_main(&mut s);
    fseek(f, pos, 0 as libc::c_int);
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_set_unpremultiply_on_load(
    mut flag_true_if_should_unpremultiply: libc::c_int,
) {
    stbi__unpremultiply_on_load_global = flag_true_if_should_unpremultiply;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_convert_iphone_png_to_rgb(
    mut flag_true_if_should_convert: libc::c_int,
) {
    stbi__de_iphone_flag_global = flag_true_if_should_convert;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_set_flip_vertically_on_load(
    mut flag_true_if_should_flip: libc::c_int,
) {
    stbi__vertically_flip_on_load_global = flag_true_if_should_flip;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_convert_iphone_png_to_rgb_thread(
    mut flag_true_if_should_convert: libc::c_int,
) {
    stbi__de_iphone_flag_local = flag_true_if_should_convert;
    stbi__de_iphone_flag_set = 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_set_flip_vertically_on_load_thread(
    mut flag_true_if_should_flip: libc::c_int,
) {
    stbi__vertically_flip_on_load_local = flag_true_if_should_flip;
    stbi__vertically_flip_on_load_set = 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_zlib_decode_malloc_guesssize(
    mut buffer: *const libc::c_char,
    mut len: libc::c_int,
    mut initial_size: libc::c_int,
    mut outlen: *mut libc::c_int,
) -> *mut libc::c_char {
    let mut a: stbi__zbuf = stbi__zbuf {
        zbuffer: 0 as *mut stbi_uc,
        zbuffer_end: 0 as *mut stbi_uc,
        num_bits: 0,
        code_buffer: 0,
        zout: 0 as *mut libc::c_char,
        zout_start: 0 as *mut libc::c_char,
        zout_end: 0 as *mut libc::c_char,
        z_expandable: 0,
        z_length: stbi__zhuffman {
            fast: [0; 512],
            firstcode: [0; 16],
            maxcode: [0; 17],
            firstsymbol: [0; 16],
            size: [0; 288],
            value: [0; 288],
        },
        z_distance: stbi__zhuffman {
            fast: [0; 512],
            firstcode: [0; 16],
            maxcode: [0; 17],
            firstsymbol: [0; 16],
            size: [0; 288],
            value: [0; 288],
        },
    };
    let mut p: *mut libc::c_char = stbi__malloc(initial_size as size_t)
        as *mut libc::c_char;
    if p.is_null() {
        return 0 as *mut libc::c_char;
    }
    a.zbuffer = buffer as *mut stbi_uc;
    a.zbuffer_end = (buffer as *mut stbi_uc).offset(len as isize);
    if stbi__do_zlib(&mut a, p, initial_size, 1 as libc::c_int, 1 as libc::c_int) != 0 {
        if !outlen.is_null() {
            *outlen = (a.zout).offset_from(a.zout_start) as libc::c_long as libc::c_int;
        }
        return a.zout_start;
    } else {
        free(a.zout_start as *mut libc::c_void);
        return 0 as *mut libc::c_char;
    };
}
#[no_mangle]
pub unsafe extern "C" fn stbi_zlib_decode_malloc(
    mut buffer: *const libc::c_char,
    mut len: libc::c_int,
    mut outlen: *mut libc::c_int,
) -> *mut libc::c_char {
    return stbi_zlib_decode_malloc_guesssize(buffer, len, 16384 as libc::c_int, outlen);
}
#[no_mangle]
pub unsafe extern "C" fn stbi_zlib_decode_buffer(
    mut obuffer: *mut libc::c_char,
    mut olen: libc::c_int,
    mut ibuffer: *const libc::c_char,
    mut ilen: libc::c_int,
) -> libc::c_int {
    let mut a: stbi__zbuf = stbi__zbuf {
        zbuffer: 0 as *mut stbi_uc,
        zbuffer_end: 0 as *mut stbi_uc,
        num_bits: 0,
        code_buffer: 0,
        zout: 0 as *mut libc::c_char,
        zout_start: 0 as *mut libc::c_char,
        zout_end: 0 as *mut libc::c_char,
        z_expandable: 0,
        z_length: stbi__zhuffman {
            fast: [0; 512],
            firstcode: [0; 16],
            maxcode: [0; 17],
            firstsymbol: [0; 16],
            size: [0; 288],
            value: [0; 288],
        },
        z_distance: stbi__zhuffman {
            fast: [0; 512],
            firstcode: [0; 16],
            maxcode: [0; 17],
            firstsymbol: [0; 16],
            size: [0; 288],
            value: [0; 288],
        },
    };
    a.zbuffer = ibuffer as *mut stbi_uc;
    a.zbuffer_end = (ibuffer as *mut stbi_uc).offset(ilen as isize);
    if stbi__do_zlib(&mut a, obuffer, olen, 0 as libc::c_int, 1 as libc::c_int) != 0 {
        return (a.zout).offset_from(a.zout_start) as libc::c_long as libc::c_int
    } else {
        return -(1 as libc::c_int)
    };
}
#[no_mangle]
pub unsafe extern "C" fn stbi_zlib_decode_noheader_malloc(
    mut buffer: *const libc::c_char,
    mut len: libc::c_int,
    mut outlen: *mut libc::c_int,
) -> *mut libc::c_char {
    let mut a: stbi__zbuf = stbi__zbuf {
        zbuffer: 0 as *mut stbi_uc,
        zbuffer_end: 0 as *mut stbi_uc,
        num_bits: 0,
        code_buffer: 0,
        zout: 0 as *mut libc::c_char,
        zout_start: 0 as *mut libc::c_char,
        zout_end: 0 as *mut libc::c_char,
        z_expandable: 0,
        z_length: stbi__zhuffman {
            fast: [0; 512],
            firstcode: [0; 16],
            maxcode: [0; 17],
            firstsymbol: [0; 16],
            size: [0; 288],
            value: [0; 288],
        },
        z_distance: stbi__zhuffman {
            fast: [0; 512],
            firstcode: [0; 16],
            maxcode: [0; 17],
            firstsymbol: [0; 16],
            size: [0; 288],
            value: [0; 288],
        },
    };
    let mut p: *mut libc::c_char = stbi__malloc(16384 as libc::c_int as size_t)
        as *mut libc::c_char;
    if p.is_null() {
        return 0 as *mut libc::c_char;
    }
    a.zbuffer = buffer as *mut stbi_uc;
    a.zbuffer_end = (buffer as *mut stbi_uc).offset(len as isize);
    if stbi__do_zlib(&mut a, p, 16384 as libc::c_int, 1 as libc::c_int, 0 as libc::c_int)
        != 0
    {
        if !outlen.is_null() {
            *outlen = (a.zout).offset_from(a.zout_start) as libc::c_long as libc::c_int;
        }
        return a.zout_start;
    } else {
        free(a.zout_start as *mut libc::c_void);
        return 0 as *mut libc::c_char;
    };
}
#[no_mangle]
pub unsafe extern "C" fn stbi_zlib_decode_noheader_buffer(
    mut obuffer: *mut libc::c_char,
    mut olen: libc::c_int,
    mut ibuffer: *const libc::c_char,
    mut ilen: libc::c_int,
) -> libc::c_int {
    let mut a: stbi__zbuf = stbi__zbuf {
        zbuffer: 0 as *mut stbi_uc,
        zbuffer_end: 0 as *mut stbi_uc,
        num_bits: 0,
        code_buffer: 0,
        zout: 0 as *mut libc::c_char,
        zout_start: 0 as *mut libc::c_char,
        zout_end: 0 as *mut libc::c_char,
        z_expandable: 0,
        z_length: stbi__zhuffman {
            fast: [0; 512],
            firstcode: [0; 16],
            maxcode: [0; 17],
            firstsymbol: [0; 16],
            size: [0; 288],
            value: [0; 288],
        },
        z_distance: stbi__zhuffman {
            fast: [0; 512],
            firstcode: [0; 16],
            maxcode: [0; 17],
            firstsymbol: [0; 16],
            size: [0; 288],
            value: [0; 288],
        },
    };
    a.zbuffer = ibuffer as *mut stbi_uc;
    a.zbuffer_end = (ibuffer as *mut stbi_uc).offset(ilen as isize);
    if stbi__do_zlib(&mut a, obuffer, olen, 0 as libc::c_int, 0 as libc::c_int) != 0 {
        return (a.zout).offset_from(a.zout_start) as libc::c_long as libc::c_int
    } else {
        return -(1 as libc::c_int)
    };
}
#[no_mangle]
pub unsafe extern "C" fn stbi__unpremultiply_on_load_thread(
    mut flag_true_if_should_unpremultiply: libc::c_int,
) {
    stbi__unpremultiply_on_load_local = flag_true_if_should_unpremultiply;
    stbi__unpremultiply_on_load_set = 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn stbi_is_16_bit_from_memory(
    mut buffer: *const stbi_uc,
    mut len: libc::c_int,
) -> libc::c_int {
    let mut s: stbi__context = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: 0 as *mut libc::c_void,
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: 0 as *mut stbi_uc,
        img_buffer_end: 0 as *mut stbi_uc,
        img_buffer_original: 0 as *mut stbi_uc,
        img_buffer_original_end: 0 as *mut stbi_uc,
    };
    stbi__start_mem(&mut s, buffer, len);
    return stbi__is_16_main(&mut s);
}
#[no_mangle]
pub unsafe extern "C" fn stbi_is_16_bit_from_callbacks(
    mut c: *const stbi_io_callbacks,
    mut user: *mut libc::c_void,
) -> libc::c_int {
    let mut s: stbi__context = stbi__context {
        img_x: 0,
        img_y: 0,
        img_n: 0,
        img_out_n: 0,
        io: stbi_io_callbacks {
            read: None,
            skip: None,
            eof: None,
        },
        io_user_data: 0 as *mut libc::c_void,
        read_from_callbacks: 0,
        buflen: 0,
        buffer_start: [0; 128],
        callback_already_read: 0,
        img_buffer: 0 as *mut stbi_uc,
        img_buffer_end: 0 as *mut stbi_uc,
        img_buffer_original: 0 as *mut stbi_uc,
        img_buffer_original_end: 0 as *mut stbi_uc,
    };
    stbi__start_callbacks(&mut s, c as *mut stbi_io_callbacks, user);
    return stbi__is_16_main(&mut s);
}
#[no_mangle]
pub unsafe extern "C" fn Tex2D_LoadRaw(
    mut path: cstr,
    mut sx: *mut libc::c_int,
    mut sy: *mut libc::c_int,
    mut components: *mut libc::c_int,
) -> *mut uchar {
    let mut data: *mut uchar = stbi_load(path, sx, sy, components, 0 as libc::c_int);
    if data.is_null() {
        Fatal(
            b"Failed to load image from '%s'\0" as *const u8 as *const libc::c_char,
            path,
        );
    }
    return data;
}
