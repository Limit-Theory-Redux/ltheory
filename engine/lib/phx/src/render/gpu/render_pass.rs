use std::sync::Arc;

use super::TexView;
use crate::render::{Renderer, Viewport};
use crate::system::{Metric, Profiler};

pub const MAX_COLOR_ATTACHMENTS: usize = 4;

/// What a pass does with an attachment's previous contents at begin.
#[luajit_ffi_gen::luajit_ffi(repr = "u32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadOp {
    /// Keep the existing contents.
    Load,
    /// Clear to the attachment's clear value.
    Clear,
    /// Contents are undefined; the pass overwrites every pixel it reads back.
    DontCare,
}

/// GL 3.3 has no discard (`glInvalidateFramebuffer` is 4.3), so this is
/// currently informational on GL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreOp {
    Store,
    Discard,
}

#[derive(Debug, Clone, Copy)]
pub struct ColorAttachment {
    pub view: TexView,
    pub load: LoadOp,
    pub clear: [f32; 4],
    pub store: StoreOp,
}

#[derive(Debug, Clone, Copy)]
pub struct DepthAttachment {
    pub view: TexView,
    pub load: LoadOp,
    pub clear: f32,
    pub store: StoreOp,
}

/// Everything a render pass needs to begin. Sent to the executor by value in
/// `RenderCommand::BeginRenderPass`.
#[derive(Debug, Clone)]
pub struct RenderPassDesc {
    pub label: Arc<str>,
    /// Contiguous from index 0.
    pub color: [Option<ColorAttachment>; MAX_COLOR_ATTACHMENTS],
    pub depth: Option<DepthAttachment>,
    /// Draw to the window's default framebuffer instead of attachments.
    pub backbuffer: bool,
    pub back_color: (LoadOp, [f32; 4]),
    pub back_depth: (LoadOp, f32),
    /// Width and height the pass renders at; the viewport is set from it.
    pub extent: [u32; 2],
}

impl RenderPassDesc {
    pub fn new(label: &str) -> Self {
        Self {
            label: Arc::from(label),
            color: [None; MAX_COLOR_ATTACHMENTS],
            depth: None,
            backbuffer: false,
            back_color: (LoadOp::Load, [0.0; 4]),
            back_depth: (LoadOp::Load, 1.0),
            extent: [0, 0],
        }
    }

    /// Backbuffer pass of the given size that keeps its contents.
    pub fn new_backbuffer(label: &str, width: i32, height: i32) -> Self {
        let mut desc = Self::new(label);
        desc.backbuffer = true;
        desc.extent = [width.max(1) as u32, height.max(1) as u32];
        desc
    }

    /// Single color attachment, for the engine's own offscreen passes.
    pub fn with_color(label: &str, view: TexView, load: LoadOp, clear: [f32; 4]) -> Self {
        let mut desc = Self::new(label);
        desc.set_color(0, view, load, clear);
        desc
    }

    pub fn set_color(&mut self, index: usize, view: TexView, load: LoadOp, clear: [f32; 4]) {
        assert!(
            index < MAX_COLOR_ATTACHMENTS,
            "RenderPassDesc '{}': color index {index} out of range (max {MAX_COLOR_ATTACHMENTS})",
            self.label
        );
        self.color[index] = Some(ColorAttachment {
            view,
            load,
            clear,
            store: StoreOp::Store,
        });
        self.extent = view.extent;
    }

    pub fn set_depth(&mut self, view: TexView, load: LoadOp, clear: f32) {
        self.depth = Some(DepthAttachment {
            view,
            load,
            clear,
            store: StoreOp::Store,
        });
        self.extent = view.extent;
    }

    pub fn color_count(&self) -> usize {
        self.color.iter().take_while(|c| c.is_some()).count()
    }

    fn validate(&self) {
        let label = &self.label;
        let count = self.color_count();
        assert!(
            self.color[count..].iter().all(|c| c.is_none()),
            "RenderPassDesc '{label}': color attachments must be contiguous from index 0"
        );
        if self.backbuffer {
            assert!(
                count == 0 && self.depth.is_none(),
                "RenderPassDesc '{label}': a backbuffer pass cannot have texture attachments"
            );
        } else {
            assert!(
                count > 0 || self.depth.is_some(),
                "RenderPassDesc '{label}': no attachments (use backbuffer() for the window)"
            );
            for view in self
                .color
                .iter()
                .flatten()
                .map(|c| c.view)
                .chain(self.depth.iter().map(|d| d.view))
            {
                assert!(
                    view.extent == self.extent,
                    "RenderPassDesc '{label}': attachment extents differ ({:?} vs {:?})",
                    view.extent,
                    self.extent
                );
            }
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl RenderPassDesc {
    #[bind(name = "Create")]
    pub fn create(label: &str) -> RenderPassDesc {
        Self::new(label)
    }

    /// Set color attachment `index` (0-3, contiguous). `r,g,b,a` is the clear
    /// value, used only with `LoadOp.Clear`.
    #[allow(clippy::too_many_arguments)]
    pub fn color(
        &mut self,
        index: i32,
        view: &TexView,
        load: LoadOp,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) {
        self.set_color(index as usize, *view, load, [r, g, b, a]);
    }

    /// Set the depth attachment. `d` is the clear value, used only with
    /// `LoadOp.Clear`.
    pub fn depth(&mut self, view: &TexView, load: LoadOp, d: f32) {
        self.set_depth(*view, load, d);
    }

    /// Target the window's backbuffer (`width` x `height` pixels) instead of
    /// texture attachments.
    #[allow(clippy::too_many_arguments)]
    pub fn backbuffer(
        &mut self,
        width: i32,
        height: i32,
        load: LoadOp,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) {
        self.backbuffer = true;
        self.extent = [width.max(1) as u32, height.max(1) as u32];
        self.back_color = (load, [r, g, b, a]);
    }

    /// Load op for the backbuffer's depth buffer.
    pub fn backbuffer_depth(&mut self, load: LoadOp, d: f32) {
        self.back_depth = (load, d);
    }
}

/// Handle of the one open render pass. `finish` ends it.
pub struct RenderPass {
    label: Arc<str>,
    finished: bool,
}

#[luajit_ffi_gen::luajit_ffi]
impl RenderPass {
    pub fn finish(&mut self, r: &mut Renderer) {
        if self.finished {
            panic!("RenderPass '{}': finish() called twice", self.label);
        }
        self.finished = true;
        r.end_pass_intern();
    }
}

impl Renderer {
    /// Begin a render pass: bind the attachments, size the viewport, apply
    /// the load ops. Only one pass may be open at a time.
    pub fn begin_pass_intern(&mut self, desc: &RenderPassDesc) -> RenderPass {
        Profiler::begin("RenderPass_Begin");
        if let Some(open) = &self.data.open_pass {
            panic!(
                "beginPass('{}'): pass '{open}' is still open (passes cannot nest; call finish() first)",
                desc.label
            );
        }
        desc.validate();

        self.data.open_pass = Some(desc.label.clone());
        Metric::FBOSwap.inc();
        self.begin_render_pass(Box::new(desc.clone()));
        Viewport::push(
            self,
            0,
            0,
            desc.extent[0] as i32,
            desc.extent[1] as i32,
            desc.backbuffer,
        );
        Profiler::end();

        RenderPass {
            label: desc.label.clone(),
            finished: false,
        }
    }

    pub fn end_pass_intern(&mut self) {
        Profiler::begin("RenderPass_End");
        if self.data.open_pass.take().is_none() {
            panic!("RenderPass finish(): no pass is open");
        }
        Metric::FBOSwap.inc();
        self.end_render_pass();
        Viewport::pop(self);
        Profiler::end();
    }
}
