use ::libc;
use super::internal::Memory::*;
extern "C" {
    pub type Tex2D;
    pub type HashMap;
    pub type FT_Face_InternalRec_;
    pub type FT_DriverRec_;
    pub type FT_Size_InternalRec_;
    pub type FT_Slot_InternalRec_;
    pub type FT_SubGlyphRec_;
    pub type FT_LibraryRec_;
    fn memset(
        _: *mut libc::c_void,
        _: libc::c_int,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn Fatal(_: cstr, _: ...);
    fn Draw_Color(
        r: libc::c_float,
        g: libc::c_float,
        b: libc::c_float,
        a: libc::c_float,
    );
    fn pow(_: libc::c_double, _: libc::c_double) -> libc::c_double;
    fn floor(_: libc::c_double) -> libc::c_double;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn HashMap_Create(keySize: uint32, capacity: uint32) -> *mut HashMap;
    fn HashMap_Get(_: *mut HashMap, key: *const libc::c_void) -> *mut libc::c_void;
    fn HashMap_Set(_: *mut HashMap, key: *const libc::c_void, value: *mut libc::c_void);
    fn Profiler_Begin(_: cstr);
    fn Profiler_End();
    fn RenderState_PushBlendMode(_: BlendMode);
    fn RenderState_PopBlendMode();
    fn Resource_GetPath(_: ResourceType, name: cstr) -> cstr;
    fn Shader_ResetTexIndex();
    fn Shader_SetTex2D(_: cstr, _: *mut Tex2D);
    fn Tex2D_Create(sx: libc::c_int, sy: libc::c_int, _: TexFormat) -> *mut Tex2D;
    fn Tex2D_DrawEx(
        _: *mut Tex2D,
        x0: libc::c_float,
        y0: libc::c_float,
        x1: libc::c_float,
        y1: libc::c_float,
        u0: libc::c_float,
        v0: libc::c_float,
        u1: libc::c_float,
        v1: libc::c_float,
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
pub type int32_t = libc::c_int;
pub type uint32_t = libc::c_uint;
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
pub type uint = libc::c_uint;
pub type uchar = libc::c_uchar;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint32 = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Font {
    pub _refCount: uint32,
    pub handle: FT_Face,
    pub glyphs: *mut HashMap,
    pub glyphsAscii: [*mut Glyph; 256],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Glyph {
    pub index: libc::c_int,
    pub tex: *mut Tex2D,
    pub x0: libc::c_int,
    pub y0: libc::c_int,
    pub x1: libc::c_int,
    pub y1: libc::c_int,
    pub sx: libc::c_int,
    pub sy: libc::c_int,
    pub advance: libc::c_int,
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
pub type FT_UShort = libc::c_ushort;
pub type FT_Encoding = FT_Encoding_;
pub type FT_Encoding_ = libc::c_uint;
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
pub type FT_UInt = libc::c_uint;
pub type FT_Outline = FT_Outline_;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FT_Outline_ {
    pub n_contours: libc::c_short,
    pub n_points: libc::c_short,
    pub points: *mut FT_Vector,
    pub tags: *mut libc::c_char,
    pub contours: *mut libc::c_short,
    pub flags: libc::c_int,
}
pub type FT_Vector = FT_Vector_;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FT_Vector_ {
    pub x: FT_Pos,
    pub y: FT_Pos,
}
pub type FT_Int = libc::c_int;
pub type FT_Bitmap = FT_Bitmap_;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FT_Bitmap_ {
    pub rows: libc::c_uint,
    pub width: libc::c_uint,
    pub pitch: libc::c_int,
    pub buffer: *mut libc::c_uchar,
    pub num_grays: libc::c_ushort,
    pub pixel_mode: libc::c_uchar,
    pub palette_mode: libc::c_uchar,
    pub palette: *mut libc::c_void,
}
pub type FT_Glyph_Format = FT_Glyph_Format_;
pub type FT_Glyph_Format_ = libc::c_uint;
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
pub type FT_Short = libc::c_short;
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
pub struct Vec2i {
    pub x: libc::c_int,
    pub y: libc::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec4i {
    pub x: libc::c_int,
    pub y: libc::c_int,
    pub z: libc::c_int,
    pub w: libc::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec4f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
    pub w: libc::c_float,
}
pub type BlendMode = int32;
pub type DataFormat = int32;
pub type PixelFormat = int32;
pub type ResourceType = int32;
pub type TexFormat = int32;
pub type FT_Error = libc::c_int;
pub type FT_ULong = libc::c_ulong;
pub type FT_Int32 = libc::c_int;
pub const FT_KERNING_DEFAULT: FT_Kerning_Mode_ = 0;
pub type FT_Kerning_Mode_ = libc::c_uint;
pub const FT_KERNING_UNSCALED: FT_Kerning_Mode_ = 2;
pub const FT_KERNING_UNFITTED: FT_Kerning_Mode_ = 1;
#[no_mangle]
pub static mut DataFormat_U8: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_I8: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_U16: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_I16: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_U32: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_I32: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_Float: DataFormat = 0;
#[inline]
unsafe extern "C" fn Floor(mut t: libc::c_double) -> libc::c_double {
    return floor(t);
}
#[inline]
unsafe extern "C" fn Pow(
    mut t: libc::c_double,
    mut p: libc::c_double,
) -> libc::c_double {
    return pow(t, p);
}
#[inline]
unsafe extern "C" fn Max(
    mut a: libc::c_double,
    mut b: libc::c_double,
) -> libc::c_double {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Min(
    mut a: libc::c_double,
    mut b: libc::c_double,
) -> libc::c_double {
    return if a < b { a } else { b };
}
#[inline]
unsafe extern "C" fn Vec4i_Create(
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut z: libc::c_int,
    mut w: libc::c_int,
) -> Vec4i {
    let mut self_0: Vec4i = {
        let mut init = Vec4i { x: x, y: y, z: z, w: w };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Vec4f_Create(
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut z: libc::c_float,
    mut w: libc::c_float,
) -> Vec4f {
    let mut self_0: Vec4f = {
        let mut init = Vec4f { x: x, y: y, z: z, w: w };
        init
    };
    return self_0;
}

#[inline]
unsafe extern "C" fn MemZero(mut dst: *mut libc::c_void, mut size: size_t) {
    memset(dst, 0 as libc::c_int, size);
}
#[no_mangle]
pub static mut ResourceType_Font: ResourceType = 0;
#[no_mangle]
pub static mut ResourceType_Mesh: ResourceType = 0;
#[no_mangle]
pub static mut ResourceType_Other: ResourceType = 0;
#[no_mangle]
pub static mut ResourceType_Script: ResourceType = 0;
#[no_mangle]
pub static mut ResourceType_Shader: ResourceType = 0;
#[no_mangle]
pub static mut ResourceType_Sound: ResourceType = 0;
#[no_mangle]
pub static mut ResourceType_Tex1D: ResourceType = 0;
#[no_mangle]
pub static mut ResourceType_Tex2D: ResourceType = 0;
#[no_mangle]
pub static mut ResourceType_Tex3D: ResourceType = 0;
#[no_mangle]
pub static mut ResourceType_TexCube: ResourceType = 0;
#[no_mangle]
pub static mut PixelFormat_Red: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_RG: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_RGB: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_BGR: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_RGBA: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_BGRA: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_Depth_Component: PixelFormat = 0;
#[no_mangle]
pub static mut TexFormat_R8: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_R16: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_R16F: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_R32F: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RG8: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RG16: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RG16F: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RG32F: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RGB8: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RGBA8: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RGBA16: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RGBA16F: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_RGBA32F: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_Depth16: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_Depth24: TexFormat = 0;
#[no_mangle]
pub static mut TexFormat_Depth32F: TexFormat = 0;
#[no_mangle]
pub static mut kGamma: libc::c_float = 1.8f32;
#[no_mangle]
pub static mut kRcpGamma: libc::c_float = unsafe { 1.0f32 / kGamma };
static mut ft: FT_Library = 0 as *const FT_LibraryRec_ as FT_Library;
unsafe extern "C" fn Font_GetGlyph(
    mut self_0: *mut Font,
    mut codepoint: uint32,
) -> *mut Glyph {
    if codepoint < 256 as libc::c_int as libc::c_uint
        && !((*self_0).glyphsAscii[codepoint as usize]).is_null()
    {
        return (*self_0).glyphsAscii[codepoint as usize];
    }
    let mut g: *mut Glyph = HashMap_Get(
        (*self_0).glyphs,
        &mut codepoint as *mut uint32 as *const libc::c_void,
    ) as *mut Glyph;
    if !g.is_null() {
        return g;
    }
    let mut face: FT_Face = (*self_0).handle;
    let mut glyph: libc::c_int = FT_Get_Char_Index(face, codepoint as FT_ULong)
        as libc::c_int;
    if glyph == 0 as libc::c_int {
        return 0 as *mut Glyph;
    }
    if FT_Load_Glyph(
        face,
        glyph as FT_UInt,
        ((1 as libc::c_long) << 5 as libc::c_int
            | (1 as libc::c_long) << 2 as libc::c_int) as FT_Int32,
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
    (*g).sx = (*bitmap).width as libc::c_int;
    (*g).sy = (*bitmap).rows as libc::c_int;
    (*g).x1 = (*g).x0 + (*g).sx;
    (*g).y1 = (*g).y0 + (*g).sy;
    (*g).advance = ((*(*face).glyph).advance.x >> 6 as libc::c_int) as libc::c_int;
    let mut buffer: *mut Vec4f = MemAlloc(
        (::core::mem::size_of::<Vec4f>())
            .wrapping_mul(((*g).sx * (*g).sy) as libc::c_ulong),
    ) as *mut Vec4f;
    let mut pBuffer: *mut Vec4f = buffer;
    let mut dy: uint = 0 as libc::c_int as uint;
    while dy < (*bitmap).rows {
        let mut dx: uint = 0 as libc::c_int as uint;
        while dx < (*bitmap).width {
            let mut a: libc::c_float = Pow(
                (*pBitmap.offset(dx as isize) as libc::c_float / 255.0f32)
                    as libc::c_double,
                kRcpGamma as libc::c_double,
            ) as libc::c_float;
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
    if codepoint < 256 as libc::c_int as libc::c_uint {
        (*self_0).glyphsAscii[codepoint as usize] = g;
    } else {
        HashMap_Set(
            (*self_0).glyphs,
            &mut codepoint as *mut uint32 as *const libc::c_void,
            g as *mut libc::c_void,
        );
    }
    return g;
}
#[inline]
unsafe extern "C" fn Font_GetKerning(
    mut self_0: *mut Font,
    mut a: libc::c_int,
    mut b: libc::c_int,
) -> libc::c_int {
    let mut kern: FT_Vector = FT_Vector { x: 0, y: 0 };
    FT_Get_Kerning(
        (*self_0).handle,
        a as FT_UInt,
        b as FT_UInt,
        FT_KERNING_DEFAULT as libc::c_int as FT_UInt,
        &mut kern,
    );
    return (kern.x >> 6 as libc::c_int) as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Font_Load(mut name: cstr, mut size: libc::c_int) -> *mut Font {
    if ft.is_null() {
        FT_Init_FreeType(&mut ft);
    }
    let mut path: cstr = Resource_GetPath(ResourceType_Font, name);
    let mut self_0: *mut Font = MemAlloc(::core::mem::size_of::<Font>())
        as *mut Font;
    (*self_0)._refCount = 1 as libc::c_int as uint32;
    if FT_New_Face(ft, path, 0 as libc::c_int as FT_Long, &mut (*self_0).handle) != 0 {
        Fatal(
            b"Font_Load: Failed to load font <%s> at <%s>\0" as *const u8
                as *const libc::c_char,
            name,
            path,
        );
    }
    FT_Set_Pixel_Sizes((*self_0).handle, 0 as libc::c_int as FT_UInt, size as FT_UInt);
    MemZero(
        ((*self_0).glyphsAscii).as_mut_ptr() as *mut libc::c_void,
        ::core::mem::size_of::<[*mut Glyph; 256]>() as libc::c_ulong,
    );
    (*self_0)
        .glyphs = HashMap_Create(
        ::core::mem::size_of::<uint32>() as libc::c_ulong as uint32,
        16 as libc::c_int as uint32,
    );
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Font_Acquire(mut self_0: *mut Font) {
    (*self_0)._refCount = ((*self_0)._refCount).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Font_Free(mut self_0: *mut Font) {
    if !self_0.is_null()
        && {
            (*self_0)._refCount = ((*self_0)._refCount).wrapping_sub(1);
            (*self_0)._refCount <= 0 as libc::c_int as libc::c_uint
        }
    {
        FT_Done_Face((*self_0).handle);
        MemFree(self_0 as *const libc::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Font_Draw(
    mut self_0: *mut Font,
    mut text: cstr,
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
) {
    Profiler_Begin(
        (*::core::mem::transmute::<&[u8; 10], &[libc::c_char; 10]>(b"Font_Draw\0"))
            .as_ptr(),
    );
    let mut glyphLast: libc::c_int = 0 as libc::c_int;
    let fresh1 = text;
    text = text.offset(1);
    let mut codepoint: uint32 = *fresh1 as uint32;
    x = Floor(x as libc::c_double) as libc::c_float;
    y = Floor(y as libc::c_double) as libc::c_float;
    RenderState_PushBlendMode(1 as libc::c_int);
    Draw_Color(r, g, b, a);
    while codepoint != 0 {
        let mut glyph: *mut Glyph = Font_GetGlyph(self_0, codepoint);
        if !glyph.is_null() {
            if glyphLast != 0 {
                x += Font_GetKerning(self_0, glyphLast, (*glyph).index) as libc::c_float;
            }
            let mut x0: libc::c_float = x + (*glyph).x0 as libc::c_float;
            let mut y0: libc::c_float = y + (*glyph).y0 as libc::c_float;
            let mut x1: libc::c_float = x + (*glyph).x1 as libc::c_float;
            let mut y1: libc::c_float = y + (*glyph).y1 as libc::c_float;
            Tex2D_DrawEx(
                (*glyph).tex,
                x0,
                y0,
                x1,
                y1,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            );
            x += (*glyph).advance as libc::c_float;
            glyphLast = (*glyph).index;
        } else {
            glyphLast = 0 as libc::c_int;
        }
        let fresh2 = text;
        text = text.offset(1);
        codepoint = *fresh2 as uint32;
    }
    Draw_Color(
        1 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
    );
    RenderState_PopBlendMode();
    Profiler_End();
}
#[no_mangle]
pub unsafe extern "C" fn Font_DrawShaded(
    mut self_0: *mut Font,
    mut text: cstr,
    mut x: libc::c_float,
    mut y: libc::c_float,
) {
    Profiler_Begin(
        (*::core::mem::transmute::<&[u8; 16], &[libc::c_char; 16]>(b"Font_DrawShaded\0"))
            .as_ptr(),
    );
    let mut glyphLast: libc::c_int = 0 as libc::c_int;
    let fresh3 = text;
    text = text.offset(1);
    let mut codepoint: uint32 = *fresh3 as uint32;
    x = Floor(x as libc::c_double) as libc::c_float;
    y = Floor(y as libc::c_double) as libc::c_float;
    while codepoint != 0 {
        let mut glyph: *mut Glyph = Font_GetGlyph(self_0, codepoint);
        if !glyph.is_null() {
            if glyphLast != 0 {
                x += Font_GetKerning(self_0, glyphLast, (*glyph).index) as libc::c_float;
            }
            let mut x0: libc::c_float = x + (*glyph).x0 as libc::c_float;
            let mut y0: libc::c_float = y + (*glyph).y0 as libc::c_float;
            let mut x1: libc::c_float = x + (*glyph).x1 as libc::c_float;
            let mut y1: libc::c_float = y + (*glyph).y1 as libc::c_float;
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
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            );
            x += (*glyph).advance as libc::c_float;
            glyphLast = (*glyph).index;
        } else {
            glyphLast = 0 as libc::c_int;
        }
        let fresh4 = text;
        text = text.offset(1);
        codepoint = *fresh4 as uint32;
    }
    Profiler_End();
}
#[no_mangle]
pub unsafe extern "C" fn Font_GetLineHeight(mut self_0: *mut Font) -> libc::c_int {
    return ((*(*(*self_0).handle).size).metrics.height >> 6 as libc::c_int)
        as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Font_GetSize(
    mut self_0: *mut Font,
    mut out: *mut Vec4i,
    mut text: cstr,
) {
    Profiler_Begin(
        (*::core::mem::transmute::<&[u8; 13], &[libc::c_char; 13]>(b"Font_GetSize\0"))
            .as_ptr(),
    );
    let mut x: libc::c_int = 0 as libc::c_int;
    let mut y: libc::c_int = 0 as libc::c_int;
    let mut lower: Vec2i = {
        let mut init = Vec2i {
            x: 2147483647 as libc::c_int,
            y: 2147483647 as libc::c_int,
        };
        init
    };
    let mut upper: Vec2i = {
        let mut init = Vec2i {
            x: -(2147483647 as libc::c_int) - 1 as libc::c_int,
            y: -(2147483647 as libc::c_int) - 1 as libc::c_int,
        };
        init
    };
    let mut glyphLast: libc::c_int = 0 as libc::c_int;
    let fresh5 = text;
    text = text.offset(1);
    let mut codepoint: uint32 = *fresh5 as uint32;
    if codepoint == 0 {
        *out = Vec4i_Create(
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
            0 as libc::c_int,
        );
        return;
    }
    while codepoint != 0 {
        let mut glyph: *mut Glyph = Font_GetGlyph(self_0, codepoint);
        if !glyph.is_null() {
            if glyphLast != 0 {
                x += Font_GetKerning(self_0, glyphLast, (*glyph).index);
            }
            lower
                .x = Min(lower.x as libc::c_double, (x + (*glyph).x0) as libc::c_double)
                as libc::c_int;
            lower
                .y = Min(lower.y as libc::c_double, (y + (*glyph).y0) as libc::c_double)
                as libc::c_int;
            upper
                .x = Max(upper.x as libc::c_double, (x + (*glyph).x1) as libc::c_double)
                as libc::c_int;
            upper
                .y = Max(upper.y as libc::c_double, (y + (*glyph).y1) as libc::c_double)
                as libc::c_int;
            x += (*glyph).advance;
            glyphLast = (*glyph).index;
        } else {
            glyphLast = 0 as libc::c_int;
        }
        let fresh6 = text;
        text = text.offset(1);
        codepoint = *fresh6 as uint32;
    }
    *out = Vec4i_Create(lower.x, lower.y, upper.x - lower.x, upper.y - lower.y);
    Profiler_End();
}
#[no_mangle]
pub unsafe extern "C" fn Font_GetSize2(
    mut self_0: *mut Font,
    mut out: *mut Vec2i,
    mut text: cstr,
) {
    Profiler_Begin(
        (*::core::mem::transmute::<&[u8; 14], &[libc::c_char; 14]>(b"Font_GetSize2\0"))
            .as_ptr(),
    );
    (*out).x = 0 as libc::c_int;
    (*out).y = 0 as libc::c_int;
    let mut glyphLast: libc::c_int = 0 as libc::c_int;
    let fresh7 = text;
    text = text.offset(1);
    let mut codepoint: uint32 = *fresh7 as uint32;
    while codepoint != 0 {
        let mut glyph: *mut Glyph = Font_GetGlyph(self_0, codepoint);
        if !glyph.is_null() {
            if glyphLast != 0 {
                (*out).x += Font_GetKerning(self_0, glyphLast, (*glyph).index);
            }
            (*out).x += (*glyph).advance;
            (*out)
                .y = Max(
                (*out).y as libc::c_double,
                (-(*glyph).y0 + 1 as libc::c_int) as libc::c_double,
            ) as libc::c_int;
            glyphLast = (*glyph).index;
        } else {
            glyphLast = 0 as libc::c_int;
        }
        let fresh8 = text;
        text = text.offset(1);
        codepoint = *fresh8 as uint32;
    }
    Profiler_End();
}
