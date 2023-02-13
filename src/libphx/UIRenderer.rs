use ::libc;
use super::internal::Memory::*;
extern "C" {
    pub type Font;
    pub type MemPool;
    pub type Shader;
    pub type Tex2D;
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
    fn Draw_Color(
        r: libc::c_float,
        g: libc::c_float,
        b: libc::c_float,
        a: libc::c_float,
    );
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
    fn MemPool_CreateAuto(elemSize: uint32) -> *mut MemPool;
    fn MemPool_Alloc(_: *mut MemPool) -> *mut libc::c_void;
    fn MemPool_Clear(_: *mut MemPool);
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
    fn Viewport_GetSize(out: *mut Vec2i);
}
pub type int32_t = libc::c_int;
pub type uint32_t = libc::c_uint;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint32 = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec2i {
    pub x: libc::c_int,
    pub y: libc::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec2f {
    pub x: libc::c_float,
    pub y: libc::c_float,
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
    pub pos: Vec2f,
    pub size: Vec2f,
    pub clip: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UIRendererText {
    pub next: *mut UIRendererText,
    pub font: *mut Font,
    pub text: cstr,
    pub pos: Vec2f,
    pub color: Vec4f,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UIRendererRect {
    pub next: *mut UIRendererRect,
    pub pos: Vec2f,
    pub size: Vec2f,
    pub color: Vec4f,
    pub outline: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UIRendererPanel {
    pub next: *mut UIRendererPanel,
    pub pos: Vec2f,
    pub size: Vec2f,
    pub color: Vec4f,
    pub bevel: libc::c_float,
    pub innerAlpha: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UIRendererImage {
    pub next: *mut UIRendererImage,
    pub image: *mut Tex2D,
    pub pos: Vec2f,
    pub size: Vec2f,
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
#[inline]
unsafe extern "C" fn Vec2f_Create(mut x: libc::c_float, mut y: libc::c_float) -> Vec2f {
    let mut self_1: Vec2f = {
        let mut init = Vec2f { x: x, y: y };
        init
    };
    return self_1;
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
static mut self_0: UIRenderer = {
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
    self_0.root = 0 as *mut UIRendererLayer;
    self_0.layer = 0 as *mut UIRendererLayer;
    self_0
        .layerPool = MemPool_CreateAuto(
        ::core::mem::size_of::<UIRendererLayer>() as usize as uint32,
    );
    self_0
        .imagePool = MemPool_CreateAuto(
        ::core::mem::size_of::<UIRendererImage>() as usize as uint32,
    );
    self_0
        .panelPool = MemPool_CreateAuto(
        ::core::mem::size_of::<UIRendererPanel>() as usize as uint32,
    );
    self_0
        .rectPool = MemPool_CreateAuto(
        ::core::mem::size_of::<UIRendererRect>() as usize as uint32,
    );
    self_0
        .textPool = MemPool_CreateAuto(
        ::core::mem::size_of::<UIRendererText>() as usize as uint32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn UIRenderer_Begin() {
    UIRenderer_Init();
    self_0.root = 0 as *mut UIRendererLayer;
    self_0.layer = 0 as *mut UIRendererLayer;
    MemPool_Clear(self_0.layerPool);
    MemPool_Clear(self_0.imagePool);
    MemPool_Clear(self_0.panelPool);
    MemPool_Clear(self_0.rectPool);
    MemPool_Clear(self_0.textPool);
    let mut vp: Vec2i = Vec2i { x: 0, y: 0 };
    Viewport_GetSize(&mut vp);
    UIRenderer_BeginLayer(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
        vp.x as libc::c_float,
        vp.y as libc::c_float,
        1 as libc::c_int != 0,
    );
    self_0.root = self_0.layer;
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
        let pad: libc::c_float = 64.0f32;
        Shader_Start(shader);
        Shader_SetFloat(b"padding\0" as *const u8 as *const libc::c_char, pad);
        let mut e: *const UIRendererPanel = (*self_1).panelList;
        while !e.is_null() {
            let mut x: libc::c_float = (*e).pos.x - pad;
            let mut y: libc::c_float = (*e).pos.y - pad;
            let mut sx: libc::c_float = (*e).size.x + 2.0f32 * pad;
            let mut sy: libc::c_float = (*e).size.y + 2.0f32 * pad;
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
    UIRenderer_DrawLayer(self_0.root);
    RenderState_PopBlendMode();
}
#[no_mangle]
pub unsafe extern "C" fn UIRenderer_BeginLayer(
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut sx: libc::c_float,
    mut sy: libc::c_float,
    mut clip: bool,
) {
    let mut layer: *mut UIRendererLayer = MemPool_Alloc(self_0.layerPool)
        as *mut UIRendererLayer;
    (*layer).parent = self_0.layer;
    (*layer).next = 0 as *mut UIRendererLayer;
    (*layer).children = 0 as *mut UIRendererLayer;
    (*layer).pos = Vec2f_Create(x, y);
    (*layer).size = Vec2f_Create(sx, sy);
    (*layer).clip = clip;
    (*layer).imageList = 0 as *mut UIRendererImage;
    (*layer).panelList = 0 as *mut UIRendererPanel;
    (*layer).rectList = 0 as *mut UIRendererRect;
    (*layer).textList = 0 as *mut UIRendererText;
    self_0.layer = layer;
}
#[no_mangle]
pub unsafe extern "C" fn UIRenderer_EndLayer() {
    if !((*self_0.layer).parent).is_null() {
        (*self_0.layer).next = (*(*self_0.layer).parent).children;
        (*(*self_0.layer).parent).children = self_0.layer;
    }
    self_0.layer = (*self_0.layer).parent;
}
#[no_mangle]
pub unsafe extern "C" fn UIRenderer_Image(
    mut image: *mut Tex2D,
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut sx: libc::c_float,
    mut sy: libc::c_float,
) {
    let mut e: *mut UIRendererImage = MemPool_Alloc(self_0.imagePool)
        as *mut UIRendererImage;
    (*e).next = (*self_0.layer).imageList;
    (*e).image = image;
    (*e).pos = Vec2f_Create(x, y);
    (*e).size = Vec2f_Create(sx, sy);
    (*self_0.layer).imageList = e;
}
#[no_mangle]
pub unsafe extern "C" fn UIRenderer_Panel(
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut sx: libc::c_float,
    mut sy: libc::c_float,
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
    mut bevel: libc::c_float,
    mut innerAlpha: libc::c_float,
) {
    let mut e: *mut UIRendererPanel = MemPool_Alloc(self_0.panelPool)
        as *mut UIRendererPanel;
    (*e).next = (*self_0.layer).panelList;
    (*e).pos = Vec2f_Create(x, y);
    (*e).size = Vec2f_Create(sx, sy);
    (*e).color = Vec4f_Create(r, g, b, a);
    (*e).bevel = bevel;
    (*e).innerAlpha = innerAlpha;
    (*self_0.layer).panelList = e;
}
#[no_mangle]
pub unsafe extern "C" fn UIRenderer_Rect(
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut sx: libc::c_float,
    mut sy: libc::c_float,
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
    mut outline: bool,
) {
    let mut e: *mut UIRendererRect = MemPool_Alloc(self_0.rectPool)
        as *mut UIRendererRect;
    (*e).next = (*self_0.layer).rectList;
    (*e).pos = Vec2f_Create(x, y);
    (*e).size = Vec2f_Create(sx, sy);
    (*e).color = Vec4f_Create(r, g, b, a);
    (*e).outline = outline;
    (*self_0.layer).rectList = e;
}
#[no_mangle]
pub unsafe extern "C" fn UIRenderer_Text(
    mut font: *mut Font,
    mut text: cstr,
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
) {
    let mut e: *mut UIRendererText = MemPool_Alloc(self_0.textPool)
        as *mut UIRendererText;
    (*e).next = (*self_0.layer).textList;
    (*e).font = font;
    (*e).text = text;
    (*e).pos = Vec2f_Create(x, y);
    (*e).color = Vec4f_Create(r, g, b, a);
    (*self_0.layer).textList = e;
}
