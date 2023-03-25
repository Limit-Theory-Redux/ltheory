use crate::internal::Memory::*;
use crate::Common::*;
use crate::DataFormat::*;
use crate::Draw::*;
use crate::HashMap::*;
use crate::Math::Vec3;
use crate::Math::Vec4;
use crate::Math::{IVec2, IVec4};
use crate::PixelFormat::*;
use crate::Profiler::*;
use crate::RenderState::*;
use crate::Resource::*;
use crate::ResourceType::*;
use crate::Shader::*;
use crate::Tex2D::*;
use crate::TexFormat::*;
use freetype_sys::*;
use libc;

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

#[no_mangle]
pub static kGamma: f32 = 1.8f32;

#[no_mangle]
pub static kRcpGamma: f32 = 1.0f32 / kGamma;

static mut ft: FT_Library = std::ptr::null_mut();

unsafe extern "C" fn Font_GetGlyph(this: *mut Font, codepoint: u32) -> *mut Glyph {
    if codepoint < 256 && !((*this).glyphsAscii[codepoint as usize]).is_null() {
        return (*this).glyphsAscii[codepoint as usize];
    }
    let mut g: *mut Glyph =
        HashMap_Get((*this).glyphs, &codepoint as *const u32 as *const _) as *mut Glyph;
    if !g.is_null() {
        return g;
    }
    let face: FT_Face = (*this).handle;
    let glyph: i32 = FT_Get_Char_Index(face, codepoint as FT_ULong) as i32;
    if glyph == 0 {
        return std::ptr::null_mut();
    }
    if FT_Load_Glyph(
        face,
        glyph as FT_UInt,
        ((1 as libc::c_long) << 5 | (1 as libc::c_long) << 2) as FT_Int32,
    ) != 0
    {
        return std::ptr::null_mut();
    }
    let bitmap: *const FT_Bitmap = &mut (*(*face).glyph).bitmap;
    let mut pBitmap: *const libc::c_uchar = (*bitmap).buffer;
    g = MemNew!(Glyph);
    (*g).index = glyph;
    (*g).x0 = (*(*face).glyph).bitmap_left;
    (*g).y0 = -(*(*face).glyph).bitmap_top;
    (*g).sx = (*bitmap).width as i32;
    (*g).sy = (*bitmap).rows as i32;
    (*g).x1 = (*g).x0 + (*g).sx;
    (*g).y1 = (*g).y0 + (*g).sy;
    (*g).advance = ((*(*face).glyph).advance.x >> 6) as i32;
    let buffer = MemNewArray!(Vec4, ((*g).sx * (*g).sy));
    let mut pBuffer = buffer;
    let mut dy: i32 = 0;
    while dy < (*bitmap).rows {
        let mut dx: i32 = 0;
        while dx < (*bitmap).width {
            let a: f32 = f64::powf(
                (*pBitmap.offset(dx as isize) as f32 / 255.0f32) as f64,
                kRcpGamma as f64,
            ) as f32;
            let fresh0 = pBuffer;
            pBuffer = pBuffer.offset(1);
            *fresh0 = Vec4::new(1.0f32, 1.0f32, 1.0f32, a);
            dx = dx.wrapping_add(1);
        }
        pBitmap = pBitmap.offset((*bitmap).pitch as isize);
        dy = dy.wrapping_add(1);
    }
    (*g).tex = Tex2D_Create((*g).sx, (*g).sy, TexFormat_RGBA8);
    Tex2D_SetData(
        (*g).tex,
        buffer as *const _,
        PixelFormat_RGBA,
        DataFormat_Float,
    );
    MemFree(buffer as *const _);
    if codepoint < 256 {
        (*this).glyphsAscii[codepoint as usize] = g;
    } else {
        HashMap_Set(
            (*this).glyphs,
            &codepoint as *const u32 as *const _,
            g as *mut _,
        );
    }
    g
}

#[inline]
unsafe extern "C" fn Font_GetKerning(this: *mut Font, a: i32, b: i32) -> i32 {
    let mut kern: FT_Vector = FT_Vector { x: 0, y: 0 };
    FT_Get_Kerning(
        (*this).handle,
        a as FT_UInt,
        b as FT_UInt,
        FT_KERNING_DEFAULT as i32 as FT_UInt,
        &mut kern,
    );
    (kern.x >> 6) as i32
}

#[no_mangle]
pub unsafe extern "C" fn Font_Load(name: *const libc::c_char, size: i32) -> *mut Font {
    if ft.is_null() {
        FT_Init_FreeType(&mut ft);
    }
    let path: *const libc::c_char = Resource_GetPath(ResourceType_Font, name);
    let this = MemNew!(Font);
    (*this)._refCount = 1;
    if FT_New_Face(ft, path, 0 as FT_Long, &mut (*this).handle) != 0 {
        Fatal(
            c_str!("Font_Load: Failed to load font <%s> at <%s>"),
            name,
            path,
        );
    }
    FT_Set_Pixel_Sizes((*this).handle, 0 as FT_UInt, size as FT_UInt);
    MemZero(
        ((*this).glyphsAscii).as_mut_ptr() as *mut _,
        std::mem::size_of::<[*mut Glyph; 256]>(),
    );
    (*this).glyphs = HashMap_Create(std::mem::size_of::<u32>() as u32, 16);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Font_Acquire(this: *mut Font) {
    (*this)._refCount = ((*this)._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Font_Free(this: *mut Font) {
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount <= 0
    } {
        FT_Done_Face((*this).handle);
        MemFree(this as *const _);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Font_Draw(
    this: *mut Font,
    mut text: *const libc::c_char,
    mut x: f32,
    mut y: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) {
    Profiler_Begin(c_str!("Font_Draw"));
    let mut glyphLast: i32 = 0;
    let fresh1 = text;
    text = text.offset(1);
    let mut codepoint: u32 = *fresh1 as u32;
    x = f64::floor(x as f64) as f32;
    y = f64::floor(y as f64) as f32;
    RenderState_PushBlendMode(1);
    Draw_Color(r, g, b, a);
    while codepoint != 0 {
        let glyph: *mut Glyph = Font_GetGlyph(this, codepoint);
        if !glyph.is_null() {
            if glyphLast != 0 {
                x += Font_GetKerning(this, glyphLast, (*glyph).index) as f32;
            }
            let x0: f32 = x + (*glyph).x0 as f32;
            let y0: f32 = y + (*glyph).y0 as f32;
            let x1: f32 = x + (*glyph).x1 as f32;
            let y1: f32 = y + (*glyph).y1 as f32;
            Tex2D_DrawEx((*glyph).tex, x0, y0, x1, y1, 0.0f32, 0.0f32, 1.0f32, 1.0f32);
            x += (*glyph).advance as f32;
            glyphLast = (*glyph).index;
        } else {
            glyphLast = 0;
        }
        let fresh2 = text;
        text = text.offset(1);
        codepoint = *fresh2 as u32;
    }
    Draw_Color(1.0f32, 1.0f32, 1.0f32, 1.0f32);
    RenderState_PopBlendMode();
    Profiler_End();
}

#[no_mangle]
pub unsafe extern "C" fn Font_DrawShaded(
    this: *mut Font,
    mut text: *const libc::c_char,
    mut x: f32,
    mut y: f32,
) {
    Profiler_Begin(c_str!("Font_DrawShaded"));
    let mut glyphLast: i32 = 0;
    let fresh3 = text;
    text = text.offset(1);
    let mut codepoint: u32 = *fresh3 as u32;
    x = f64::floor(x as f64) as f32;
    y = f64::floor(y as f64) as f32;
    while codepoint != 0 {
        let glyph: *mut Glyph = Font_GetGlyph(this, codepoint);
        if !glyph.is_null() {
            if glyphLast != 0 {
                x += Font_GetKerning(this, glyphLast, (*glyph).index) as f32;
            }
            let x0: f32 = x + (*glyph).x0 as f32;
            let y0: f32 = y + (*glyph).y0 as f32;
            let x1: f32 = x + (*glyph).x1 as f32;
            let y1: f32 = y + (*glyph).y1 as f32;
            Shader_ResetTexIndex();
            Shader_SetTex2D(c_str!("glyph"), (*glyph).tex);
            Tex2D_DrawEx((*glyph).tex, x0, y0, x1, y1, 0.0f32, 0.0f32, 1.0f32, 1.0f32);
            x += (*glyph).advance as f32;
            glyphLast = (*glyph).index;
        } else {
            glyphLast = 0;
        }
        let fresh4 = text;
        text = text.offset(1);
        codepoint = *fresh4 as u32;
    }
    Profiler_End();
}

#[no_mangle]
pub unsafe extern "C" fn Font_GetLineHeight(this: *mut Font) -> i32 {
    ((*(*(*this).handle).size).metrics.height >> 6) as i32
}

#[no_mangle]
pub unsafe extern "C" fn Font_GetSize(
    this: *mut Font,
    out: *mut IVec4,
    mut text: *const libc::c_char,
) {
    Profiler_Begin(c_str!("Font_GetSize"));
    let mut x: i32 = 0;
    let y: i32 = 0;
    let mut lower = IVec2::new(i32::MAX, i32::MAX);
    let mut upper = IVec2::new(i32::MIN, i32::MIN);
    let mut glyphLast: i32 = 0;
    let fresh5 = text;
    text = text.offset(1);
    let mut codepoint: u32 = *fresh5 as u32;
    if codepoint == 0 {
        *out = IVec4::ZERO;
        return;
    }
    while codepoint != 0 {
        let glyph: *mut Glyph = Font_GetGlyph(this, codepoint);
        if !glyph.is_null() {
            if glyphLast != 0 {
                x += Font_GetKerning(this, glyphLast, (*glyph).index);
            }
            lower.x = f64::min(lower.x as f64, (x + (*glyph).x0) as f64) as i32;
            lower.y = f64::min(lower.y as f64, (y + (*glyph).y0) as f64) as i32;
            upper.x = f64::max(upper.x as f64, (x + (*glyph).x1) as f64) as i32;
            upper.y = f64::max(upper.y as f64, (y + (*glyph).y1) as f64) as i32;
            x += (*glyph).advance;
            glyphLast = (*glyph).index;
        } else {
            glyphLast = 0;
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
    this: *mut Font,
    out: *mut IVec2,
    mut text: *const libc::c_char,
) {
    Profiler_Begin(c_str!("Font_GetSize2"));
    (*out).x = 0;
    (*out).y = 0;
    let mut glyphLast: i32 = 0;
    let fresh7 = text;
    text = text.offset(1);
    let mut codepoint: u32 = *fresh7 as u32;
    while codepoint != 0 {
        let glyph: *mut Glyph = Font_GetGlyph(this, codepoint);
        if !glyph.is_null() {
            if glyphLast != 0 {
                (*out).x += Font_GetKerning(this, glyphLast, (*glyph).index);
            }
            (*out).x += (*glyph).advance;
            (*out).y = f64::max((*out).y as f64, (-(*glyph).y0 + 1) as f64) as i32;
            glyphLast = (*glyph).index;
        } else {
            glyphLast = 0;
        }
        let fresh8 = text;
        text = text.offset(1);
        codepoint = *fresh8 as u32;
    }
    Profiler_End();
}
