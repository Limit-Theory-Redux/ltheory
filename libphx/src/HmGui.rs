use ::libc;
use glam::Vec3;
use glam::{IVec2, Vec2};
use crate::internal::Memory::*;
use crate::Button::*;

extern "C" {
    pub type Font;
    pub type HashMap;
    pub type Tex2D;
    fn Font_Load(name: cstr, size: libc::c_int) -> *mut Font;
    fn Font_GetSize2(_: *mut Font, out: *mut IVec2, text: cstr);
    fn Hash_FNV64_Init() -> uint64;
    fn Hash_FNV64_Incremental(
        _: uint64,
        buf: *const libc::c_void,
        len: libc::c_int,
    ) -> uint64;
    fn HashMap_Create(keySize: uint32, capacity: uint32) -> *mut HashMap;
    fn HashMap_GetRaw(_: *mut HashMap, keyHash: uint64) -> *mut libc::c_void;
    fn HashMap_SetRaw(_: *mut HashMap, keyHash: uint64, value: *mut libc::c_void);
    fn Input_GetPressed(_: Button) -> bool;
    fn Input_GetDown(_: Button) -> bool;
    fn Input_GetMouseDelta(_: *mut IVec2);
    fn Input_GetMousePosition(_: *mut IVec2);
    fn Input_GetMouseScroll(_: *mut IVec2);
    fn Profiler_Begin(_: cstr);
    fn Profiler_End();
    fn RenderState_PushBlendMode(_: BlendMode);
    fn RenderState_PopBlendMode();
    fn UIRenderer_Begin();
    fn UIRenderer_End();
    fn UIRenderer_Draw();
    fn UIRenderer_BeginLayer(
        x: libc::c_float,
        y: libc::c_float,
        sx: libc::c_float,
        sy: libc::c_float,
        clip: bool,
    );
    fn UIRenderer_EndLayer();
    fn UIRenderer_Image(
        _: *mut Tex2D,
        x: libc::c_float,
        y: libc::c_float,
        sx: libc::c_float,
        sy: libc::c_float,
    );
    fn UIRenderer_Panel(
        x: libc::c_float,
        y: libc::c_float,
        sx: libc::c_float,
        sy: libc::c_float,
        r: libc::c_float,
        g: libc::c_float,
        b: libc::c_float,
        a: libc::c_float,
        bevel: libc::c_float,
        innerAlpha: libc::c_float,
    );
    fn UIRenderer_Rect(
        x: libc::c_float,
        y: libc::c_float,
        sx: libc::c_float,
        sy: libc::c_float,
        r: libc::c_float,
        g: libc::c_float,
        b: libc::c_float,
        a: libc::c_float,
        outline: bool,
    );
    fn UIRenderer_Text(
        font: *mut Font,
        text: cstr,
        x: libc::c_float,
        y: libc::c_float,
        r: libc::c_float,
        g: libc::c_float,
        b: libc::c_float,
        a: libc::c_float,
    );
}
pub type int32_t = libc::c_int;
pub type uint32_t = libc::c_uint;
pub type uint64_t = libc::c_ulonglong;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint32 = uint32_t;
pub type uint64 = uint64_t;


#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec4f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
    pub w: libc::c_float,
}
pub type BlendMode = int32;
pub type Button = int32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiGroup {
    pub widget: HmGuiWidget,
    pub head: *mut HmGuiWidget,
    pub tail: *mut HmGuiWidget,
    pub layout: uint32,
    pub children: uint32,
    pub focusStyle: uint32,
    pub paddingLower: Vec2,
    pub paddingUpper: Vec2,
    pub offset: Vec2,
    pub maxSize: Vec2,
    pub totalStretch: Vec2,
    pub spacing: libc::c_float,
    pub frameOpacity: libc::c_float,
    pub clip: bool,
    pub expand: bool,
    pub focusable: [bool; 2],
    pub storeSize: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiWidget {
    pub parent: *mut HmGuiGroup,
    pub next: *mut HmGuiWidget,
    pub prev: *mut HmGuiWidget,
    pub hash: uint64,
    pub type_0: uint32,
    pub pos: Vec2,
    pub size: Vec2,
    pub minSize: Vec2,
    pub align: Vec2,
    pub stretch: Vec2,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGui {
    pub group: *mut HmGuiGroup,
    pub root: *mut HmGuiGroup,
    pub last: *mut HmGuiWidget,
    pub style: *mut HmGuiStyle,
    pub clipRect: *mut HmGuiClipRect,
    pub data: *mut HashMap,
    pub focus: [uint64; 2],
    pub focusPos: Vec2,
    pub activate: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiClipRect {
    pub prev: *mut HmGuiClipRect,
    pub lower: Vec2,
    pub upper: Vec2,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiStyle {
    pub prev: *mut HmGuiStyle,
    pub font: *mut Font,
    pub spacing: libc::c_float,
    pub colorPrimary: Vec4f,
    pub colorFrame: Vec4f,
    pub colorText: Vec4f,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiText {
    pub widget: HmGuiWidget,
    pub font: *mut Font,
    pub text: cstr,
    pub color: Vec4f,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiData {
    pub offset: Vec2,
    pub minSize: Vec2,
    pub size: Vec2,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiImage {
    pub widget: HmGuiWidget,
    pub image: *mut Tex2D,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiRect {
    pub widget: HmGuiWidget,
    pub color: Vec4f,
}

#[inline]
unsafe extern "C" fn Clamp(
    mut t: libc::c_double,
    mut lower: libc::c_double,
    mut upper: libc::c_double,
) -> libc::c_double {
    t = if t > upper { upper } else { t };
    t = if t < lower { lower } else { t };
    return t;
}
#[inline]
unsafe extern "C" fn Lerp(
    mut a: libc::c_double,
    mut b: libc::c_double,
    mut t: libc::c_double,
) -> libc::c_double {
    return a + t * (b - a);
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
unsafe extern "C" fn Vec4f_Create(
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut z: libc::c_float,
    mut w: libc::c_float,
) -> Vec4f {
    let mut self_1: Vec4f = {
        let mut init = Vec4f { x: x, y: y, z: z, w: w };
        init
    };
    return self_1;
}

static mut this: HmGui = HmGui {
    group: 0 as *const HmGuiGroup as *mut HmGuiGroup,
    root: 0 as *const HmGuiGroup as *mut HmGuiGroup,
    last: 0 as *const HmGuiWidget as *mut HmGuiWidget,
    style: 0 as *const HmGuiStyle as *mut HmGuiStyle,
    clipRect: 0 as *const HmGuiClipRect as *mut HmGuiClipRect,
    data: 0 as *const HashMap as *mut HashMap,
    focus: [0; 2],
    focusPos: Vec2::ZERO,
    activate: false,
};
static mut init_hmgui: bool = 0 as libc::c_int != 0;
unsafe extern "C" fn HmGui_InitWidget(mut e: *mut HmGuiWidget, mut type_0: uint32) {
    (*e).parent = this.group;
    (*e).next = 0 as *mut HmGuiWidget;
    (*e)
        .prev = if !(this.group).is_null() {
        (*this.group).tail
    } else {
        0 as *mut HmGuiWidget
    };
    if !((*e).parent).is_null() {
        (*(*e).parent).children = ((*(*e).parent).children).wrapping_add(1);
        (*e)
            .hash = Hash_FNV64_Incremental(
            (*(*e).parent).widget.hash,
            &mut (*(*e).parent).children as *mut uint32 as *const libc::c_void,
            ::core::mem::size_of::<uint32>() as usize as libc::c_int,
        );
        if !((*e).next).is_null() {
            (*(*e).next).prev = e;
        } else {
            (*(*e).parent).tail = e;
        }
        if !((*e).prev).is_null() {
            (*(*e).prev).next = e;
        } else {
            (*(*e).parent).head = e;
        }
    } else {
        (*e).hash = Hash_FNV64_Init();
    }
    (*e).type_0 = type_0;
    (*e)
        .pos = Vec2::new(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    (*e)
        .size = Vec2::new(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    (*e)
        .minSize = Vec2::new(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    (*e)
        .align = Vec2::new(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    (*e)
        .stretch = Vec2::new(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    this.last = e;
}
unsafe extern "C" fn HmGui_BeginGroup(mut layout: uint32) {
    let mut e: *mut HmGuiGroup = MemAlloc(
        ::core::mem::size_of::<HmGuiGroup>() as usize,
    ) as *mut HmGuiGroup;
    HmGui_InitWidget(&mut (*e).widget, 0 as libc::c_int as uint32);
    (*e).head = 0 as *mut HmGuiWidget;
    (*e).tail = 0 as *mut HmGuiWidget;
    (*e).layout = layout;
    (*e).children = 0 as libc::c_int as uint32;
    (*e).focusStyle = 0 as libc::c_int as uint32;
    (*e)
        .paddingLower = Vec2::new(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    (*e)
        .paddingUpper = Vec2::new(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    (*e)
        .offset = Vec2::new(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    (*e).maxSize = Vec2::new(1e30f32, 1e30f32);
    (*e).spacing = (*this.style).spacing;
    (*e).frameOpacity = 0.0f32;
    (*e).clip = 0 as libc::c_int != 0;
    (*e).expand = 1 as libc::c_int != 0;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 2 as libc::c_int {
        (*e).focusable[i as usize] = 0 as libc::c_int != 0;
        i += 1;
    }
    (*e).storeSize = 0 as libc::c_int != 0;
    this.group = e;
    match layout {
        1 => {
            (*e)
                .widget
                .stretch = Vec2::new(
                1 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            );
        }
        2 => {
            (*e)
                .widget
                .stretch = Vec2::new(
                1 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
            );
        }
        3 => {
            (*e)
                .widget
                .stretch = Vec2::new(
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            );
        }
        _ => {}
    };
}
unsafe extern "C" fn HmGui_FreeText(mut e: *mut HmGuiText) {
    StrFree((*e).text);
    MemFree(e as *const libc::c_void);
}
unsafe extern "C" fn HmGui_FreeGroup(mut g: *mut HmGuiGroup) {
    let mut e: *mut HmGuiWidget = (*g).head;
    while !e.is_null() {
        let mut next: *mut HmGuiWidget = (*e).next;
        match (*e).type_0 {
            0 => {
                HmGui_FreeGroup(e as *mut HmGuiGroup);
            }
            1 => {
                HmGui_FreeText(e as *mut HmGuiText);
            }
            _ => {
                MemFree(e as *const libc::c_void);
            }
        }
        e = next;
    }
    MemFree(g as *const libc::c_void);
}
unsafe extern "C" fn HmGui_GetData(mut g: *mut HmGuiGroup) -> *mut HmGuiData {
    let mut data: *mut HmGuiData = HashMap_GetRaw(this.data, (*g).widget.hash)
        as *mut HmGuiData;
    if data.is_null() {
        data = MemAlloc(::core::mem::size_of::<HmGuiData>())
            as *mut HmGuiData;
        (*data)
            .offset = Vec2::new(
            0 as libc::c_int as libc::c_float,
            0 as libc::c_int as libc::c_float,
        );
        (*data)
            .minSize = Vec2::new(
            0 as libc::c_int as libc::c_float,
            0 as libc::c_int as libc::c_float,
        );
        (*data)
            .size = Vec2::new(
            0 as libc::c_int as libc::c_float,
            0 as libc::c_int as libc::c_float,
        );
        HashMap_SetRaw(this.data, (*g).widget.hash, data as *mut libc::c_void);
    }
    return data;
}
unsafe extern "C" fn HmGui_ComputeSize(mut g: *mut HmGuiGroup) {
    let mut e: *mut HmGuiWidget = (*g).head;
    while !e.is_null() {
        if (*e).type_0 == 0 as libc::c_int as libc::c_uint {
            HmGui_ComputeSize(e as *mut HmGuiGroup);
        }
        e = (*e).next;
    }
    (*g)
        .widget
        .minSize = Vec2::new(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    let mut e_0: *mut HmGuiWidget = (*g).head;
    while !e_0.is_null() {
        match (*g).layout {
            1 => {
                (*g)
                    .widget
                    .minSize
                    .x = Max(
                    (*g).widget.minSize.x as libc::c_double,
                    (*e_0).minSize.x as libc::c_double,
                ) as libc::c_float;
                (*g)
                    .widget
                    .minSize
                    .y = Max(
                    (*g).widget.minSize.y as libc::c_double,
                    (*e_0).minSize.y as libc::c_double,
                ) as libc::c_float;
            }
            2 => {
                (*g)
                    .widget
                    .minSize
                    .x = Max(
                    (*g).widget.minSize.x as libc::c_double,
                    (*e_0).minSize.x as libc::c_double,
                ) as libc::c_float;
                (*g).widget.minSize.y += (*e_0).minSize.y;
                if e_0 != (*g).head {
                    (*g).widget.minSize.y += (*g).spacing;
                }
            }
            3 => {
                (*g).widget.minSize.x += (*e_0).minSize.x;
                (*g)
                    .widget
                    .minSize
                    .y = Max(
                    (*g).widget.minSize.y as libc::c_double,
                    (*e_0).minSize.y as libc::c_double,
                ) as libc::c_float;
                if e_0 != (*g).head {
                    (*g).widget.minSize.x += (*g).spacing;
                }
            }
            _ => {}
        }
        e_0 = (*e_0).next;
    }
    (*g).widget.minSize.x += (*g).paddingLower.x + (*g).paddingUpper.x;
    (*g).widget.minSize.y += (*g).paddingLower.y + (*g).paddingUpper.y;
    if (*g).storeSize {
        let mut data: *mut HmGuiData = HmGui_GetData(g);
        (*data).minSize = (*g).widget.minSize;
    }
    (*g)
        .widget
        .minSize
        .x = Min(
        (*g).widget.minSize.x as libc::c_double,
        (*g).maxSize.x as libc::c_double,
    ) as libc::c_float;
    (*g)
        .widget
        .minSize
        .y = Min(
        (*g).widget.minSize.y as libc::c_double,
        (*g).maxSize.y as libc::c_double,
    ) as libc::c_float;
}
unsafe extern "C" fn HmGui_LayoutWidget(
    mut e: *mut HmGuiWidget,
    mut pos: Vec2,
    mut sx: libc::c_float,
    mut sy: libc::c_float,
) {
    (*e).pos = pos;
    (*e).size = (*e).minSize;
    (*e).size.x += (*e).stretch.x * (sx - (*e).minSize.x);
    (*e).size.y += (*e).stretch.y * (sy - (*e).minSize.y);
    (*e).pos.x += (*e).align.x * (sx - (*e).size.x);
    (*e).pos.y += (*e).align.y * (sy - (*e).size.y);
}
unsafe extern "C" fn HmGui_LayoutGroup(mut g: *mut HmGuiGroup) {
    let mut pos = (*g).widget.pos;
    let mut size = (*g).widget.size;
    let mut extra: libc::c_float = 0 as libc::c_int as libc::c_float;
    let mut totalStretch: libc::c_float = 0 as libc::c_int as libc::c_float;
    pos.x += (*g).paddingLower.x + (*g).offset.x;
    pos.y += (*g).paddingLower.y + (*g).offset.y;
    size.x -= (*g).paddingLower.x + (*g).paddingUpper.x;
    size.y -= (*g).paddingLower.y + (*g).paddingUpper.y;
    if (*g).expand {
        if (*g).layout == 2 as libc::c_int as libc::c_uint {
            extra = (*g).widget.size.y - (*g).widget.minSize.y;
            let mut e: *mut HmGuiWidget = (*g).head;
            while !e.is_null() {
                totalStretch += (*e).stretch.y;
                e = (*e).next;
            }
        } else if (*g).layout == 3 as libc::c_int as libc::c_uint {
            extra = (*g).widget.size.x - (*g).widget.minSize.x;
            let mut e_0: *mut HmGuiWidget = (*g).head;
            while !e_0.is_null() {
                totalStretch += (*e_0).stretch.x;
                e_0 = (*e_0).next;
            }
        }
        if totalStretch > 0 as libc::c_int as libc::c_float {
            extra /= totalStretch;
        }
    }
    let mut s: libc::c_float = 0.;
    let mut e_1: *mut HmGuiWidget = (*g).head;
    while !e_1.is_null() {
        match (*g).layout {
            0 => {
                HmGui_LayoutWidget(e_1, (*e_1).pos, size.x, size.y);
            }
            1 => {
                HmGui_LayoutWidget(e_1, pos, size.x, size.y);
            }
            2 => {
                s = (*e_1).minSize.y;
                if extra > 0 as libc::c_int as libc::c_float {
                    s += (*e_1).stretch.y * extra;
                }
                HmGui_LayoutWidget(e_1, pos, size.x, s);
                pos.y += (*e_1).size.y + (*g).spacing;
            }
            3 => {
                s = (*e_1).minSize.x;
                if extra > 0 as libc::c_int as libc::c_float {
                    s += (*e_1).stretch.x * extra;
                }
                HmGui_LayoutWidget(e_1, pos, s, size.y);
                pos.x += (*e_1).size.x + (*g).spacing;
            }
            _ => {}
        }
        if (*e_1).type_0 == 0 as libc::c_int as libc::c_uint {
            HmGui_LayoutGroup(e_1 as *mut HmGuiGroup);
        }
        e_1 = (*e_1).next;
    }
    if (*g).storeSize {
        let mut data: *mut HmGuiData = HmGui_GetData(g);
        (*data).size = (*g).widget.size;
    }
}
#[inline]
unsafe extern "C" fn IsClipped(mut g: *mut HmGuiGroup, mut p: Vec2) -> bool {
    return p.x < (*g).widget.pos.x || p.y < (*g).widget.pos.y
        || (*g).widget.pos.x + (*g).widget.size.x < p.x
        || (*g).widget.pos.y + (*g).widget.size.y < p.y;
}
unsafe extern "C" fn HmGui_CheckFocus(mut g: *mut HmGuiGroup) {
    if (*g).clip as libc::c_int != 0 && IsClipped(g, this.focusPos) as libc::c_int != 0
    {
        return;
    }
    let mut e: *mut HmGuiWidget = (*g).tail;
    while !e.is_null() {
        if (*e).type_0 == 0 as libc::c_int as libc::c_uint {
            HmGui_CheckFocus(e as *mut HmGuiGroup);
        }
        e = (*e).prev;
    }
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 2 as libc::c_int {
        if this.focus[i as usize] == 0 as libc::c_ulonglong
            && (*g).focusable[i as usize] as libc::c_int != 0
        {
            if (*g).widget.pos.x <= this.focusPos.x
                && (*g).widget.pos.y <= this.focusPos.y
                && this.focusPos.x <= (*g).widget.pos.x + (*g).widget.size.x
                && this.focusPos.y <= (*g).widget.pos.y + (*g).widget.size.y
            {
                this.focus[i as usize] = (*g).widget.hash;
            }
        }
        i += 1;
    }
}
unsafe extern "C" fn HmGui_DrawText(mut e: *mut HmGuiText) {
    UIRenderer_Text(
        (*e).font,
        (*e).text,
        (*e).widget.pos.x,
        (*e).widget.pos.y + (*e).widget.minSize.y,
        (*e).color.x,
        (*e).color.y,
        (*e).color.z,
        (*e).color.w,
    );
}
unsafe extern "C" fn HmGui_DrawRect(mut e: *mut HmGuiRect) {
    UIRenderer_Rect(
        (*e).widget.pos.x,
        (*e).widget.pos.y,
        (*e).widget.size.x,
        (*e).widget.size.y,
        (*e).color.x,
        (*e).color.y,
        (*e).color.z,
        (*e).color.w,
        0 as libc::c_int != 0,
    );
}
unsafe extern "C" fn HmGui_DrawImage(mut e: *mut HmGuiImage) {
    UIRenderer_Image(
        (*e).image,
        (*e).widget.pos.x,
        (*e).widget.pos.y,
        (*e).widget.size.x,
        (*e).widget.size.y,
    );
}
unsafe extern "C" fn HmGui_DrawGroup(mut g: *mut HmGuiGroup) {
    UIRenderer_BeginLayer(
        (*g).widget.pos.x,
        (*g).widget.pos.y,
        (*g).widget.size.x,
        (*g).widget.size.y,
        (*g).clip,
    );
    let mut e: *mut HmGuiWidget = (*g).tail;
    while !e.is_null() {
        match (*e).type_0 {
            0 => {
                HmGui_DrawGroup(e as *mut HmGuiGroup);
            }
            1 => {
                HmGui_DrawText(e as *mut HmGuiText);
            }
            2 => {
                HmGui_DrawRect(e as *mut HmGuiRect);
            }
            3 => {
                HmGui_DrawImage(e as *mut HmGuiImage);
            }
            _ => {}
        }
        e = (*e).prev;
    }
    if (*g).focusable[0] {
        let mut focus: bool = this.focus[0]
            == (*g).widget.hash;
        if (*g).focusStyle == 0 as libc::c_int as libc::c_uint {
            UIRenderer_Panel(
                (*g).widget.pos.x,
                (*g).widget.pos.y,
                (*g).widget.size.x,
                (*g).widget.size.y,
                0.1f32,
                0.12f32,
                0.13f32,
                1.0f32,
                8.0f32,
                (*g).frameOpacity,
            );
        } else if (*g).focusStyle == 1 as libc::c_int as libc::c_uint {
            if focus {
                UIRenderer_Panel(
                    (*g).widget.pos.x,
                    (*g).widget.pos.y,
                    (*g).widget.size.x,
                    (*g).widget.size.y,
                    0.1f32,
                    0.5f32,
                    1.0f32,
                    1.0f32,
                    0.0f32,
                    1.0f32,
                );
            } else {
                UIRenderer_Panel(
                    (*g).widget.pos.x,
                    (*g).widget.pos.y,
                    (*g).widget.size.x,
                    (*g).widget.size.y,
                    0.15f32,
                    0.15f32,
                    0.15f32,
                    0.8f32,
                    0.0f32,
                    (*g).frameOpacity,
                );
            }
        } else if (*g).focusStyle == 2 as libc::c_int as libc::c_uint {
            if focus {
                UIRenderer_Rect(
                    (*g).widget.pos.x,
                    (*g).widget.pos.y,
                    (*g).widget.size.x,
                    (*g).widget.size.y,
                    0.1f32,
                    0.5f32,
                    1.0f32,
                    1.0f32,
                    1 as libc::c_int != 0,
                );
            }
        } else if (*g).focusStyle == 3 as libc::c_int as libc::c_uint {
            UIRenderer_Rect(
                (*g).widget.pos.x,
                (*g).widget.pos.y,
                (*g).widget.size.x,
                (*g).widget.size.y,
                0.3f32,
                0.3f32,
                0.3f32,
                if focus as libc::c_int != 0 { 0.5f32 } else { (*g).frameOpacity },
                0 as libc::c_int != 0,
            );
        }
    }
    UIRenderer_EndLayer();
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_Begin(mut sx: libc::c_float, mut sy: libc::c_float) {
    if !init_hmgui {
        init_hmgui = 1 as libc::c_int != 0;
        this.group = 0 as *mut HmGuiGroup;
        this.root = 0 as *mut HmGuiGroup;
        this
            .style = MemAlloc(::core::mem::size_of::<HmGuiStyle>())
            as *mut HmGuiStyle;
        (*this.style).prev = 0 as *mut HmGuiStyle;
        (*this.style)
            .font = Font_Load(
            b"Rajdhani\0" as *const u8 as *const libc::c_char,
            14 as libc::c_int,
        );
        (*this.style).spacing = 6 as libc::c_int as libc::c_float;
        (*this.style).colorPrimary = Vec4f_Create(0.1f32, 0.5f32, 1.0f32, 1.0f32);
        (*this.style).colorFrame = Vec4f_Create(0.1f32, 0.1f32, 0.1f32, 0.5f32);
        (*this.style)
            .colorText = Vec4f_Create(
            1 as libc::c_int as libc::c_float,
            1 as libc::c_int as libc::c_float,
            1 as libc::c_int as libc::c_float,
            1 as libc::c_int as libc::c_float,
        );
        this.clipRect = 0 as *mut HmGuiClipRect;
        this
            .data = HashMap_Create(
            0 as libc::c_int as uint32,
            128 as libc::c_int as uint32,
        );
        let mut i: libc::c_int = 0 as libc::c_int;
        while i < 2 as libc::c_int {
            this.focus[i as usize] = 0 as libc::c_int as uint64;
            i += 1;
        }
        this.activate = 0 as libc::c_int != 0;
    }
    if !(this.root).is_null() {
        HmGui_FreeGroup(this.root);
        this.root = 0 as *mut HmGuiGroup;
    }
    this.last = 0 as *mut HmGuiWidget;
    this.activate = Input_GetPressed(Button_Mouse_Left);
    HmGui_BeginGroup(0 as libc::c_int as uint32);
    (*this.group).clip = 1 as libc::c_int != 0;
    (*this.group)
        .widget
        .pos = Vec2::new(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    (*this.group).widget.size = Vec2::new(sx, sy);
    this.root = this.group;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_End() {
    Profiler_Begin(
        (*::core::mem::transmute::<&[u8; 10], &[libc::c_char; 10]>(b"HmGui_End\0"))
            .as_ptr(),
    );
    HmGui_EndGroup();
    HmGui_ComputeSize(this.root);
    HmGui_LayoutGroup(this.root);
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 2 as libc::c_int {
        this.focus[i as usize] = 0 as libc::c_int as uint64;
        i += 1;
    }
    let mut mouse: IVec2 = IVec2 { x: 0, y: 0 };
    Input_GetMousePosition(&mut mouse);
    this.focusPos = Vec2::new(mouse.x as libc::c_float, mouse.y as libc::c_float);
    HmGui_CheckFocus(this.root);
    Profiler_End();
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_Draw() {
    Profiler_Begin(
        (*::core::mem::transmute::<&[u8; 11], &[libc::c_char; 11]>(b"HmGui_Draw\0"))
            .as_ptr(),
    );
    RenderState_PushBlendMode(1 as libc::c_int);
    UIRenderer_Begin();
    HmGui_DrawGroup(this.root);
    UIRenderer_End();
    RenderState_PopBlendMode();
    UIRenderer_Draw();
    Profiler_End();
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_BeginGroupX() {
    HmGui_BeginGroup(3 as libc::c_int as uint32);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_BeginGroupY() {
    HmGui_BeginGroup(2 as libc::c_int as uint32);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_BeginGroupStack() {
    HmGui_BeginGroup(1 as libc::c_int as uint32);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_EndGroup() {
    this.last = &mut (*this.group).widget;
    this.group = (*this.group).widget.parent;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_BeginScroll(mut maxSize: libc::c_float) {
    HmGui_BeginGroupX();
    HmGui_SetStretch(
        1 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
    );
    (*this.group).clip = 1 as libc::c_int != 0;
    HmGui_SetSpacing(2 as libc::c_int as libc::c_float);
    HmGui_BeginGroupY();
    HmGui_SetPadding(
        6 as libc::c_int as libc::c_float,
        6 as libc::c_int as libc::c_float,
    );
    HmGui_SetStretch(
        1 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
    );
    (*this.group).expand = 0 as libc::c_int != 0;
    (*this.group).storeSize = 1 as libc::c_int != 0;
    (*this.group).maxSize.y = maxSize;
    let mut data: *mut HmGuiData = HmGui_GetData(this.group);
    (*this.group).offset.y = -(*data).offset.y;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_EndScroll() {
    let mut data: *mut HmGuiData = HmGui_GetData(this.group);
    if HmGui_GroupHasFocus(1 as libc::c_int) {
        let mut scroll: IVec2 = IVec2 { x: 0, y: 0 };
        Input_GetMouseScroll(&mut scroll);
        (*data).offset.y -= 10.0f32 * scroll.y as libc::c_float;
    }
    let mut maxScroll: libc::c_float = Max(
        0.0f32 as libc::c_double,
        ((*data).minSize.y - (*data).size.y) as libc::c_double,
    ) as libc::c_float;
    (*data)
        .offset
        .y = Clamp(
        (*data).offset.y as libc::c_double,
        0.0f32 as libc::c_double,
        maxScroll as libc::c_double,
    ) as libc::c_float;
    HmGui_EndGroup();
    HmGui_BeginGroupY();
    HmGui_SetStretch(
        0 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
    );
    HmGui_SetSpacing(0 as libc::c_int as libc::c_float);
    if maxScroll > 0 as libc::c_int as libc::c_float {
        let mut handleSize: libc::c_float = (*data).size.y
            * ((*data).size.y / (*data).minSize.y);
        let mut handlePos: libc::c_float = Lerp(
            0.0f32 as libc::c_double,
            ((*data).size.y - handleSize) as libc::c_double,
            ((*data).offset.y / maxScroll) as libc::c_double,
        ) as libc::c_float;
        HmGui_Rect(4.0f32, handlePos, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
        HmGui_Rect(
            4.0f32,
            handleSize,
            (*this.style).colorFrame.x,
            (*this.style).colorFrame.y,
            (*this.style).colorFrame.z,
            (*this.style).colorFrame.w,
        );
    } else {
        HmGui_Rect(4.0f32, 16.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
    }
    HmGui_EndGroup();
    HmGui_EndGroup();
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_BeginWindow(mut title: cstr) {
    HmGui_BeginGroupStack();
    HmGui_SetStretch(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    (*this.group).focusStyle = 0 as libc::c_int as uint32;
    (*this.group).frameOpacity = 0.95f32;
    let mut data: *mut HmGuiData = HmGui_GetData(this.group);
    if HmGui_GroupHasFocus(0 as libc::c_int) {
        if Input_GetDown(Button_Mouse_Left) {
            let mut md: IVec2 = IVec2 { x: 0, y: 0 };
            Input_GetMouseDelta(&mut md);
            (*data).offset.x += md.x as libc::c_float;
            (*data).offset.y += md.y as libc::c_float;
        }
    }
    (*this.group).widget.pos.x += (*data).offset.x;
    (*this.group).widget.pos.y += (*data).offset.y;
    HmGui_BeginGroupY();
    (*this.group).clip = 1 as libc::c_int != 0;
    HmGui_SetPadding(
        8 as libc::c_int as libc::c_float,
        8 as libc::c_int as libc::c_float,
    );
    HmGui_SetStretch(
        1 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_EndWindow() {
    HmGui_EndGroup();
    HmGui_EndGroup();
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_Button(mut label: cstr) -> bool {
    HmGui_BeginGroupStack();
    (*this.group).focusStyle = 1 as libc::c_int as uint32;
    (*this.group).frameOpacity = 0.5f32;
    let mut focus: bool = HmGui_GroupHasFocus(0 as libc::c_int);
    HmGui_SetPadding(
        8 as libc::c_int as libc::c_float,
        8 as libc::c_int as libc::c_float,
    );
    HmGui_Text(label);
    HmGui_SetAlign(0.5f32, 0.5f32);
    HmGui_EndGroup();
    return focus as libc::c_int != 0 && this.activate as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_Checkbox(mut label: cstr, mut value: bool) -> bool {
    HmGui_BeginGroupX();
    (*this.group).focusStyle = 3 as libc::c_int as uint32;
    if HmGui_GroupHasFocus(0 as libc::c_int) as libc::c_int != 0
        && this.activate as libc::c_int != 0
    {
        value = !value;
    }
    HmGui_SetPadding(
        4 as libc::c_int as libc::c_float,
        4 as libc::c_int as libc::c_float,
    );
    HmGui_SetSpacing(8 as libc::c_int as libc::c_float);
    HmGui_SetStretch(
        1 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    HmGui_Text(label);
    HmGui_SetAlign(0.0f32, 0.5f32);
    HmGui_SetStretch(
        1 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    HmGui_BeginGroupStack();
    HmGui_Rect(
        16 as libc::c_int as libc::c_float,
        16 as libc::c_int as libc::c_float,
        (*this.style).colorFrame.x,
        (*this.style).colorFrame.y,
        (*this.style).colorFrame.z,
        (*this.style).colorFrame.w,
    );
    if value {
        HmGui_Rect(
            10 as libc::c_int as libc::c_float,
            10 as libc::c_int as libc::c_float,
            (*this.style).colorPrimary.x,
            (*this.style).colorPrimary.y,
            (*this.style).colorPrimary.z,
            (*this.style).colorPrimary.w,
        );
        HmGui_SetAlign(0.5f32, 0.5f32);
    }
    HmGui_EndGroup();
    HmGui_SetStretch(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    HmGui_EndGroup();
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_Slider(
    mut lower: libc::c_float,
    mut upper: libc::c_float,
    mut value: libc::c_float,
) -> libc::c_float {
    HmGui_BeginGroupStack();
    HmGui_Rect(
        0 as libc::c_int as libc::c_float,
        2 as libc::c_int as libc::c_float,
        0.5f32,
        0.5f32,
        0.5f32,
        1.0f32,
    );
    HmGui_SetAlign(0.5f32, 0.5f32);
    HmGui_SetStretch(
        1 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    HmGui_EndGroup();
    return 0.0f32;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_Image(mut image: *mut Tex2D) {
    let mut e: *mut HmGuiImage = MemAlloc(
        ::core::mem::size_of::<HmGuiImage>() as usize,
    ) as *mut HmGuiImage;
    HmGui_InitWidget(&mut (*e).widget, 3 as libc::c_int as uint32);
    (*e).image = image;
    (*e)
        .widget
        .stretch = Vec2::new(
        1 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_Rect(
    mut sx: libc::c_float,
    mut sy: libc::c_float,
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
) {
    let mut e: *mut HmGuiRect = MemAlloc(
        ::core::mem::size_of::<HmGuiRect>() as usize,
    ) as *mut HmGuiRect;
    HmGui_InitWidget(&mut (*e).widget, 2 as libc::c_int as uint32);
    (*e).color = Vec4f_Create(r, g, b, a);
    (*e).widget.minSize = Vec2::new(sx, sy);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_Text(mut text: cstr) {
    HmGui_TextEx(
        (*this.style).font,
        text,
        (*this.style).colorText.x,
        (*this.style).colorText.y,
        (*this.style).colorText.z,
        (*this.style).colorText.w,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_TextColored(
    mut text: cstr,
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
) {
    HmGui_TextEx((*this.style).font, text, r, g, b, a);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_TextEx(
    mut font: *mut Font,
    mut text: cstr,
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
) {
    let mut e: *mut HmGuiText = MemAlloc(
        ::core::mem::size_of::<HmGuiText>() as usize,
    ) as *mut HmGuiText;
    HmGui_InitWidget(&mut (*e).widget, 1 as libc::c_int as uint32);
    (*e).font = font;
    (*e).text = StrDup(text);
    (*e).color = Vec4f_Create(r, g, b, a);
    let mut size: IVec2 = IVec2 { x: 0, y: 0 };
    Font_GetSize2((*e).font, &mut size, (*e).text);
    (*e).widget.minSize = Vec2::new(size.x as libc::c_float, size.y as libc::c_float);
    HmGui_SetAlign(0.0f32, 1.0f32);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_SetAlign(mut ax: libc::c_float, mut ay: libc::c_float) {
    (*this.last).align = Vec2::new(ax, ay);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_SetPadding(mut px: libc::c_float, mut py: libc::c_float) {
    (*this.group).paddingLower = Vec2::new(px, py);
    (*this.group).paddingUpper = Vec2::new(px, py);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_SetPaddingEx(
    mut left: libc::c_float,
    mut top: libc::c_float,
    mut right: libc::c_float,
    mut bottom: libc::c_float,
) {
    (*this.group).paddingLower = Vec2::new(left, top);
    (*this.group).paddingUpper = Vec2::new(right, bottom);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_SetPaddingLeft(mut padding: libc::c_float) {
    (*this.group).paddingLower.x = padding;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_SetPaddingTop(mut padding: libc::c_float) {
    (*this.group).paddingLower.y = padding;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_SetPaddingRight(mut padding: libc::c_float) {
    (*this.group).paddingUpper.x = padding;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_SetPaddingBottom(mut padding: libc::c_float) {
    (*this.group).paddingUpper.y = padding;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_SetSpacing(mut spacing: libc::c_float) {
    (*this.group).spacing = spacing;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_SetStretch(mut x: libc::c_float, mut y: libc::c_float) {
    (*this.last).stretch = Vec2::new(x, y);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_GroupHasFocus(mut type_0: libc::c_int) -> bool {
    (*this.group).focusable[type_0 as usize] = 1 as libc::c_int != 0;
    return this.focus[type_0 as usize] == (*this.group).widget.hash;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_PushStyle() {
    let mut style: *mut HmGuiStyle = MemAlloc(
        ::core::mem::size_of::<HmGuiStyle>() as usize,
    ) as *mut HmGuiStyle;
    *style = *this.style;
    (*style).prev = this.style;
    this.style = style;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_PushFont(mut font: *mut Font) {
    HmGui_PushStyle();
    (*this.style).font = font;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_PushTextColor(
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
) {
    HmGui_PushStyle();
    (*this.style).colorText = Vec4f_Create(r, g, b, a);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_PopStyle(mut depth: libc::c_int) {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < depth {
        let mut style: *mut HmGuiStyle = this.style;
        this.style = (*style).prev;
        MemFree(style as *const libc::c_void);
        i += 1;
    }
}
