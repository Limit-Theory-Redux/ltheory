use crate::math::*;

pub enum PrimitiveType {
    Points,
    Lines,
    Triangles,
    Polygon,
    Quads,
}

const IMMEDIATE_DRAW_SET_BUFFER_SIZE: usize = 8192;

// A type which constructs dynamic vertex / index buffers like the OpenGL immediate mode API.
//
// Inspired by raylib:
// https://github.com/raysan5/raylib/blob/master/src/rlgl.h#L1277
pub struct ImmediateDrawSet {
    primitive: PrimitiveType,

    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    texcoords: Vec<Vec2>,
    colors: Vec<Vec4>,
    included_normals: bool,
    included_texcoords: bool,
    included_colors: bool,
    next_normal: Vec3,
    next_texcoord: Vec2,
    next_color: Vec4,
    // Diligent::RefCntAutoPtr<Diligent::IBuffer> positionsVB;
    // Diligent::RefCntAutoPtr<Diligent::IBuffer> normalsVB;
    // Diligent::RefCntAutoPtr<Diligent::IBuffer> texcoordsVB;
    // Diligent::RefCntAutoPtr<Diligent::IBuffer> colorsVB;
}

impl ImmediateDrawSet {
    pub fn new() -> ImmediateDrawSet {
        // Diligent::BufferDesc bd;
        // bd.Usage = Diligent::USAGE_DYNAMIC;
        // bd.CPUAccessFlags = Diligent::CPU_ACCESS_WRITE;
        // bd.BindFlags = Diligent::BIND_VERTEX_BUFFER;

        // // Positions.
        // bd.Size = sizeof(Vec3f) * IMMEDIATE_DRAW_SET_BUFFER_SIZE;
        // rs->device->CreateBuffer(bd, nullptr, &positionsVB);
        // // Normals.
        // bd.Size = sizeof(Vec3f) * IMMEDIATE_DRAW_SET_BUFFER_SIZE;
        // rs->device->CreateBuffer(bd, nullptr, &normalsVB);
        // // Texcoords.
        // bd.Size = sizeof(Vec2f) * IMMEDIATE_DRAW_SET_BUFFER_SIZE;
        // rs->device->CreateBuffer(bd, nullptr, &texcoordsVB);
        // // Colors.
        // bd.Size = sizeof(Vec4f) * IMMEDIATE_DRAW_SET_BUFFER_SIZE;
        // rs->device->CreateBuffer(bd, nullptr, &colorsVB);

        ImmediateDrawSet {
            primitive: PrimitiveType::Triangles,
            positions: vec![],
            normals: vec![],
            texcoords: vec![],
            colors: vec![],
            included_normals: false,
            included_texcoords: false,
            included_colors: false,
            next_normal: Vec3::ZERO,
            next_texcoord: Vec2::ZERO,
            next_color: Vec4::ZERO,
        }
    }

    pub fn free() {
        //   colorsVB.Release();
        //   texcoordsVB.Release();
        //   normalsVB.Release();
        //   positionsVB.Release();
    }

    pub fn lineWidth(width: f32) {
        // TODO
    }

    pub fn pointSize(size: f32) {
        // TODO
    }

    pub fn begin(&mut self, primitive_type: PrimitiveType) {
        self.primitive = primitive_type;

        self.included_normals = false;
        self.included_texcoords = false;
        self.included_colors = false;
        self.next_normal = Vec3::ZERO;
        self.next_texcoord = Vec2::ZERO;
        self.next_color = Vec4::ZERO;
    }

    pub fn end(&mut self) {
        self.flush_and_draw();
    }

    pub fn color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.included_colors = true;
        self.next_color = Vec4::new(r, g, b, a);
    }

    pub fn texcoord(&mut self, tc: Vec2) {
        self.included_texcoords = true;
        self.next_texcoord = tc;
    }

    pub fn normal(&mut self, n: Vec3) {
        self.included_normals = true;
        self.next_normal = n;
    }

    pub fn vertex3(&mut self, p: Vec3) {
        // IDEA: We have a "current batch" which gets filled from here. One array for each vertex attribute.
        // Then when the batch hits the limit of the VB (8192 vertices?) or end() is called, a draw call is made by uploading
        // the data to the dynamic vertex buffers (one buffer for positions, one for normals, etc). We only bind the relevant
        // buffers when drawing.
        self.positions.push(p);
        self.normals.push(self.next_normal);
        self.texcoords.push(self.next_texcoord);
        self.colors.push(self.next_color);

        if self.positions.len() >= IMMEDIATE_DRAW_SET_BUFFER_SIZE {
            self.flush_and_draw();
        }
    }

    pub fn vertex2(&mut self, p: Vec2) {
        // TODO: What z value should go here?
        self.vertex3(Vec3::new(p.x, p.y, 0.0));
    }

    fn flush_and_draw(&mut self) {
        // RendererState* rs = Window_GetCurrentRS();

        // // Update vertex buffers.
        // /* position always included */ {
        // Diligent::MapHelper<Vec3f> positionsMapped(rs->immediateContext, positionsVB, Diligent::MAP_WRITE,
        //                                             Diligent::MAP_FLAG_DISCARD);
        // memcpy((Vec3f*)positionsMapped, positions.data(), sizeof(Vec3f) * positions.size());
        // }
        // if (includedNormals) {
        // Diligent::MapHelper<Vec3f> normalsMapped(rs->immediateContext, normalsVB, Diligent::MAP_WRITE,
        //                                         Diligent::MAP_FLAG_DISCARD);
        // memcpy((Vec3f*)normalsMapped, normals.data(), sizeof(Vec3f) * normals.size());
        // }
        // if (includedTexcoords) {
        // Diligent::MapHelper<Vec2f> texcoordsMapped(rs->immediateContext, texcoordsVB, Diligent::MAP_WRITE,
        //                                             Diligent::MAP_FLAG_DISCARD);
        // memcpy((Vec2f*)texcoordsMapped, texcoords.data(), sizeof(Vec2f) * texcoords.size());
        // }
        // if (includedColors) {
        // Diligent::MapHelper<Vec4f> colorsMapped(rs->immediateContext, colorsVB, Diligent::MAP_WRITE,
        //                                         Diligent::MAP_FLAG_DISCARD);
        // memcpy((Vec4f*)colorsMapped, colors.data(), sizeof(Vec4f) * colors.size());
        // }

        // // Bind vertex buffers.
        // std::vector<Diligent::IBuffer*> vbuffers;
        // vbuffers.push_back(positionsVB);
        // rs->immediateContext->SetVertexBuffers(0, vbuffers.size(), vbuffers.data(), nullptr, Diligent::RESOURCE_STATE_TRANSITION_MODE_TRANSITION, Diligent::SET_VERTEX_BUFFERS_FLAG_RESET);

        // // Issue draw call.
        // Diligent::DrawAttribs attribs;
        // attribs.NumVertices = positions.size();
        // attribs.Flags = Diligent::DRAW_FLAG_VERIFY_ALL;
        // rs->immediateContext->Draw(attribs);

        // // Create data for next draw call.
        // positions.clear();
        // normals.clear();
        // texcoords.clear();
        // colors.clear();
    }
}
