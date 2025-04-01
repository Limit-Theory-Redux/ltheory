use glam::Vec3;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Sphere {
    pub p: Vec3,
    pub r: f32,
}

#[luajit_ffi_gen::luajit_ffi(
    clone = true,
    typedef = "
        float px;
        float py;
        float pz;
        float r;"
)]
impl Sphere {}
