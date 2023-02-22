use crate::internal::Memory::*;
use crate::Button::*;
use glam::Vec3;
use glam::{IVec2, Vec2};
use libc;

extern "C" {
    pub type Font;
    pub type HashMap;
    pub type MemPool;
    pub type Shader;
    pub type Tex2D;
    fn Fatal(_: *const libc::c_char, _: ...);
    fn ClipRect_PushCombined(x: f32, y: f32, sx: f32, sy: f32);
    fn ClipRect_Pop();
    fn Draw_Rect(x: f32, y: f32, sx: f32, sy: f32);
    fn Draw_Border(s: f32, x: f32, y: f32, w: f32, h: f32);
    fn Draw_LineWidth(width: f32);
    fn Draw_Color(r: f32, g: f32, b: f32, a: f32);
    fn Font_Load(name: *const libc::c_char, size: i32) -> *mut Font;
    fn Font_Draw(_: *mut Font, text: *const libc::c_char, x: f32, y: f32, r: f32, g: f32, b: f32, a: f32);
    fn Font_GetLineHeight(_: *mut Font) -> i32;
    fn Font_GetSize2(_: *mut Font, out: *mut IVec2, text: *const libc::c_char);
    fn Hash_FNV64_Init() -> u64;
    fn Hash_FNV64_Incremental(_: u64, buf: *const libc::c_void, len: i32) -> u64;
    fn HashMap_Create(keySize: u32, capacity: u32) -> *mut HashMap;
    fn HashMap_GetRaw(_: *mut HashMap, keyHash: u64) -> *mut libc::c_void;
    fn HashMap_SetRaw(_: *mut HashMap, keyHash: u64, value: *mut libc::c_void);
    fn Input_GetPressed(_: Button) -> bool;
    fn Input_GetDown(_: Button) -> bool;
    fn Input_GetMouseDelta(_: *mut IVec2);
    fn Input_GetMousePosition(_: *mut IVec2);
    fn Input_GetMouseScroll(_: *mut IVec2);
    fn MemPool_CreateAuto(elemSize: u32) -> *mut MemPool;
    fn MemPool_Alloc(_: *mut MemPool) -> *mut libc::c_void;
    fn MemPool_Clear(_: *mut MemPool);
    fn MemPool_Dealloc(_: *mut MemPool, _: *mut libc::c_void);
    fn RenderState_PushBlendMode(_: BlendMode);
    fn RenderState_PopBlendMode();
    fn Shader_Load(vertName: *const libc::c_char, fragName: *const libc::c_char) -> *mut Shader;
    fn Shader_Start(_: *mut Shader);
    fn Shader_Stop(_: *mut Shader);
    fn Shader_SetFloat(_: *const libc::c_char, _: f32);
    fn Shader_SetFloat2(_: *const libc::c_char, _: f32, _: f32);
    fn Shader_SetFloat4(_: *const libc::c_char, _: f32, _: f32, _: f32, _: f32);
    fn Tex2D_Draw(_: *mut Tex2D, x: f32, y: f32, sx: f32, sy: f32);
    fn Tex2D_GetSize(_: *mut Tex2D, out: *mut IVec2);
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec4f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
pub type BlendMode = i32;
pub type Button = i32;
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
    pub focus: [u64; 3],
    pub dragging: u64,
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
    pub hash: u64,
    pub index: u32,
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
    pub styleVars: i32,
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
    pub hash: u64,
    pub index: u32,
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
    pub text: *const libc::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ImGuiPanel {
    pub next: *mut ImGuiPanel,
    pub color: Vec4f,
    pub pos: Vec2,
    pub size: Vec2,
    pub innerAlpha: f32,
    pub bevel: f32,
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
    pub scroll: f32,
}

#[inline]
unsafe extern "C" fn Clamp(mut t: f64, mut lower: f64, mut upper: f64) -> f64 {
    t = if t > upper { upper } else { t };
    t = if t < lower { lower } else { t };
    return t;
}
#[inline]
unsafe extern "C" fn Max(mut a: f64, mut b: f64) -> f64 {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Min(mut a: f64, mut b: f64) -> f64 {
    return if a < b { a } else { b };
}
#[inline]
unsafe extern "C" fn Vec4f_Create(mut x: f32, mut y: f32, mut z: f32, mut w: f32) -> Vec4f {
    let mut self_1: Vec4f = Vec4f {
        x: x,
        y: y,
        z: z,
        w: w,
    };
    return self_1;
}

#[no_mangle]
pub static mut FocusType_Mouse: i32 = 0 as i32;
#[no_mangle]
pub static mut FocusType_Keyboard: i32 = 1 as i32;
#[no_mangle]
pub static mut FocusType_Scroll: i32 = 2 as i32;
#[no_mangle]
pub static mut FocusType_SIZE: i32 = 3 as i32;
static mut this: ImGui = ImGui {
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
    let mut e: *mut ImGuiLine = MemPool_Alloc(this.linePool) as *mut ImGuiLine;
    (*e).color = color;
    (*e).p1 = p1;
    (*e).p2 = p2;
    (*e).next = (*this.layer).lineList;
    (*this.layer).lineList = e;
}
#[inline]
unsafe extern "C" fn EmitPanel(
    mut color: Vec4f,
    mut pos: Vec2,
    mut size: Vec2,
    mut innerAlpha: f32,
    mut bevel: f32,
) {
    let mut e: *mut ImGuiPanel = MemPool_Alloc(this.panelPool) as *mut ImGuiPanel;
    (*e).color = color;
    (*e).pos = pos;
    (*e).size = size;
    (*e).innerAlpha = innerAlpha;
    (*e).bevel = bevel;
    (*e).next = (*this.layer).panelList;
    (*this.layer).panelList = e;
}
#[inline]
unsafe extern "C" fn EmitRect(mut color: Vec4f, mut pos: Vec2, mut size: Vec2, mut outline: bool) {
    let mut e: *mut ImGuiRect = MemPool_Alloc(this.rectPool) as *mut ImGuiRect;
    (*e).color = color;
    (*e).pos = pos;
    (*e).size = size;
    (*e).outline = outline;
    (*e).next = (*this.layer).rectList;
    (*this.layer).rectList = e;
}
#[inline]
unsafe extern "C" fn EmitTex2D(mut tex: *mut Tex2D, mut pos: Vec2, mut size: Vec2) {
    let mut e: *mut ImGuiTex2D = MemPool_Alloc(this.tex2DPool) as *mut ImGuiTex2D;
    (*e).tex = tex;
    (*e).pos = pos;
    (*e).size = size;
    (*e).next = (*this.layer).tex2DList;
    (*this.layer).tex2DList = e;
}
#[inline]
unsafe extern "C" fn EmitText(
    mut font: *mut Font,
    mut color: Vec4f,
    mut pos: Vec2,
    mut text: *const libc::c_char,
) {
    let mut e: *mut ImGuiText = MemPool_Alloc(this.textPool) as *mut ImGuiText;
    (*e).font = font;
    (*e).color = color;
    (*e).pos = pos;
    (*e).text = StrDup(text);
    (*e).next = (*this.layer).textList;
    (*this.layer).textList = e;
}
#[inline]
unsafe extern "C" fn GetData(mut hash: u64) -> *mut ImGuiData {
    let mut data: *mut ImGuiData = HashMap_GetRaw(this.data, hash) as *mut ImGuiData;
    if data.is_null() {
        data = MemAlloc(::core::mem::size_of::<ImGuiData>()) as *mut ImGuiData;
        (*data).size = Vec2::new(0.0f32, 0.0f32);
        (*data).offset = Vec2::new(0.0f32, 0.0f32);
        (*data).scroll = 0.0f32;
        HashMap_SetRaw(this.data, hash, data as *mut libc::c_void);
    }
    return data;
}
unsafe extern "C" fn ImGui_PushDefaultStyle() {
    static mut font: *mut Font = 0 as *const Font as *mut Font;
    static mut fontSubheading: *mut Font = 0 as *const Font as *mut Font;
    if font.is_null() {
        font = Font_Load(b"Share\0" as *const u8 as *const libc::c_char, 16 as i32);
        fontSubheading = Font_Load(b"Iceland\0" as *const u8 as *const libc::c_char, 18 as i32);
    }
    let mut style: *mut ImGuiStyle = MemPool_Alloc(this.stylePool) as *mut ImGuiStyle;
    (*style).prev = this.style;
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
    this.style = style;
}
unsafe extern "C" fn ImGui_PushClipRect(mut pos: Vec2, mut size: Vec2) {
    let mut rect: *mut ImGuiClipRect =
        MemAlloc(::core::mem::size_of::<ImGuiClipRect>() as usize) as *mut ImGuiClipRect;
    let mut prev: *mut ImGuiClipRect = this.clipRect;
    (*rect).prev = prev;
    (*rect).p1 = pos;
    (*rect).p2 = pos + size;
    if !prev.is_null() {
        (*rect).p1.x = Max((*rect).p1.x as f64, (*prev).p1.x as f64) as f32;
        (*rect).p1.y = Max((*rect).p1.y as f64, (*prev).p1.y as f64) as f32;
        (*rect).p2.x = Min((*rect).p2.x as f64, (*prev).p2.x as f64) as f32;
        (*rect).p2.y = Min((*rect).p2.y as f64, (*prev).p2.y as f64) as f32;
    }
    this.clipRect = rect;
}
unsafe extern "C" fn ImGui_PopClipRect() {
    let mut rect: *mut ImGuiClipRect = this.clipRect;
    this.clipRect = (*rect).prev;
    MemFree(rect as *const libc::c_void);
}
#[inline]
unsafe extern "C" fn IsClipped(mut p: Vec2) -> bool {
    if (this.clipRect).is_null() {
        return 0 as i32 != 0;
    }
    return p.x < (*this.clipRect).p1.x
        || p.y < (*this.clipRect).p1.y
        || (*this.clipRect).p2.x < p.x
        || (*this.clipRect).p2.y < p.y;
}
#[inline]
unsafe extern "C" fn Advance(mut size: Vec2) {
    if (*this.layout).horizontal {
        this.cursor.x += size.x;
        (*this.layout).spacing.x = (*this.style).spacing.x;
    } else {
        this.cursor.y += size.y;
        (*this.layout).spacing.y = (*this.style).spacing.y;
    };
}
#[inline]
unsafe extern "C" fn HashGet() -> u64 {
    return Hash_FNV64_Incremental(
        (*this.widget).hash,
        &mut (*this.widget).index as *mut u32 as *const libc::c_void,
        ::core::mem::size_of::<u32>() as usize as i32,
    );
}
#[inline]
unsafe extern "C" fn HashNext() -> u64 {
    (*this.widget).index = ((*this.widget).index).wrapping_add(1);
    return HashGet();
}
#[inline]
unsafe extern "C" fn HashPeekNext() -> u64 {
    let mut index: u32 = ((*this.widget).index).wrapping_add(1 as i32 as u32);
    return Hash_FNV64_Incremental(
        (*this.widget).hash,
        &mut index as *mut u32 as *const libc::c_void,
        ::core::mem::size_of::<u32>() as usize as i32,
    );
}
#[inline]
unsafe extern "C" fn TransformPos(mut x: *mut f32, mut y: *mut f32) {
    if *x < 0.0f32 {
        *x = (*this.layout).upper.x + *x;
    }
    if *y < 0.0f32 {
        *y = (*this.layout).upper.y + *y;
    }
}
#[inline]
unsafe extern "C" fn TransformSize(mut sx: *mut f32, mut sy: *mut f32) {
    if *sx <= 0.0f32 {
        *sx = (*this.layout).upper.x - this.cursor.x + *sx;
    }
    if *sy <= 0.0f32 {
        *sy = (*this.layout).upper.y - this.cursor.y + *sy;
    }
}
#[inline]
unsafe extern "C" fn RectContains(mut pos: Vec2, mut size: Vec2, mut p: Vec2) -> bool {
    return pos.x <= p.x && p.x <= pos.x + size.x && pos.y <= p.y && p.y <= pos.y + size.y;
}
#[inline]
unsafe extern "C" fn Spacing() {
    this.cursor.x += (*this.layout).spacing.x;
    this.cursor.y += (*this.layout).spacing.y;
    (*this.layout).spacing.x = 0.0f32;
    (*this.layout).spacing.y = 0.0f32;
}
unsafe extern "C" fn ImGui_PushLayout(mut sx: f32, mut sy: f32, mut horizontal: bool) {
    TransformSize(&mut sx, &mut sy);
    let mut layout: *mut ImGuiLayout = MemPool_Alloc(this.layoutPool) as *mut ImGuiLayout;
    (*layout).prev = this.layout;
    (*layout).lower = this.cursor;
    (*layout).upper = Vec2::new(this.cursor.x + sx, this.cursor.y + sy);
    (*layout).size = Vec2::new(sx, sy);
    (*layout).styleVars = 0 as i32;
    (*layout).horizontal = horizontal;
    this.layout = layout;
}
unsafe extern "C" fn ImGui_PopLayout() {
    let mut layout: *mut ImGuiLayout = this.layout;
    let mut i: i32 = 0 as i32;
    while i < (*layout).styleVars {
        ImGui_PopStyle();
        i += 1;
    }
    this.layout = (*layout).prev;
    MemPool_Dealloc(this.layoutPool, layout as *mut libc::c_void);
}
#[inline]
unsafe extern "C" fn ImGui_Pad(mut mx: f32, mut my: f32) {
    let mut px: f32 = mx * (*this.style).padding.x;
    let mut py: f32 = my * (*this.style).padding.y;
    this.cursor.x += px;
    this.cursor.y += py;
    (*this.layout).lower.x += px;
    (*this.layout).lower.y += py;
    (*this.layout).upper.x -= px;
    (*this.layout).upper.y -= py;
    (*this.layout).size.x -= 2.0f32 * px;
    (*this.layout).size.y -= 2.0f32 * py;
}
unsafe extern "C" fn ImGui_Unpad(mut mx: f32, mut my: f32) {
    let mut px: f32 = mx * (*this.style).padding.x;
    let mut py: f32 = my * (*this.style).padding.y;
    (*this.layout).lower.x -= px;
    (*this.layout).lower.y -= py;
    (*this.layout).upper.x += px;
    (*this.layout).upper.y += py;
    (*this.layout).size.x += 2.0f32 * px;
    (*this.layout).size.y += 2.0f32 * py;
    this.cursor.x -= px;
    this.cursor.y -= py;
}
unsafe extern "C" fn ImGui_BeginWidget(mut sx: f32, mut sy: f32) {
    Spacing();
    TransformSize(&mut sx, &mut sy);
    let mut widget: *mut ImGuiWidget = MemPool_Alloc(this.widgetPool) as *mut ImGuiWidget;
    (*widget).prev = this.widget;
    (*widget).index = 0 as i32 as u32;
    (*widget).pos = Vec2::new(this.cursor.x, this.cursor.y);
    (*widget).size = Vec2::new(sx, sy);
    if !(this.widget).is_null() {
        (*this.widget).index = ((*this.widget).index).wrapping_add(1);
        (*widget).hash = Hash_FNV64_Incremental(
            (*this.widget).hash,
            &mut (*this.widget).index as *mut u32 as *const libc::c_void,
            ::core::mem::size_of::<u32>() as usize as i32,
        );
    } else {
        (*widget).hash = Hash_FNV64_Init();
    }
    this.widget = widget;
}
unsafe extern "C" fn ImGui_EndWidget() {
    if !(this.widgetLast).is_null() {
        MemPool_Dealloc(this.widgetPool, this.widgetLast as *mut libc::c_void);
    }
    this.cursor = (*this.widget).pos;
    this.widgetLast = this.widget;
    this.widget = (*this.widget).prev;
    Advance((*this.widgetLast).size);
}
unsafe extern "C" fn ImGui_Focus(mut widget: *mut ImGuiWidget, mut focusType: i32) -> bool {
    if this.focus[focusType as usize] == 0 as u64 {
        if !IsClipped(this.mouse)
            && RectContains((*widget).pos, (*widget).size, this.mouse) as i32 != 0
        {
            this.focus[focusType as usize] = (*widget).hash;
        }
    }
    return this.focus[focusType as usize] == (*widget).hash;
}
#[inline]
unsafe extern "C" fn ImGui_FocusCurrent(mut focusType: i32) -> bool {
    return ImGui_Focus(this.widget, focusType);
}
#[inline]
unsafe extern "C" fn ImGui_FocusLast(mut focusType: i32) -> bool {
    return ImGui_Focus(this.widgetLast, focusType);
}
#[inline]
unsafe extern "C" fn TryFocusRect(
    mut hash: u64,
    mut focusType: i32,
    mut pos: Vec2,
    mut size: Vec2,
) -> bool {
    if this.focus[focusType as usize] == 0 as u64 {
        if !IsClipped(this.mouse) && RectContains(pos, size, this.mouse) as i32 != 0 {
            this.focus[focusType as usize] = hash;
        }
    }
    return this.focus[focusType as usize] == hash;
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
    let mut layer: *mut ImGuiLayer =
        MemAlloc(::core::mem::size_of::<ImGuiLayer>() as usize) as *mut ImGuiLayer;
    (*layer).parent = this.layer;
    (*layer).children = 0 as *mut ImGuiLayer;
    (*layer).next = 0 as *mut ImGuiLayer;
    (*layer).pos = (*this.layout).lower;
    (*layer).size = (*this.layout).size;
    (*layer).index = 0 as i32 as u32;
    (*layer).clip = clip;
    (*layer).tex2DList = 0 as *mut ImGuiTex2D;
    (*layer).panelList = 0 as *mut ImGuiPanel;
    (*layer).rectList = 0 as *mut ImGuiRect;
    (*layer).textList = 0 as *mut ImGuiText;
    (*layer).lineList = 0 as *mut ImGuiLine;
    if !(this.layer).is_null() {
        (*layer).next = (*this.layer).children;
        (*this.layer).children = layer;
        (*layer).hash = HashNext();
    } else {
        (*layer).hash = Hash_FNV64_Init();
    }
    this.layer = layer;
    if clip {
        ImGui_PushClipRect((*this.layer).pos, (*this.layer).size);
    }
    return layer;
}
unsafe extern "C" fn ImGui_PopLayer() {
    if (*this.layer).clip {
        ImGui_PopClipRect();
    }
    this.layerLast = this.layer;
    this.layer = (*this.layer).parent;
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
        Draw_Color(1.0f32, 1.0f32, 1.0f32, 1.0f32);
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
        let pad: f32 = 64.0f32;
        Shader_Start(shader);
        Shader_SetFloat(b"padding\0" as *const u8 as *const libc::c_char, pad);
        let mut e_0: *const ImGuiPanel = (*self_1).panelList;
        while !e_0.is_null() {
            let mut x: f32 = (*e_0).pos.x - pad;
            let mut y: f32 = (*e_0).pos.y - pad;
            let mut sx: f32 = (*e_0).size.x + 2.0f32 * pad;
            let mut sy: f32 = (*e_0).size.y + 2.0f32 * pad;
            Shader_SetFloat(
                b"innerAlpha\0" as *const u8 as *const libc::c_char,
                (*e_0).innerAlpha,
            );
            Shader_SetFloat(b"bevel\0" as *const u8 as *const libc::c_char, (*e_0).bevel);
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
        Draw_Color(
            (*e_1).color.x,
            (*e_1).color.y,
            (*e_1).color.z,
            (*e_1).color.w,
        );
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
        RenderState_PushBlendMode(0 as i32);
        static mut shader_0: *mut Shader = 0 as *const Shader as *mut Shader;
        if shader_0.is_null() {
            shader_0 = Shader_Load(
                b"vertex/ui\0" as *const u8 as *const libc::c_char,
                b"fragment/ui/line\0" as *const u8 as *const libc::c_char,
            );
        }
        let pad_0: f32 = 64.0f32;
        Shader_Start(shader_0);
        let mut e_2: *const ImGuiLine = (*self_1).lineList;
        while !e_2.is_null() {
            let mut xMin: f32 = (Min((*e_2).p1.x as f64, (*e_2).p2.x as f64) - pad_0 as f64) as f32;
            let mut yMin: f32 = (Min((*e_2).p1.y as f64, (*e_2).p2.y as f64) - pad_0 as f64) as f32;
            let mut xMax: f32 = (Max((*e_2).p1.x as f64, (*e_2).p2.x as f64) + pad_0 as f64) as f32;
            let mut yMax: f32 = (Max((*e_2).p1.y as f64, (*e_2).p2.y as f64) + pad_0 as f64) as f32;
            let mut sx_0: f32 = xMax - xMin;
            let mut sy_0: f32 = yMax - yMin;
            Shader_SetFloat2(b"origin\0" as *const u8 as *const libc::c_char, xMin, yMin);
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
static mut init_imgui: bool = 0 as i32 != 0;
unsafe extern "C" fn ImGui_Init() {
    if init_imgui {
        return;
    }
    init_imgui = 1 as i32 != 0;
    this.layer = 0 as *mut ImGuiLayer;
    this.layerLast = 0 as *mut ImGuiLayer;
    this.style = 0 as *mut ImGuiStyle;
    this.clipRect = 0 as *mut ImGuiClipRect;
    this.cursorStack = 0 as *mut ImGuiCursor;
    this.dragging = 0 as i32 as u64;
    this.data = HashMap_Create(0 as i32 as u32, 128 as i32 as u32);
    this.layoutPool = MemPool_CreateAuto(::core::mem::size_of::<ImGuiLayout>() as usize as u32);
    this.widgetPool = MemPool_CreateAuto(::core::mem::size_of::<ImGuiWidget>() as usize as u32);
    this.stylePool = MemPool_CreateAuto(::core::mem::size_of::<ImGuiStyle>() as usize as u32);
    this.clipRectPool = MemPool_CreateAuto(::core::mem::size_of::<ImGuiClipRect>() as usize as u32);
    this.cursorPool = MemPool_CreateAuto(::core::mem::size_of::<ImGuiCursor>() as usize as u32);
    this.tex2DPool = MemPool_CreateAuto(::core::mem::size_of::<ImGuiTex2D>() as usize as u32);
    this.panelPool = MemPool_CreateAuto(::core::mem::size_of::<ImGuiPanel>() as usize as u32);
    this.rectPool = MemPool_CreateAuto(::core::mem::size_of::<ImGuiRect>() as usize as u32);
    this.textPool = MemPool_CreateAuto(::core::mem::size_of::<ImGuiText>() as usize as u32);
    this.linePool = MemPool_CreateAuto(::core::mem::size_of::<ImGuiLine>() as usize as u32);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_Begin(mut sx: f32, mut sy: f32) {
    ImGui_Init();
    let mut i: i32 = 0 as i32;
    while i < FocusType_SIZE {
        this.focus[i as usize] = 0 as i32 as u64;
        i += 1;
    }
    if !Input_GetDown(Button_Mouse_Left) {
        this.dragging = 0 as i32 as u64;
    }
    if this.dragging != 0 {
        this.focus[FocusType_Mouse as usize] = this.dragging;
    }
    this.cursor = Vec2::new(0.0f32, 0.0f32);
    if !(this.layerLast).is_null() {
        ImGuiLayer_Free(this.layerLast);
        this.layerLast = 0 as *mut ImGuiLayer;
    }
    MemPool_Clear(this.layoutPool);
    MemPool_Clear(this.widgetPool);
    MemPool_Clear(this.stylePool);
    MemPool_Clear(this.clipRectPool);
    MemPool_Clear(this.cursorPool);
    MemPool_Clear(this.tex2DPool);
    MemPool_Clear(this.panelPool);
    MemPool_Clear(this.rectPool);
    MemPool_Clear(this.textPool);
    MemPool_Clear(this.linePool);
    this.style = 0 as *mut ImGuiStyle;
    ImGui_PushDefaultStyle();
    this.layout = 0 as *mut ImGuiLayout;
    ImGui_PushLayout(sx, sy, 0 as i32 != 0);
    this.widget = 0 as *mut ImGuiWidget;
    this.widgetLast = 0 as *mut ImGuiWidget;
    ImGui_BeginWidget(sx, sy);
    this.layer = 0 as *mut ImGuiLayer;
    ImGui_PushLayer(1 as i32 != 0);
    let mut mouse: IVec2 = IVec2 { x: 0, y: 0 };
    Input_GetMousePosition(&mut mouse);
    this.mouse.x = mouse.x as f32;
    this.mouse.y = mouse.y as f32;
    this.activate = Input_GetPressed(Button_Mouse_Left);
    this.forceSize = Vec2::new(0.0f32, 0.0f32);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_End() {
    ImGui_PopLayer();
    ImGui_EndWidget();
    ImGui_PopLayout();
    if !(this.layer).is_null() {
        Fatal(b"ImGui_End: layer stack not empty\0" as *const u8 as *const libc::c_char);
    }
    if !(this.widget).is_null() {
        Fatal(b"ImGui_End: widget stack not empty\0" as *const u8 as *const libc::c_char);
    }
    if !(this.layout).is_null() {
        Fatal(b"ImGui_End: layout stack not empty\0" as *const u8 as *const libc::c_char);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_Draw() {
    RenderState_PushBlendMode(1 as i32);
    Draw_LineWidth(1.0f32);
    ImGui_DrawLayer(this.layerLast);
    RenderState_PopBlendMode();
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_AlignCursor(
    mut sx: f32,
    mut sy: f32,
    mut alignX: f32,
    mut alignY: f32,
) {
    TransformSize(&mut sx, &mut sy);
    ImGui_SetCursor(
        (*this.layout).lower.x + alignX * ((*this.layout).size.x - sx),
        (*this.layout).lower.y + alignY * ((*this.layout).size.y - sy),
    );
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_GetCursorX() -> f32 {
    return this.cursor.x;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_GetCursorY() -> f32 {
    return this.cursor.y;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_PushCursor() {
    let mut cursor: *mut ImGuiCursor = MemPool_Alloc(this.cursorPool) as *mut ImGuiCursor;
    (*cursor).prev = this.cursorStack;
    (*cursor).pos = this.cursor;
    this.cursorStack = cursor;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_PopCursor() {
    let mut cursor: *mut ImGuiCursor = this.cursorStack;
    this.cursor = (*cursor).pos;
    this.cursorStack = (*cursor).prev;
    MemPool_Dealloc(this.cursorPool, cursor as *mut libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_SetCursor(mut cx: f32, mut cy: f32) {
    TransformPos(&mut cx, &mut cy);
    this.cursor = Vec2::new(cx, cy);
    (*this.layout).spacing = Vec2::new(0.0f32, 0.0f32);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_SetCursorX(mut x: f32) {
    ImGui_SetCursor(x, this.cursor.y);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_SetCursorY(mut y: f32) {
    ImGui_SetCursor(this.cursor.x, y);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_Indent() {
    this.cursor.x += 2.0f32 * (*this.style).padding.x;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_Undent() {
    this.cursor.x -= 2.0f32 * (*this.style).padding.x;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_BeginGroup(mut sx: f32, mut sy: f32, mut horizontal: bool) {
    ImGui_BeginWidget(sx, sy);
    ImGui_PushLayout(sx, sy, horizontal);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_BeginGroupX(mut sy: f32) {
    ImGui_BeginWidget(0.0f32, sy);
    ImGui_PushLayout(0.0f32, sy, 1 as i32 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_BeginGroupY(mut sx: f32) {
    ImGui_BeginWidget(sx, 0.0f32);
    ImGui_PushLayout(sx, 0.0f32, 0 as i32 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_EndGroup() {
    ImGui_PopLayout();
    ImGui_EndWidget();
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_BeginPanel(mut sx: f32, mut sy: f32) {
    ImGui_BeginGroup(sx, sy, 0 as i32 != 0);
    ImGui_PushLayer(0 as i32 != 0);
    ImGui_PushLayer(1 as i32 != 0);
    ImGui_Pad(1.0f32, 1.0f32);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_EndPanel() {
    ImGui_Unpad(1.0f32, 1.0f32);
    ImGui_PopLayer();
    EmitPanel(
        (*this.style).frameColor,
        (*this.widget).pos,
        (*this.widget).size,
        1.0f32,
        12.0f32,
    );
    ImGui_PopLayer();
    ImGui_EndGroup();
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_BeginWindow(mut title: *const libc::c_char, mut sx: f32, mut sy: f32) {
    let mut hash: u64 = HashPeekNext();
    let mut data: *mut ImGuiData = GetData(hash);
    this.cursor.x += (*data).offset.x;
    this.cursor.y += (*data).offset.y;
    ImGui_BeginPanel(sx, sy);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_EndWindow() {
    ImGui_EndPanel();
    let mut data: *mut ImGuiData = GetData((*this.widgetLast).hash);
    this.cursor.x -= (*data).offset.x;
    this.cursor.y -= (*data).offset.y;
    if ImGui_FocusLast(FocusType_Mouse) {
        if Input_GetDown(Button_Mouse_Left) {
            let mut delta: IVec2 = IVec2 { x: 0, y: 0 };
            Input_GetMouseDelta(&mut delta);
            (*data).offset.x += delta.x as f32;
            (*data).offset.y += delta.y as f32;
            this.dragging = (*this.widgetLast).hash;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_BeginScrollFrame(mut sx: f32, mut sy: f32) {
    ImGui_BeginGroup(sx, sy, 0 as i32 != 0);
    ImGui_PushLayer(1 as i32 != 0);
    ImGui_Pad(1.0f32, 1.0f32);
    let mut data: *mut ImGuiData = GetData((*this.widget).hash);
    this.cursor.y -= (*data).scroll;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_EndScrollFrame() {
    let mut data: *mut ImGuiData = GetData((*this.widget).hash);
    this.cursor.y += (*data).scroll;
    let mut layout: *mut ImGuiLayout = this.layout;
    ImGui_PopLayer();
    let mut scroll: f32 = (*data).scroll;
    let mut virtualSize: f32 = this.cursor.y - (*layout).lower.y;
    let mut scrollMax: f32 = virtualSize - (*layout).size.y;
    let mut scrollPos = Vec2::new((*layout).lower.x + (*layout).size.x, (*layout).lower.y);
    let mut scrollSize = Vec2::new((*this.style).scrollBarSize.x, (*layout).size.y);
    let mut handleHash: u64 = HashNext();
    if (*layout).size.y < virtualSize {
        let mut handleSizeY: f32 = (*layout).size.y * ((*layout).size.y / virtualSize);
        handleSizeY = Clamp(handleSizeY as f64, 16.0f32 as f64, 128.0f32 as f64) as f32;
        let mut handleOffset: f32 = ((*layout).size.y - handleSizeY) * (scroll / scrollMax);
        let mut handlePos = Vec2::new(scrollPos.x, scrollPos.y + handleOffset);
        let mut handleSize = Vec2::new((*this.style).scrollBarSize.x, handleSizeY);
        let mut handleFocus: bool =
            TryFocusRect(handleHash, FocusType_Mouse, handlePos, handleSize);
        EmitPanel(
            if handleFocus as i32 != 0 {
                (*this.style).buttonColorFocus
            } else {
                Vec4f_Create(0.3f32, 0.4f32, 0.5f32, 1.0f32)
            },
            handlePos,
            handleSize,
            if handleFocus as i32 != 0 {
                0.5f32
            } else {
                0.25f32
            },
            4.0f32,
        );
    }
    ImGui_Unpad(1.0f32, 1.0f32);
    ImGui_EndGroup();
    EmitPanel(
        Vec4f_Create(0.0f32, 0.0f32, 0.0f32, 0.5f32),
        (*this.widgetLast).pos,
        (*this.widgetLast).size,
        0.25f32,
        4.0f32,
    );
    if ImGui_FocusLast(FocusType_Scroll) {
        let mut scroll_0: IVec2 = IVec2 { x: 0, y: 0 };
        Input_GetMouseScroll(&mut scroll_0);
        (*data).scroll -= 10.0f32 * scroll_0.y as f32;
    }
    (*data).scroll = Clamp((*data).scroll as f64, 0.0f32 as f64, scrollMax as f64) as f32;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_SetNextWidth(mut sx: f32) {
    this.forceSize.x = sx;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_SetNextHeight(mut sy: f32) {
    this.forceSize.y = sy;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_PushStyle() {
    let mut style: *mut ImGuiStyle = MemPool_Alloc(this.stylePool) as *mut ImGuiStyle;
    MemCpy(
        style as *mut libc::c_void,
        this.style as *const libc::c_void,
        ::core::mem::size_of::<ImGuiStyle>() as usize,
    );
    (*style).prev = this.style;
    this.style = style;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_PushStyleFont(mut font: *mut Font) {
    ImGui_PushStyle();
    (*this.style).font = font;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_PushStylePadding(mut px: f32, mut py: f32) {
    ImGui_PushStyle();
    (*this.style).padding = Vec2::new(px, py);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_PushStyleSpacing(mut x: f32, mut y: f32) {
    ImGui_PushStyle();
    (*this.style).spacing = Vec2::new(x, y);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_PushStyleTextColor(mut r: f32, mut g: f32, mut b: f32, mut a: f32) {
    ImGui_PushStyle();
    (*this.style).textColor = Vec4f_Create(r, g, b, a);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_PopStyle() {
    if ((*this.style).prev).is_null() {
        Fatal(
            b"ImGui_PopStyle: Attempting to pop an empty stack\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut style: *mut ImGuiStyle = this.style;
    this.style = (*style).prev;
    MemPool_Dealloc(this.stylePool, style as *mut libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_SetFont(mut font: *mut Font) {
    ImGui_PushStyleFont(font);
    (*this.layout).styleVars += 1;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_SetSpacing(mut sx: f32, mut sy: f32) {
    ImGui_PushStyleSpacing(sx, sy);
    (*this.layout).styleVars += 1;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_Button(mut label: *const libc::c_char) -> bool {
    return ImGui_ButtonEx(label, 0.0f32, 32.0f32);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_ButtonEx(mut label: *const libc::c_char, mut sx: f32, mut sy: f32) -> bool {
    ImGui_BeginWidget(sx, sy);
    let mut focus: bool = ImGui_FocusCurrent(FocusType_Mouse);
    let mut color: Vec4f = if focus as i32 != 0 {
        (*this.style).buttonColorFocus
    } else {
        (*this.style).buttonColor
    };
    EmitPanel(
        color,
        (*this.widget).pos,
        (*this.widget).size,
        if focus as i32 != 0 { 1.0f32 } else { 0.5f32 },
        4.0f32,
    );
    let mut bound: IVec2 = IVec2 { x: 0, y: 0 };
    Font_GetSize2((*this.style).font, &mut bound, label);
    let mut labelPos = Vec2::new(
        (*this.widget).pos.x + 0.5f32 * ((*this.widget).size.x - bound.x as f32),
        (*this.widget).pos.y + 0.5f32 * ((*this.widget).size.y - bound.y as f32),
    );
    let mut labelSize = Vec2::new(bound.x as f32, bound.y as f32);
    EmitText(
        (*this.style).font,
        if focus as i32 != 0 {
            (*this.style).textColorFocus
        } else {
            (*this.style).textColor
        },
        Vec2::new(labelPos.x, labelPos.y + bound.y as f32),
        label,
    );
    ImGui_EndWidget();
    return focus as i32 != 0 && this.activate as i32 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_Checkbox(mut value: bool) -> bool {
    ImGui_BeginWidget(16.0f32, 16.0f32);
    let mut focus: bool = ImGui_FocusCurrent(FocusType_Mouse);
    if focus as i32 != 0 && this.activate as i32 != 0 {
        value = !value;
    }
    if focus {
        EmitRect(
            (*this.style).buttonColorFocus,
            (*this.widget).pos,
            (*this.widget).size,
            1 as i32 != 0,
        );
    }
    EmitPanel(
        if value as i32 != 0 {
            (*this.style).buttonColorFocus
        } else {
            (*this.style).buttonColor
        },
        Vec2::new((*this.widget).pos.x + 2.0f32, (*this.widget).pos.y + 2.0f32),
        Vec2::new(
            (*this.widget).size.x - 4.0f32,
            (*this.widget).size.y - 4.0f32,
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
        (if (*this.layout).horizontal as i32 != 0 {
            2 as i32
        } else {
            0 as i32
        }) as f32,
        (if (*this.layout).horizontal as i32 != 0 {
            0 as i32
        } else {
            2 as i32
        }) as f32,
    );
    EmitLine(
        (*this.style).buttonColorFocus,
        (*this.widget).pos,
        Vec2::new(
            (*this.widget).pos.x
                + (if (*this.layout).horizontal as i32 != 0 {
                    0.0f32
                } else {
                    (*this.widget).size.x
                }),
            (*this.widget).pos.y
                + (if (*this.layout).horizontal as i32 != 0 {
                    (*this.widget).size.y
                } else {
                    0.0f32
                }),
        ),
    );
    ImGui_EndWidget();
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_Selectable(mut label: *const libc::c_char) -> bool {
    let mut bound: IVec2 = IVec2 { x: 0, y: 0 };
    Font_GetSize2((*this.style).font, &mut bound, label);
    ImGui_BeginWidget(
        if (*this.layout).horizontal as i32 != 0 {
            bound.x as f32 + 4.0f32
        } else {
            0.0f32
        },
        if (*this.layout).horizontal as i32 != 0 {
            0.0f32
        } else {
            4.0f32 + Font_GetLineHeight((*this.style).font) as f32
        },
    );
    let mut focus: bool = ImGui_FocusCurrent(FocusType_Mouse);
    if focus {
        EmitRect(
            (*this.style).buttonColorFocus,
            (*this.widget).pos,
            (*this.widget).size,
            0 as i32 != 0,
        );
    }
    EmitText(
        (*this.style).font,
        if focus as i32 != 0 {
            (*this.style).textColorFocus
        } else {
            (*this.style).textColor
        },
        Vec2::new(
            (*this.widget).pos.x + 2.0f32,
            (*this.widget).pos.y
                + bound.y as f32
                + 0.5f32 * ((*this.widget).size.y - bound.y as f32),
        ),
        label,
    );
    ImGui_EndWidget();
    return focus as i32 != 0 && this.activate as i32 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_Tex2D(mut tex: *mut Tex2D) {
    let mut size: IVec2 = IVec2 { x: 0, y: 0 };
    Tex2D_GetSize(tex, &mut size);
    let mut sizef = Vec2::new(size.x as f32, size.y as f32);
    ImGui_BeginWidget(size.x as f32, size.y as f32);
    EmitTex2D(tex, this.cursor, sizef);
    ImGui_EndWidget();
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_Text(mut text: *const libc::c_char) {
    ImGui_TextEx(
        (*this.style).font,
        text,
        (*this.style).textColor.x,
        (*this.style).textColor.y,
        (*this.style).textColor.z,
        (*this.style).textColor.w,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_TextColored(
    mut text: *const libc::c_char,
    mut r: f32,
    mut g: f32,
    mut b: f32,
    mut a: f32,
) {
    ImGui_TextEx((*this.style).font, text, r, g, b, a);
}
#[no_mangle]
pub unsafe extern "C" fn ImGui_TextEx(
    mut font: *mut Font,
    mut text: *const libc::c_char,
    mut r: f32,
    mut g: f32,
    mut b: f32,
    mut a: f32,
) {
    let mut bound: IVec2 = IVec2 { x: 0, y: 0 };
    Font_GetSize2((*this.style).font, &mut bound, text);
    ImGui_BeginWidget(
        bound.x as f32,
        (if (*this.layout).horizontal as i32 != 0 {
            0 as i32
        } else {
            Font_GetLineHeight((*this.style).font)
        }) as f32,
    );
    EmitText(
        font,
        Vec4f_Create(r, g, b, a),
        Vec2::new(
            (*this.widget).pos.x,
            (*this.widget).pos.y
                + bound.y as f32
                + 0.5f32 * ((*this.widget).size.y - bound.y as f32),
        ),
        text,
    );
    ImGui_EndWidget();
}
