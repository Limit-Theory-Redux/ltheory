use crate::math::*;
use crate::render::*;

// TODO: optimize Octree and Node structures if possible - get rid of Option<Box<>>

#[derive(Clone)]
#[repr(C)]
pub struct Octree {
    pub box_0: Box3,
    pub child: [Option<Box<Octree>>; 8],
    pub elems: Option<Box<Node>>,
}

#[derive(Clone)]
#[repr(C)]
pub struct Node {
    pub next: Option<Box<Node>>,
    pub id: u64,
    pub box_0: Box3,
}

impl Octree {
    fn get_avg_load_impl(&self, load: &mut f64, nodes: &mut f64) {
        *nodes += 1.0;
        let mut elem = self.elems.as_ref();
        while let Some(node) = elem {
            *load += 1.0;
            elem = node.next.as_ref();
        }

        for child in self.child.iter().flatten() {
            child.get_avg_load_impl(load, nodes);
        }
    }

    fn intersect_ray_impl(&self, o: Vec3, di: Vec3) -> bool {
        if !self.box_0.intersects_ray(o, di) {
            return false;
        }
        let mut elem = self.elems.as_ref();
        while let Some(node) = elem {
            if node.box_0.intersects_ray(o, di) {
                return true;
            }
            elem = node.next.as_ref();
        }
        for child in self.child.iter().flatten() {
            if child.intersect_ray_impl(o, di) {
                return true;
            }
        }
        false
    }

    fn insert(&mut self, box_0: Box3, id: u64) {
        let elem = Node {
            box_0,
            id,
            next: self.elems.take(),
        };

        self.elems = Some(elem.into());
    }

    fn add_depth(&mut self, box_0: Box3, id: u64) {
        let l = self.box_0.lower;
        let u = self.box_0.upper;
        let c = self.box_0.center();
        let child_bound = [
            Box3 { lower: l, upper: c },
            Box3 {
                lower: Vec3::new(c.x, l.y, l.z),
                upper: Vec3::new(u.x, c.y, c.z),
            },
            Box3 {
                lower: Vec3::new(l.x, c.y, l.z),
                upper: Vec3::new(c.x, u.y, c.z),
            },
            Box3 {
                lower: Vec3::new(c.x, c.y, l.z),
                upper: Vec3::new(u.x, u.y, c.z),
            },
            Box3 {
                lower: Vec3::new(l.x, l.y, c.z),
                upper: Vec3::new(c.x, c.y, u.z),
            },
            Box3 {
                lower: Vec3::new(c.x, l.y, c.z),
                upper: Vec3::new(u.x, c.y, u.z),
            },
            Box3 {
                lower: Vec3::new(l.x, c.y, c.z),
                upper: Vec3::new(c.x, u.y, u.z),
            },
            Box3 { lower: c, upper: u },
        ];

        let mut intersections = 0;
        let mut last_intersection = 0;
        for (i, child) in child_bound.iter().enumerate() {
            if Box3::intersects_box(box_0, *child) {
                intersections += 1;
                last_intersection = i;
            }
        }
        if intersections == 0 {
            return;
        }
        if intersections == 1 {
            let octree = self.child[last_intersection]
                .get_or_insert_with(|| Self::new(child_bound[last_intersection]).into());

            octree.add_depth(
                Box3::intersection(box_0, child_bound[last_intersection]),
                id,
            );
            return;
        }
        self.insert(box_0, id);
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Octree {
    #[bind(name = "Create")]
    pub fn new(box_0: Box3) -> Self {
        Self {
            box_0,
            child: Default::default(),
            elems: None,
        }
    }

    pub fn from_mesh(mesh: &mut Mesh) -> Self {
        let mut mesh_box = Box3 {
            lower: Vec3::ZERO,
            upper: Vec3::ZERO,
        };
        mesh.get_bound(&mut mesh_box);
        let mut this = Octree::new(mesh_box);

        let index_data = mesh.get_index_data();
        let vertex_data = mesh.get_vertex_data();

        for i in (0..index_data.len()).step_by(3) {
            let v0 = &vertex_data[index_data[i] as usize];
            let v1 = &vertex_data[index_data[i + 1] as usize];
            let v2 = &vertex_data[index_data[i + 2] as usize];
            let box_0 = Box3::new(
                Vec3::min(v0.p, Vec3::min(v1.p, v2.p)),
                Vec3::max(v0.p, Vec3::max(v1.p, v2.p)),
            );

            this.add(box_0, (i / 3) as u64);
        }
        this
    }

    pub fn get_avg_load(&self) -> f64 {
        let mut load = 0.0;
        let mut nodes = 0.0;
        self.get_avg_load_impl(&mut load, &mut nodes);
        load / nodes
    }

    pub fn get_max_load(&self) -> i32 {
        let mut load = 0;
        let mut elem = self.elems.as_ref();
        while let Some(node) = elem {
            load += 1;
            elem = node.next.as_ref();
        }

        for child in self.child.iter().flatten() {
            load = i32::max(load, child.get_max_load());
        }
        load
    }

    pub fn get_memory(&self) -> usize {
        let mut memory = std::mem::size_of::<Octree>();
        for child in self.child.iter().flatten() {
            memory += child.get_memory();
        }
        let mut elem = self.elems.as_ref();
        while let Some(node) = elem {
            memory = (memory as usize).wrapping_add(std::mem::size_of::<Node>());
            elem = node.next.as_ref();
        }
        memory
    }

    pub fn intersect_ray(&self, matrix: &mut Matrix, ro: &Vec3, rd: &Vec3) -> bool {
        let inv = matrix.inverse();
        let inv_ro = inv.mul_point(ro);
        let inv_rd = inv.mul_dir(rd);
        self.intersect_ray_impl(inv_ro, inv_rd.recip())
    }

    pub fn add(&mut self, box_0: Box3, id: u64) {
        self.add_depth(box_0, id);
    }

    #[allow(unsafe_code)]
    pub fn draw(&mut self) {
        unsafe {
            Draw_Color(1.0, 1.0, 1.0, 1.0);
            Draw_Box3(&self.box_0);
            Draw_Color(0.0, 1.0, 0.0, 1.0);
        }
        let mut elem = self.elems.as_ref();
        while let Some(node) = elem {
            unsafe {
                Draw_Box3(&node.box_0);
            }
            elem = node.next.as_ref();
        }
        for child in self.child.iter_mut().flatten() {
            child.draw();
        }
    }
}
