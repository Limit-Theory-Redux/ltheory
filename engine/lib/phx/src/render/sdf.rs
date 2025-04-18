use glam::{IVec3, Vec3};

use super::{DataFormat, Mesh, PixelFormat, Tex3D};
use crate::math::saturate;

#[derive(Clone)]
pub struct Sdf {
    size: IVec3,
    data: Vec<Cell>,
}

#[derive(Clone, Default)]
struct Cell {
    pub value: f32,
    pub normal: Vec3,
}

#[luajit_ffi_gen::luajit_ffi(name = "SDF")]
impl Sdf {
    #[bind(name = "Create")]
    pub fn new(sx: i32, sy: i32, sz: i32) -> Self {
        Self {
            size: IVec3::new(sx, sy, sz),
            data: vec![
                Cell {
                    value: 0.0,
                    normal: Vec3::ZERO,
                };
                (sx * sy * sz) as usize
            ],
        }
    }

    #[bind(name = "FromTex3D")]
    pub fn from_tex3d(tex: &mut Tex3D) -> Self {
        Self {
            size: tex.get_size(),
            data: tex.get_data(PixelFormat::RGBA, DataFormat::Float),
        }
    }

    pub fn to_mesh(&self) -> Mesh {
        let mut mesh = Mesh::new();
        let cells = self.size - 1;
        let cells_f = Vec3::new(cells.x as f32, cells.y as f32, cells.z as f32);
        let stride = IVec3 {
            x: 1,
            y: self.size.x,
            z: self.size.x * self.size.y,
        };
        let cell_stride = IVec3 {
            x: 1,
            y: cells.x,
            z: cells.x * cells.y,
        };
        let mut indices = vec![0; (cells.x * cells.y * cells.z) as usize];

        const VP: [Vec3; 8] = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
        ];
        const EDGE_TABLE: [[usize; 2]; 12] = [
            [0, 1],
            [2, 3],
            [4, 5],
            [6, 7],
            [0, 2],
            [1, 3],
            [4, 6],
            [5, 7],
            [0, 4],
            [1, 5],
            [2, 6],
            [3, 7],
        ];

        for z in 0..cells.z {
            let z0 = z as f32 / cells_f.z;
            for y in 0..cells.y {
                let y0 = y as f32 / cells_f.y;
                for x in 0..cells.x {
                    let x0 = x as f32 / cells_f.x;
                    let cell = IVec3 { x, y, z };
                    let cell_index = IVec3::dot(cell_stride, IVec3::new(x, y, z)) as usize;

                    let base_idx = IVec3::dot(stride, IVec3::new(x, y, z)) as usize;
                    let v = [
                        &self.data[base_idx],
                        &self.data[base_idx + stride.x as usize],
                        &self.data[base_idx + stride.y as usize],
                        &self.data[base_idx + (stride.x + stride.y) as usize],
                        &self.data[base_idx + stride.z as usize],
                        &self.data[base_idx + (stride.z + stride.x) as usize],
                        &self.data[base_idx + (stride.z + stride.y) as usize],
                        &self.data[base_idx + (stride.z + stride.y + stride.x) as usize],
                    ];

                    let mut mask = 0;
                    mask |= if v[0].value > 0.0 { 0x1 } else { 0 };
                    mask |= if v[1].value > 0.0 { 0x2 } else { 0 };
                    mask |= if v[2].value > 0.0 { 0x4 } else { 0 };
                    mask |= if v[3].value > 0.0 { 0x8 } else { 0 };
                    mask |= if v[4].value > 0.0 { 0x10 } else { 0 };
                    mask |= if v[5].value > 0.0 { 0x20 } else { 0 };
                    mask |= if v[6].value > 0.0 { 0x40 } else { 0 };
                    mask |= if v[7].value > 0.0 { 0x80 } else { 0 };

                    if mask == 0 || mask == 0xff {
                        indices[cell_index] = -1;
                    } else {
                        let mut tw = 0.0;
                        let mut offset = Vec3::ZERO;
                        let mut n = Vec3::ZERO;
                        for edge in EDGE_TABLE {
                            let i0 = edge[0];
                            let i1 = edge[1];
                            let v0 = &v[i0];
                            let v1 = &v[i1];
                            if (v0.value > 0.0) != (v1.value > 0.0) {
                                let t = saturate((v0.value / (v0.value - v1.value)) as f64) as f32;
                                offset += VP[i0].lerp(VP[i1], t);
                                n += v0.normal.lerp(v1.normal, t);
                                tw += 1.0;
                            }
                        }
                        offset /= tw;
                        n = n.normalize();

                        let mut p = Vec3::new(x0, y0, z0) + (offset / cells_f);
                        p = p * 2.0 - 1.0;
                        indices[cell_index] = mesh.get_vertex_count();
                        mesh.add_vertex(p.x, p.y, p.z, n.x, n.y, n.z, 1.0, 0.0);

                        let i0_0 = indices[cell_index];
                        for i_0 in 0..3 {
                            let j = (i_0 + 1) % 3;
                            let k = (i_0 + 2) % 3;
                            if !(cell[j] == 0 || cell[k] == 0) {
                                let du = cell_stride[j] as usize;
                                let dv = cell_stride[k] as usize;
                                let i1_0 = indices[cell_index - du];
                                let i2 = indices[cell_index - du - dv];
                                let i3 = indices[cell_index - dv];
                                if !(i1_0 < 0 || i2 < 0 || i3 < 0) {
                                    if v[0].value > 0.0 {
                                        mesh.add_quad(i0_0, i3, i2, i1_0);
                                    } else {
                                        mesh.add_quad(i0_0, i1_0, i2, i3);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        mesh
    }

    pub fn clear(&mut self, value: f32) {
        for cell in &mut self.data {
            cell.value = value;
        }
    }

    pub fn compute_normals(&mut self) {
        let stride = IVec3 {
            x: 1,
            y: self.size.x,
            z: self.size.x * self.size.y,
        };
        for z in 1..self.size.z - 1 {
            for y in 1..self.size.y - 1 {
                for x in 1..self.size.x - 1 {
                    let cell_idx = ((x * stride.x) + (y * stride.y) + (z * stride.z)) as usize;
                    let x0 = &self.data[cell_idx - stride.x as usize];
                    let x1 = &self.data[cell_idx + stride.x as usize];
                    let y0 = &self.data[cell_idx - stride.y as usize];
                    let y1 = &self.data[cell_idx + stride.y as usize];
                    let z0 = &self.data[cell_idx - stride.z as usize];
                    let z1 = &self.data[cell_idx + stride.z as usize];

                    self.data[cell_idx].normal = Vec3::new(
                        x1.value - x0.value,
                        y1.value - y0.value,
                        z1.value - z0.value,
                    )
                    .normalize();
                }
            }
        }
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, value: f32) {
        self.data[(x + self.size.x * (y + self.size.y * z)) as usize].value = value;
    }

    pub fn set_normal(&mut self, x: i32, y: i32, z: i32, normal: &Vec3) {
        self.data[(x + self.size.x * (y + self.size.y * z)) as usize].normal = *normal;
    }
}
