use ::libc;
use glam::Vec3;
use glam::{IVec2, Vec2, Vec4};
use crate::internal::Memory::*;

extern "C" {
    pub type Font;
    pub type MemPool;
    pub type Shader;
    pub type Tex2D;
    fn ClipRect_PushCombined(
        x: f32,
        y: f32,
        sx: f32,
        sy: f32,
    );
    fn ClipRect_Pop();
    fn Draw_Rect(
        x: f32,
        y: f32,
        sx: f32,
        sy: f32,
    );
    fn Draw_Border(
        s: f32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    );
    fn Draw_Color(
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    );
    fn Font_Draw(
        _: *mut Font,
        text: cstr,
        x: f32,
        y: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    );
    fn MemPool_CreateAuto(elemSize: uint32) -> *mut MemPool;
    fn MemPool_Alloc(_: *mut MemPool) -> *mut libc::c_void;
    fn MemPool_Clear(_: *mut MemPool);
    fn RenderState_PushBlendMode(_: BlendMode);
    fn RenderState_PopBlendMode();
    fn Shader_Load(vertName: cstr, fragName: cstr) -> *mut Shader;
    fn Shader_Start(_: *mut Shader);
    fn Shader_Stop(_: *mut Shader);
    fn Shader_SetFloat(_: cstr, _: f32);
    fn Shader_SetFloat2(_: cstr, _: f32, _: f32);
    fn Shader_SetFloat4(
        _: cstr,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
    );
    fn Tex2D_Draw(
        _: *mut Tex2D,
        x: f32,
        y: f32,
        sx: f32,
        sy: f32,
    );
    fn Viewport_GetSize(out: *mut IVec2);
}
pub type int32_t = libc::c_int;
pub type uint32_t = libc::c_uint;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint32 = uint32_t;

pub type BlendMode = int32;
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
    pub text: cstr,
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

static mut this: UIRenderer = {
    let mut init = UIRenderer {
        root: 0 as *const UIRendererLayer as *mut UIRendererLayer,
        layer: 0 as *const UIRendererLayer as *mut UIRendererLayer,
        layerPool: 0 as *const MemPool as *mut MemPool,
        imagePool: 0 as *const MemPool as *mut MemPool,
        panelPool: 0 as *const MemPool as *mut MemPool,
        rectPool: 0 as *const MemPool as *mut MemPool,
        textPool: 0 as *const MemPool as *mut MemPool,
    };
    init
};
unsafe extern "C" fn UIRenderer_Init() {
    static mut init: bool = 0 as libc::c_int != 0;
    if init {
        return;
    }
    init = 1 as libc::c_int != 0;
    this.root = 0 as *mut UIRendererLayer;
    this.layer = 0 as *mut UIRendererLayer;
    this
        .layerPool = MemPool_CreateAuto(
        ::core::mem::size_of::<UIRendererLayer>() as usize as uint32,
    );
    this
        .imagePool = MemPool_CreateAuto(
        ::core::mem::size_of::<UIRendererImage>() as usize as uint32,
    );
    this
        .panelPool = MemPool_CreateAuto(
        ::core::mem::size_of::<UIRendererPanel>() as usize as uint32,
    );
    this
        .rectPool = MemPool_CreateAuto(
        ::core::mem::size_of::<UIRendererRect>() as usize as uint32,
    );
    this
        .textPool = MemPool_CreateAuto(
        ::core::mem::size_of::<UIRendererText>() as usize as uint32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn UIRenderer_Begin() {
    UIRenderer_Init();
    this.root = 0 as *mut UIRendererLayer;
    this.layer = 0 as *mut UIRendererLayer;
    MemPool_Clear(this.layerPool);
    MemPool_Clear(this.imagePool);
    MemPool_Clear(this.panelPool);
    MemPool_Clear(this.rectPool);
    MemPool_Clear(this.textPool);
    let mut vp: IVec2 = IVec2 { x: 0, y: 0 };
    Viewport_GetSize(&mut vp);
    UIRenderer_BeginLayer(
        0 as libc::c_int as f32,
        0 as libc::c_int as f32,
        vp.x as f32,
        vp.y as f32,
        1 as libc::c_int != 0,
    );
    this.root = this.layer;
}
#[no_mangle]
pub unsafe extern "C" fn UIRenderer_End() {
    UIRenderer_EndLayer();
}
unsafe extern "C" fn UIRenderer_DrawLayer(mut self_1: *const UIRendererLayer) {
    if (*self_1).clip {
        ClipRect_PushCombined(
            (*self_1).pos.x,
            (*self_1).pos.y,
            (*self_1).size.x,
            (*self_1).size.y,
        );
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
        let mut e: *const UIRendererPanel = (*self_1).panelList;
        while !e.is_null() {
            let mut x: f32 = (*e).pos.x - pad;
            let mut y: f32 = (*e).pos.y - pad;
            let mut sx: f32 = (*e).size.x + 2.0f32 * pad;
            let mut sy: f32 = (*e).size.y + 2.0f32 * pad;
            Shader_SetFloat(
                b"innerAlpha\0" as *const u8 as *const libc::c_char,
                (*e).innerAlpha,
            );
            Shader_SetFloat(b"bevel\0" as *const u8 as *const libc::c_char, (*e).bevel);
            Shader_SetFloat2(b"size\0" as *const u8 as *const libc::c_char, sx, sy);
            Shader_SetFloat4(
                b"color\0" as *const u8 as *const libc::c_char,
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
            (*e_0).image,
            (*e_0).pos.x,
            (*e_0).pos.y,
            (*e_0).size.x,
            (*e_0).size.y,
        );
        e_0 = (*e_0).next;
    }
    let mut e_1: *const UIRendererRect = (*self_1).rectList;
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
    let mut e_2: *const UIRendererText = (*self_1).textList;
    while !e_2.is_null() {
        Font_Draw(
            (*e_2).font,
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
    RenderState_PushBlendMode(1 as libc::c_int);
    UIRenderer_DrawLayer(this.root);
    RenderState_PopBlendMode();
}
#[no_mangle]
pub unsafe extern "C" fn UIRenderer_BeginLayer(
    mut x: f32,
    mut y: f32,
    mut sx: f32,
    mut sy: f32,
    mut clip: bool,
) {
    let mut layer: *mut UIRendererLayer = MemPool_Alloc(this.layerPool)
        as *mut UIRendererLayer;
    (*layer).parent = this.layer;
    (*layer).next = 0 as *mut UIRendererLayer;
    (*layer).children = 0 as *mut UIRendererLayer;
    (*layer).pos = Vec2::new(x, y);
    (*layer).size = Vec2::new(sx, sy);
    (*layer).clip = clip;
    (*layer).imageList = 0 as *mut UIRendererImage;
    (*layer).panelList = 0 as *mut UIRendererPanel;
    (*layer).rectList = 0 as *mut UIRendererRect;
    (*layer).textList = 0 as *mut UIRendererText;
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
pub unsafe extern "C" fn UIRenderer_Image(
    mut image: *mut Tex2D,
    mut x: f32,
    mut y: f32,
    mut sx: f32,
    mut sy: f32,
) {
    let mut e: *mut UIRendererImage = MemPool_Alloc(this.imagePool)
        as *mut UIRendererImage;
    (*e).next = (*this.layer).imageList;
    (*e).image = image;
    (*e).pos = Vec2::new(x, y);
    (*e).size = Vec2::new(sx, sy);
    (*this.layer).imageList = e;
}
#[no_mangle]
pub unsafe extern "C" fn UIRenderer_Panel(
    mut x: f32,
    mut y: f32,
    mut sx: f32,
    mut sy: f32,
    mut r: f32,
    mut g: f32,
    mut b: f32,
    mut a: f32,
    mut bevel: f32,
    mut innerAlpha: f32,
) {
    let mut e: *mut UIRendererPanel = MemPool_Alloc(this.panelPool)
        as *mut UIRendererPanel;
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
    mut x: f32,
    mut y: f32,
    mut sx: f32,
    mut sy: f32,
    mut r: f32,
    mut g: f32,
    mut b: f32,
    mut a: f32,
    mut outline: bool,
) {
    let mut e: *mut UIRendererRect = MemPool_Alloc(this.rectPool)
        as *mut UIRendererRect;
    (*e).next = (*this.layer).rectList;
    (*e).pos = Vec2::new(x, y);
    (*e).size = Vec2::new(sx, sy);
    (*e).color = Vec4::new(r, g, b, a);
    (*e).outline = outline;
    (*this.layer).rectList = e;
}
#[no_mangle]
pub unsafe extern "C" fn UIRenderer_Text(
    mut font: *mut Font,
    mut text: cstr,
    mut x: f32,
    mut y: f32,
    mut r: f32,
    mut g: f32,
    mut b: f32,
    mut a: f32,
) {
    let mut e: *mut UIRendererText = MemPool_Alloc(this.textPool)
        as *mut UIRendererText;
    (*e).next = (*this.layer).textList;
    (*e).font = font;
    (*e).text = text;
    (*e).pos = Vec2::new(x, y);
    (*e).color = Vec4::new(r, g, b, a);
    (*this.layer).textList = e;
}
