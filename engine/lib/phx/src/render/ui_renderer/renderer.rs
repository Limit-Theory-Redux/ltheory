use super::*;
use crate::common::*;
use crate::math::*;
use crate::render::*;
use crate::system::*;

#[derive(Clone)]
pub struct UIRenderer {
    root: *mut UIRendererLayer,
    layer: *mut UIRendererLayer,
    layer_pool: *mut MemPool,
    image_pool: *mut MemPool,
    panel_pool: *mut MemPool,
    rect_pool: *mut MemPool,
    text_pool: *mut MemPool,
}

impl Default for UIRenderer {
    fn default() -> Self {
        unsafe {
            Self {
                root: std::ptr::null_mut(),
                layer: std::ptr::null_mut(),
                layer_pool: MemPool_CreateAuto(std::mem::size_of::<UIRendererLayer>() as u32),
                image_pool: MemPool_CreateAuto(std::mem::size_of::<UIRendererImage>() as u32),
                panel_pool: MemPool_CreateAuto(std::mem::size_of::<UIRendererPanel>() as u32),
                rect_pool: MemPool_CreateAuto(std::mem::size_of::<UIRendererRect>() as u32),
                text_pool: MemPool_CreateAuto(std::mem::size_of::<UIRendererText>() as u32),
            }
        }
    }
}

// #[luajit_ffi_gen::luajit_ffi]
impl UIRenderer {
    pub fn begin(&mut self) {
        unsafe {
            self.root = std::ptr::null_mut();
            self.layer = std::ptr::null_mut();

            MemPool_Clear(&mut *self.layer_pool);
            MemPool_Clear(&mut *self.image_pool);
            MemPool_Clear(&mut *self.panel_pool);
            MemPool_Clear(&mut *self.rect_pool);
            MemPool_Clear(&mut *self.text_pool);

            let mut vp = IVec2::ZERO;
            Viewport_GetSize(&mut vp);

            self.begin_layer(Vec2::ZERO, Vec2::new(vp.x as f32, vp.y as f32), true);

            self.root = self.layer;
        }
    }

    pub fn end(&mut self) {
        self.end_layer();
    }

    pub fn draw(&self) {
        unsafe {
            RenderState_PushBlendMode(1);

            (&*self.root).draw();

            RenderState_PopBlendMode();
        }
    }

    pub fn begin_layer(&mut self, pos: Vec2, size: Vec2, clip: bool) {
        unsafe {
            let layer = MemPool_Alloc(&mut *self.layer_pool) as *mut UIRendererLayer;

            (*layer).parent = self.layer;
            (*layer).next = std::ptr::null_mut();
            (*layer).children = std::ptr::null_mut();
            (*layer).pos = pos;
            (*layer).size = size;
            (*layer).clip = clip;
            (*layer).image_list = std::ptr::null_mut();
            (*layer).panel_list = std::ptr::null_mut();
            (*layer).rect_list = std::ptr::null_mut();
            (*layer).text_list = std::ptr::null_mut();

            self.layer = layer;
        }
    }

    pub fn end_layer(&mut self) {
        unsafe {
            if !((*self.layer).parent).is_null() {
                (*self.layer).next = (*(*self.layer).parent).children;
                (*(*self.layer).parent).children = self.layer;
            }
            self.layer = (*self.layer).parent;
        }
    }

    pub fn image(&self, image: *mut Tex2D, pos: Vec2, size: Vec2) {
        unsafe {
            let e = MemPool_Alloc(&mut *self.image_pool) as *mut UIRendererImage;
            (*e).next = (*self.layer).image_list;
            (*e).image = image;
            (*e).pos = pos;
            (*e).size = size;

            (*self.layer).image_list = e;
        }
    }

    pub fn panel(&self, pos: Vec2, size: Vec2, color: Vec4, bevel: f32, innerAlpha: f32) {
        unsafe {
            let e = MemPool_Alloc(&mut *self.panel_pool) as *mut UIRendererPanel;
            (*e).next = (*self.layer).panel_list;
            (*e).pos = pos;
            (*e).size = size;
            (*e).color = color;
            (*e).bevel = bevel;
            (*e).inner_alpha = innerAlpha;

            (*self.layer).panel_list = e;
        }
    }

    pub fn rect(&self, pos: Vec2, size: Vec2, color: Vec4, outline: bool) {
        unsafe {
            let e = MemPool_Alloc(&mut *self.rect_pool) as *mut UIRendererRect;
            (*e).next = (*self.layer).rect_list;
            (*e).pos = pos;
            (*e).size = size;
            (*e).color = color;
            (*e).outline = outline;

            (*self.layer).rect_list = e;
        }
    }

    pub fn text(&self, font: &Font, text: &str, pos: Vec2, color: Vec4) {
        unsafe {
            let e = MemPool_Alloc(&mut *self.text_pool) as *mut UIRendererText;
            (*e).next = (*self.layer).text_list;
            (*e).font = font as _;
            (*e).text = text.into();
            (*e).pos = pos;
            (*e).color = color;

            (*self.layer).text_list = e;
        }
    }
}
