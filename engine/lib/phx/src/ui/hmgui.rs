use super::*;
use crate::common::*;
// use crate::input::*;
use crate::input2::CursorControl;
use crate::input2::Input;
use crate::input2::MouseControl;
use crate::internal::*;
use crate::math::*;
use crate::render::*;
use crate::system::*;
use crate::*;

pub const Layout_None: u32 = 0;
pub const Layout_Stack: u32 = 1;
pub const Layout_Vertical: u32 = 2;
pub const Layout_Horizontal: u32 = 3;

pub const Widget_Group: u32 = 0;
pub const Widget_Text: u32 = 1;
pub const Widget_Rect: u32 = 2;
pub const Widget_Image: u32 = 3;

pub const FocusStyle_None: u32 = 0;
pub const FocusStyle_Fill: u32 = 1;
pub const FocusStyle_Outline: u32 = 2;
pub const FocusStyle_Underline: u32 = 3;

pub const FocusType_Mouse: i32 = 0;
pub const FocusType_Scroll: i32 = 1;
pub const FocusType_SIZE: i32 = 2;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiGroup {
    pub widget: HmGuiWidget,
    pub head: *mut HmGuiWidget,
    pub tail: *mut HmGuiWidget,
    pub layout: u32,
    pub children: u32,
    pub focusStyle: u32,
    pub paddingLower: Vec2,
    pub paddingUpper: Vec2,
    pub offset: Vec2,
    pub maxSize: Vec2,
    pub totalStretch: Vec2,
    pub spacing: f32,
    pub frameOpacity: f32,
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
    pub hash: u64,
    pub ty: u32,
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
    pub focus: [u64; 2],
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
    pub spacing: f32,
    pub colorPrimary: Vec4,
    pub colorFrame: Vec4,
    pub colorText: Vec4,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiText {
    pub widget: HmGuiWidget,
    pub font: *mut Font,
    pub text: *const libc::c_char,
    pub color: Vec4,
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
    pub color: Vec4,
}

static mut this: HmGui = HmGui {
    group: std::ptr::null_mut(),
    root: std::ptr::null_mut(),
    last: std::ptr::null_mut(),
    style: std::ptr::null_mut(),
    clipRect: std::ptr::null_mut(),
    data: std::ptr::null_mut(),
    focus: [0; 2],
    focusPos: Vec2::ZERO,
    activate: false,
};
static mut init_hmgui: bool = false;

unsafe extern "C" fn HmGui_InitWidget(e: *mut HmGuiWidget, ty: u32) {
    (*e).parent = this.group;
    (*e).next = std::ptr::null_mut();
    (*e).prev = if !(this.group).is_null() {
        (*this.group).tail
    } else {
        std::ptr::null_mut()
    };

    if !((*e).parent).is_null() {
        (*(*e).parent).children = ((*(*e).parent).children).wrapping_add(1);
        (*e).hash = Hash_FNV64_Incremental(
            (*(*e).parent).widget.hash,
            &mut (*(*e).parent).children as *mut u32 as *const _,
            std::mem::size_of::<u32>() as i32,
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

    (*e).ty = ty;
    (*e).pos = Vec2::ZERO;
    (*e).size = Vec2::ZERO;
    (*e).minSize = Vec2::ZERO;
    (*e).align = Vec2::ZERO;
    (*e).stretch = Vec2::ZERO;

    this.last = e;
}

unsafe extern "C" fn HmGui_BeginGroup(layout: u32) {
    let e = MemNew!(HmGuiGroup);
    HmGui_InitWidget(&mut (*e).widget, 0);
    (*e).head = std::ptr::null_mut();
    (*e).tail = std::ptr::null_mut();
    (*e).layout = layout;
    (*e).children = 0;
    (*e).focusStyle = FocusStyle_None;
    (*e).paddingLower = Vec2::ZERO;
    (*e).paddingUpper = Vec2::ZERO;
    (*e).offset = Vec2::ZERO;
    (*e).maxSize = Vec2::new(1e30f32, 1e30f32);
    (*e).spacing = (*this.style).spacing;
    (*e).frameOpacity = 0.0f32;
    (*e).clip = false;
    (*e).expand = true;
    for i in 0..FocusType_SIZE {
        (*e).focusable[i as usize] = false;
    }
    (*e).storeSize = false;
    this.group = e;

    match layout {
        Layout_Stack => {
            (*e).widget.stretch = Vec2::ONE;
        }
        Layout_Vertical => {
            (*e).widget.stretch = Vec2::X;
        }
        Layout_Horizontal => {
            (*e).widget.stretch = Vec2::Y;
        }
        _ => {}
    };
}

unsafe extern "C" fn HmGui_FreeText(e: *mut HmGuiText) {
    StrFree((*e).text);
    MemFree(e as *const _);
}

unsafe extern "C" fn HmGui_FreeGroup(g: *mut HmGuiGroup) {
    let mut e: *mut HmGuiWidget = (*g).head;
    while !e.is_null() {
        let next: *mut HmGuiWidget = (*e).next;
        match (*e).ty {
            Widget_Group => {
                HmGui_FreeGroup(e as *mut HmGuiGroup);
            }
            Widget_Text => {
                HmGui_FreeText(e as *mut HmGuiText);
            }
            _ => {
                MemFree(e as *const _);
            }
        }
        e = next;
    }
    MemFree(g as *const _);
}

unsafe extern "C" fn HmGui_GetData(g: *mut HmGuiGroup) -> *mut HmGuiData {
    let mut data: *mut HmGuiData = HashMap_GetRaw(this.data, (*g).widget.hash) as *mut HmGuiData;
    if data.is_null() {
        data = MemNew!(HmGuiData);
        (*data).offset = Vec2::ZERO;
        (*data).minSize = Vec2::ZERO;
        (*data).size = Vec2::ZERO;
        HashMap_SetRaw(this.data, (*g).widget.hash, data as *mut _);
    }
    data
}

unsafe extern "C" fn HmGui_ComputeSize(g: *mut HmGuiGroup) {
    let mut e: *mut HmGuiWidget = (*g).head;
    while !e.is_null() {
        if (*e).ty == Widget_Group {
            HmGui_ComputeSize(e as *mut HmGuiGroup);
        }
        e = (*e).next;
    }

    (*g).widget.minSize = Vec2::ZERO;

    let mut e: *mut HmGuiWidget = (*g).head;
    while !e.is_null() {
        match (*g).layout {
            Layout_Stack => {
                (*g).widget.minSize.x = f32::max((*g).widget.minSize.x, (*e).minSize.x);
                (*g).widget.minSize.y = f32::max((*g).widget.minSize.y, (*e).minSize.y);
            }
            Layout_Vertical => {
                (*g).widget.minSize.x = f32::max((*g).widget.minSize.x, (*e).minSize.x);
                (*g).widget.minSize.y += (*e).minSize.y;
                if e != (*g).head {
                    (*g).widget.minSize.y += (*g).spacing;
                }
            }
            Layout_Horizontal => {
                (*g).widget.minSize.x += (*e).minSize.x;
                (*g).widget.minSize.y = f32::max((*g).widget.minSize.y, (*e).minSize.y);
                if e != (*g).head {
                    (*g).widget.minSize.x += (*g).spacing;
                }
            }
            _ => {}
        }
        e = (*e).next;
    }

    (*g).widget.minSize.x += (*g).paddingLower.x + (*g).paddingUpper.x;
    (*g).widget.minSize.y += (*g).paddingLower.y + (*g).paddingUpper.y;

    if (*g).storeSize {
        let data: *mut HmGuiData = HmGui_GetData(g);
        (*data).minSize = (*g).widget.minSize;
    }

    (*g).widget.minSize.x = f32::min((*g).widget.minSize.x, (*g).maxSize.x);
    (*g).widget.minSize.y = f32::min((*g).widget.minSize.y, (*g).maxSize.y);
}

unsafe extern "C" fn HmGui_LayoutWidget(e: *mut HmGuiWidget, pos: Vec2, sx: f32, sy: f32) {
    (*e).pos = pos;
    (*e).size = (*e).minSize;
    (*e).size.x += (*e).stretch.x * (sx - (*e).minSize.x);
    (*e).size.y += (*e).stretch.y * (sy - (*e).minSize.y);
    (*e).pos.x += (*e).align.x * (sx - (*e).size.x);
    (*e).pos.y += (*e).align.y * (sy - (*e).size.y);
}

unsafe extern "C" fn HmGui_LayoutGroup(g: *mut HmGuiGroup) {
    let mut pos = (*g).widget.pos;
    let mut size = (*g).widget.size;
    let mut extra: f32 = 0.0f32;
    let mut totalStretch: f32 = 0.0f32;

    pos.x += (*g).paddingLower.x + (*g).offset.x;
    pos.y += (*g).paddingLower.y + (*g).offset.y;
    size.x -= (*g).paddingLower.x + (*g).paddingUpper.x;
    size.y -= (*g).paddingLower.y + (*g).paddingUpper.y;

    if (*g).expand {
        if (*g).layout == Layout_Vertical {
            extra = (*g).widget.size.y - (*g).widget.minSize.y;
            let mut e: *mut HmGuiWidget = (*g).head;
            while !e.is_null() {
                totalStretch += (*e).stretch.y;
                e = (*e).next;
            }
        } else if (*g).layout == 3 {
            extra = (*g).widget.size.x - (*g).widget.minSize.x;
            let mut e: *mut HmGuiWidget = (*g).head;
            while !e.is_null() {
                totalStretch += (*e).stretch.x;
                e = (*e).next;
            }
        }

        if totalStretch > 0.0f32 {
            extra /= totalStretch;
        }
    }

    let mut e: *mut HmGuiWidget = (*g).head;
    while !e.is_null() {
        match (*g).layout {
            Layout_None => {
                HmGui_LayoutWidget(e, (*e).pos, size.x, size.y);
            }
            Layout_Stack => {
                HmGui_LayoutWidget(e, pos, size.x, size.y);
            }
            Layout_Vertical => {
                let mut s = (*e).minSize.y;
                if extra > 0.0f32 {
                    s += (*e).stretch.y * extra;
                }
                HmGui_LayoutWidget(e, pos, size.x, s);
                pos.y += (*e).size.y + (*g).spacing;
            }
            Layout_Horizontal => {
                let mut s = (*e).minSize.x;
                if extra > 0.0f32 {
                    s += (*e).stretch.x * extra;
                }
                HmGui_LayoutWidget(e, pos, s, size.y);
                pos.x += (*e).size.x + (*g).spacing;
            }
            _ => {}
        }

        if (*e).ty == Widget_Group {
            HmGui_LayoutGroup(e as *mut HmGuiGroup);
        }

        e = (*e).next;
    }

    if (*g).storeSize {
        let data: *mut HmGuiData = HmGui_GetData(g);
        (*data).size = (*g).widget.size;
    }
}

#[inline]
unsafe extern "C" fn IsClipped(g: *mut HmGuiGroup, p: Vec2) -> bool {
    p.x < (*g).widget.pos.x
        || p.y < (*g).widget.pos.y
        || (*g).widget.pos.x + (*g).widget.size.x < p.x
        || (*g).widget.pos.y + (*g).widget.size.y < p.y
}

unsafe extern "C" fn HmGui_CheckFocus(g: *mut HmGuiGroup) {
    if (*g).clip as i32 != 0 && IsClipped(g, this.focusPos) as i32 != 0 {
        return;
    }

    let mut e: *mut HmGuiWidget = (*g).tail;
    while !e.is_null() {
        if (*e).ty == Widget_Group {
            HmGui_CheckFocus(e as *mut HmGuiGroup);
        }
        e = (*e).prev;
    }

    for i in 0..FocusType_SIZE {
        if this.focus[i as usize] == 0
            && (*g).focusable[i as usize] as i32 != 0
            && (*g).widget.pos.x <= this.focusPos.x
            && (*g).widget.pos.y <= this.focusPos.y
            && this.focusPos.x <= (*g).widget.pos.x + (*g).widget.size.x
            && this.focusPos.y <= (*g).widget.pos.y + (*g).widget.size.y
        {
            this.focus[i as usize] = (*g).widget.hash;
        }
    }
}

unsafe extern "C" fn HmGui_DrawText(e: *mut HmGuiText) {
    // #if HMGUI_DRAW_GROUP_FRAMES
    //   Draw_Color(0.5f, 0.2f, 0.2f, 0.5f);
    //   Draw_Border(1.0f, e->pos.x, e->pos.y, e->size.x, e->size.y);
    //#endif

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

unsafe extern "C" fn HmGui_DrawRect(e: *mut HmGuiRect) {
    UIRenderer_Rect(
        (*e).widget.pos.x,
        (*e).widget.pos.y,
        (*e).widget.size.x,
        (*e).widget.size.y,
        (*e).color.x,
        (*e).color.y,
        (*e).color.z,
        (*e).color.w,
        false,
    );
}

unsafe extern "C" fn HmGui_DrawImage(e: *mut HmGuiImage) {
    UIRenderer_Image(
        (*e).image,
        (*e).widget.pos.x,
        (*e).widget.pos.y,
        (*e).widget.size.x,
        (*e).widget.size.y,
    );
}

unsafe extern "C" fn HmGui_DrawGroup(g: *mut HmGuiGroup) {
    // #if HMGUI_DRAW_GROUP_FRAMES
    //   Draw_Color(0.2f, 0.2f, 0.2f, 0.5f);
    //   Draw_Border(2.0f, g->pos.x, g->pos.y, g->size.x, g->size.y);
    // #endif

    UIRenderer_BeginLayer(
        (*g).widget.pos.x,
        (*g).widget.pos.y,
        (*g).widget.size.x,
        (*g).widget.size.y,
        (*g).clip,
    );

    let mut e: *mut HmGuiWidget = (*g).tail;
    while !e.is_null() {
        match (*e).ty {
            Widget_Group => {
                HmGui_DrawGroup(e as *mut HmGuiGroup);
            }
            Widget_Text => {
                HmGui_DrawText(e as *mut HmGuiText);
            }
            Widget_Rect => {
                HmGui_DrawRect(e as *mut HmGuiRect);
            }
            Widget_Image => {
                HmGui_DrawImage(e as *mut HmGuiImage);
            }
            _ => {}
        }
        e = (*e).prev;
    }

    if (*g).focusable[FocusType_Mouse as usize] {
        let focus: bool = this.focus[FocusType_Mouse as usize] == (*g).widget.hash;
        if (*g).focusStyle == FocusStyle_None {
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
        } else if (*g).focusStyle == FocusStyle_Fill {
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
        } else if (*g).focusStyle == FocusStyle_Outline {
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
                    true,
                );
            }
        } else if (*g).focusStyle == FocusStyle_Underline {
            UIRenderer_Rect(
                (*g).widget.pos.x,
                (*g).widget.pos.y,
                (*g).widget.size.x,
                (*g).widget.size.y,
                0.3f32,
                0.3f32,
                0.3f32,
                if focus as i32 != 0 {
                    0.5f32
                } else {
                    (*g).frameOpacity
                },
                false,
            );
        }
    }
    UIRenderer_EndLayer();
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_Begin(sx: f32, sy: f32, input: &Input) {
    if !init_hmgui {
        init_hmgui = true;
        this.group = std::ptr::null_mut();
        this.root = std::ptr::null_mut();

        this.style = MemNew!(HmGuiStyle);
        (*this.style).prev = std::ptr::null_mut();
        (*this.style).font = Font_Load(c_str!("Rajdhani"), 14);
        (*this.style).spacing = 6.0f32;

        (*this.style).colorPrimary = Vec4::new(0.1f32, 0.5f32, 1.0f32, 1.0f32);
        (*this.style).colorFrame = Vec4::new(0.1f32, 0.1f32, 0.1f32, 0.5f32);
        (*this.style).colorText = Vec4::ONE;

        this.clipRect = std::ptr::null_mut();
        this.data = HashMap_Create(0, 128);

        for i in 0..FocusType_SIZE {
            this.focus[i as usize] = 0;
        }
        this.activate = false;
    }

    if !(this.root).is_null() {
        HmGui_FreeGroup(this.root);
        this.root = std::ptr::null_mut();
    }
    this.last = std::ptr::null_mut();
    this.activate = input.mouse().is_pressed(MouseControl::Left);

    HmGui_BeginGroup(0);
    (*this.group).clip = true;
    (*this.group).widget.pos = Vec2::ZERO;
    (*this.group).widget.size = Vec2::new(sx, sy);
    this.root = this.group;
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_End(input: &Input) {
    Profiler_Begin(c_str!("HmGui_End"));
    HmGui_EndGroup();
    HmGui_ComputeSize(this.root);
    HmGui_LayoutGroup(this.root);

    for i in 0..FocusType_SIZE {
        this.focus[i as usize] = 0;
    }

    let cursor = input.cursor();

    this.focusPos = Vec2::new(
        cursor.value(CursorControl::X),
        cursor.value(CursorControl::Y),
    );

    HmGui_CheckFocus(this.root);
    Profiler_End();
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_Draw() {
    Profiler_Begin(c_str!("HmGui_Draw"));
    RenderState_PushBlendMode(1);
    UIRenderer_Begin();
    HmGui_DrawGroup(this.root);
    UIRenderer_End();
    RenderState_PopBlendMode();
    UIRenderer_Draw();
    Profiler_End();
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_BeginGroupX() {
    HmGui_BeginGroup(Layout_Horizontal);
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_BeginGroupY() {
    HmGui_BeginGroup(Layout_Vertical);
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_BeginGroupStack() {
    HmGui_BeginGroup(Layout_Stack);
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_EndGroup() {
    this.last = &mut (*this.group).widget;
    this.group = (*this.group).widget.parent;
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_BeginScroll(maxSize: f32) {
    HmGui_BeginGroupX();
    HmGui_SetStretch(1.0f32, 1.0f32);
    (*this.group).clip = true;
    HmGui_SetSpacing(2.0f32);
    HmGui_BeginGroupY();
    HmGui_SetPadding(6.0f32, 6.0f32);
    HmGui_SetStretch(1.0f32, 1.0f32);
    (*this.group).expand = false;
    (*this.group).storeSize = true;
    (*this.group).maxSize.y = maxSize;

    let data: *mut HmGuiData = HmGui_GetData(this.group);
    (*this.group).offset.y = -(*data).offset.y;
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_EndScroll(input: Input) {
    let data = HmGui_GetData(this.group);

    if HmGui_GroupHasFocus(1) {
        let scroll_y = input.mouse().value(MouseControl::ScrollY);

        (*data).offset.y -= 10.0f32 * scroll_y as f32;
    }

    let maxScroll: f32 = f32::max(0.0f32, (*data).minSize.y - (*data).size.y);
    (*data).offset.y = f32::clamp((*data).offset.y, 0.0f32, maxScroll);

    HmGui_EndGroup();

    HmGui_BeginGroupY();
    HmGui_SetStretch(0.0f32, 1.0f32);
    HmGui_SetSpacing(0.0f32);
    if maxScroll > 0.0f32 {
        let handleSize: f32 = (*data).size.y * ((*data).size.y / (*data).minSize.y);
        let handlePos: f32 = Lerp(
            0.0f64,
            ((*data).size.y - handleSize) as f64,
            ((*data).offset.y / maxScroll) as f64,
        ) as f32;
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
pub unsafe extern "C" fn HmGui_BeginWindow(_title: *const libc::c_char, input: Input) {
    HmGui_BeginGroupStack();
    HmGui_SetStretch(0.0f32, 0.0f32);

    (*this.group).focusStyle = FocusStyle_None;
    (*this.group).frameOpacity = 0.95f32;

    let data = HmGui_GetData(this.group);
    let mouse = input.mouse();

    if HmGui_GroupHasFocus(0) && mouse.is_down(MouseControl::Left) {
        (*data).offset.x += mouse.value(MouseControl::DeltaX);
        (*data).offset.y += mouse.value(MouseControl::DeltaY);
    }

    (*this.group).widget.pos.x += (*data).offset.x;
    (*this.group).widget.pos.y += (*data).offset.y;

    HmGui_BeginGroupY();
    (*this.group).clip = true;
    HmGui_SetPadding(8.0f32, 8.0f32);
    HmGui_SetStretch(1.0f32, 1.0f32);
    // HmGui_TextColored(title, 1.0f, 1.0f, 1.0f, 0.3f);
    // HmGui_SetAlign(0.5f, 0.0f);
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_EndWindow() {
    HmGui_EndGroup();
    HmGui_EndGroup();
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_Button(label: *const libc::c_char) -> bool {
    HmGui_BeginGroupStack();
    (*this.group).focusStyle = FocusStyle_Fill;
    (*this.group).frameOpacity = 0.5f32;
    let focus: bool = HmGui_GroupHasFocus(FocusType_Mouse);
    HmGui_SetPadding(8.0f32, 8.0f32);
    HmGui_Text(label);
    HmGui_SetAlign(0.5f32, 0.5f32);
    HmGui_EndGroup();
    focus as i32 != 0 && this.activate as i32 != 0
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_Checkbox(label: *const libc::c_char, mut value: bool) -> bool {
    HmGui_BeginGroupX();
    (*this.group).focusStyle = FocusStyle_Underline;
    if HmGui_GroupHasFocus(FocusType_Mouse) as i32 != 0 && this.activate as i32 != 0 {
        value = !value;
    }
    HmGui_SetPadding(4.0f32, 4.0f32);
    HmGui_SetSpacing(8.0f32);
    HmGui_SetStretch(1.0f32, 0.0f32);

    HmGui_Text(label);
    HmGui_SetAlign(0.0f32, 0.5f32);
    HmGui_SetStretch(1.0f32, 0.0f32);

    HmGui_BeginGroupStack();
    HmGui_Rect(
        16.0f32,
        16.0f32,
        (*this.style).colorFrame.x,
        (*this.style).colorFrame.y,
        (*this.style).colorFrame.z,
        (*this.style).colorFrame.w,
    );
    if value {
        HmGui_Rect(
            10.0f32,
            10.0f32,
            (*this.style).colorPrimary.x,
            (*this.style).colorPrimary.y,
            (*this.style).colorPrimary.z,
            (*this.style).colorPrimary.w,
        );
        HmGui_SetAlign(0.5f32, 0.5f32);
    }
    HmGui_EndGroup();
    HmGui_SetStretch(0.0f32, 0.0f32);
    HmGui_EndGroup();
    value
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_Slider(_lower: f32, _upper: f32, _value: f32) -> f32 {
    HmGui_BeginGroupStack();
    HmGui_Rect(0.0f32, 2.0f32, 0.5f32, 0.5f32, 0.5f32, 1.0f32);
    HmGui_SetAlign(0.5f32, 0.5f32);
    HmGui_SetStretch(1.0f32, 0.0f32);
    HmGui_EndGroup();
    0.0f32
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_Image(image: *mut Tex2D) {
    let e = MemNew!(HmGuiImage);
    HmGui_InitWidget(&mut (*e).widget, Widget_Image);
    (*e).image = image;
    (*e).widget.stretch = Vec2::ONE;
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_Rect(sx: f32, sy: f32, r: f32, g: f32, b: f32, a: f32) {
    let e = MemNew!(HmGuiRect);
    HmGui_InitWidget(&mut (*e).widget, Widget_Rect);
    (*e).color = Vec4::new(r, g, b, a);
    (*e).widget.minSize = Vec2::new(sx, sy);
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_Text(text: *const libc::c_char) {
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
    text: *const libc::c_char,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) {
    HmGui_TextEx((*this.style).font, text, r, g, b, a);
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_TextEx(
    font: *mut Font,
    text: *const libc::c_char,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) {
    let e = MemNew!(HmGuiText);

    HmGui_InitWidget(&mut (*e).widget, Widget_Text);

    (*e).font = font;
    (*e).text = StrDup(text);
    (*e).color = Vec4::new(r, g, b, a);

    let mut size = IVec2::ZERO;

    Font_GetSize2(&mut *(*e).font, &mut size, (*e).text);

    (*e).widget.minSize = Vec2::new(size.x as f32, size.y as f32);

    HmGui_SetAlign(0.0f32, 1.0f32);
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_SetAlign(ax: f32, ay: f32) {
    (*this.last).align = Vec2::new(ax, ay);
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_SetPadding(px: f32, py: f32) {
    (*this.group).paddingLower = Vec2::new(px, py);
    (*this.group).paddingUpper = Vec2::new(px, py);
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_SetPaddingEx(left: f32, top: f32, right: f32, bottom: f32) {
    (*this.group).paddingLower = Vec2::new(left, top);
    (*this.group).paddingUpper = Vec2::new(right, bottom);
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_SetPaddingLeft(padding: f32) {
    (*this.group).paddingLower.x = padding;
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_SetPaddingTop(padding: f32) {
    (*this.group).paddingLower.y = padding;
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_SetPaddingRight(padding: f32) {
    (*this.group).paddingUpper.x = padding;
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_SetPaddingBottom(padding: f32) {
    (*this.group).paddingUpper.y = padding;
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_SetSpacing(spacing: f32) {
    (*this.group).spacing = spacing;
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_SetStretch(x: f32, y: f32) {
    (*this.last).stretch = Vec2::new(x, y);
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_GroupHasFocus(ty: i32) -> bool {
    (*this.group).focusable[ty as usize] = true;
    this.focus[ty as usize] == (*this.group).widget.hash
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_PushStyle() {
    let style = MemNew!(HmGuiStyle);
    *style = *this.style;
    (*style).prev = this.style;
    this.style = style;
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_PushFont(font: *mut Font) {
    HmGui_PushStyle();
    (*this.style).font = font;
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_PushTextColor(r: f32, g: f32, b: f32, a: f32) {
    HmGui_PushStyle();
    (*this.style).colorText = Vec4::new(r, g, b, a);
}

#[no_mangle]
pub unsafe extern "C" fn HmGui_PopStyle(depth: i32) {
    for _ in 0..depth {
        let style: *mut HmGuiStyle = this.style;
        this.style = (*style).prev;
        MemFree(style as *const _);
    }
}
