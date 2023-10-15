use glam::Vec2;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiData {
    pub offset: Vec2,
    pub minSize: Vec2,
    pub size: Vec2,
}
