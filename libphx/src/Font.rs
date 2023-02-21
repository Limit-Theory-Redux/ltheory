use ::libc;
use glam::Vec3;
use glam::{IVec2, IVec4};
use crate::internal::Memory::*;
use crate::DataFormat::*;
use crate::PixelFormat::*;
use crate::TexFormat::*;
use crate::ResourceType::*;

extern "C" {
    pub type Tex2D;
    pub type HashMap;
    pub type FT_Face_InternalRec_;
    pub type FT_DriverRec_;
    pub type FT_Size_InternalRec_;
    pub type FT_Slot_InternalRec_;
    pub type FT_SubGlyphRec_;
    pub type FT_LibraryRec_;
    fn Fatal(_: cstr, _: ...);
    fn Draw_Color(
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    );
    fn pow(_: f64, _: f64) -> f64;
    fn floor(_: f64) -> f64;
    fn HashMap_Create(keySize: u32, capacity: u32) -> *mut HashMap;
    fn HashMap_Get(_: *mut HashMap, key: *const libc::c_void) -> *mut libc::c_void;
    fn HashMap_Set(_: *mut HashMap, key: *const libc::c_void, value: *mut libc::c_void);
    fn Profiler_Begin(_: cstr);
    fn Profiler_End();
    fn RenderState_PushBlendMode(_: BlendMode);
    fn RenderState_PopBlendMode();
    fn Resource_GetPath(_: ResourceType, name: cstr) -> cstr;
    fn Shader_ResetTexIndex();
    fn Shader_SetTex2D(_: cstr, _: *mut Tex2D);
    fn Tex2D_Create(sx: i32, sy: i32, _: TexFormat) -> *mut Tex2D;
    fn Tex2D_DrawEx(
        _: *mut Tex2D,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        u0: f32,
        v0: f32,
        u1: f32,
        v1: f32,
    );
    fn Tex2D_SetData(
        _: *mut Tex2D,
        _: *const libc::c_void,
        _: PixelFormat,
        _: DataFormat,
    );
    fn FT_Init_FreeType(alibrary: *mut FT_Library) -> FT_Error;
    fn FT_New_Face(
        library: FT_Library,
        filepathname: *const libc::c_char,
        face_index: FT_Long,
        aface: *mut FT_Face,
    ) -> FT_Error;
    fn FT_Set_Pixel_Sizes(
        face: FT_Face,
        pixel_width: FT_UInt,
        pixel_height: FT_UInt,
    ) -> FT_Error;
    fn FT_Done_Face(face: FT_Face) -> FT_Error;
    fn FT_Load_Glyph(
        face: FT_Face,
        glyph_index: FT_UInt,
        load_flags: FT_Int32,
    ) -> FT_Error;
    fn FT_Get_Kerning(
        face: FT_Face,
        left_glyph: FT_UInt,
        right_glyph: FT_UInt,
        kern_mode: FT_UInt,
        akerning: *mut FT_Vector,
    ) -> FT_Error;
    fn FT_Get_Char_Index(face: FT_Face, charcode: FT_ULong) -> FT_UInt;
}
pub type uint = u32;
pub type uchar = libc::c_uchar;
pub type cstr = *const libc::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Font {
    pub _refCount: u32,
    pub handle: FT_Face,
    pub glyphs: *mut HashMap,
    pub glyphsAscii: [*mut Glyph; 256],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Glyph {
    pub index: i32,
    pub tex: *mut Tex2D,
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub sx: i32,
    pub sy: i32,
    pub advance: i32,
}
pub type FT_Face = *mut FT_FaceRec_;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FT_FaceRec_ {
    pub num_faces: FT_Long,
    pub face_index: FT_Long,
    pub face_flags: FT_Long,
    pub style_flags: FT_Long,
    pub num_glyphs: FT_Long,
    pub family_name: *mut FT_String,
    pub style_name: *mut FT_String,
    pub num_fixed_sizes: FT_Int,
    pub available_sizes: *mut FT_Bitmap_Size,
    pub num_charmaps: FT_Int,
    pub charmaps: *mut FT_CharMap,
    pub generic: FT_Generic,
    pub bbox: FT_BBox,
    pub units_per_EM: FT_UShort,
    pub ascender: FT_Short,
    pub descender: FT_Short,
    pub height: FT_Short,
    pub max_advance_width: FT_Short,
    pub max_advance_height: FT_Short,
    pub underline_position: FT_Short,
    pub underline_thickness: FT_Short,
    pub glyph: FT_GlyphSlot,
    pub size: FT_Size,
    pub charmap: FT_CharMap,
    pub driver: FT_Driver,
    pub memory: FT_Memory,
    pub stream: FT_Stream,
    pub sizes_list: FT_ListRec,
    pub autohint: FT_Generic,
    pub extensions: *mut libc::c_void,
    pub internal: FT_Face_Internal,
}
pub type FT_Face_Internal = *mut FT_Face_InternalRec_;
pub type FT_Generic = FT_Generic_;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FT_Generic_ {
    pub data: *mut libc::c_void,
    pub finalizer: FT_Generic_Finalizer,
}
pub type FT_Generic_Finalizer = Option::<unsafe extern "C" fn(*mut libc::c_void) -> ()>;
pub type FT_ListRec = FT_ListRec_;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FT_ListRec_ {
    pub head: FT_ListNode,
    pub tail: FT_ListNode,
}
pub type FT_ListNode = *mut FT_ListNodeRec_;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FT_ListNodeRec_ {
    pub prev: FT_ListNode,
    pub next: FT_ListNode,
    pub data: *mut libc::c_void,
}
pub type FT_Stream = *mut FT_StreamRec_;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FT_StreamRec_ {
    pub base: *mut libc::c_uchar,
    pub size: libc::c_ulong,
    pub pos: libc::c_ulong,
    pub descriptor: FT_StreamDesc,
    pub pathname: FT_StreamDesc,
    pub read: FT_Stream_IoFunc,
    pub close: FT_Stream_CloseFunc,
    pub memory: FT_Memory,
    pub cursor: *mut libc::c_uchar,
    pub limit: *mut libc::c_uchar,
}
pub type FT_Memory = *mut FT_MemoryRec_;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FT_MemoryRec_ {
    pub user: *mut libc::c_void,
    pub alloc: FT_Alloc_Func,
    pub free: FT_Free_Func,
    pub realloc: FT_Realloc_Func,
}
pub type FT_Realloc_Func = Option::<
    unsafe extern "C" fn(
        FT_Memory,
        libc::c_long,
        libc::c_long,
        *mut libc::c_void,
    ) -> *mut libc::c_void,
>;
pub type FT_Free_Func = Option::<
    unsafe extern "C" fn(FT_Memory, *mut libc::c_void) -> (),
>;
pub type FT_Alloc_Func = Option::<
    unsafe extern "C" fn(FT_Memory, libc::c_long) -> *mut libc::c_void,
>;
pub type FT_Stream_CloseFunc = Option::<unsafe extern "C" fn(FT_Stream) -> ()>;
pub type FT_Stream_IoFunc = Option::<
    unsafe extern "C" fn(
        FT_Stream,
        libc::c_ulong,
        *mut libc::c_uchar,
        libc::c_ulong,
    ) -> libc::c_ulong,
>;
pub type FT_StreamDesc = FT_StreamDesc_;
#[derive(Copy, Clone)]
#[repr(C)]
pub union FT_StreamDesc_ {
    pub value: libc::c_long,
    pub pointer: *mut libc::c_void,
}
pub type FT_Driver = *mut FT_DriverRec_;
pub type FT_CharMap = *mut FT_CharMapRec_;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FT_CharMapRec_ {
    pub face: FT_Face,
    pub encoding: FT_Encoding,
    pub platform_id: FT_UShort,
    pub encoding_id: FT_UShort,
}
pub type FT_UShort = u16;
pub type FT_Encoding = FT_Encoding_;
pub type FT_Encoding_ = u32;
pub const FT_ENCODING_APPLE_ROMAN: FT_Encoding_ = 1634889070;
pub const FT_ENCODING_OLD_LATIN_2: FT_Encoding_ = 1818326066;
pub const FT_ENCODING_ADOBE_LATIN_1: FT_Encoding_ = 1818326065;
pub const FT_ENCODING_ADOBE_CUSTOM: FT_Encoding_ = 1094992451;
pub const FT_ENCODING_ADOBE_EXPERT: FT_Encoding_ = 1094992453;
pub const FT_ENCODING_ADOBE_STANDARD: FT_Encoding_ = 1094995778;
pub const FT_ENCODING_MS_JOHAB: FT_Encoding_ = 1785686113;
pub const FT_ENCODING_MS_WANSUNG: FT_Encoding_ = 2002873971;
pub const FT_ENCODING_MS_BIG5: FT_Encoding_ = 1651074869;
pub const FT_ENCODING_MS_GB2312: FT_Encoding_ = 1734484000;
pub const FT_ENCODING_MS_SJIS: FT_Encoding_ = 1936353651;
pub const FT_ENCODING_GB2312: FT_Encoding_ = 1734484000;
pub const FT_ENCODING_JOHAB: FT_Encoding_ = 1785686113;
pub const FT_ENCODING_WANSUNG: FT_Encoding_ = 2002873971;
pub const FT_ENCODING_BIG5: FT_Encoding_ = 1651074869;
pub const FT_ENCODING_PRC: FT_Encoding_ = 1734484000;
pub const FT_ENCODING_SJIS: FT_Encoding_ = 1936353651;
pub const FT_ENCODING_UNICODE: FT_Encoding_ = 1970170211;
pub const FT_ENCODING_MS_SYMBOL: FT_Encoding_ = 1937337698;
pub const FT_ENCODING_NONE: FT_Encoding_ = 0;
pub type FT_Size = *mut FT_SizeRec_;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FT_SizeRec_ {
    pub face: FT_Face,
    pub generic: FT_Generic,
    pub metrics: FT_Size_Metrics,
    pub internal: FT_Size_Internal,
}
pub type FT_Size_Internal = *mut FT_Size_InternalRec_;
pub type FT_Size_Metrics = FT_Size_Metrics_;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FT_Size_Metrics_ {
    pub x_ppem: FT_UShort,
    pub y_ppem: FT_UShort,
    pub x_scale: FT_Fixed,
    pub y_scale: FT_Fixed,
    pub ascender: FT_Pos,
    pub descender: FT_Pos,
    pub height: FT_Pos,
    pub max_advance: FT_Pos,
}
pub type FT_Pos = libc::c_long;
pub type FT_Fixed = libc::c_long;
pub type FT_GlyphSlot = *mut FT_GlyphSlotRec_;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FT_GlyphSlotRec_ {
    pub library: FT_Library,
    pub face: FT_Face,
    pub next: FT_GlyphSlot,
    pub glyph_index: FT_UInt,
    pub generic: FT_Generic,
    pub metrics: FT_Glyph_Metrics,
    pub linearHoriAdvance: FT_Fixed,
    pub linearVertAdvance: FT_Fixed,
    pub advance: FT_Vector,
    pub format: FT_Glyph_Format,
    pub bitmap: FT_Bitmap,
    pub bitmap_left: FT_Int,
    pub bitmap_top: FT_Int,
    pub outline: FT_Outline,
    pub num_subglyphs: FT_UInt,
    pub subglyphs: FT_SubGlyph,
    pub control_data: *mut libc::c_void,
    pub control_len: libc::c_long,
    pub lsb_delta: FT_Pos,
    pub rsb_delta: FT_Pos,
    pub other: *mut libc::c_void,
    pub internal: FT_Slot_Internal,
}
pub type FT_Slot_Internal = *mut FT_Slot_InternalRec_;
pub type FT_SubGlyph = *mut FT_SubGlyphRec_;
pub type FT_UInt = u32;
pub type FT_Outline = FT_Outline_;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FT_Outline_ {
    pub n_contours: i16,
    pub n_points: i16,
    pub points: *mut FT_Vector,
    pub tags: *mut libc::c_char,
    pub contours: *mut i16,
    pub flags: i32,
}
pub type FT_Vector = FT_Vector_;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FT_Vector_ {
    pub x: FT_Pos,
    pub y: FT_Pos,
}
pub type FT_Int = i32;
pub type FT_Bitmap = FT_Bitmap_;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FT_Bitmap_ {
    pub rows: u32,
    pub width: u32,
    pub pitch: i32,
    pub buffer: *mut libc::c_uchar,
    pub num_grays: u16,
    pub pixel_mode: libc::c_uchar,
    pub palette_mode: libc::c_uchar,
    pub palette: *mut libc::c_void,
}
pub type FT_Glyph_Format = FT_Glyph_Format_;
pub type FT_Glyph_Format_ = u32;
pub const FT_GLYPH_FORMAT_SVG: FT_Glyph_Format_ = 1398163232;
pub const FT_GLYPH_FORMAT_PLOTTER: FT_Glyph_Format_ = 1886154612;
pub const FT_GLYPH_FORMAT_OUTLINE: FT_Glyph_Format_ = 1869968492;
pub const FT_GLYPH_FORMAT_BITMAP: FT_Glyph_Format_ = 1651078259;
pub const FT_GLYPH_FORMAT_COMPOSITE: FT_Glyph_Format_ = 1668246896;
pub const FT_GLYPH_FORMAT_NONE: FT_Glyph_Format_ = 0;
pub type FT_Glyph_Metrics = FT_Glyph_Metrics_;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FT_Glyph_Metrics_ {
    pub width: FT_Pos,
    pub height: FT_Pos,
    pub horiBearingX: FT_Pos,
    pub horiBearingY: FT_Pos,
    pub horiAdvance: FT_Pos,
    pub vertBearingX: FT_Pos,
    pub vertBearingY: FT_Pos,
    pub vertAdvance: FT_Pos,
}
pub type FT_Library = *mut FT_LibraryRec_;
pub type FT_Short = i16;
pub type FT_BBox = FT_BBox_;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FT_BBox_ {
    pub xMin: FT_Pos,
    pub yMin: FT_Pos,
    pub xMax: FT_Pos,
    pub yMax: FT_Pos,
}
pub type FT_Bitmap_Size = FT_Bitmap_Size_;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FT_Bitmap_Size_ {
    pub height: FT_Short,
    pub width: FT_Short,
    pub size: FT_Pos,
    pub x_ppem: FT_Pos,
    pub y_ppem: FT_Pos,
}
pub type FT_String = libc::c_char;
pub type FT_Long = libc::c_long;


#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec4f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
pub type BlendMode = i32;
pub type DataFormat = i32;
pub type PixelFormat = i32;
pub type ResourceType = i32;
pub type TexFormat = i32;
pub type FT_Error = i32;
pub type FT_ULong = libc::c_ulong;
pub type FT_Int32 = i32;
pub const FT_KERNING_DEFAULT: FT_Kerning_Mode_ = 0;
pub type FT_Kerning_Mode_ = u32;
pub const FT_KERNING_UNSCALED: FT_Kerning_Mode_ = 2;
pub const FT_KERNING_UNFITTED: FT_Kerning_Mode_ = 1;
#[inline]
unsafe extern "C" fn Floor(mut t: f64) -> f64 {
    return floor(t);
}
#[inline]
unsafe extern "C" fn Pow(
    mut t: f64,
    mut p: f64,
) -> f64 {
    return pow(t, p);
}
#[inline]
unsafe extern "C" fn Max(
    mut a: f64,
    mut b: f64,
) -> f64 {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Min(
    mut a: f64,
    mut b: f64,
) -> f64 {
    return if a < b { a } else { b };
}

#[inline]
unsafe extern "C" fn Vec4f_Create(
    mut x: f32,
    mut y: f32,
    mut z: f32,
    mut w: f32,
) -> Vec4f {
    let mut this: Vec4f =  Vec4f { x: x, y: y, z: z, w: w };
    return this;
}

#[no_mangle]
pub static mut kGamma: f32 = 1.8f32;
#[no_mangle]
pub static mut kRcpGamma: f32 = unsafe { 1.0f32 / kGamma };
static mut ft: FT_Library = 0 as *const FT_LibraryRec_ as FT_Library;
unsafe extern "C" fn Font_GetGlyph(
    mut this: *mut Font,
    mut codepoint: u32,
) -> *mut Glyph {
    if codepoint < 256 as i32 as u32
        && !((*this).glyphsAscii[codepoint as usize]).is_null()
    {
        return (*this).glyphsAscii[codepoint as usize];
    }
    let mut g: *mut Glyph = HashMap_Get(
        (*this).glyphs,
        &mut codepoint as *mut u32 as *const libc::c_void,
    ) as *mut Glyph;
    if !g.is_null() {
        return g;
    }
    let mut face: FT_Face = (*this).handle;
    let mut glyph: i32 = FT_Get_Char_Index(face, codepoint as FT_ULong)
        as i32;
    if glyph == 0 as i32 {
        return 0 as *mut Glyph;
    }
    if FT_Load_Glyph(
        face,
        glyph as FT_UInt,
        ((1 as libc::c_long) << 5 as i32
            | (1 as libc::c_long) << 2 as i32) as FT_Int32,
    ) != 0
    {
        return 0 as *mut Glyph;
    }
    let mut bitmap: *const FT_Bitmap = &mut (*(*face).glyph).bitmap;
    let mut pBitmap: *const uchar = (*bitmap).buffer;
    g = MemAlloc(::core::mem::size_of::<Glyph>()) as *mut Glyph;
    (*g).index = glyph;
    (*g).x0 = (*(*face).glyph).bitmap_left;
    (*g).y0 = -(*(*face).glyph).bitmap_top;
    (*g).sx = (*bitmap).width as i32;
    (*g).sy = (*bitmap).rows as i32;
    (*g).x1 = (*g).x0 + (*g).sx;
    (*g).y1 = (*g).y0 + (*g).sy;
    (*g).advance = ((*(*face).glyph).advance.x >> 6 as i32) as i32;
    let mut buffer: *mut Vec4f = MemAlloc(
        (::core::mem::size_of::<Vec4f>())
            .wrapping_mul(((*g).sx * (*g).sy) as usize),
    ) as *mut Vec4f;
    let mut pBuffer: *mut Vec4f = buffer;
    let mut dy: uint = 0 as i32 as uint;
    while dy < (*bitmap).rows {
        let mut dx: uint = 0 as i32 as uint;
        while dx < (*bitmap).width {
            let mut a: f32 = Pow(
                (*pBitmap.offset(dx as isize) as f32 / 255.0f32)
                    as f64,
                kRcpGamma as f64,
            ) as f32;
            let fresh0 = pBuffer;
            pBuffer = pBuffer.offset(1);
            *fresh0 = Vec4f_Create(1.0f32, 1.0f32, 1.0f32, a);
            dx = dx.wrapping_add(1);
        }
        pBitmap = pBitmap.offset((*bitmap).pitch as isize);
        dy = dy.wrapping_add(1);
    }
    (*g).tex = Tex2D_Create((*g).sx, (*g).sy, TexFormat_RGBA8);
    Tex2D_SetData(
        (*g).tex,
        buffer as *const libc::c_void,
        PixelFormat_RGBA,
        DataFormat_Float,
    );
    MemFree(buffer as *const libc::c_void);
    if codepoint < 256 as i32 as u32 {
        (*this).glyphsAscii[codepoint as usize] = g;
    } else {
        HashMap_Set(
            (*this).glyphs,
            &mut codepoint as *mut u32 as *const libc::c_void,
            g as *mut libc::c_void,
        );
    }
    return g;
}
#[inline]
unsafe extern "C" fn Font_GetKerning(
    mut this: *mut Font,
    mut a: i32,
    mut b: i32,
) -> i32 {
    let mut kern: FT_Vector = FT_Vector { x: 0, y: 0 };
    FT_Get_Kerning(
        (*this).handle,
        a as FT_UInt,
        b as FT_UInt,
        FT_KERNING_DEFAULT as i32 as FT_UInt,
        &mut kern,
    );
    return (kern.x >> 6 as i32) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn Font_Load(mut name: cstr, mut size: i32) -> *mut Font {
    if ft.is_null() {
        FT_Init_FreeType(&mut ft);
    }
    let mut path: cstr = Resource_GetPath(ResourceType_Font, name);
    let mut this: *mut Font = MemAlloc(::core::mem::size_of::<Font>())
        as *mut Font;
    (*this)._refCount = 1 as i32 as u32;
    if FT_New_Face(ft, path, 0 as i32 as FT_Long, &mut (*this).handle) != 0 {
        Fatal(
            b"Font_Load: Failed to load font <%s> at <%s>\0" as *const u8
                as *const libc::c_char,
            name,
            path,
        );
    }
    FT_Set_Pixel_Sizes((*this).handle, 0 as i32 as FT_UInt, size as FT_UInt);
    MemZero(
        ((*this).glyphsAscii).as_mut_ptr() as *mut libc::c_void,
        ::core::mem::size_of::<[*mut Glyph; 256]>(),
    );
    (*this)
        .glyphs = HashMap_Create(
        ::core::mem::size_of::<u32>() as usize as u32,
        16 as i32 as u32,
    );
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Font_Acquire(mut this: *mut Font) {
    (*this)._refCount = ((*this)._refCount).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Font_Free(mut this: *mut Font) {
    if !this.is_null()
        && {
            (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
            (*this)._refCount <= 0 as i32 as u32
        }
    {
        FT_Done_Face((*this).handle);
        MemFree(this as *const libc::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Font_Draw(
    mut this: *mut Font,
    mut text: cstr,
    mut x: f32,
    mut y: f32,
    mut r: f32,
    mut g: f32,
    mut b: f32,
    mut a: f32,
) {
    Profiler_Begin(
        (*::core::mem::transmute::<&[u8; 10], &[libc::c_char; 10]>(b"Font_Draw\0"))
            .as_ptr(),
    );
    let mut glyphLast: i32 = 0 as i32;
    let fresh1 = text;
    text = text.offset(1);
    let mut codepoint: u32 = *fresh1 as u32;
    x = Floor(x as f64) as f32;
    y = Floor(y as f64) as f32;
    RenderState_PushBlendMode(1 as i32);
    Draw_Color(r, g, b, a);
    while codepoint != 0 {
        let mut glyph: *mut Glyph = Font_GetGlyph(this, codepoint);
        if !glyph.is_null() {
            if glyphLast != 0 {
                x += Font_GetKerning(this, glyphLast, (*glyph).index) as f32;
            }
            let mut x0: f32 = x + (*glyph).x0 as f32;
            let mut y0: f32 = y + (*glyph).y0 as f32;
            let mut x1: f32 = x + (*glyph).x1 as f32;
            let mut y1: f32 = y + (*glyph).y1 as f32;
            Tex2D_DrawEx(
                (*glyph).tex,
                x0,
                y0,
                x1,
                y1,
                0.0f32,
                0.0f32,
                1.0f32,
                1.0f32,
            );
            x += (*glyph).advance as f32;
            glyphLast = (*glyph).index;
        } else {
            glyphLast = 0 as i32;
        }
        let fresh2 = text;
        text = text.offset(1);
        codepoint = *fresh2 as u32;
    }
    Draw_Color(
        1.0f32,
        1.0f32,
        1.0f32,
        1.0f32,
    );
    RenderState_PopBlendMode();
    Profiler_End();
}
#[no_mangle]
pub unsafe extern "C" fn Font_DrawShaded(
    mut this: *mut Font,
    mut text: cstr,
    mut x: f32,
    mut y: f32,
) {
    Profiler_Begin(
        (*::core::mem::transmute::<&[u8; 16], &[libc::c_char; 16]>(b"Font_DrawShaded\0"))
            .as_ptr(),
    );
    let mut glyphLast: i32 = 0 as i32;
    let fresh3 = text;
    text = text.offset(1);
    let mut codepoint: u32 = *fresh3 as u32;
    x = Floor(x as f64) as f32;
    y = Floor(y as f64) as f32;
    while codepoint != 0 {
        let mut glyph: *mut Glyph = Font_GetGlyph(this, codepoint);
        if !glyph.is_null() {
            if glyphLast != 0 {
                x += Font_GetKerning(this, glyphLast, (*glyph).index) as f32;
            }
            let mut x0: f32 = x + (*glyph).x0 as f32;
            let mut y0: f32 = y + (*glyph).y0 as f32;
            let mut x1: f32 = x + (*glyph).x1 as f32;
            let mut y1: f32 = y + (*glyph).y1 as f32;
            Shader_ResetTexIndex();
            Shader_SetTex2D(
                b"glyph\0" as *const u8 as *const libc::c_char,
                (*glyph).tex,
            );
            Tex2D_DrawEx(
                (*glyph).tex,
                x0,
                y0,
                x1,
                y1,
                0.0f32,
                0.0f32,
                1.0f32,
                1.0f32,
            );
            x += (*glyph).advance as f32;
            glyphLast = (*glyph).index;
        } else {
            glyphLast = 0 as i32;
        }
        let fresh4 = text;
        text = text.offset(1);
        codepoint = *fresh4 as u32;
    }
    Profiler_End();
}
#[no_mangle]
pub unsafe extern "C" fn Font_GetLineHeight(mut this: *mut Font) -> i32 {
    return ((*(*(*this).handle).size).metrics.height >> 6 as i32)
        as i32;
}
#[no_mangle]
pub unsafe extern "C" fn Font_GetSize(
    mut this: *mut Font,
    mut out: *mut IVec4,
    mut text: cstr,
) {
    Profiler_Begin(
        (*::core::mem::transmute::<&[u8; 13], &[libc::c_char; 13]>(b"Font_GetSize\0"))
            .as_ptr(),
    );
    let mut x: i32 = 0 as i32;
    let mut y: i32 = 0 as i32;
    let mut lower = IVec2::new(i32::MAX, i32::MAX);
    let mut upper = IVec2::new(i32::MIN, i32::MIN);
    let mut glyphLast: i32 = 0 as i32;
    let fresh5 = text;
    text = text.offset(1);
    let mut codepoint: u32 = *fresh5 as u32;
    if codepoint == 0 {
        *out = IVec4::new(
            0 as i32,
            0 as i32,
            0 as i32,
            0 as i32,
        );
        return;
    }
    while codepoint != 0 {
        let mut glyph: *mut Glyph = Font_GetGlyph(this, codepoint);
        if !glyph.is_null() {
            if glyphLast != 0 {
                x += Font_GetKerning(this, glyphLast, (*glyph).index);
            }
            lower
                .x = Min(lower.x as f64, (x + (*glyph).x0) as f64)
                as i32;
            lower
                .y = Min(lower.y as f64, (y + (*glyph).y0) as f64)
                as i32;
            upper
                .x = Max(upper.x as f64, (x + (*glyph).x1) as f64)
                as i32;
            upper
                .y = Max(upper.y as f64, (y + (*glyph).y1) as f64)
                as i32;
            x += (*glyph).advance;
            glyphLast = (*glyph).index;
        } else {
            glyphLast = 0 as i32;
        }
        let fresh6 = text;
        text = text.offset(1);
        codepoint = *fresh6 as u32;
    }
    *out = IVec4::new(lower.x, lower.y, upper.x - lower.x, upper.y - lower.y);
    Profiler_End();
}
#[no_mangle]
pub unsafe extern "C" fn Font_GetSize2(
    mut this: *mut Font,
    mut out: *mut IVec2,
    mut text: cstr,
) {
    Profiler_Begin(
        (*::core::mem::transmute::<&[u8; 14], &[libc::c_char; 14]>(b"Font_GetSize2\0"))
            .as_ptr(),
    );
    (*out).x = 0 as i32;
    (*out).y = 0 as i32;
    let mut glyphLast: i32 = 0 as i32;
    let fresh7 = text;
    text = text.offset(1);
    let mut codepoint: u32 = *fresh7 as u32;
    while codepoint != 0 {
        let mut glyph: *mut Glyph = Font_GetGlyph(this, codepoint);
        if !glyph.is_null() {
            if glyphLast != 0 {
                (*out).x += Font_GetKerning(this, glyphLast, (*glyph).index);
            }
            (*out).x += (*glyph).advance;
            (*out)
                .y = Max(
                (*out).y as f64,
                (-(*glyph).y0 + 1 as i32) as f64,
            ) as i32;
            glyphLast = (*glyph).index;
        } else {
            glyphLast = 0 as i32;
        }
        let fresh8 = text;
        text = text.offset(1);
        codepoint = *fresh8 as u32;
    }
    Profiler_End();
}
