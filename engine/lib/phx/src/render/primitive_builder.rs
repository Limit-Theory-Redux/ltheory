use crate::render::{CmdPrimitiveType, ImmVertex};

#[derive(PartialEq)]
pub enum PrimitiveType {
    /// A list of points.
    Points,

    /// A list of lines.
    Lines,

    /// A list of triangles.
    Triangles,

    /// A list of quads. `CommandExecutor::draw_immediate` fans each group of
    /// 4 vertices as a triangle fan - no CPU-side triangle expansion needed.
    Quads,

    /// A single convex polygon, using the first vertex as the point shared by all triangles.
    Polygon,
}

impl PrimitiveType {
    fn to_cmd(&self) -> CmdPrimitiveType {
        match self {
            PrimitiveType::Points => CmdPrimitiveType::Points,
            PrimitiveType::Lines => CmdPrimitiveType::Lines,
            PrimitiveType::Triangles => CmdPrimitiveType::Triangles,
            PrimitiveType::Quads => CmdPrimitiveType::Quads,
            PrimitiveType::Polygon => CmdPrimitiveType::TriangleFan,
        }
    }
}

/// Accumulates vertices for immediate-mode drawing, owned by `Renderer` (was
/// `Draw`'s owned `PrimitiveBuilder`, which held its own GL buffers created
/// on whichever thread first touched `Draw`). Pure CPU-side now: GL
/// buffers/VAO live in `CommandExecutor`, fed by `RenderCommand::DrawImmediate`
/// once `end()` hands the accumulated vertices to the caller to submit.
pub struct PrimitiveBuilder {
    primitive: PrimitiveType,
    vertices: Vec<ImmVertex>,

    // The current attributes that will be used to seed the next vertex.
    current_normal: [f32; 3],
    current_texcoord: [f32; 2],
    current_color: [f32; 4],
}

impl PrimitiveBuilder {
    pub fn new() -> PrimitiveBuilder {
        PrimitiveBuilder {
            primitive: PrimitiveType::Triangles,
            vertices: vec![],
            current_normal: [0.0, 0.0, 0.0],
            current_texcoord: [0.0, 0.0],
            current_color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn begin(&mut self, primitive_type: PrimitiveType) {
        self.primitive = primitive_type;
        self.current_normal = [0.0, 0.0, 0.0];
        self.current_texcoord = [0.0, 0.0];
        self.current_color = [1.0, 1.0, 1.0, 1.0];
    }

    /// Take the accumulated vertices for the caller to submit as a
    /// `DrawImmediate` command. Returns `None` if nothing was drawn.
    pub fn end(&mut self) -> Option<(CmdPrimitiveType, Vec<ImmVertex>)> {
        if self.vertices.is_empty() {
            return None;
        }
        Some((self.primitive.to_cmd(), std::mem::take(&mut self.vertices)))
    }

    pub fn color3(&mut self, r: f32, g: f32, b: f32) {
        self.current_color = [r, g, b, 1.0];
    }

    pub fn color4(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.current_color = [r, g, b, a];
    }

    pub fn texcoord2(&mut self, s: f32, t: f32) {
        self.current_texcoord = [s, t];
    }

    pub fn normal3(&mut self, x: f32, y: f32, z: f32) {
        self.current_normal = [x, y, z];
    }

    pub fn vertex2(&mut self, x: f32, y: f32) {
        self.finish_vertex(x, y, 0.0);
    }

    pub fn vertex3(&mut self, x: f32, y: f32, z: f32) {
        self.finish_vertex(x, y, z);
    }

    fn finish_vertex(&mut self, x: f32, y: f32, z: f32) {
        self.vertices.push(ImmVertex {
            pos: [x, y, z],
            normal: self.current_normal,
            uv: self.current_texcoord,
            color: self.current_color,
        });
    }
}

impl Default for PrimitiveBuilder {
    fn default() -> Self {
        Self::new()
    }
}
