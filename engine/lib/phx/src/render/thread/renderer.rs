use crate::window::WindowGlContext;

pub struct Renderer {}

impl Renderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start(&mut self, _context: WindowGlContext) -> bool {
        false
    }

    pub fn stop(&mut self) -> Option<WindowGlContext> {
        None
    }
}
