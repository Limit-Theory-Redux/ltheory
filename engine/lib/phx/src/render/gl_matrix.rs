use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::math::*;

// OpenGL 2.1 style matrix stack emulation.

pub enum MatrixMode {
    ModelView,
    Projection,
}

pub struct MatrixStack {
    mode: MatrixMode,
    modelview: Vec<Matrix>,
    projection: Vec<Matrix>,
}

impl MatrixStack {
    fn count(&self) -> usize {
        match self.mode {
            MatrixMode::ModelView => self.modelview.len(),
            MatrixMode::Projection => self.projection.len(),
        }
    }

    fn push(&mut self) {
        let m = self.top().unwrap_or(&Matrix::IDENTITY).clone();
        match self.mode {
            MatrixMode::ModelView => self.modelview.push(m),
            MatrixMode::Projection => self.projection.push(m),
        }
    }

    fn pop(&mut self) {
        match self.mode {
            MatrixMode::ModelView => self.modelview.pop(),
            MatrixMode::Projection => self.projection.pop(),
        };
    }

    fn top(&self) -> Option<&Matrix> {
        if self.count() == 0 {
            None
        } else {
            Some(match self.mode {
                MatrixMode::ModelView => &self.modelview[self.modelview.len() - 1],
                MatrixMode::Projection => &self.projection[self.projection.len() - 1],
            })
        }
    }

    fn top_mut(&mut self) -> Option<&mut Matrix> {
        if self.count() == 0 {
            None
        } else {
            Some(match self.mode {
                MatrixMode::ModelView => {
                    let len = self.modelview.len();
                    &mut self.modelview[len - 1]
                }
                MatrixMode::Projection => {
                    let len = self.projection.len();
                    &mut self.projection[len - 1]
                }
            })
        }
    }

    fn load(&mut self, m: Matrix) {
        self.top_mut().map(|top| {
            *top = m;
        });
    }

    fn mult(&mut self, m: Matrix) {
        self.top_mut().map(|top| {
            *top *= m;
        });
    }
}
pub struct GLMatrix;

impl GLMatrix {
    fn inst() -> MutexGuard<'static, MatrixStack> {
        static INST: OnceLock<Mutex<MatrixStack>> = OnceLock::new();
        INST.get_or_init(|| {
            Mutex::new(MatrixStack {
                mode: MatrixMode::ModelView,
                modelview: vec![],
                projection: vec![],
            })
        })
        .lock()
        .unwrap()
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl GLMatrix {
    pub fn clear() {
        Self::inst().load(Matrix::IDENTITY);
    }

    pub fn load(matrix: &Matrix) {
        Self::inst().load(*matrix);
    }

    pub fn look_at(eye: &DVec3, at: &DVec3, up: &DVec3) {
        let dmatrix = glam::DMat4::look_at_rh(*eye, *at, *up);
        Self::inst().mult(dmatrix.as_mat4());
    }

    pub fn mode_p() {
        Self::inst().mode = MatrixMode::Projection;
    }

    pub fn mode_w_v() {
        Self::inst().mode = MatrixMode::ModelView;
    }

    pub fn mult(matrix: &mut Matrix) {
        Self::inst().mult(*matrix);
    }

    pub fn perspective(fovy: f64, aspect: f64, z0: f64, z1: f64) {
        let dmatrix = glam::DMat4::perspective_rh_gl(fovy, aspect, z0, z1);
        Self::inst().mult(dmatrix.as_mat4());
    }

    pub fn pop() {
        Self::inst().pop();
    }

    pub fn push() {
        Self::inst().push();
    }

    pub fn push_clear() {
        Self::inst().push();
        Self::inst().load(Matrix::IDENTITY);
    }

    pub fn get() -> Option<Matrix> {
        Self::inst().top().map(|ref_mut: &Mat4| ref_mut.clone())
    }

    pub fn rotate_x(angle: f64) {
        Self::inst().mult(glam::DMat4::from_rotation_x(angle).as_mat4())
    }

    pub fn rotate_y(angle: f64) {
        Self::inst().mult(glam::DMat4::from_rotation_y(angle).as_mat4())
    }

    pub fn rotate_z(angle: f64) {
        Self::inst().mult(glam::DMat4::from_rotation_z(angle).as_mat4())
    }

    pub fn scale(x: f64, y: f64, z: f64) {
        Self::inst().mult(glam::DMat4::from_scale(DVec3::new(x, y, z)).as_mat4())
    }

    pub fn translate(x: f64, y: f64, z: f64) {
        Self::inst().mult(glam::DMat4::from_translation(DVec3::new(x, y, z)).as_mat4())
    }
}
