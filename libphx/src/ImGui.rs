use ::libc;
use glam::{IVec2, Vec2};
use crate::internal::Memory::*;
use crate::Button::*;

extern "C" {
    pub type Font;
    pub type HashMap;
    pub type MemPool;
    pub type Shader;
    pub type Tex2D;
    fn Fatal(_: cstr, _: ...);
    fn ClipRect_PushCombined(
        x: libc::c_float,
        y: libc::c_float,
        sx: libc::c_float,
        sy: libc::c_float,
    );
    fn ClipRect_Pop();
    fn Draw_Rect(
        x: libc::c_float,
        y: libc::c_float,
        sx: libc::c_float,
        sy: libc::c_float,
    );
    fn Draw_Border(
        s: libc::c_float,
        x: libc::c_float,
        y: libc::c_float,
        w: libc::c_float,
        h: libc::c_float,
    );
    fn Draw_LineWidth(width: libc::c_float);
    fn Draw_Color(
        r: libc::c_float,
        g: libc::c_float,
        b: libc::c_float,
        a: libc::c_float,
    );
    fn Font_Load(name: cstr, size: libc::c_int) -> *mut Font;
    fn Font_Draw(
        _: *mut Font,
        text: cstr,
        x: libc::c_float,
        y: libc::c_float,
        r: libc::c_float,
        g: libc::c_float,
        b: libc::c_float,
        a: libc::c_float,
    );
    fn Font_GetLineHeight(_: *mut Font) -> libc::c_int;
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
    fn MemPool_CreateAuto(elemSize: uint32) -> *mut MemPool;
    fn MemPool_Alloc(_: *mut MemPool) -> *mut libc::c_void;
    fn MemPool_Clear(_: *mut MemPool);
    fn MemPool_Dealloc(_: *mut MemPool, _: *mut libc::c_void);
    fn RenderState_PushBlendMode(_: BlendMode);
    fn RenderState_PopBlendMode();
    fn Shader_Load(vertName: cstr, fragName: cstr) -> *mut Shader;
    fn Shader_Start(_: *mut Shader);
    fn Shader_Stop(_: *mut Shader);
    fn Shader_SetFloat(_: cstr, _: libc::c_float);
    fn Shader_SetFloat2(_: cstr, _: libc::c_float, _: libc::c_float);
    fn Shader_SetFloat4(
        _: cstr,
        _: libc::c_float,
        _: libc::c_float,
        _: libc::c_float,
        _: libc::c_float,
    );
    fn Tex2D_Draw(
        _: *mut Tex2D,
        x: libc::c_float,
        y: libc::c_float,
        sx: libc::c_float,
        sy: libc::c_float,
    );
    fn Tex2D_GetSize(_: *mut Tex2D, out: *mut IVec2);
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
pub struct ImGui {
    pub layer: *mut ImGuiLayer,
    pub layerLast: *mut ImGuiLayer,
    pub layout: *mut ImGuiLayout,
    pub widget: *mut ImGuiWidget,
    pub widgetLast: *mut ImGuiWidget,
    pub style: *mut ImGuiStyle,
    pub clipRect: *mut ImGuiClipRect,
    pub cursorStack: *mut ImGuiCursor,
    pub cursor: Vec2,
    pub mouse: Vec2,
    pub focus: [uint64; 3],
    pub dragging: uint64,
    pub activate: bool,
    pub forceSize: Vec2,
    pub data: *mut HashMap,
    pub layoutPool: *mut MemPool,
    pub widgetPool: *mut MemPool,
    pub stylePool: *mut MemPool,
    pub clipRectPool: *mut MemPool,
    pub cursorPool: *mut MemPool,
    pub tex2DPool: *mut MemPool,
    pub panelPool: *mut MemPool,
    pub rectPool: *mut MemPool,
    pub textPool: *mut MemPool,
    pub linePool: *mut MemPool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ImGuiCursor {
    pub prev: *mut ImGuiCursor,
    pub pos: Vec2,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ImGuiClipRect {
    pub prev: *mut ImGuiClipRect,
    pub p1: Vec2,
    pub p2: Vec2,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ImGuiStyle {
    pub prev: *mut ImGuiStyle,
    pub font: *mut Font,
    pub fontSubheading: *mut Font,
    pub spacing: Vec2,
    pub padding: Vec2,
    pub scrollBarSize: Vec2,
    pub buttonColor: Vec4f,
    pub buttonColorFocus: Vec4f,
    pub frameColor: Vec4f,
    pub textColor: Vec4f,
    pub textColorFocus: Vec4f,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ImGuiWidget {
    pub prev: *mut ImGuiWidget,
    pub hash: uint64,
    pub index: uint32,
    pub pos: Vec2,
    pub size: Vec2,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ImGuiLayout {
    pub prev: *mut ImGuiLayout,
    pub lower: Vec2,
    pub upper: Vec2,
    pub size: Vec2,
    pub spacing: Vec2,
    pub styleVars: libc::c_int,
    pub horizontal: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ImGuiLayer {
    pub parent: *mut ImGuiLayer,
    pub next: *mut ImGuiLayer,
    pub children: *mut ImGuiLayer,
    pub pos: Vec2,
    pub size: Vec2,
    pub hash: uint64,
    pub index: uint32,
    pub clip: bool,
    pub tex2DList: *mut ImGuiTex2D,
    pub rectList: *mut ImGuiRect,
    pub panelList: *mut ImGuiPanel,
    pub textList: *mut ImGuiText,
    pub lineList: *mut ImGuiLine,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ImGuiLine {
    pub next: *mut ImGuiLine,
    pub color: Vec4f,
    pub p1: Vec2,
    pub p2: Vec2,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ImGuiText {
    pub next: *mut ImGuiText,
    pub font: *mut Font,
    pub color: Vec4f,
    pub pos: Vec2,
    pub text: cstr,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ImGuiPanel {
    pub next: *mut ImGuiPanel,
    pub color: Vec4f,
    pub pos: Vec2,
    pub size: Vec2,
    pub innerAlpha: libc::c_float,
    pub bevel: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ImGuiRect {
    pub next: *mut ImGuiRect,
    pub color: Vec4f,
    pub pos: Vec2,
    pub size: Vec2,
    pub outline: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ImGuiTex2D {
    pub next: *mut ImGuiTex2D,
    pub tex: *mut Tex2D,
    pub pos: Vec2,
    pub size: Vec2,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ImGuiData {
    pub size: Vec2,
    pub offset: Vec2,
    pub scroll: libc::c_float,
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

#[no_mangle]
pub static mut FocusType_Mouse: libc::c_int = 0 as libc::c_int;
#[no_mangle]
pub static mut FocusType_Keyboard: libc::c_int = 1 as libc::c_int;
#[no_mangle]
pub static mut FocusType_Scroll: libc::c_int = 2 as libc::c_int;
#[no_mangle]
pub static mut FocusType_SIZE: libc::c_int = 3 as libc::c_int;
static mut self_0: ImGui = ImGui {
    layer: 0 as *const ImGuiLayer as *mut ImGuiLayer,
    layerLast: 0 as *const ImGuiLayer as *mut ImGuiLayer,
    layout: 0 as *const ImGuiLayout as *mut ImGuiLayout,
    widget: 0 as *const ImGuiWidget as *mut ImGuiWidget,
    widgetLast: 0 as *const ImGuiWidget as *mut ImGuiWidget,
    style: 0 as *const ImGuiStyle as *mut ImGuiStyle,
    clipRect: 0 as *const ImGuiClipRect as *mut ImGuiClipRect,
    cursorStack: 0 as *const ImGuiCursor as *mut ImGuiCursor,
    cursor: Vec2::ZERO,
    mouse: Vec2::ZERO,
    focus: [0; 3],
    dragging: 0,
    activate: false,
    forceSize: Vec2::ZERO,
    data: 0 as *const HashMap as *mut HashMap,
    layoutPool: 0 as *const MemPool as *mut MemPool,
    widgetPool: 0 as *const MemPool as *mut MemPool,
    stylePool: 0 as *const MemPool as *mut MemPool,
    clipRectPool: 0 as *const MemPool as *mut MemPool,
    cursorPool: 0 as *const MemPool as *mut MemPool,
    tex2DPool: 0 as *const MemPool as *mut MemPool,
    panelPool: 0 as *const MemPool as *mut MemPool,
    rectPool: 0 as *const MemPool as *mut MemPool,
    textPool: 0 as *const MemPool as *mut MemPool,
    linePool: 0 as *const MemPool as *mut MemPool,
};
#[inline]
unsafe extern "C" fn EmitLine(mut color: Vec4f, mut p1: Vec2, mut p2: Vec2) {
    let mut e: *mut ImGuiLine = MemPool_Alloc(self_0.linePool) as *mut ImGuiLine;
    (*e).color = color;
    (*e).p1 = p1;
    (*e).p2 = p2;
    (*e).next = (*self_0.layer).lineList;
    (*self_0.layer).lineList = e;
}
#[inline]
unsafe extern "C" fn EmitPanel(
    mut color: Vec4f,
    mut pos: Vec2,
    mut size: Vec2,
    mut innerAlpha: libc::c_float,
    mut bevel: libc::c_float,
) {
    let mut e: *mut ImGuiPanel = MemPool_Alloc(self_0.panelPool) as *mut ImGuiPanel;
    (*e).color = color;
    (*e).pos = pos;
    (*e).size = size;
    (*e).innerAlpha = innerAlpha;
    (*e).bevel = bevel;
    (*e).next = (*self_0.layer).panelList;
    (*self_0.layer).panelList = e;
}
#[inline]
unsafe extern "C" fn EmitRect(
    mut color: Vec4f,
    mut pos: Vec2,
    mut size: Vec2,
    mut outline: bool,
) {
    let mut e: *mut ImGuiRect = MemPool_Alloc(self_0.rectPool) as *mut ImGuiRect;
    (*e).color = color;
    (*e).pos = pos;
    (*e).size = size;
    (*e).outline = outline;
    (*e).next = (*self_0.layer).rectList;
    (*self_0.layer).rectList = e;
}
#[inline]
unsafe extern "C" fn EmitTex2D(mut tex: *mut Tex2D, mut pos: Vec2, mut size: Vec2) {
    let mut e: *mut ImGuiTex2D = MemPool_Alloc(self_0.tex2DPool) as *mut ImGuiTex2D;
    (*e).tex = tex;
    (*e).pos = pos;
    (*e).size = size;
    (*e).next = (*self_0.layer).tex2DList;
    (*self_0.layer).tex2DList = e;
}
#[inline]
unsafe extern "C" fn EmitText(
    mut font: *mut Font,
    mut color: Vec4f,
    mut pos: Vec2,
    mut text: cstr,
) {
    let mut e: *mut ImGuiText = MemPool_Alloc(self_0.textPool) as *mut ImGuiText;
    (*e).font = font;
    (*e).color = color;
    (*e).pos = pos;
    (*e).text = StrDup(text);
    (*e).next = (*self_0.layer).textList;
    (*self_0.layer).textList = e;
}
#[inline]
unsafe extern "C" fn GetData(mut hash: uint64) -> *mut ImGuiData {
    let mut data: *mut ImGuiData = HashMap_GetRaw(self_0.data, hash) as *mut ImGuiData;
    if data.is_null() {
        data = MemAlloc(::core::mem::size_of::<ImGuiData>())
            as *mut ImGuiData;
        (*data)
            .size = Vec2::new(
            0 as libc::c_int as libc::c_float,
            0 as libc::c_int as libc::c_float,
        );
        (*data)
            .offset = Vec2::new(
            0 as libc::c_int as libc::c_float,
            0 as libc::c_int as libc::c_float,
        );
        (*data).scroll = 0 as libc::c_int as libc::c_float;
        HashMap_SetRaw(self_0.data, hash, data as *mut libc::c_void);
    }
    return data;
}
unsafe extern "C" fn ImGui_PushDefaultStyle() {
    static mut font: *mut Font = 0 as *const Font as *mut Font;
    static mut fontSubheading: *mut Font = 0 as *const Font as *mut Font;
    if font.is_null() {
        font = Font_Load(
            b"Share\0" as *const u8 as *const libc::c_char,
            16 as libc::c_int,
        );
        fontSubheading = Font_Load(
            b"Iceland\0" as *const u8 as *const libc::c_char,
            18 as libc::c_int,
        );
    }
    let mut style: *mut ImGuiStyle = MemPool_Alloc(self_0.stylePool) as *mut ImGuiStyle;
    (*style).prev = self_0.style;
    (*style).font = font;
    (*style).fontSubheading = fontSubheading;
    (*style).spacing = Vec2::new(8.0f32, 8.0f32);
    (*style).padding = Vec2::new(8.0f32, 8.0f32);
    (*style).scrollBarSize = Vec2::new(4.0f32, 4.0f32);
    (*style).buttonColor = Vec4f_Create(0.1f32, 0.12f32, 0.15f32, 1.0f32);
    (*style).buttonColorFocus = Vec4f_Create(0.1f32, 0.6f32, 1.0f32, 1.0f32);
    (*style).frameColor = Vec4f_Create(0.1f32, 0.12f32, 0.15f32, 0.95f32);
    (*style).textColor = Vec4f_Create(1.0f32, 1.0f32, 1.0f32, 1.0f32);
    (*style).textColorFocus = Vec4f_Create(0.1f32, 0.1f32, 0.1f32, 1.0f32);
    self_0.style = style;
}
unsafe extern "C" fn ImGui_PushClipRect(mut pos: Vec2, mut size: Vec2) {
    let mut rect: *mut ImGuiClipRect = MemAlloc(
        ::core::mem::size_of::<ImGuiClipRect>() as usize,
    ) as *mut ImGuiClipRect;
    let mut prev: *mut ImGuiClipRect = self_0.clipRect;
    (*rect).prev = prev;
    (*rect).p1 = pos;
    (*rect).p2 = pos + size;
    if !prev.is_null() {
        (*rect)
            .p1
            .x = Max((*rect).p1.x as libc::c_double, (*prev).p1.x as libc::c_double)
            as libc::c_float;
        (*rect)
            .p1
            .y = Max((*rect).p1.y as libc::c_double, (*prev).p1.y as libc::c_double)
            as libc::c_float;
        (*rect)
            .p2
            .x = Min((*rect).p2.x as libc::c_double, (*prev).p2.x as libc::c_double)
            as libc::c_float;
        (*rect)
            .p2
            .y = Min((*rect).p2.y as libc::c_double, (*prev).p2.y as libc::c_double)
            as libc::c_float;
    }
    self_0.clipRect = rect;
}
unsafe extern "C" fn ImGui_PopClipRect() {
    let mut rect: *mut ImGuiClipRect = self_0.clipRect;
    self_0.clipRect = (*rect).prev;
    MemFree(rect as *const libc::c_void);
}
#[inline]
unsafe extern "C" fn IsClipped(mut p: Vec2) -> bool {
    if (self_0.clipRect).is_null() {
        return 0 as libc::c_int != 0;
    }
    return p.x < (*self_0.clipRect).p1.x || p.y < (*self_0.clipRect).p1.y
        || (*self_0.clipRect).p2.x < p.x || (*self_0.clipRect).p2.y < p.y;
}
#[inline]
unsafe extern "C" fn Advance(mut size: Vec2) {
    if (*self_0.layout).horizontal {
        self_0.cursor.x += size.x;
        (*self_0.layout).spacing.x = (*self_0.style).spacing.x;
    } else {
        self_0.cursor.y += size.y;
        (*self_0.layout).spacing.y = (*self_0.style).spacing.y;
    };
}
#[inline]
unsafe extern "C" fn HashGet() -> uint64 {
    return Hash_FNV64_Incremental(
        (*self_0.widget).hash,
        &mut (*self_0.widget).index as *mut uint32 as *const libc::c_void,
        ::core::mem::size_of::<uint32>() as usize as libc::c_int,
    );
}
#[inline]
unsafe extern "C" fn HashNext() -> uint64 {
    (*self_0.widget).index = ((*self_0.widget).index).wrapping_add(1);
    return HashGet();
}
#[inline]
unsafe extern "C" fn HashPeekNext() -> uint64 {
    let mut index: uint32 = ((*self_0.widget).index)
        .wrapping_add(1 as libc::c_int as libc::c_uint);
    return Hash_FNV64_Incremental(
        (*self_0.widget).hash,
        &mut index as *mut uint32 as *const libc::c_void,
        ::core::mem::size_of::<uint32>() as usize as libc::c_int,
    );
}
#[inline]
unsafe extern "C" fn TransformPos(mut x: *mut libc::c_float, mut y: *mut libc::c_float) {
    if *x < 0.0f32 {
        *x = (*self_0.layout).upper.x + *x;
    }
    if *y < 0.0f32 {
        *y = (*self_0.layout).upper.y + *y;
    }
}
#[inline]
unsafe extern "C" fn TransformSize(
    mut sx: *mut libc::c_float,
    mut sy: *mut libc::c_float,
) {
    if *sx <= 0.0f32 {
        *sx = (*self_0.layout).upper.x - self_0.cursor.x + *sx;
    }
    if *sy <= 0.0f32 {
        *sy = (*self_0.layout).upper.y - self_0.cursor.y + *sy;
    }
}
#[inline]
unsafe extern "C" fn RectContains(
    mut pos: Vec2,
    mut size: Vec2,
    mut p: Vec2,
) -> bool {
    return pos.x <= p.x && p.x <= pos.x + size.x && pos.y <= p.y
        && p.y <= pos.y + size.y;
}
#[inline]
unsafe extern "C" fn Spacing() {
    self_0.cursor.x += (*self_0.layout).spacing.x;
    self_0.cursor.y += (*self_0.layout).spacing.y;
    (*self_0.layout).spacing.x = 0 as libc::c_int as libc::c_float;
    (*self_0.layout).spacing.y = 0 as libc::c_int as libc::c_float;
}
unsafe extern "C" fn ImGui_PushLayout(
    mut sx: libc::c_float,
    mut sy: libc::c_float,
    mut horizontal: bool,
) {
    TransformSize(&mut sx, &mut sy);
    let mut layout: *mut ImGuiLayout = MemPool_Alloc(self_0.layoutPool)
        as *mut ImGuiLayout;
    (*layout).prev = self_0.layout;
    (*layout).lower = self_0.cursor;
    (*layout).upper = Vec2::new(self_0.cursor.x + sx, self_0.cursor.y + sy);
    (*layout).size = Vec2::new(sx, sy);
    (*layout).styleVars = 0 as libc::c_int;
    (*layout).horizontal = horizontal;
    self_0.layout = layout;
}
unsafe extern "C" fn ImGui_PopLayout() {
    let mut layout: *mut ImGuiLayout = self_0.layout;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < (*layout).styleVars {
        ImGui_PopStyle();
        i += 1;
    }
    self_0.layout = (*layout).prev;
    MemPool_Dealloc(self_0.layoutPool, layout as *mut libc::c_void);
}
#[inline]
unsafe extern "C" fn ImGui_Pad(mut mx: libc::c_float, mut my: libc::c_float) {
    let mut px: libc::c_float = mx * (*self_0.style).padding.x;
    let mut py: libc::c_float = my * (*self_0.style).padding.y;
    self_0.cursor.x += px;
    self_0.cursor.y += py;
    (*self_0.layout).lower.x += px;
    (*self_0.layout).lower.y += py;
    (*self_0.layout).upper.x -= px;
    (*self_0.layout).upper.y -= py;
    (*self_0.layout).size.x -= 2.0f32 * px;
    (*self_0.layout).size.y -= 2.0f32 * py;
}
unsafe extern "C" fn ImGui_Unpad(mut mx: libc::c_float, mut my: libc::c_float) {
    let mut px: libc::c_float = mx * (*self_0.style).padding.x;
    let mut py: libc::c_float = my * (*self_0.style).padding.y;
    (*self_0.layout).lower.x -= px;
    (*self_0.layout).lower.y -= py;
    (*self_0.layout).upper.x += px;
    (*self_0.layout).upper.y += py;
    (*self_0.layout).size.x += 2.0f32 * px;
    (*self_0.layout).size.y += 2.0f32 * py;
    self_0.cursor.x -= px;
    self_0.cursor.y -= py;
}
unsafe extern "C" fn ImGui_BeginWidget(mut sx: libc::c_float, mut sy: libc::c_float) {
    Spacing();
    TransformSize(&mut sx, &mut sy);
    let mut widget: *mut ImGuiWidget = MemPool_Alloc(self_0.widgetPool)
        as *mut ImGuiWidget;
    (*widget).prev = self_0.widget;
    (*widget).index = 0 as libc::c_int as uint32;
    (*widget).pos = Vec2::new(self_0.cursor.x, self_0.cursor.y);
    (*widget).size = Vec2::new(sx, sy);
    if !(self_0.widget).is_null() {
        (*self_0.widget).index = ((*self_0.widget).index).wrapping_add(1);
        (*widget)
            .hash = Hash_FNV64_Incremental(
            (*self_0.widget).hash,
            &mut (*self_0.widget).index as *mut uint32 as *const libc::c_void,
            ::core::mem::size_of::<uint32>() as usize as libc::c_int,
        );
    } else {
        (*widget).hash = Hash_FNV64_Init();
    }
    self_0.widget = widget;
}
unsafe extern "C" fn ImGui_EndWidget() {
    if !(self_0.widgetLast).is_null() {
        MemPool_Dealloc(self_0.widgetPool, self_0.widgetLast as *mut libc::c_void);
    }
    self_0.cursor = (*self_0.widget).pos;
    self_0.widgetLast = self_0.widget;
    self_0.widget = (*self_0.widget).prev;
    Advance((*self_0.widgetLast).size);
}
unsafe extern "C" fn ImGui_Focus(
    mut widget: *mut ImGuiWidget,
    mut focusType: libc::c_int,
) -> bool {
    if self_0.focus[focusType as usize] == 0 as libc::c_ulonglong {
        if !IsClipped(self_0.mouse)
            && RectContains((*widget).pos, (*widget).size, self_0.mouse) as libc::c_int
                != 0
        {
            self_0.focus[focusType as usize] = (*widget).hash;
        }
    }
    return self_0.focus[focusType as usize] == (*widget).hash;
}
#[inline]
unsafe extern "C" fn ImGui_FocusCurrent(mut focusType: libc::c_int) -> bool {
    return ImGui_Focus(self_0.widget, focusType);
}
#[inline]
unsafe extern "C" fn ImGui_FocusLast(mut focusType: libc::c_int) -> bool {
    return ImGui_Focus(self_0.widgetLast, focusType);
}
#[inline]
unsafe extern "C" fn TryFocusRect(
    mut hash: uint64,
    mut focusType: libc::c_int,
    mut pos: Vec2,
    mut size: Vec2,
) -> bool {
    if self_0.focus[focusType as usize] == 0 as libc::c_ulonglong {
        if !IsClipped(self_0.mouse)
            && RectContains(pos, size, self_0.mouse) as libc::c_int != 0
        {
            self_0.focus[focusType as usize] = hash;
        }
    }
    return self_0.focus[focusType as usize] == hash;
}
unsafe extern "C" fn ImGuiLayer_Free(mut self_1: *mut ImGuiLayer) {
    let mut child: *mut ImGuiLayer = (*self_1).children;
    while !child.is_null() {
        let mut next: *mut ImGuiLayer = (*child).next;
        ImGuiLayer_Free(child);
        child = next;
    }
    let mut e: *mut ImGuiText = (*self_1).textList;
    while !e.is_null() {
        StrFree((*e).text);
        e = (*e).next;
    }
    MemFree(self_1 as *const libc::c_void);
}
unsafe extern "C" fn ImGui_PushLayer(mut clip: bool) -> *mut ImGuiLayer {
    let mut layer: *mut ImGuiLayer = MemAlloc(
        ::core::mem::size_of::<ImGuiLayer>() as usize,
    ) as *mut ImGuiLayer;
    (*layer).parent = self_0.layer;
    (*layer).children = 0 as *mut ImGuiLayer;
    (*layer).next = 0 as *mut ImGuiLayer;
    (*layer).pos = (*self_0.layout).lower;
    (*layer).size = (*self_0.layout).size;
    (*layer).index = 0 as libc::c_int as uint32;
    (*layer).clip = clip;
    (*layer).tex2DList = 0 as *mut ImGuiTex2D;
    (*layer).panelList = 0 as *mut ImGuiPanel;
    (*layer).rectList = 0 as *mut ImGuiRect;
    (*layer).textList = 0 as *mut ImGuiText;
    (*layer).lineList = 0 as *mut ImGuiLine;
    if !(self_0.layer).is_null() {
        (*layer).next = (*self_0.layer).children;
        (*self_0.layer).children = layer;
        (*layer).hash = HashNext();
    } else {
        (*layer).hash = Hash_FNV64_Init();
    }
    self_0.layer = layer;
    if clip {
        ImGui_PushClipRect((*self_0.layer).pos, (*self_0.layer).size);
    }
    return layer;
}
unsafe extern "C" fn ImGui_PopLayer() {
    if (*self_0.layer).clip {
        ImGui_PopClipRect();
    }
    self_0.layerLast = self_0.layer;
    self_0.layer = (*self_0.layer).parent;
}
unsafe extern "C" fn ImGui_DrawLayer(mut self_1: *const ImGuiLayer) {
    if (*self_1).clip {
        ClipRect_PushCombined(
            (*self_1).pos.x,
            (*self_1).pos.y,
            (*self_1).size.x,
            (*self_1).size.y,
        );
    }
    let mut e: *const ImGuiTex2D = (*self_1).tex2DList;
    while !e.is_null() {
        Draw_Color(
            1 as libc::c_int as libc::c_float,
            1 as libc::c_int as libc::c_float,
            1 as libc::c_int as libc::c_float,
            1 as libc::c_int as libc::c_float,
        );
        Tex2D_Draw((*e).tex, (*e).pos.x, (*e).pos.y, (*e).size.x, (*e).size.y);
        e = (*e).next;
    }
    if !((*self_1).panelList).is_null() {
        static mut shader: *mut Shader = 0 as *const Shader as *mut Shader;
        if shader.is_null() {
            shader = Shader_Load(
                b"vertex/ui\0" as *const u8 as *const libc::c_char,
                b"fragment/ui/panel\0" as *const u8 as *const libc::c_char,
            );
        }
        let pad: libc::c_float = 64.0f32;
        Shader_Start(shader);
        Shader_SetFloat(b"padding\0" as *const u8 as *const libc::c_char, pad);
        let mut e_0: *const ImGuiPanel = (*self_1).panelList;
        while !e_0.is_null() {
            let mut x: libc::c_float = (*e_0).pos.x - pad;
            let mut y: libc::c_float = (*e_0).pos.y - pad;
            let mut sx: libc::c_float = (*e_0).size.x + 2.0f32 * pad;
            let mut sy: libc::c_float = (*e_0).size.y + 2.0f32 * pad;
            Shader_SetFloat(
                b"innerAlpha\0" as *const u8 as *const libc::c_char,
                (*e_0).innerAlpha,
            );
            Shader_SetFloat(
                b"bevel\0" as *const u8 as *const libc::c_char,
                (*e_0).bevel,
            );
            Shader_SetFloat2(b"size\0" as *const u8 as *const libc::c_char, sx, sy);
            Shader_SetFloat4(
                b"color\0" as *const u8 as *const libc::c_char,
                (*e_0).color.x,
                (*e_0).color.y,
                (*e_0).color.z,
                (*e_0).color.w,
            );
            Draw_Rect(x, y, sx, sy);
            e_0 = (*e_0).next;
        }
        Shader_Stop(shader);
    }
    let mut e_1: *const ImGuiRect = (*self_1).rectList;
    while !e_1.is_null() {
        Draw_Color((*e_1).color.x, (*e_1).color.y, (*e_1).color.z, (*e_1).color.w);
        if (*e_1).outline {
            Draw_Border(
                1.0f32,
                (*e_1).pos.x,
                (*e_1).pos.y,
                (*e_1).size.x,
                (*e_1).size.y,
            );
        } else {
            Draw_Rect((*e_1).pos.x, (*e_1).pos.y, (*e_1).size.x, (*e_1).size.y);
        }
        e_1 = (*e_1).next;
    }
    if !((*self_1).lineList).is_null() {
        RenderState_PushBlendMode(0 as libc::c_int);
        static mut shader_0: *mut Shader = 0 as *const Shader as *mut Shader;
        if shader_0.is_null() {
            shader_0 = Shader_Load(
                b"vertex/ui\0" as *const u8 as *const libc::c_char,
                b"fragment/ui/line\0" as *const u8 as *const libc::c_char,
            );
        }
        let pad_0: libc::c_float = 64.0f32;
        Shader_Start(shader_0);
        let mut e_2: *const ImGuiLine = (*self_1).lineList;
        while !e_2.is_null() {
            let mut xMin: libc::c_float = (Min(
                (*e_2).p1.x as libc::c_double,
                (*e_2).p2.x as libc::c_double,
            ) - pad_0 as libc::c_double) as libc::c_float;
            let mut yMin: libc::c_float = (Min(
                (*e_2).p1.y as libc::c_double,
                (*e_2).p2.y as libc::c_double,
            ) - pad_0 as libc::c_double) as libc::c_float;
            let mut xMax: libc::c_float = (Max(
                (*e_2).p1.x as libc::c_double,
                (*e_2).p2.x as libc::c_double,
            ) + pad_0 as libc::c_double) as libc::c_float;
            let mut yMax: libc::c_float = (Max(
                (*e_2).p1.y as libc::c_double,
                (*e_2).p2.y as libc::c_double,
            ) + pad_0 as libc::c_double) as libc::c_float;
            let mut sx_0: libc::c_float = xMax - xMin;
            let mut sy_0: libc::c_float = yMax - yMin;
            Shader_SetFloat2(
                b"origin\0" as *const u8 as *const libc::c_char,
                xMin,
                yMin,
            );
            Shader_SetFloat2(b"size\0" as *const u8 as *const libc::c_char, sx_0, sy_0);
            Shader_SetFloat2(
                b"p1\0" as *const u8 as *const libc::c_char,
                (*e_2).p1.x,
                (*e_2).p1.y,
            );
            Shader_SetFloat2(
                b"p2\0" as *const u8 as *const libc::c_char,
                (*e_2).p2.x,
                (*e_2).p2.y,
            );
            Shader_SetFloat4(
                b"color\0" as *const u8 as *const libc::c_char,
                (*e_2).color.x,
                (*e_2).color.y,
                (*e_2).color.z,
                (*e_2).color.w,
            );
            Draw_Rect(xMin, yMin, sx_0, sy_0);
            e_2 = (*e_2).next;
        }
        Shader_Stop(shader_0);
        RenderState_PopBlendMode();
    }
    let mut e_3: *const ImGuiText = (*self_1).textList;
    while !e_3.is_null() {
        Font_Draw(
            (*e_3).font,
            (*e_3).text,
            (*e_3).pos.x,
            (*e_3).pos.y,
            (*e_3).color.x,
            (*e_3).color.y,
            (*e_3).color.z,
            (*e_3).color.w,
        );
        e_3 = (*e_3).next;
    }
    let mut e_4: *const ImGuiLayer = (*self_1).children;
    while !e_4.is_null() {
        ImGui_DrawLayer(e_4);
        e_4 = (*e_4).next;
    }
    if (*self_1).clip {
        ClipRect_Pop();
    }
}
static mut init_imgui: bool = 0 as libc::c_int != 0;
unsafe extern "C" fn ImGui_Init() {
    if init_imgui {
        return;
    }
    init_imgui = 1 as libc::c_int != 0;
    self_0.layer = 0 as *mut ImGuiLayer;
    self_0.layerLast = 0 as *mut ImGuiLayer;
    self_0.style = 0 as *mut ImGuiStyle;
    self_0.clipRect = 0 as *mut ImGuiClipRect;
    self_0.cursorStack = 0 as *mut ImGuiCursor;
    self_0.dragging = 0 as libc::c_int as uint64;
    self_0
        .data = HashMap_Create(0 as libc::c_int as uint32, 128 as libc::c_int as uint32);
    self_0
        .layoutPool = MemPool_CreateAuto(
        ::core::mem::size_of::<ImGuiLayout>() as usize as uint32,
    );
    self_0
        .widgetPool = MemPool_CreateAuto(
        ::core::mem::size_of::<ImGuiWidget>() as usize as uint32,
    );
    self_0
        .stylePool = MemPool_CreateAuto(
        ::core::mem::size_of::<ImGuiStyle>() as usize as uint32,
    );
    self_0
        .clipRectPool = MemPool_CreateAuto(
        ::core::mem::size_of::<ImGuiClipRect>() as usize as uint32,
    );
    self_0
        .cursorPool = MemPool_CreateAuto(
        ::core::mem::size_of::<ImGuiCursor>() as usize as uint32,
    );
    self_0
        .tex2DPool = MemPool_CreateAuto(
        ::core::mem::size_of::<ImGuiTex2D>() as usize as uint32,
    );
    self_0
        .panelPool = MemPool_CreateAuto(
        ::core::mem::size_of::<ImGuiPanel>() as usize as uint32,
    );
    self_0
        .rectPool = MemPool_CreateAuto(
        ::core::mem::size_of::<ImGuiRect>() as usize as uint32,
    );
    self_0
        .textPool = MemPool_CreateAuto(
        ::core::mem::size_of::<ImGuiText>() as usize as uint32,
    );
    self_0
        .linePool = MemPool_CreateAuto(
        ::core::mem::size_of::<ImGuiLine>() as usize as uint32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_Begin(mut sx: libc::c_float, mut sy: libc::c_float) {
    ImGui_Init();
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < FocusType_SIZE {
        self_0.focus[i as usize] = 0 as libc::c_int as uint64;
        i += 1;
    }
    if !Input_GetDown(Button_Mouse_Left) {
        self_0.dragging = 0 as libc::c_int as uint64;
    }
    if self_0.dragging != 0 {
        self_0.focus[FocusType_Mouse as usize] = self_0.dragging;
    }
    self_0
        .cursor = Vec2::new(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    if !(self_0.layerLast).is_null() {
        ImGuiLayer_Free(self_0.layerLast);
        self_0.layerLast = 0 as *mut ImGuiLayer;
    }
    MemPool_Clear(self_0.layoutPool);
    MemPool_Clear(self_0.widgetPool);
    MemPool_Clear(self_0.stylePool);
    MemPool_Clear(self_0.clipRectPool);
    MemPool_Clear(self_0.cursorPool);
    MemPool_Clear(self_0.tex2DPool);
    MemPool_Clear(self_0.panelPool);
    MemPool_Clear(self_0.rectPool);
    MemPool_Clear(self_0.textPool);
    MemPool_Clear(self_0.linePool);
    self_0.style = 0 as *mut ImGuiStyle;
    ImGui_PushDefaultStyle();
    self_0.layout = 0 as *mut ImGuiLayout;
    ImGui_PushLayout(sx, sy, 0 as libc::c_int != 0);
    self_0.widget = 0 as *mut ImGuiWidget;
    self_0.widgetLast = 0 as *mut ImGuiWidget;
    ImGui_BeginWidget(sx, sy);
    self_0.layer = 0 as *mut ImGuiLayer;
    ImGui_PushLayer(1 as libc::c_int != 0);
    let mut mouse: IVec2 = IVec2 { x: 0, y: 0 };
    Input_GetMousePosition(&mut mouse);
    self_0.mouse.x = mouse.x as libc::c_float;
    self_0.mouse.y = mouse.y as libc::c_float;
    self_0.activate = Input_GetPressed(Button_Mouse_Left);
    self_0
        .forceSize = Vec2::new(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_End() {
    ImGui_PopLayer();
    ImGui_EndWidget();
    ImGui_PopLayout();
    if !(self_0.layer).is_null() {
        Fatal(b"ImGui_End: layer stack not empty\0" as *const u8 as *const libc::c_char);
    }
    if !(self_0.widget).is_null() {
        Fatal(
            b"ImGui_End: widget stack not empty\0" as *const u8 as *const libc::c_char,
        );
    }
    if !(self_0.layout).is_null() {
        Fatal(
            b"ImGui_End: layout stack not empty\0" as *const u8 as *const libc::c_char,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_Draw() {
    RenderState_PushBlendMode(1 as libc::c_int);
    Draw_LineWidth(1 as libc::c_int as libc::c_float);
    ImGui_DrawLayer(self_0.layerLast);
    RenderState_PopBlendMode();
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_AlignCursor(
    mut sx: libc::c_float,
    mut sy: libc::c_float,
    mut alignX: libc::c_float,
    mut alignY: libc::c_float,
) {
    TransformSize(&mut sx, &mut sy);
    ImGui_SetCursor(
        (*self_0.layout).lower.x + alignX * ((*self_0.layout).size.x - sx),
        (*self_0.layout).lower.y + alignY * ((*self_0.layout).size.y - sy),
    );
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_GetCursorX() -> libc::c_float {
    return self_0.cursor.x;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_GetCursorY() -> libc::c_float {
    return self_0.cursor.y;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_PushCursor() {
    let mut cursor: *mut ImGuiCursor = MemPool_Alloc(self_0.cursorPool)
        as *mut ImGuiCursor;
    (*cursor).prev = self_0.cursorStack;
    (*cursor).pos = self_0.cursor;
    self_0.cursorStack = cursor;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_PopCursor() {
    let mut cursor: *mut ImGuiCursor = self_0.cursorStack;
    self_0.cursor = (*cursor).pos;
    self_0.cursorStack = (*cursor).prev;
    MemPool_Dealloc(self_0.cursorPool, cursor as *mut libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_SetCursor(mut cx: libc::c_float, mut cy: libc::c_float) {
    TransformPos(&mut cx, &mut cy);
    self_0.cursor = Vec2::new(cx, cy);
    (*self_0.layout)
        .spacing = Vec2::new(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_SetCursorX(mut x: libc::c_float) {
    ImGui_SetCursor(x, self_0.cursor.y);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_SetCursorY(mut y: libc::c_float) {
    ImGui_SetCursor(self_0.cursor.x, y);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_Indent() {
    self_0.cursor.x += 2.0f32 * (*self_0.style).padding.x;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_Undent() {
    self_0.cursor.x -= 2.0f32 * (*self_0.style).padding.x;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_BeginGroup(
    mut sx: libc::c_float,
    mut sy: libc::c_float,
    mut horizontal: bool,
) {
    ImGui_BeginWidget(sx, sy);
    ImGui_PushLayout(sx, sy, horizontal);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_BeginGroupX(mut sy: libc::c_float) {
    ImGui_BeginWidget(0 as libc::c_int as libc::c_float, sy);
    ImGui_PushLayout(0 as libc::c_int as libc::c_float, sy, 1 as libc::c_int != 0);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_BeginGroupY(mut sx: libc::c_float) {
    ImGui_BeginWidget(sx, 0 as libc::c_int as libc::c_float);
    ImGui_PushLayout(sx, 0 as libc::c_int as libc::c_float, 0 as libc::c_int != 0);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_EndGroup() {
    ImGui_PopLayout();
    ImGui_EndWidget();
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_BeginPanel(mut sx: libc::c_float, mut sy: libc::c_float) {
    ImGui_BeginGroup(sx, sy, 0 as libc::c_int != 0);
    ImGui_PushLayer(0 as libc::c_int != 0);
    ImGui_PushLayer(1 as libc::c_int != 0);
    ImGui_Pad(1.0f32, 1.0f32);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_EndPanel() {
    ImGui_Unpad(1.0f32, 1.0f32);
    ImGui_PopLayer();
    EmitPanel(
        (*self_0.style).frameColor,
        (*self_0.widget).pos,
        (*self_0.widget).size,
        1.0f32,
        12.0f32,
    );
    ImGui_PopLayer();
    ImGui_EndGroup();
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_BeginWindow(
    mut title: cstr,
    mut sx: libc::c_float,
    mut sy: libc::c_float,
) {
    let mut hash: uint64 = HashPeekNext();
    let mut data: *mut ImGuiData = GetData(hash);
    self_0.cursor.x += (*data).offset.x;
    self_0.cursor.y += (*data).offset.y;
    ImGui_BeginPanel(sx, sy);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_EndWindow() {
    ImGui_EndPanel();
    let mut data: *mut ImGuiData = GetData((*self_0.widgetLast).hash);
    self_0.cursor.x -= (*data).offset.x;
    self_0.cursor.y -= (*data).offset.y;
    if ImGui_FocusLast(FocusType_Mouse) {
        if Input_GetDown(Button_Mouse_Left) {
            let mut delta: IVec2 = IVec2 { x: 0, y: 0 };
            Input_GetMouseDelta(&mut delta);
            (*data).offset.x += delta.x as libc::c_float;
            (*data).offset.y += delta.y as libc::c_float;
            self_0.dragging = (*self_0.widgetLast).hash;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_BeginScrollFrame(
    mut sx: libc::c_float,
    mut sy: libc::c_float,
) {
    ImGui_BeginGroup(sx, sy, 0 as libc::c_int != 0);
    ImGui_PushLayer(1 as libc::c_int != 0);
    ImGui_Pad(1.0f32, 1.0f32);
    let mut data: *mut ImGuiData = GetData((*self_0.widget).hash);
    self_0.cursor.y -= (*data).scroll;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_EndScrollFrame() {
    let mut data: *mut ImGuiData = GetData((*self_0.widget).hash);
    self_0.cursor.y += (*data).scroll;
    let mut layout: *mut ImGuiLayout = self_0.layout;
    ImGui_PopLayer();
    let mut scroll: libc::c_float = (*data).scroll;
    let mut virtualSize: libc::c_float = self_0.cursor.y - (*layout).lower.y;
    let mut scrollMax: libc::c_float = virtualSize - (*layout).size.y;
    let mut scrollPos = Vec2::new(
        (*layout).lower.x + (*layout).size.x,
        (*layout).lower.y,
    );
    let mut scrollSize = Vec2::new(
        (*self_0.style).scrollBarSize.x,
        (*layout).size.y,
    );
    let mut handleHash: uint64 = HashNext();
    if (*layout).size.y < virtualSize {
        let mut handleSizeY: libc::c_float = (*layout).size.y
            * ((*layout).size.y / virtualSize);
        handleSizeY = Clamp(
            handleSizeY as libc::c_double,
            16.0f32 as libc::c_double,
            128.0f32 as libc::c_double,
        ) as libc::c_float;
        let mut handleOffset: libc::c_float = ((*layout).size.y - handleSizeY)
            * (scroll / scrollMax);
        let mut handlePos = Vec2::new(scrollPos.x, scrollPos.y + handleOffset);
        let mut handleSize = Vec2::new(
            (*self_0.style).scrollBarSize.x,
            handleSizeY,
        );
        let mut handleFocus: bool = TryFocusRect(
            handleHash,
            FocusType_Mouse,
            handlePos,
            handleSize,
        );
        EmitPanel(
            if handleFocus as libc::c_int != 0 {
                (*self_0.style).buttonColorFocus
            } else {
                Vec4f_Create(0.3f32, 0.4f32, 0.5f32, 1.0f32)
            },
            handlePos,
            handleSize,
            if handleFocus as libc::c_int != 0 { 0.5f32 } else { 0.25f32 },
            4.0f32,
        );
    }
    ImGui_Unpad(1.0f32, 1.0f32);
    ImGui_EndGroup();
    EmitPanel(
        Vec4f_Create(0.0f32, 0.0f32, 0.0f32, 0.5f32),
        (*self_0.widgetLast).pos,
        (*self_0.widgetLast).size,
        0.25f32,
        4.0f32,
    );
    if ImGui_FocusLast(FocusType_Scroll) {
        let mut scroll_0: IVec2 = IVec2 { x: 0, y: 0 };
        Input_GetMouseScroll(&mut scroll_0);
        (*data).scroll -= 10.0f32 * scroll_0.y as libc::c_float;
    }
    (*data)
        .scroll = Clamp(
        (*data).scroll as libc::c_double,
        0.0f32 as libc::c_double,
        scrollMax as libc::c_double,
    ) as libc::c_float;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_SetNextWidth(mut sx: libc::c_float) {
    self_0.forceSize.x = sx;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_SetNextHeight(mut sy: libc::c_float) {
    self_0.forceSize.y = sy;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_PushStyle() {
    let mut style: *mut ImGuiStyle = MemPool_Alloc(self_0.stylePool) as *mut ImGuiStyle;
    MemCpy(
        style as *mut libc::c_void,
        self_0.style as *const libc::c_void,
        ::core::mem::size_of::<ImGuiStyle>() as usize,
    );
    (*style).prev = self_0.style;
    self_0.style = style;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_PushStyleFont(mut font: *mut Font) {
    ImGui_PushStyle();
    (*self_0.style).font = font;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_PushStylePadding(
    mut px: libc::c_float,
    mut py: libc::c_float,
) {
    ImGui_PushStyle();
    (*self_0.style).padding = Vec2::new(px, py);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_PushStyleSpacing(
    mut x: libc::c_float,
    mut y: libc::c_float,
) {
    ImGui_PushStyle();
    (*self_0.style).spacing = Vec2::new(x, y);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_PushStyleTextColor(
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
) {
    ImGui_PushStyle();
    (*self_0.style).textColor = Vec4f_Create(r, g, b, a);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_PopStyle() {
    if ((*self_0.style).prev).is_null() {
        Fatal(
            b"ImGui_PopStyle: Attempting to pop an empty stack\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut style: *mut ImGuiStyle = self_0.style;
    self_0.style = (*style).prev;
    MemPool_Dealloc(self_0.stylePool, style as *mut libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_SetFont(mut font: *mut Font) {
    ImGui_PushStyleFont(font);
    (*self_0.layout).styleVars += 1;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_SetSpacing(mut sx: libc::c_float, mut sy: libc::c_float) {
    ImGui_PushStyleSpacing(sx, sy);
    (*self_0.layout).styleVars += 1;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_Button(mut label: cstr) -> bool {
    return ImGui_ButtonEx(
        label,
        0 as libc::c_int as libc::c_float,
        32 as libc::c_int as libc::c_float,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_ButtonEx(
    mut label: cstr,
    mut sx: libc::c_float,
    mut sy: libc::c_float,
) -> bool {
    ImGui_BeginWidget(sx, sy);
    let mut focus: bool = ImGui_FocusCurrent(FocusType_Mouse);
    let mut color: Vec4f = if focus as libc::c_int != 0 {
        (*self_0.style).buttonColorFocus
    } else {
        (*self_0.style).buttonColor
    };
    EmitPanel(
        color,
        (*self_0.widget).pos,
        (*self_0.widget).size,
        if focus as libc::c_int != 0 { 1.0f32 } else { 0.5f32 },
        4.0f32,
    );
    let mut bound: IVec2 = IVec2 { x: 0, y: 0 };
    Font_GetSize2((*self_0.style).font, &mut bound, label);
    let mut labelPos = Vec2::new(
        (*self_0.widget).pos.x
            + 0.5f32 * ((*self_0.widget).size.x - bound.x as libc::c_float),
        (*self_0.widget).pos.y
            + 0.5f32 * ((*self_0.widget).size.y - bound.y as libc::c_float),
    );
    let mut labelSize = Vec2::new(
        bound.x as libc::c_float,
        bound.y as libc::c_float,
    );
    EmitText(
        (*self_0.style).font,
        if focus as libc::c_int != 0 {
            (*self_0.style).textColorFocus
        } else {
            (*self_0.style).textColor
        },
        Vec2::new(labelPos.x, labelPos.y + bound.y as libc::c_float),
        label,
    );
    ImGui_EndWidget();
    return focus as libc::c_int != 0 && self_0.activate as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_Checkbox(mut value: bool) -> bool {
    ImGui_BeginWidget(16.0f32, 16.0f32);
    let mut focus: bool = ImGui_FocusCurrent(FocusType_Mouse);
    if focus as libc::c_int != 0 && self_0.activate as libc::c_int != 0 {
        value = !value;
    }
    if focus {
        EmitRect(
            (*self_0.style).buttonColorFocus,
            (*self_0.widget).pos,
            (*self_0.widget).size,
            1 as libc::c_int != 0,
        );
    }
    EmitPanel(
        if value as libc::c_int != 0 {
            (*self_0.style).buttonColorFocus
        } else {
            (*self_0.style).buttonColor
        },
        Vec2::new(
            (*self_0.widget).pos.x + 2 as libc::c_int as libc::c_float,
            (*self_0.widget).pos.y + 2 as libc::c_int as libc::c_float,
        ),
        Vec2::new(
            (*self_0.widget).size.x - 4 as libc::c_int as libc::c_float,
            (*self_0.widget).size.y - 4 as libc::c_int as libc::c_float,
        ),
        1.0f32,
        4.0f32,
    );
    ImGui_EndWidget();
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_Divider() {
    ImGui_BeginWidget(
        (if (*self_0.layout).horizontal as libc::c_int != 0 {
            2 as libc::c_int
        } else {
            0 as libc::c_int
        }) as libc::c_float,
        (if (*self_0.layout).horizontal as libc::c_int != 0 {
            0 as libc::c_int
        } else {
            2 as libc::c_int
        }) as libc::c_float,
    );
    EmitLine(
        (*self_0.style).buttonColorFocus,
        (*self_0.widget).pos,
        Vec2::new(
            (*self_0.widget).pos.x
                + (if (*self_0.layout).horizontal as libc::c_int != 0 {
                    0 as libc::c_int as libc::c_float
                } else {
                    (*self_0.widget).size.x
                }),
            (*self_0.widget).pos.y
                + (if (*self_0.layout).horizontal as libc::c_int != 0 {
                    (*self_0.widget).size.y
                } else {
                    0 as libc::c_int as libc::c_float
                }),
        ),
    );
    ImGui_EndWidget();
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_Selectable(mut label: cstr) -> bool {
    let mut bound: IVec2 = IVec2 { x: 0, y: 0 };
    Font_GetSize2((*self_0.style).font, &mut bound, label);
    ImGui_BeginWidget(
        if (*self_0.layout).horizontal as libc::c_int != 0 {
            bound.x as libc::c_float + 4.0f32
        } else {
            0 as libc::c_int as libc::c_float
        },
        if (*self_0.layout).horizontal as libc::c_int != 0 {
            0 as libc::c_int as libc::c_float
        } else {
            4.0f32 + Font_GetLineHeight((*self_0.style).font) as libc::c_float
        },
    );
    let mut focus: bool = ImGui_FocusCurrent(FocusType_Mouse);
    if focus {
        EmitRect(
            (*self_0.style).buttonColorFocus,
            (*self_0.widget).pos,
            (*self_0.widget).size,
            0 as libc::c_int != 0,
        );
    }
    EmitText(
        (*self_0.style).font,
        if focus as libc::c_int != 0 {
            (*self_0.style).textColorFocus
        } else {
            (*self_0.style).textColor
        },
        Vec2::new(
            (*self_0.widget).pos.x + 2.0f32,
            (*self_0.widget).pos.y + bound.y as libc::c_float
                + 0.5f32 * ((*self_0.widget).size.y - bound.y as libc::c_float),
        ),
        label,
    );
    ImGui_EndWidget();
    return focus as libc::c_int != 0 && self_0.activate as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_Tex2D(mut tex: *mut Tex2D) {
    let mut size: IVec2 = IVec2 { x: 0, y: 0 };
    Tex2D_GetSize(tex, &mut size);
    let mut sizef = Vec2::new(
        size.x as libc::c_float,
        size.y as libc::c_float,
    );
    ImGui_BeginWidget(size.x as libc::c_float, size.y as libc::c_float);
    EmitTex2D(tex, self_0.cursor, sizef);
    ImGui_EndWidget();
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_Text(mut text: cstr) {
    ImGui_TextEx(
        (*self_0.style).font,
        text,
        (*self_0.style).textColor.x,
        (*self_0.style).textColor.y,
        (*self_0.style).textColor.z,
        (*self_0.style).textColor.w,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_TextColored(
    mut text: cstr,
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
) {
    ImGui_TextEx((*self_0.style).font, text, r, g, b, a);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_TextEx(
    mut font: *mut Font,
    mut text: cstr,
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
) {
    let mut bound: IVec2 = IVec2 { x: 0, y: 0 };
    Font_GetSize2((*self_0.style).font, &mut bound, text);
    ImGui_BeginWidget(
        bound.x as libc::c_float,
        (if (*self_0.layout).horizontal as libc::c_int != 0 {
            0 as libc::c_int
        } else {
            Font_GetLineHeight((*self_0.style).font)
        }) as libc::c_float,
    );
    EmitText(
        font,
        Vec4f_Create(r, g, b, a),
        Vec2::new(
            (*self_0.widget).pos.x,
            (*self_0.widget).pos.y + bound.y as libc::c_float
                + 0.5f32 * ((*self_0.widget).size.y - bound.y as libc::c_float),
        ),
        text,
    );
    ImGui_EndWidget();
}
