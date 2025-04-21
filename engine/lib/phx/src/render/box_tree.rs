use glam::Vec3;

use super::{Draw, Mesh};
use crate::math::{Box3, Matrix};

#[derive(Clone)]
pub struct BoxTree {
    pub root: Option<Node>,
}

#[derive(Clone)]
pub struct Node {
    pub box3: Box3,
    pub data: Vec<u8>,
    pub sub: [Option<Box<Node>>; 2],
}

#[luajit_ffi_gen::luajit_ffi]
impl BoxTree {
    #[bind(name = "Create")]
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn from_mesh(mesh: &Mesh) -> Self {
        let mut this = Self::new();
        let index_data = mesh.get_index_data();
        let vertex_data = mesh.get_vertex_data();

        for i in (0..index_data.len()).step_by(3) {
            let v0 = &vertex_data[index_data[i] as usize];
            let v1 = &vertex_data[index_data[i + 1] as usize];
            let v2 = &vertex_data[index_data[i + 2] as usize];
            let box3 = Box3::new(
                Vec3::min(v0.p, Vec3::min(v1.p, v2.p)),
                Vec3::max(v0.p, Vec3::max(v1.p, v2.p)),
            );
            this.add(box3, &[]);
        }
        this
    }

    pub fn add(&mut self, box3: Box3, data: &[u8]) {
        if let Some(root) = self.root.take() {
            let node = Node::new(box3, data);
            self.root = Some(Node::merge(root.into(), node));
        }
    }

    pub fn get_memory(&self) -> usize {
        let mut memory = std::mem::size_of::<BoxTree>();
        if let Some(root) = &self.root {
            memory += root.get_memory();
        }
        memory
    }

    pub fn intersect_ray(&self, matrix: &mut Matrix, ro: &Vec3, rd: &Vec3) -> bool {
        if let Some(root) = &self.root {
            let inv = matrix.inverse();
            let inv_ro = inv.mul_point(ro);
            let inv_rd = inv.mul_dir(rd);
            root.intersect_ray(inv_ro, inv_rd.recip())
        } else {
            false
        }
    }

    pub fn draw(&self, max_depth: i32) {
        if let Some(root) = &self.root {
            root.draw_node(max_depth);
        }
    }
}

#[inline]
fn cost(box3: Box3) -> f32 {
    box3.volume()
}

#[inline]
fn cost_merge(a: Box3, b: Box3) -> f32 {
    cost(Box3::union(a, b))
}

impl Node {
    #[inline]
    fn new(box3: Box3, data: &[u8]) -> Self {
        Self {
            box3,
            sub: Default::default(),
            data: data.into(),
        }
    }

    fn merge(mut this: Box<Node>, src: Node) -> Self {
        if let Some(sub0) = this.sub[0].take() {
            if let Some(sub1) = this.sub[1].take() {
                // Contained by current sub-tree
                if Box3::contains(this.box3, src.box3) {
                    let mut node = Node {
                        box3: this.box3,
                        data: this.data,
                        sub: Default::default(),
                    };
                    let cost0 = cost_merge(sub0.box3, src.box3) + cost(sub1.box3);
                    let cost1 = cost_merge(sub1.box3, src.box3) + cost(sub0.box3);
                    if cost0 < cost1 {
                        node.sub[0] = Some(Self::merge(sub0, src).into());
                        node.sub[1] = Some(sub1);
                    } else {
                        node.sub[0] = Some(sub0);
                        node.sub[1] = Some(Self::merge(sub1, src).into());
                    }
                    return node;
                } else {
                    /* Not contained, need new parent. */
                    let mut parent = Node::new(Box3::union(this.box3, src.box3), &[]);

                    let cost_base = cost(this.box3) + cost(src.box3);
                    let cost0 = cost_merge(sub0.box3, src.box3) + cost(sub1.box3);
                    let cost1 = cost_merge(sub1.box3, src.box3) + cost(sub0.box3);

                    if cost_base <= cost0 && cost_base <= cost1 {
                        parent.sub[0] = Some(this);
                        parent.sub[1] = Some(src.into());
                    } else if cost0 <= cost_base && cost0 <= cost1 {
                        parent.sub[0] = Some(Self::merge(sub0, src).into());
                        parent.sub[1] = Some(sub1);
                    } else {
                        parent.sub[0] = Some(sub0);
                        parent.sub[1] = Some(Self::merge(sub1, src).into());
                    }
                    return parent;
                }
            }

            let node = Node {
                box3: this.box3,
                data: this.data,
                sub: [Some(sub0), this.sub[1].take()],
            };
            return node;
        } else {
            // Leaf node
            let mut parent = Node::new(Box3::union(this.box3, src.box3), &[]);
            parent.sub[0] = Some(this);
            parent.sub[1] = Some(src.into());
            return parent;
        }
    }

    fn get_memory(&self) -> usize {
        let mut memory = std::mem::size_of::<Node>();
        if let Some(sub) = &self.sub[0] {
            memory += sub.get_memory(); // TODO: + self.data.len() ?
        }
        if let Some(sub) = &self.sub[1] {
            memory += sub.get_memory(); // TODO: + self.data.len() ?
        }
        memory
    }

    fn intersect_ray(&self, o: Vec3, di: Vec3) -> bool {
        if !self.box3.intersects_ray(o, di) {
            return false;
        }

        if let Some(sub0) = &self.sub[0] {
            if sub0.intersect_ray(o, di) {
                return true;
            }
            if let Some(sub1) = &self.sub[1] {
                if sub1.intersect_ray(o, di) {
                    return true;
                }
            }
            false
        } else {
            true
        }
    }

    fn draw_node(&self, max_depth: i32) {
        if max_depth < 0 {
            return;
        }
        if self.sub[0].is_some() || self.sub[1].is_some() {
            Draw::color(1.0f32, 1.0f32, 1.0f32, 1.0f32);
            Draw::box3(&self.box3);
        } else {
            Draw::color(0.0f32, 1.0f32, 0.0f32, 1.0f32);
            Draw::box3(&self.box3);
        }
        if let Some(sub) = &self.sub[0] {
            sub.draw_node(max_depth - 1);
        }
        if let Some(sub) = &self.sub[1] {
            sub.draw_node(max_depth - 1);
        }
    }
}
