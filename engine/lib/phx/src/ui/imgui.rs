use super::*;
use crate::common::*;
use crate::input::*;
use crate::internal::*;
use crate::math::*;
use crate::render::*;
use crate::system::*;
use crate::*;

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

    pub focus: [u64; FocusType_SIZE as usize],
    pub dragging: u64,

    pub activate: bool,
    pub forceSize: Vec2,

    pub data: *mut HashMap,

    /* TODO : Stack allocator. */
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

    pub buttonColor: Vec4,
    pub buttonColorFocus: Vec4,
    pub frameColor: Vec4,
    pub textColor: Vec4,
    pub textColorFocus: Vec4,
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
    pub color: Vec4,
    pub p1: Vec2,
    pub p2: Vec2,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ImGuiText {
    pub next: *mut ImGuiText,
    pub font: *mut Font,
    pub color: Vec4,
    pub pos: Vec2,
    pub text: *const libc::c_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ImGuiPanel {
    pub next: *mut ImGuiPanel,
    pub color: Vec4,
    pub pos: Vec2,
    pub size: Vec2,
    pub innerAlpha: f32,
    pub bevel: f32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ImGuiRect {
    pub next: *mut ImGuiRect,
    pub color: Vec4,
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

pub const FocusType_Mouse: i32 = 0;
pub const FocusType_Keyboard: i32 = 1;
pub const FocusType_Scroll: i32 = 2;
pub const FocusType_SIZE: i32 = 3;

static mut this: ImGui = ImGui {
    layer: std::ptr::null_mut(),
    layerLast: std::ptr::null_mut(),
    layout: std::ptr::null_mut(),
    widget: std::ptr::null_mut(),
    widgetLast: std::ptr::null_mut(),
    style: std::ptr::null_mut(),
    clipRect: std::ptr::null_mut(),
    cursorStack: std::ptr::null_mut(),
    cursor: Vec2::ZERO,
    mouse: Vec2::ZERO,
    focus: [0; FocusType_SIZE as usize],
    dragging: 0,
    activate: false,
    forceSize: Vec2::ZERO,
    data: std::ptr::null_mut(),
    layoutPool: std::ptr::null_mut(),
    widgetPool: std::ptr::null_mut(),
    stylePool: std::ptr::null_mut(),
    clipRectPool: std::ptr::null_mut(),
    cursorPool: std::ptr::null_mut(),
    tex2DPool: std::ptr::null_mut(),
    panelPool: std::ptr::null_mut(),
    rectPool: std::ptr::null_mut(),
    textPool: std::ptr::null_mut(),
    linePool: std::ptr::null_mut(),
};

#[inline]
unsafe extern "C" fn EmitLine(color: Vec4, p1: Vec2, p2: Vec2) {
    let e: *mut ImGuiLine = MemPool_Alloc(&mut *this.linePool) as *mut ImGuiLine;
    (*e).color = color;
    (*e).p1 = p1;
    (*e).p2 = p2;
    (*e).next = (*this.layer).lineList;
    (*this.layer).lineList = e;
}

#[inline]
unsafe extern "C" fn EmitPanel(color: Vec4, pos: Vec2, size: Vec2, innerAlpha: f32, bevel: f32) {
    let e: *mut ImGuiPanel = MemPool_Alloc(&mut *this.panelPool) as *mut ImGuiPanel;
    (*e).color = color;
    (*e).pos = pos;
    (*e).size = size;
    (*e).innerAlpha = innerAlpha;
    (*e).bevel = bevel;
    (*e).next = (*this.layer).panelList;
    (*this.layer).panelList = e;
}

#[inline]
unsafe extern "C" fn EmitRect(color: Vec4, pos: Vec2, size: Vec2, outline: bool) {
    let e: *mut ImGuiRect = MemPool_Alloc(&mut *this.rectPool) as *mut ImGuiRect;
    (*e).color = color;
    (*e).pos = pos;
    (*e).size = size;
    (*e).outline = outline;
    (*e).next = (*this.layer).rectList;
    (*this.layer).rectList = e;
}

#[inline]
unsafe extern "C" fn EmitTex2D(tex: *mut Tex2D, pos: Vec2, size: Vec2) {
    let e: *mut ImGuiTex2D = MemPool_Alloc(&mut *this.tex2DPool) as *mut ImGuiTex2D;
    (*e).tex = tex;
    (*e).pos = pos;
    (*e).size = size;
    (*e).next = (*this.layer).tex2DList;
    (*this.layer).tex2DList = e;
}

#[inline]
unsafe extern "C" fn EmitText(font: *mut Font, color: Vec4, pos: Vec2, text: *const libc::c_char) {
    let e: *mut ImGuiText = MemPool_Alloc(&mut *this.textPool) as *mut ImGuiText;

    (*e).font = font;
    (*e).color = color;
    (*e).pos = pos;
    (*e).text = StrDup(text);
    (*e).next = (*this.layer).textList;
    (*this.layer).textList = e;
}

#[inline]
unsafe extern "C" fn GetData(hash: u64) -> *mut ImGuiData {
    let mut data: *mut ImGuiData = HashMap_GetRaw(this.data, hash) as *mut ImGuiData;
    if data.is_null() {
        data = MemNew!(ImGuiData);
        (*data).size = Vec2::ZERO;
        (*data).offset = Vec2::ZERO;
        (*data).scroll = 0.0f32;
        HashMap_SetRaw(this.data, hash, data as *mut _);
    }
    data
}

unsafe extern "C" fn ImGui_PushDefaultStyle() {
    static mut font: *mut Font = std::ptr::null_mut();
    static mut fontSubheading: *mut Font = std::ptr::null_mut();
    if font.is_null() {
        font = Font_Load(c_str!("Share"), 16);
        fontSubheading = Font_Load(c_str!("Iceland"), 18);
    }

    let style: *mut ImGuiStyle = MemPool_Alloc(&mut *this.stylePool) as *mut ImGuiStyle;
    (*style).prev = this.style;
    (*style).font = font;
    (*style).fontSubheading = fontSubheading;
    (*style).spacing = Vec2::new(8.0f32, 8.0f32);
    (*style).padding = Vec2::new(8.0f32, 8.0f32);
    (*style).scrollBarSize = Vec2::new(4.0f32, 4.0f32);
    (*style).buttonColor = Vec4::new(0.1f32, 0.12f32, 0.15f32, 1.0f32);
    (*style).buttonColorFocus = Vec4::new(0.1f32, 0.6f32, 1.0f32, 1.0f32);
    (*style).frameColor = Vec4::new(0.1f32, 0.12f32, 0.15f32, 0.95f32);
    (*style).textColor = Vec4::ONE;
    (*style).textColorFocus = Vec4::new(0.1f32, 0.1f32, 0.1f32, 1.0f32);
    this.style = style;
}

unsafe extern "C" fn ImGui_PushClipRect(pos: Vec2, size: Vec2) {
    let rect = MemNew!(ImGuiClipRect);
    let prev = this.clipRect;
    (*rect).prev = prev;
    (*rect).p1 = pos;
    (*rect).p2 = pos + size;
    if !prev.is_null() {
        (*rect).p1.x = f32::max((*rect).p1.x, (*prev).p1.x);
        (*rect).p1.y = f32::max((*rect).p1.y, (*prev).p1.y);
        (*rect).p2.x = f32::min((*rect).p2.x, (*prev).p2.x);
        (*rect).p2.y = f32::min((*rect).p2.y, (*prev).p2.y);
    }
    this.clipRect = rect;
}

unsafe extern "C" fn ImGui_PopClipRect() {
    let rect = this.clipRect;
    this.clipRect = (*rect).prev;
    MemFree(rect as *const _);
}

#[inline]
unsafe extern "C" fn IsClipped(p: Vec2) -> bool {
    if (this.clipRect).is_null() {
        return false;
    }
    p.x < (*this.clipRect).p1.x
        || p.y < (*this.clipRect).p1.y
        || (*this.clipRect).p2.x < p.x
        || (*this.clipRect).p2.y < p.y
}

#[inline]
unsafe extern "C" fn Advance(size: Vec2) {
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
    Hash_FNV64_Incremental(
        (*this.widget).hash,
        &mut (*this.widget).index as *mut u32 as *const _,
        std::mem::size_of::<u32>() as i32,
    )
}

#[inline]
unsafe extern "C" fn HashNext() -> u64 {
    (*this.widget).index = ((*this.widget).index).wrapping_add(1);
    HashGet()
}

#[inline]
unsafe extern "C" fn HashPeekNext() -> u64 {
    let mut index: u32 = ((*this.widget).index).wrapping_add(1);
    Hash_FNV64_Incremental(
        (*this.widget).hash,
        &mut index as *mut u32 as *const _,
        std::mem::size_of::<u32>() as i32,
    )
}

#[inline]
unsafe extern "C" fn TransformPos(x: *mut f32, y: *mut f32) {
    if *x < 0.0f32 {
        *x += (*this.layout).upper.x;
    }
    if *y < 0.0f32 {
        *y += (*this.layout).upper.y;
    }
}

#[inline]
unsafe extern "C" fn TransformSize(sx: *mut f32, sy: *mut f32) {
    if *sx <= 0.0f32 {
        *sx += (*this.layout).upper.x - this.cursor.x;
    }
    if *sy <= 0.0f32 {
        *sy += (*this.layout).upper.y - this.cursor.y;
    }
}

#[inline]
extern "C" fn RectContains(pos: Vec2, size: Vec2, p: Vec2) -> bool {
    pos.x <= p.x && p.x <= pos.x + size.x && pos.y <= p.y && p.y <= pos.y + size.y
}

#[inline]
unsafe extern "C" fn Spacing() {
    this.cursor.x += (*this.layout).spacing.x;
    this.cursor.y += (*this.layout).spacing.y;
    (*this.layout).spacing.x = 0.0f32;
    (*this.layout).spacing.y = 0.0f32;
}

unsafe extern "C" fn ImGui_PushLayout(mut sx: f32, mut sy: f32, horizontal: bool) {
    TransformSize(&mut sx, &mut sy);
    let layout: *mut ImGuiLayout = MemPool_Alloc(&mut *this.layoutPool) as *mut ImGuiLayout;
    (*layout).prev = this.layout;
    (*layout).lower = this.cursor;
    (*layout).upper = Vec2::new(this.cursor.x + sx, this.cursor.y + sy);
    (*layout).size = Vec2::new(sx, sy);
    (*layout).styleVars = 0;
    (*layout).horizontal = horizontal;
    this.layout = layout;
}

unsafe extern "C" fn ImGui_PopLayout() {
    let layout: *mut ImGuiLayout = this.layout;
    for _ in 0..(*layout).styleVars {
        ImGui_PopStyle();
    }
    this.layout = (*layout).prev;
    MemPool_Dealloc(&mut *this.layoutPool, layout as *mut _);
}

#[inline]
unsafe extern "C" fn ImGui_Pad(mx: f32, my: f32) {
    let px: f32 = mx * (*this.style).padding.x;
    let py: f32 = my * (*this.style).padding.y;
    this.cursor.x += px;
    this.cursor.y += py;
    (*this.layout).lower.x += px;
    (*this.layout).lower.y += py;
    (*this.layout).upper.x -= px;
    (*this.layout).upper.y -= py;
    (*this.layout).size.x -= 2.0f32 * px;
    (*this.layout).size.y -= 2.0f32 * py;
}

unsafe extern "C" fn ImGui_Unpad(mx: f32, my: f32) {
    let px: f32 = mx * (*this.style).padding.x;
    let py: f32 = my * (*this.style).padding.y;
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
    let widget: *mut ImGuiWidget = MemPool_Alloc(&mut *this.widgetPool) as *mut _;

    (*widget).prev = this.widget;
    (*widget).index = 0;
    (*widget).pos = Vec2::new(this.cursor.x, this.cursor.y);
    (*widget).size = Vec2::new(sx, sy);

    /* Compute this widget's hash. */
    if !(this.widget).is_null() {
        (*this.widget).index = ((*this.widget).index).wrapping_add(1);
        (*widget).hash = Hash_FNV64_Incremental(
            (*this.widget).hash,
            &mut (*this.widget).index as *mut u32 as *const _,
            std::mem::size_of::<u32>() as i32,
        );
    } else {
        (*widget).hash = Hash_FNV64_Init();
    }

    this.widget = widget;
}

unsafe extern "C" fn ImGui_EndWidget() {
    if !(this.widgetLast).is_null() {
        MemPool_Dealloc(&mut *this.widgetPool, this.widgetLast as *mut _);
    }
    this.cursor = (*this.widget).pos;
    this.widgetLast = this.widget;
    this.widget = (*this.widget).prev;
    Advance((*this.widgetLast).size);
}

unsafe extern "C" fn ImGui_Focus(widget: *mut ImGuiWidget, focusType: i32) -> bool {
    if this.focus[focusType as usize] == 0
        && !IsClipped(this.mouse)
        && RectContains((*widget).pos, (*widget).size, this.mouse)
    {
        this.focus[focusType as usize] = (*widget).hash;
    }
    this.focus[focusType as usize] == (*widget).hash
}

#[inline]
unsafe extern "C" fn ImGui_FocusCurrent(focusType: i32) -> bool {
    ImGui_Focus(this.widget, focusType)
}

#[inline]
unsafe extern "C" fn ImGui_FocusLast(focusType: i32) -> bool {
    ImGui_Focus(this.widgetLast, focusType)
}

#[inline]
unsafe extern "C" fn TryFocusRect(hash: u64, focusType: i32, pos: Vec2, size: Vec2) -> bool {
    if this.focus[focusType as usize] == 0
        && !IsClipped(this.mouse)
        && RectContains(pos, size, this.mouse)
    {
        this.focus[focusType as usize] = hash;
    }

    this.focus[focusType as usize] == hash
}

unsafe extern "C" fn ImGuiLayer_Free(self_1: *mut ImGuiLayer) {
    let mut child: *mut ImGuiLayer = (*self_1).children;
    while !child.is_null() {
        let next: *mut ImGuiLayer = (*child).next;
        ImGuiLayer_Free(child);
        child = next;
    }

    /* TODO : Stack allocation for strs. */
    let mut e: *mut ImGuiText = (*self_1).textList;
    while !e.is_null() {
        StrFree((*e).text);
        e = (*e).next;
    }
    MemFree(self_1 as *const _);
}

unsafe extern "C" fn ImGui_PushLayer(clip: bool) -> *mut ImGuiLayer {
    let layer = MemNew!(ImGuiLayer);
    (*layer).parent = this.layer;
    (*layer).children = std::ptr::null_mut();
    (*layer).next = std::ptr::null_mut();

    (*layer).pos = (*this.layout).lower;
    (*layer).size = (*this.layout).size;
    (*layer).index = 0;
    (*layer).clip = clip;

    (*layer).tex2DList = std::ptr::null_mut();
    (*layer).panelList = std::ptr::null_mut();
    (*layer).rectList = std::ptr::null_mut();
    (*layer).textList = std::ptr::null_mut();
    (*layer).lineList = std::ptr::null_mut();

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
    layer
}

unsafe extern "C" fn ImGui_PopLayer() {
    if (*this.layer).clip {
        ImGui_PopClipRect();
    }
    this.layerLast = this.layer;
    this.layer = (*this.layer).parent;
}

unsafe extern "C" fn ImGui_DrawLayer(self_1: *const ImGuiLayer) {
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
        Tex2D_Draw(
            &mut *(*e).tex,
            (*e).pos.x,
            (*e).pos.y,
            (*e).size.x,
            (*e).size.y,
        );
        e = (*e).next;
    }

    if !((*self_1).panelList).is_null() {
        // TODO: Store shader properly
        static mut shader: *mut Shader = std::ptr::null_mut();
        if shader.is_null() {
            shader = Box::into_raw(Shader_Load(
                c_str!("vertex/ui"),
                c_str!("fragment/ui/panel"),
            ));
        }

        let pad: f32 = 64.0f32;
        Shader_Start(&mut *shader);
        Shader_SetFloat(c_str!("padding"), pad);

        let mut e: *const ImGuiPanel = (*self_1).panelList;
        while !e.is_null() {
            let x: f32 = (*e).pos.x - pad;
            let y: f32 = (*e).pos.y - pad;
            let sx: f32 = (*e).size.x + 2.0f32 * pad;
            let sy: f32 = (*e).size.y + 2.0f32 * pad;

            Shader_SetFloat(c_str!("innerAlpha"), (*e).innerAlpha);
            Shader_SetFloat(c_str!("bevel"), (*e).bevel);
            Shader_SetFloat2(c_str!("size"), sx, sy);
            Shader_SetFloat4(
                c_str!("color"),
                (*e).color.x,
                (*e).color.y,
                (*e).color.z,
                (*e).color.w,
            );
            Draw_Rect(x, y, sx, sy);
            e = (*e).next;
        }

        Shader_Stop(shader);
    }

    let mut e: *const ImGuiRect = (*self_1).rectList;
    while !e.is_null() {
        Draw_Color((*e).color.x, (*e).color.y, (*e).color.z, (*e).color.w);
        if (*e).outline {
            Draw_Border(1.0f32, (*e).pos.x, (*e).pos.y, (*e).size.x, (*e).size.y);
        } else {
            Draw_Rect((*e).pos.x, (*e).pos.y, (*e).size.x, (*e).size.y);
        }
        e = (*e).next;
    }

    if !((*self_1).lineList).is_null() {
        RenderState_PushBlendMode(0);
        // TODO: Store shader properly
        static mut shader: *mut Shader = std::ptr::null_mut();
        if shader.is_null() {
            shader = Box::into_raw(Shader_Load(c_str!("vertex/ui"), c_str!("fragment/ui/line")));
        }
        let pad: f32 = 64.0f32;
        Shader_Start(&mut *shader);
        let mut e: *const ImGuiLine = (*self_1).lineList;
        while !e.is_null() {
            let xMin: f32 = f32::min((*e).p1.x, (*e).p2.x) - pad;
            let yMin: f32 = f32::min((*e).p1.y, (*e).p2.y) - pad;
            let xMax: f32 = f32::max((*e).p1.x, (*e).p2.x) + pad;
            let yMax: f32 = f32::max((*e).p1.y, (*e).p2.y) + pad;
            let sx_0: f32 = xMax - xMin;
            let sy_0: f32 = yMax - yMin;
            Shader_SetFloat2(c_str!("origin"), xMin, yMin);
            Shader_SetFloat2(c_str!("size"), sx_0, sy_0);
            Shader_SetFloat2(c_str!("p1"), (*e).p1.x, (*e).p1.y);
            Shader_SetFloat2(c_str!("p2"), (*e).p2.x, (*e).p2.y);
            Shader_SetFloat4(
                c_str!("color"),
                (*e).color.x,
                (*e).color.y,
                (*e).color.z,
                (*e).color.w,
            );
            Draw_Rect(xMin, yMin, sx_0, sy_0);
            e = (*e).next;
        }
        Shader_Stop(shader);
        RenderState_PopBlendMode();
    }

    let mut e: *const ImGuiText = (*self_1).textList;
    while !e.is_null() {
        Font_Draw(
            &mut *(*e).font,
            (*e).text,
            (*e).pos.x,
            (*e).pos.y,
            (*e).color.x,
            (*e).color.y,
            (*e).color.z,
            (*e).color.w,
        );
        e = (*e).next;
    }

    let mut e: *const ImGuiLayer = (*self_1).children;
    while !e.is_null() {
        ImGui_DrawLayer(e);
        e = (*e).next;
    }

    if (*self_1).clip {
        ClipRect_Pop();
    }
}

static mut init_imgui: bool = false;

unsafe extern "C" fn ImGui_Init() {
    if init_imgui {
        return;
    }

    init_imgui = true;
    this.layer = std::ptr::null_mut();
    this.layerLast = std::ptr::null_mut();
    this.style = std::ptr::null_mut();
    this.clipRect = std::ptr::null_mut();
    this.cursorStack = std::ptr::null_mut();
    this.dragging = 0;

    this.data = HashMap_Create(0, 128);
    this.layoutPool = MemPool_CreateAuto(std::mem::size_of::<ImGuiLayout>() as u32);
    this.widgetPool = MemPool_CreateAuto(std::mem::size_of::<ImGuiWidget>() as u32);
    this.stylePool = MemPool_CreateAuto(std::mem::size_of::<ImGuiStyle>() as u32);
    this.clipRectPool = MemPool_CreateAuto(std::mem::size_of::<ImGuiClipRect>() as u32);
    this.cursorPool = MemPool_CreateAuto(std::mem::size_of::<ImGuiCursor>() as u32);
    this.tex2DPool = MemPool_CreateAuto(std::mem::size_of::<ImGuiTex2D>() as u32);
    this.panelPool = MemPool_CreateAuto(std::mem::size_of::<ImGuiPanel>() as u32);
    this.rectPool = MemPool_CreateAuto(std::mem::size_of::<ImGuiRect>() as u32);
    this.textPool = MemPool_CreateAuto(std::mem::size_of::<ImGuiText>() as u32);
    this.linePool = MemPool_CreateAuto(std::mem::size_of::<ImGuiLine>() as u32);
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_Begin(sx: f32, sy: f32) {
    ImGui_Init();

    for i in 0..FocusType_SIZE {
        this.focus[i as usize] = 0;
    }

    if !Input_GetDown(Button_Mouse_Left) {
        this.dragging = 0;
    }
    if this.dragging != 0 {
        this.focus[FocusType_Mouse as usize] = this.dragging;
    }

    this.cursor = Vec2::ZERO;

    if !(this.layerLast).is_null() {
        ImGuiLayer_Free(this.layerLast);
        this.layerLast = std::ptr::null_mut();
    }

    MemPool_Clear(&mut *this.layoutPool);
    MemPool_Clear(&mut *this.widgetPool);
    MemPool_Clear(&mut *this.stylePool);
    MemPool_Clear(&mut *this.clipRectPool);
    MemPool_Clear(&mut *this.cursorPool);
    MemPool_Clear(&mut *this.tex2DPool);
    MemPool_Clear(&mut *this.panelPool);
    MemPool_Clear(&mut *this.rectPool);
    MemPool_Clear(&mut *this.textPool);
    MemPool_Clear(&mut *this.linePool);

    this.style = std::ptr::null_mut();
    ImGui_PushDefaultStyle();
    this.layout = std::ptr::null_mut();
    ImGui_PushLayout(sx, sy, false);

    this.widget = std::ptr::null_mut();
    this.widgetLast = std::ptr::null_mut();
    ImGui_BeginWidget(sx, sy);

    this.layer = std::ptr::null_mut();
    ImGui_PushLayer(true);

    let mut mouse: IVec2 = IVec2::ZERO;
    Input_GetMousePosition(&mut mouse);
    this.mouse.x = mouse.x as f32;
    this.mouse.y = mouse.y as f32;

    this.activate = Input_GetPressed(Button_Mouse_Left);
    this.forceSize = Vec2::ZERO;
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_End() {
    ImGui_PopLayer();
    ImGui_EndWidget();
    ImGui_PopLayout();

    if !(this.layer).is_null() {
        panic!("ImGui_End: layer stack not empty");
    }
    if !(this.widget).is_null() {
        panic!("ImGui_End: widget stack not empty");
    }
    if !(this.layout).is_null() {
        panic!("ImGui_End: layout stack not empty");
    }
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_Draw() {
    RenderState_PushBlendMode(1);
    Draw_LineWidth(1.0f32);
    ImGui_DrawLayer(this.layerLast);
    RenderState_PopBlendMode();
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_AlignCursor(mut sx: f32, mut sy: f32, alignX: f32, alignY: f32) {
    TransformSize(&mut sx, &mut sy);
    ImGui_SetCursor(
        (*this.layout).lower.x + alignX * ((*this.layout).size.x - sx),
        (*this.layout).lower.y + alignY * ((*this.layout).size.y - sy),
    );
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_GetCursorX() -> f32 {
    this.cursor.x
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_GetCursorY() -> f32 {
    this.cursor.y
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_PushCursor() {
    let cursor: *mut ImGuiCursor = MemPool_Alloc(&mut *this.cursorPool) as *mut ImGuiCursor;
    (*cursor).prev = this.cursorStack;
    (*cursor).pos = this.cursor;
    this.cursorStack = cursor;
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_PopCursor() {
    let cursor: *mut ImGuiCursor = this.cursorStack;
    this.cursor = (*cursor).pos;
    this.cursorStack = (*cursor).prev;
    MemPool_Dealloc(&mut *this.cursorPool, cursor as *mut _);
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_SetCursor(mut cx: f32, mut cy: f32) {
    TransformPos(&mut cx, &mut cy);
    this.cursor = Vec2::new(cx, cy);
    (*this.layout).spacing = Vec2::ZERO;
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_SetCursorX(x: f32) {
    ImGui_SetCursor(x, this.cursor.y);
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_SetCursorY(y: f32) {
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
pub unsafe extern "C" fn ImGui_BeginGroup(sx: f32, sy: f32, horizontal: bool) {
    ImGui_BeginWidget(sx, sy);
    ImGui_PushLayout(sx, sy, horizontal);
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_BeginGroupX(sy: f32) {
    ImGui_BeginWidget(0.0f32, sy);
    ImGui_PushLayout(0.0f32, sy, true);
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_BeginGroupY(sx: f32) {
    ImGui_BeginWidget(sx, 0.0f32);
    ImGui_PushLayout(sx, 0.0f32, false);
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_EndGroup() {
    ImGui_PopLayout();
    ImGui_EndWidget();
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_BeginPanel(sx: f32, sy: f32) {
    ImGui_BeginGroup(sx, sy, false);
    ImGui_PushLayer(false);
    ImGui_PushLayer(true);
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
pub unsafe extern "C" fn ImGui_BeginWindow(_title: *const libc::c_char, sx: f32, sy: f32) {
    let hash: u64 = HashPeekNext();
    let data: *mut ImGuiData = GetData(hash);
    this.cursor.x += (*data).offset.x;
    this.cursor.y += (*data).offset.y;
    ImGui_BeginPanel(sx, sy);
    // ImGui_TextEx(self.style->fontSubheading, title, 0.4f, 0.4f, 0.4f, 1.0f);
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_EndWindow() {
    ImGui_EndPanel();
    let data: *mut ImGuiData = GetData((*this.widgetLast).hash);
    this.cursor.x -= (*data).offset.x;
    this.cursor.y -= (*data).offset.y;

    if ImGui_FocusLast(FocusType_Mouse) && Input_GetDown(Button_Mouse_Left) {
        let mut delta: IVec2 = IVec2::ZERO;

        Input_GetMouseDelta(&mut delta);

        (*data).offset.x += delta.x as f32;
        (*data).offset.y += delta.y as f32;

        this.dragging = (*this.widgetLast).hash;
    }
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_BeginScrollFrame(sx: f32, sy: f32) {
    ImGui_BeginGroup(sx, sy, false);
    ImGui_PushLayer(true);
    ImGui_Pad(1.0f32, 1.0f32);

    let data: *mut ImGuiData = GetData((*this.widget).hash);
    this.cursor.y -= (*data).scroll;
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_EndScrollFrame() {
    let data: *mut ImGuiData = GetData((*this.widget).hash);
    this.cursor.y += (*data).scroll;

    let layout: *mut ImGuiLayout = this.layout;
    ImGui_PopLayer();

    let scroll: f32 = (*data).scroll;
    let virtualSize: f32 = this.cursor.y - (*layout).lower.y;
    let scrollMax: f32 = virtualSize - (*layout).size.y;
    let scrollPos = Vec2::new((*layout).lower.x + (*layout).size.x, (*layout).lower.y);
    let _scrollSize = Vec2::new((*this.style).scrollBarSize.x, (*layout).size.y);

    let handleHash: u64 = HashNext();
    if (*layout).size.y < virtualSize {
        let mut handleSizeY: f32 = (*layout).size.y * ((*layout).size.y / virtualSize);
        handleSizeY = f32::clamp(handleSizeY, 16.0f32, 128.0f32);
        let handleOffset: f32 = ((*layout).size.y - handleSizeY) * (scroll / scrollMax);

        let handlePos = Vec2::new(scrollPos.x, scrollPos.y + handleOffset);
        let handleSize = Vec2::new((*this.style).scrollBarSize.x, handleSizeY);
        let handleFocus: bool = TryFocusRect(handleHash, FocusType_Mouse, handlePos, handleSize);

        EmitPanel(
            if handleFocus {
                (*this.style).buttonColorFocus
            } else {
                Vec4::new(0.3f32, 0.4f32, 0.5f32, 1.0f32)
            },
            handlePos,
            handleSize,
            if handleFocus { 0.5f32 } else { 0.25f32 },
            4.0f32,
        );
    }

    ImGui_Unpad(1.0f32, 1.0f32);
    ImGui_EndGroup();
    EmitPanel(
        Vec4::new(0.0f32, 0.0f32, 0.0f32, 0.5f32),
        (*this.widgetLast).pos,
        (*this.widgetLast).size,
        0.25f32,
        4.0f32,
    );

    if ImGui_FocusLast(FocusType_Scroll) {
        let mut scroll: IVec2 = IVec2::ZERO;
        Input_GetMouseScroll(&mut scroll);
        (*data).scroll -= 10.0f32 * scroll.y as f32;
    }

    (*data).scroll = f32::clamp((*data).scroll, 0.0f32, scrollMax);
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_SetNextWidth(sx: f32) {
    this.forceSize.x = sx;
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_SetNextHeight(sy: f32) {
    this.forceSize.y = sy;
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_PushStyle() {
    let style: *mut ImGuiStyle = MemPool_Alloc(&mut *this.stylePool) as *mut ImGuiStyle;
    MemCpy(
        style as *mut _,
        this.style as *const _,
        std::mem::size_of::<ImGuiStyle>(),
    );
    (*style).prev = this.style;
    this.style = style;
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_PushStyleFont(font: *mut Font) {
    ImGui_PushStyle();
    (*this.style).font = font;
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_PushStylePadding(px: f32, py: f32) {
    ImGui_PushStyle();
    (*this.style).padding = Vec2::new(px, py);
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_PushStyleSpacing(x: f32, y: f32) {
    ImGui_PushStyle();
    (*this.style).spacing = Vec2::new(x, y);
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_PushStyleTextColor(r: f32, g: f32, b: f32, a: f32) {
    ImGui_PushStyle();
    (*this.style).textColor = Vec4::new(r, g, b, a);
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_PopStyle() {
    if ((*this.style).prev).is_null() {
        panic!("ImGui_PopStyle: Attempting to pop an empty stack");
    }
    let style: *mut ImGuiStyle = this.style;
    this.style = (*style).prev;
    MemPool_Dealloc(&mut *this.stylePool, style as *mut _);
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_SetFont(font: *mut Font) {
    ImGui_PushStyleFont(font);
    (*this.layout).styleVars += 1;
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_SetSpacing(sx: f32, sy: f32) {
    ImGui_PushStyleSpacing(sx, sy);
    (*this.layout).styleVars += 1;
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_Button(label: *const libc::c_char) -> bool {
    ImGui_ButtonEx(label, 0.0f32, 32.0f32)
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_ButtonEx(label: *const libc::c_char, sx: f32, sy: f32) -> bool {
    ImGui_BeginWidget(sx, sy);
    let focus: bool = ImGui_FocusCurrent(FocusType_Mouse);
    let color: Vec4 = if focus {
        (*this.style).buttonColorFocus
    } else {
        (*this.style).buttonColor
    };
    EmitPanel(
        color,
        (*this.widget).pos,
        (*this.widget).size,
        if focus { 1.0f32 } else { 0.5f32 },
        4.0f32,
    );

    let mut bound: IVec2 = IVec2::ZERO;
    Font_GetSize2(&mut *(*this.style).font, &mut bound, label);
    let labelPos = Vec2::new(
        (*this.widget).pos.x + 0.5f32 * ((*this.widget).size.x - bound.x as f32),
        (*this.widget).pos.y + 0.5f32 * ((*this.widget).size.y - bound.y as f32),
    );
    let _labelSize = Vec2::new(bound.x as f32, bound.y as f32);

    EmitText(
        (*this.style).font,
        if focus {
            (*this.style).textColorFocus
        } else {
            (*this.style).textColor
        },
        Vec2::new(labelPos.x, labelPos.y + bound.y as f32),
        label,
    );

    ImGui_EndWidget();
    focus && this.activate
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_Checkbox(mut value: bool) -> bool {
    ImGui_BeginWidget(16.0f32, 16.0f32);
    let focus: bool = ImGui_FocusCurrent(FocusType_Mouse);
    if focus && this.activate {
        value = !value;
    }
    if focus {
        EmitRect(
            (*this.style).buttonColorFocus,
            (*this.widget).pos,
            (*this.widget).size,
            true,
        );
    }

    EmitPanel(
        if value {
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
    value
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_Divider() {
    ImGui_BeginWidget(
        if (*this.layout).horizontal {
            2.0f32
        } else {
            0.0f32
        },
        if (*this.layout).horizontal {
            0.0f32
        } else {
            2.0f32
        },
    );
    EmitLine(
        (*this.style).buttonColorFocus,
        (*this.widget).pos,
        Vec2::new(
            (*this.widget).pos.x
                + (if (*this.layout).horizontal {
                    0.0f32
                } else {
                    (*this.widget).size.x
                }),
            (*this.widget).pos.y
                + (if (*this.layout).horizontal {
                    (*this.widget).size.y
                } else {
                    0.0f32
                }),
        ),
    );
    ImGui_EndWidget();
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_Selectable(label: *const libc::c_char) -> bool {
    let mut bound: IVec2 = IVec2::ZERO;
    Font_GetSize2(&mut *(*this.style).font, &mut bound, label);
    ImGui_BeginWidget(
        if (*this.layout).horizontal {
            bound.x as f32 + 4.0f32
        } else {
            0.0f32
        },
        if (*this.layout).horizontal {
            0.0f32
        } else {
            4.0f32 + Font_GetLineHeight(&mut *(*this.style).font) as f32
        },
    );

    let focus: bool = ImGui_FocusCurrent(FocusType_Mouse);
    if focus {
        EmitRect(
            (*this.style).buttonColorFocus,
            (*this.widget).pos,
            (*this.widget).size,
            false,
        );
    }

    EmitText(
        (*this.style).font,
        if focus {
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
    focus && this.activate
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_Tex2D(tex: *mut Tex2D) {
    let mut size: IVec2 = IVec2::ZERO;
    Tex2D_GetSize(&mut *tex, &mut size);
    let sizef = Vec2::new(size.x as f32, size.y as f32);
    ImGui_BeginWidget(size.x as f32, size.y as f32);
    EmitTex2D(tex, this.cursor, sizef);
    ImGui_EndWidget();
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_Text(text: *const libc::c_char) {
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
    text: *const libc::c_char,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) {
    ImGui_TextEx((*this.style).font, text, r, g, b, a);
}

#[no_mangle]
pub unsafe extern "C" fn ImGui_TextEx(
    font: *mut Font,
    text: *const libc::c_char,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) {
    let mut bound: IVec2 = IVec2::ZERO;
    Font_GetSize2(&mut *(*this.style).font, &mut bound, text);
    ImGui_BeginWidget(
        bound.x as f32,
        if (*this.layout).horizontal {
            0.0f32
        } else {
            Font_GetLineHeight(&mut *(*this.style).font) as f32
        },
    );
    EmitText(
        font,
        Vec4::new(r, g, b, a),
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
