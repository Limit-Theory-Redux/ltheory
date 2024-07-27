use super::*;
use crate::math::*;
use crate::render::*;

pub struct UIRenderer {
    panel_shader: Shader,
    image_shader: Shader,
    rect_shader: Shader,

    current_layer_id: Option<UIRendererLayerId>,

    layers: Vec<UIRendererLayer>,
    images: Vec<UIRendererImage>,
    panels: Vec<UIRendererPanel>,
    rects: Vec<UIRendererRect>,
    texts: Vec<UIRendererText>,
}

impl UIRenderer {
    pub fn new() -> UIRenderer {
        UIRenderer {
            panel_shader: Shader::load("vertex/ui", "fragment/ui/panel"),
            image_shader: Shader::load("vertex/ui", "fragment/simple_image"),
            rect_shader: Shader::load("vertex/ui", "fragment/simple_color"),
            current_layer_id: None,
            layers: Vec::new(),
            images: Vec::new(),
            panels: Vec::new(),
            rects: Vec::new(),
            texts: Vec::new(),
        }
    }

    pub fn begin(&mut self) {
        self.current_layer_id = None;

        self.layers.clear();
        self.images.clear();
        self.panels.clear();
        self.rects.clear();
        self.texts.clear();

        let mut vp = IVec2::ZERO;
        unsafe { Viewport_GetSize(&mut vp) };

        self.begin_layer(Vec2::ZERO, Vec2::new(vp.x as f32, vp.y as f32), true);
    }

    pub fn end(&mut self) {
        self.end_layer();
    }

    pub fn draw(&mut self) {
        unsafe { RenderState_PushBlendMode(BlendMode::Alpha) };

        if let Some(root) = self.layers.first() {
            root.draw(
                &mut self.panel_shader,
                &mut self.image_shader,
                &mut self.rect_shader,
                &self.layers,
                &self.images,
                &self.panels,
                &self.rects,
                &self.texts,
            );
        } else {
            unreachable!("No layers defined");
        }

        unsafe { RenderState_PopBlendMode() };
    }

    pub fn begin_layer(&mut self, pos: Vec2, size: Vec2, clip: bool) {
        let layer = UIRendererLayer {
            parent: self.current_layer_id,
            pos,
            size,
            clip,
            ..Default::default()
        };

        let next_layer_id = self.layers.len();

        self.layers.push(layer);

        self.current_layer_id = Some(next_layer_id.into());
    }

    pub fn end_layer(&mut self) {
        if let Some(current_layer_id) = self.current_layer_id {
            let parent_id = self.layers[*current_layer_id].parent;

            if let Some(parent_id) = parent_id {
                self.layers[*current_layer_id].next = self.layers[*parent_id].children;
                self.layers[*parent_id].children = self.current_layer_id;
            }

            self.current_layer_id = self.layers[*current_layer_id].parent;
        } else {
            unreachable!();
        }
    }

    pub fn image(&mut self, image: *mut Tex2D, pos: Vec2, size: Vec2) {
        if let Some(current_layer_id) = self.current_layer_id {
            let next = self.layers[*current_layer_id].image_id;
            let item = UIRendererImage {
                next,
                pos,
                size,
                image,
            };

            self.layers[*current_layer_id].image_id = Some(self.images.len().into());
            self.images.push(item);
        } else {
            unreachable!();
        }
    }

    pub fn panel(&mut self, pos: Vec2, size: Vec2, color: Color, bevel: f32, inner_alpha: f32) {
        if let Some(current_layer_id) = self.current_layer_id {
            let next = self.layers[*current_layer_id].panel_id;
            let item = UIRendererPanel {
                next,
                pos,
                size,
                color,
                bevel,
                inner_alpha,
            };

            self.layers[*current_layer_id].panel_id = Some(self.panels.len().into());
            self.panels.push(item);
        } else {
            unreachable!();
        }
    }

    pub fn rect(&mut self, pos: Vec2, size: Vec2, color: Color, outline: Option<f32>) {
        if let Some(current_layer_id) = self.current_layer_id {
            let next = self.layers[*current_layer_id].rect_id;
            let item = UIRendererRect {
                next,
                pos,
                size,
                color,
                outline,
            };

            self.layers[*current_layer_id].rect_id = Some(self.rects.len().into());
            self.rects.push(item);
        } else {
            unreachable!();
        }
    }

    pub fn text(&mut self, font: &Font, text: &str, pos: Vec2, color: Color) {
        if let Some(current_layer_id) = self.current_layer_id {
            let next = self.layers[*current_layer_id].text_id;
            let item = UIRendererText {
                next,
                pos,
                font: font as _,
                text: text.into(),
                color,
            };

            self.layers[*current_layer_id].text_id = Some(self.texts.len().into());
            self.texts.push(item);
        } else {
            unreachable!();
        }
    }
}
