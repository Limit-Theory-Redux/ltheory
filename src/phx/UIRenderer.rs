
use crate::phx::ClipRect::*;
use crate::phx::Common::*;
use crate::phx::Draw::*;
use crate::phx::Font::*;

use crate::phx::Math::{IVec2, Vec2, Vec4};
use crate::phx::MemPool::*;
use crate::phx::RenderState::*;
use crate::phx::Shader::*;
use crate::phx::Tex2D::*;
use crate::phx::Viewport::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct UIRendererLayer {
    pub parent: *mut UIRendererLayer,
    pub next: *mut UIRendererLayer,
    pub children: *mut UIRendererLayer,
    pub imageList: *mut UIRendererImage,
    pub panelList: *mut UIRendererPanel,
    pub rectList: *mut UIRendererRect,
    pub textList: *mut UIRendererText,
    pub pos: Vec2,
    pub size: Vec2,
    pub clip: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct UIRendererText {
    pub next: *mut UIRendererText,
    pub font: *mut Font,
    pub text: *const libc::c_char,
    pub pos: Vec2,
    pub color: Vec4,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct UIRendererRect {
    pub next: *mut UIRendererRect,
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Vec4,
    pub outline: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct UIRendererPanel {
    pub next: *mut UIRendererPanel,
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Vec4,
    pub bevel: f32,
    pub innerAlpha: f32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct UIRendererImage {
    pub next: *mut UIRendererImage,
    pub image: *mut Tex2D,
    pub pos: Vec2,
    pub size: Vec2,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct UIRenderer {
    pub root: *mut UIRendererLayer,
    pub layer: *mut UIRendererLayer,
    pub layerPool: *mut MemPool,
    pub imagePool: *mut MemPool,
    pub panelPool: *mut MemPool,
    pub rectPool: *mut MemPool,
    pub textPool: *mut MemPool,
}

static mut this: UIRenderer = UIRenderer {
    root: std::ptr::null_mut(),
    layer: std::ptr::null_mut(),
    layerPool: std::ptr::null_mut(),
    imagePool: std::ptr::null_mut(),
    panelPool: std::ptr::null_mut(),
    rectPool: std::ptr::null_mut(),
    textPool: std::ptr::null_mut(),
};

unsafe extern "C" fn UIRenderer_Init() {
    static mut init: bool = false;
    if init {
        return;
    }
    init = true;
    this.root = std::ptr::null_mut();
    this.layer = std::ptr::null_mut();
    this.layerPool = MemPool_CreateAuto(std::mem::size_of::<UIRendererLayer>() as u32);
    this.imagePool = MemPool_CreateAuto(std::mem::size_of::<UIRendererImage>() as u32);
    this.panelPool = MemPool_CreateAuto(std::mem::size_of::<UIRendererPanel>() as u32);
    this.rectPool = MemPool_CreateAuto(std::mem::size_of::<UIRendererRect>() as u32);
    this.textPool = MemPool_CreateAuto(std::mem::size_of::<UIRendererText>() as u32);
}

#[no_mangle]
pub unsafe extern "C" fn UIRenderer_Begin() {
    UIRenderer_Init();
    this.root = std::ptr::null_mut();
    this.layer = std::ptr::null_mut();
    MemPool_Clear(&mut *this.layerPool);
    MemPool_Clear(&mut *this.imagePool);
    MemPool_Clear(&mut *this.panelPool);
    MemPool_Clear(&mut *this.rectPool);
    MemPool_Clear(&mut *this.textPool);
    let mut vp: IVec2 = IVec2::ZERO;
    Viewport_GetSize(&mut vp);
    UIRenderer_BeginLayer(0.0f32, 0.0f32, vp.x as f32, vp.y as f32, true);
    this.root = this.layer;
}

#[no_mangle]
pub unsafe extern "C" fn UIRenderer_End() {
    UIRenderer_EndLayer();
}

unsafe extern "C" fn UIRenderer_DrawLayer(self_1: *const UIRendererLayer) {
    if (*self_1).clip {
        ClipRect_PushCombined(
            (*self_1).pos.x,
            (*self_1).pos.y,
            (*self_1).size.x,
            (*self_1).size.y,
        );
    }
    if !((*self_1).panelList).is_null() {
        static mut shader: *mut Shader = std::ptr::null_mut();
        if shader.is_null() {
            shader = Shader_Load(c_str!("vertex/ui"), c_str!("fragment/ui/panel"));
        }
        let pad: f32 = 64.0f32;
        Shader_Start(&mut *shader);
        Shader_SetFloat(c_str!("padding"), pad);
        let mut e: *const UIRendererPanel = (*self_1).panelList;
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
    let mut e_0: *const UIRendererImage = (*self_1).imageList;
    while !e_0.is_null() {
        Tex2D_Draw(
            &mut *(*e_0).image,
            (*e_0).pos.x,
            (*e_0).pos.y,
            (*e_0).size.x,
            (*e_0).size.y,
        );
        e_0 = (*e_0).next;
    }
    let mut e_1: *const UIRendererRect = (*self_1).rectList;
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
    let mut e_2: *const UIRendererText = (*self_1).textList;
    while !e_2.is_null() {
        Font_Draw(
            &mut *(*e_2).font,
            (*e_2).text,
            (*e_2).pos.x,
            (*e_2).pos.y,
            (*e_2).color.x,
            (*e_2).color.y,
            (*e_2).color.z,
            (*e_2).color.w,
        );
        e_2 = (*e_2).next;
    }
    let mut e_3: *const UIRendererLayer = (*self_1).children;
    while !e_3.is_null() {
        UIRenderer_DrawLayer(e_3);
        e_3 = (*e_3).next;
    }
    if (*self_1).clip {
        ClipRect_Pop();
    }
}

#[no_mangle]
pub unsafe extern "C" fn UIRenderer_Draw() {
    RenderState_PushBlendMode(1);
    UIRenderer_DrawLayer(this.root);
    RenderState_PopBlendMode();
}

#[no_mangle]
pub unsafe extern "C" fn UIRenderer_BeginLayer(x: f32, y: f32, sx: f32, sy: f32, clip: bool) {
    let layer: *mut UIRendererLayer = MemPool_Alloc(&mut *this.layerPool) as *mut UIRendererLayer;
    (*layer).parent = this.layer;
    (*layer).next = std::ptr::null_mut();
    (*layer).children = std::ptr::null_mut();
    (*layer).pos = Vec2::new(x, y);
    (*layer).size = Vec2::new(sx, sy);
    (*layer).clip = clip;
    (*layer).imageList = std::ptr::null_mut();
    (*layer).panelList = std::ptr::null_mut();
    (*layer).rectList = std::ptr::null_mut();
    (*layer).textList = std::ptr::null_mut();
    this.layer = layer;
}

#[no_mangle]
pub unsafe extern "C" fn UIRenderer_EndLayer() {
    if !((*this.layer).parent).is_null() {
        (*this.layer).next = (*(*this.layer).parent).children;
        (*(*this.layer).parent).children = this.layer;
    }
    this.layer = (*this.layer).parent;
}

#[no_mangle]
pub unsafe extern "C" fn UIRenderer_Image(image: *mut Tex2D, x: f32, y: f32, sx: f32, sy: f32) {
    let e: *mut UIRendererImage = MemPool_Alloc(&mut *this.imagePool) as *mut UIRendererImage;
    (*e).next = (*this.layer).imageList;
    (*e).image = image;
    (*e).pos = Vec2::new(x, y);
    (*e).size = Vec2::new(sx, sy);
    (*this.layer).imageList = e;
}

#[no_mangle]
pub unsafe extern "C" fn UIRenderer_Panel(
    x: f32,
    y: f32,
    sx: f32,
    sy: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
    bevel: f32,
    innerAlpha: f32,
) {
    let e: *mut UIRendererPanel = MemPool_Alloc(&mut *this.panelPool) as *mut UIRendererPanel;
    (*e).next = (*this.layer).panelList;
    (*e).pos = Vec2::new(x, y);
    (*e).size = Vec2::new(sx, sy);
    (*e).color = Vec4::new(r, g, b, a);
    (*e).bevel = bevel;
    (*e).innerAlpha = innerAlpha;
    (*this.layer).panelList = e;
}

#[no_mangle]
pub unsafe extern "C" fn UIRenderer_Rect(
    x: f32,
    y: f32,
    sx: f32,
    sy: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
    outline: bool,
) {
    let e: *mut UIRendererRect = MemPool_Alloc(&mut *this.rectPool) as *mut UIRendererRect;
    (*e).next = (*this.layer).rectList;
    (*e).pos = Vec2::new(x, y);
    (*e).size = Vec2::new(sx, sy);
    (*e).color = Vec4::new(r, g, b, a);
    (*e).outline = outline;
    (*this.layer).rectList = e;
}

#[no_mangle]
pub unsafe extern "C" fn UIRenderer_Text(
    font: *mut Font,
    text: *const libc::c_char,
    x: f32,
    y: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) {
    let e: *mut UIRendererText = MemPool_Alloc(&mut *this.textPool) as *mut UIRendererText;
    (*e).next = (*this.layer).textList;
    (*e).font = font;
    (*e).text = text;
    (*e).pos = Vec2::new(x, y);
    (*e).color = Vec4::new(r, g, b, a);
    (*this.layer).textList = e;
}
