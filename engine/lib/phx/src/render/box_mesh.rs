use glam::Vec3;

use crate::math::Matrix;

use super::Mesh;

#[derive(Clone)]
pub struct BoxMesh {
    pub elem: Vec<Box0>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Box0 {
    pub p: Vec3,
    pub s: Vec3,
    pub r: Vec3,
    pub b: Vec3,
}

const K_FACE_ORIGIN: [Vec3; 6] = [
    Vec3::new(-1.0, -1.0, 1.0),
    Vec3::new(-1.0, -1.0, -1.0),
    Vec3::new(1.0, -1.0, -1.0),
    Vec3::new(-1.0, -1.0, -1.0),
    Vec3::new(-1.0, 1.0, -1.0),
    Vec3::new(-1.0, -1.0, -1.0),
];

const K_FACE_U: [Vec3; 6] = [
    Vec3::new(2.0, 0.0, 0.0),
    Vec3::new(0.0, 2.0, 0.0),
    Vec3::new(0.0, 2.0, 0.0),
    Vec3::new(0.0, 0.0, 2.0),
    Vec3::new(0.0, 0.0, 2.0),
    Vec3::new(2.0, 0.0, 0.0),
];

const K_FACE_V: [Vec3; 6] = [
    Vec3::new(0.0, 2.0, 0.0),
    Vec3::new(2.0, 0.0, 0.0),
    Vec3::new(0.0, 0.0, 2.0),
    Vec3::new(0.0, 2.0, 0.0),
    Vec3::new(2.0, 0.0, 0.0),
    Vec3::new(0.0, 0.0, 2.0),
];

#[luajit_ffi_gen::luajit_ffi]
impl BoxMesh {
    #[bind(name = "Create")]
    pub fn new() -> BoxMesh {
        BoxMesh { elem: Vec::new() }
    }

    pub fn add(&mut self, p: &Vec3, s: &Vec3, r: &Vec3, b: &Vec3) {
        self.elem.push(Box0 {
            p: *p,
            s: *s,
            r: *r,
            b: *b,
        });
    }

    pub fn get_mesh(&self, res: i32) -> Mesh {
        let mut mesh = Mesh::new();
        mesh.reserve_vertex_data(6 * res * res * self.elem.len() as i32);
        mesh.reserve_index_data(12 * (res - 1) * (res - 1));

        for box3 in &self.elem {
            let lower = Vec3::new(box3.b.x - 1.0, box3.b.y - 1.0, box3.b.z - 1.0);
            let upper = Vec3::new(1.0 - box3.b.x, 1.0 - box3.b.y, 1.0 - box3.b.z);
            let rot = Matrix::yaw_pitch_roll(box3.r.x, box3.r.y, box3.r.z);

            for face in 0..6 {
                let o = K_FACE_ORIGIN[face as usize];
                let du = K_FACE_U[face as usize];
                let dv = K_FACE_V[face as usize];
                let n = Vec3::cross(du, dv).normalize();

                for iu in 0..res {
                    let u = iu as f32 / (res - 1) as f32;
                    for iv in 0..res {
                        let v = iv as f32 / (res - 1) as f32;
                        let mut p = o + (du * u) + (dv * v);
                        let clamped = Vec3::clamp(p, lower, upper);
                        let proj = p - clamped;
                        p = clamped + (proj.normalize() * box3.b);
                        p *= box3.s;
                        let rp = rot.mul_point(&p);
                        p = rp + box3.p;

                        if iu != 0 && iv != 0 {
                            let off = mesh.get_vertex_count();
                            mesh.add_quad(off, off - res, off - res - 1, off - 1);
                        }
                        mesh.add_vertex(p.x, p.y, p.z, n.x, n.y, n.z, u, v);
                    }
                }
            }
        }

        mesh
    }
}
