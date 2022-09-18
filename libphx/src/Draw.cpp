#include "Draw.h"
#include "Metric.h"
#include "OpenGL.h"
#include "Window.h"
#include "Vec4.h"

#include "Graphics/GraphicsTools/interface/MapHelper.hpp"
#include <vector>

#define MAX_STACK_DEPTH 16

static float alphaStack[MAX_STACK_DEPTH];
static int alphaIndex = -1;
static Vec4f color = { 1, 1, 1, 1 };
static Vec4f drawColor = { 1, 1, 1, 1 };

enum class PrimitiveType {
  Points,
  Lines,
  Triangles,
  Polygon,
  Quads
};

// A class which constructs dynamic vertex / index buffers like the OpenGL immediate mode API.
//
// Inspired by raylib:
// https://github.com/raysan5/raylib/blob/master/src/rlgl.h#L1277
#define IMMEDIATE_DRAW_SET_BUFFER_SIZE 8192
class ImmediateDrawSet {
public:
  void createResources(Window* w) {
    RendererState* rs = Window_GetRS(w);

    Diligent::BufferDesc bd;
    bd.Usage = Diligent::USAGE_DYNAMIC;
    bd.CPUAccessFlags = Diligent::CPU_ACCESS_WRITE;
    bd.BindFlags = Diligent::BIND_VERTEX_BUFFER;

    // Positions.
    bd.Size = sizeof(Vec3f) * IMMEDIATE_DRAW_SET_BUFFER_SIZE;
    rs->device->CreateBuffer(bd, nullptr, &positionsVB);
    // Normals.
    bd.Size = sizeof(Vec3f) * IMMEDIATE_DRAW_SET_BUFFER_SIZE;
    rs->device->CreateBuffer(bd, nullptr, &normalsVB);
    // Texcoords.
    bd.Size = sizeof(Vec2f) * IMMEDIATE_DRAW_SET_BUFFER_SIZE;
    rs->device->CreateBuffer(bd, nullptr, &texcoordsVB);
    // Colors.
    bd.Size = sizeof(Vec4f) * IMMEDIATE_DRAW_SET_BUFFER_SIZE;
    rs->device->CreateBuffer(bd, nullptr, &colorsVB);
  }

  void free() {
    colorsVB.Release();
    texcoordsVB.Release();
    normalsVB.Release();
    positionsVB.Release();
  }

  void lineWidth(float width) {
    // TODO
  }

  void pointSize(float size) {
    // TODO
  }

  void begin(PrimitiveType type) {
    primitive = type;

    includedNormals = false;
    includedTexcoords = false;
    includedColors = false;
    nextNormal = Vec3f_Create(0.0f, 0.0f, 0.0f);
    nextTexcoord = Vec2f_Create(0.0f, 0.0f);
    nextColor = Vec4f_Create(0.0f, 0.0f, 0.0f, 0.0f);
  }

  void end() {
    flushAndDraw();
  }

  void color(float r, float g, float b, float a) {
    includedColors = true;
    nextColor = Vec4f_Create(r, g, b, a);
  }

  void texCoord(Vec2f tc) {
    includedTexcoords = true;
    nextTexcoord = tc;
  }

  void normal(Vec3f n) {
    includedNormals = true;
    nextNormal = n;
  }

  void vertex3(Vec3f p) {
    // IDEA: We have a "current batch" which gets filled from here. One array for each vertex attribute.
    // Then when the batch hits the limit of the VB (8192 vertices?) or end() is called, a draw call is made by uploading
    // the data to the dynamic vertex buffers (one buffer for positions, one for normals, etc). We only bind the relevant
    // buffers when drawing.
    positions.push_back(p);
    normals.push_back(nextNormal);
    texcoords.push_back(nextTexcoord);
    colors.push_back(nextColor);

    if (positions.size() > 8192) {
      flushAndDraw();
    }
  }

  void vertex2(Vec2f p) {
    // TODO: What z value should go here?
    vertex3(Vec3f{UNPACK2(p), 0.0f});
  }

private:
  PrimitiveType primitive = PrimitiveType::Triangles;

  std::vector<Vec3f> positions;
  std::vector<Vec3f> normals;
  std::vector<Vec2f> texcoords;
  std::vector<Vec4f> colors;
  bool includedNormals;
  bool includedTexcoords;
  bool includedColors;
  Vec3f nextNormal;
  Vec2f nextTexcoord;
  Vec4f nextColor;

  Diligent::RefCntAutoPtr<Diligent::IBuffer> positionsVB;
  Diligent::RefCntAutoPtr<Diligent::IBuffer> normalsVB;
  Diligent::RefCntAutoPtr<Diligent::IBuffer> texcoordsVB;
  Diligent::RefCntAutoPtr<Diligent::IBuffer> colorsVB;

  void flushAndDraw() {
    RendererState* rs = Window_GetCurrentRS();

    // Update vertex buffers.
    /* position always included */ {
      Diligent::MapHelper<Vec3f> positionsMapped(rs->immediateContext, positionsVB, Diligent::MAP_WRITE,
                                                 Diligent::MAP_FLAG_DISCARD);
      memcpy((Vec3f*)positionsMapped, positions.data(), sizeof(Vec3f) * positions.size());
    }
    if (includedNormals) {
      Diligent::MapHelper<Vec3f> normalsMapped(rs->immediateContext, normalsVB, Diligent::MAP_WRITE,
                                                 Diligent::MAP_FLAG_DISCARD);
      memcpy((Vec3f*)normalsMapped, normals.data(), sizeof(Vec3f) * normals.size());
    }
    if (includedTexcoords) {
      Diligent::MapHelper<Vec2f> texcoordsMapped(rs->immediateContext, texcoordsVB, Diligent::MAP_WRITE,
                                                 Diligent::MAP_FLAG_DISCARD);
      memcpy((Vec2f*)texcoordsMapped, texcoords.data(), sizeof(Vec2f) * texcoords.size());
    }
    if (includedColors) {
      Diligent::MapHelper<Vec4f> colorsMapped(rs->immediateContext, colorsVB, Diligent::MAP_WRITE,
                                                 Diligent::MAP_FLAG_DISCARD);
      memcpy((Vec4f*)colorsMapped, colors.data(), sizeof(Vec4f) * colors.size());
    }

    // Bind vertex buffers.
    std::vector<Diligent::IBuffer*> vbuffers;
    vbuffers.push_back(positionsVB);
    rs->immediateContext->SetVertexBuffers(0, vbuffers.size(), vbuffers.data(), nullptr, Diligent::RESOURCE_STATE_TRANSITION_MODE_TRANSITION, Diligent::SET_VERTEX_BUFFERS_FLAG_RESET);

    // Issue draw call.
    Diligent::DrawAttribs attribs;
    attribs.NumVertices = positions.size();
    attribs.Flags = Diligent::DRAW_FLAG_VERIFY_ALL;
    rs->immediateContext->Draw(attribs);

    // Create data for next draw call.
    positions.clear();
    normals.clear();
    texcoords.clear();
    colors.clear();
  }
};

static ImmediateDrawSet ids;

//static bgfx::ProgramHandle programPOnly;
//static bgfx::ProgramHandle programPAndTC;
//
//
//struct Vertex_P {
//  Vec3f p;
//
//  static const bgfx::VertexLayout& layout() {
//    static bgfx::VertexLayout layout = []() -> bgfx::VertexLayout {
//        bgfx::VertexLayout layout;
//        layout.add(bgfx::Attrib::Position, 3, bgfx::AttribType::Float, false, false);
//        return layout;
//    }();
//    return layout;
//  }
//};
//
//struct Vertex_P_TC {
//  Vec3f p;
//  Vec2f tc;
//
//  static const bgfx::VertexLayout& layout() {
//    static bgfx::VertexLayout layout = []() -> bgfx::VertexLayout {
//        bgfx::VertexLayout layout;
//        layout.add(bgfx::Attrib::Position, 3, bgfx::AttribType::Float, false, false);
//        layout.add(bgfx::Attrib::TexCoord0, 2, bgfx::AttribType::Float, false, false);
//        return layout;
//    }();
//    return layout;
//  }
//};

void Draw_PushAlpha (float a) {
  if (alphaIndex + 1 >= MAX_STACK_DEPTH)
      Fatal("Draw_PushAlpha: Maximum alpha stack depth exceeded");

  float prevAlpha = alphaIndex >= 0 ? alphaStack[alphaIndex] : 1;
  float alpha = a * prevAlpha;
  alphaStack[++alphaIndex] = alpha;
  drawColor = Vec4f_Create(color.x, color.y, color.z, color.w * alpha);
}

void Draw_PopAlpha () {
  if (alphaIndex < 0)
      Fatal("Draw_PopAlpha Attempting to pop an empty alpha stack");

  alphaIndex--;
  float alpha = alphaIndex >= 0 ? alphaStack[alphaIndex] : 1;
  drawColor = Vec4f_Create(color.x, color.y, color.z, color.w * alpha);
}

void Draw_Axes (
  Vec3f const* pos,
  Vec3f const* x,
  Vec3f const* y,
  Vec3f const* z,
  float scale,
  float _alpha)
{
  Vec3f left    = Vec3f_Add(*pos, Vec3f_Muls(*x, scale));
  Vec3f up      = Vec3f_Add(*pos, Vec3f_Muls(*y, scale));
  Vec3f forward = Vec3f_Add(*pos, Vec3f_Muls(*z, scale));

  ids.begin(PrimitiveType::Lines);
  ids.color(1, 0.25f, 0.25f, _alpha);
  ids.vertex3(*pos);
  ids.vertex3(left);
  ids.color(0.25f, 1, 0.25f, _alpha);
  ids.vertex3(*pos);
  ids.vertex3(up);
  ids.color(0.25f, 0.25f, 1, _alpha);
  ids.vertex3(*pos);
  ids.vertex3(forward);
  ids.end();

  ids.begin(PrimitiveType::Points);
  ids.color(1, 1, 1, _alpha);
  ids.vertex3(*pos);
  ids.end();
}

void Draw_Border (float s, float x, float y, float w, float h) {
  Draw_Rect(x, y, w, s);
  Draw_Rect(x, y + h - s, w, s);
  Draw_Rect(x, y + s, s, h - 2*s);
  Draw_Rect(x + w - s, y + s, s, h - 2*s);
}

void Draw_Box3 (Box3f const* self) {
  Metric_AddDrawImm(6, 12, 24);

  ids.begin(PrimitiveType::Quads);

  /* Left. */
  ids.vertex3(Vec3f_Create(self->lower.x, self->lower.y, self->lower.z));
  ids.vertex3(Vec3f_Create(self->lower.x, self->lower.y, self->upper.z));
  ids.vertex3(Vec3f_Create(self->lower.x, self->upper.y, self->upper.z));
  ids.vertex3(Vec3f_Create(self->lower.x, self->upper.y, self->lower.z));

  /* Right. */
  ids.vertex3(Vec3f_Create(self->upper.x, self->lower.y, self->lower.z));
  ids.vertex3(Vec3f_Create(self->upper.x, self->upper.y, self->lower.z));
  ids.vertex3(Vec3f_Create(self->upper.x, self->upper.y, self->upper.z));
  ids.vertex3(Vec3f_Create(self->upper.x, self->lower.y, self->upper.z));

  /* Front. */
  ids.vertex3(Vec3f_Create(self->lower.x, self->lower.y, self->upper.z));
  ids.vertex3(Vec3f_Create(self->upper.x, self->lower.y, self->upper.z));
  ids.vertex3(Vec3f_Create(self->upper.x, self->upper.y, self->upper.z));
  ids.vertex3(Vec3f_Create(self->lower.x, self->upper.y, self->upper.z));

  /* Back. */
  ids.vertex3(Vec3f_Create(self->lower.x, self->lower.y, self->lower.z));
  ids.vertex3(Vec3f_Create(self->lower.x, self->upper.y, self->lower.z));
  ids.vertex3(Vec3f_Create(self->upper.x, self->upper.y, self->lower.z));
  ids.vertex3(Vec3f_Create(self->upper.x, self->lower.y, self->lower.z));

  /* Top. */
  ids.vertex3(Vec3f_Create(self->lower.x, self->upper.y, self->lower.z));
  ids.vertex3(Vec3f_Create(self->lower.x, self->upper.y, self->upper.z));
  ids.vertex3(Vec3f_Create(self->upper.x, self->upper.y, self->upper.z));
  ids.vertex3(Vec3f_Create(self->upper.x, self->upper.y, self->lower.z));

  /* Bottom. */
  ids.vertex3(Vec3f_Create(self->lower.x, self->lower.y, self->lower.z));
  ids.vertex3(Vec3f_Create(self->upper.x, self->lower.y, self->lower.z));
  ids.vertex3(Vec3f_Create(self->upper.x, self->lower.y, self->upper.z));
  ids.vertex3(Vec3f_Create(self->lower.x, self->lower.y, self->upper.z));

//  /* Write indices */
//  uint16_t* indices = (uint16_t*)tib.data;
//  for (int i = 0; i < 6; ++i) {
//    /* for each quad */
//    uint16_t firstVertex = i * 4;
//    *indices++ = firstVertex;
//    *indices++ = firstVertex + 1;
//    *indices++ = firstVertex + 2;
//    *indices++ = firstVertex;
//    *indices++ = firstVertex + 2;
//    *indices++ = firstVertex + 3;
//  }

  ids.end();
}

void Draw_Clear (float r, float g, float b, float a) {
//  auto status = glCheckFramebufferStatus(GL_FRAMEBUFFER);
//  if (status != GL_FRAMEBUFFER_COMPLETE) {
//    Warn("Framebuffer is incomplete, skipping clear: %d", status);
//  } else {
//    GLCALL(glClearColor(r, g, b, a))
//    GLCALL(glClear(GL_COLOR_BUFFER_BIT))
//  }
}

void Draw_ClearDepth (float d) {
//  bgfx::setViewClear(0, BGFX_CLEAR_DEPTH, 0, d, 0);
}

void Draw_Color (float r, float g, float b, float a) {
  float alpha = alphaIndex >= 0 ? alphaStack[alphaIndex] : 1;
  color = Vec4f_Create(r, g, b, a);
  drawColor = Vec4f_Create(r, g, b, a * alpha);
}

void Draw_Flush () {
  Metric_Inc(Metric_Flush);
//  GLCALL(glFinish())
}

void Draw_Line (float x1, float y1, float x2, float y2) {
  ids.begin(PrimitiveType::Lines);
  ids.vertex2(Vec2f_Create(x1, y1));
  ids.vertex2(Vec2f_Create(x2, y2));
  ids.end();
}

void Draw_Line3 (Vec3f const* p1, Vec3f const* p2) {
  ids.begin(PrimitiveType::Lines);
  ids.vertex3(*p1);
  ids.vertex3(*p2);
  ids.end();
}

void Draw_LineWidth (float width) {
  ids.lineWidth(width);
}

void Draw_Plane (Vec3f const* p, Vec3f const* n, float scale) {
  Vec3f e1 = Abs(n->x) < 0.7f ? Vec3f_Create(1, 0, 0) : Vec3f_Create(0, 1, 0);
  e1 = Vec3f_Normalize(Vec3f_Reject(e1, *n));
  Vec3f e2 = Vec3f_Cross(*n, e1);

  Vec3f p0 = Vec3f_Add(*p, Vec3f_Add(Vec3f_Muls(e1, -scale), Vec3f_Muls(e2, -scale)));
  Vec3f p1 = Vec3f_Add(*p, Vec3f_Add(Vec3f_Muls(e1,  scale), Vec3f_Muls(e2, -scale)));
  Vec3f p2 = Vec3f_Add(*p, Vec3f_Add(Vec3f_Muls(e1,  scale), Vec3f_Muls(e2,  scale)));
  Vec3f p3 = Vec3f_Add(*p, Vec3f_Add(Vec3f_Muls(e1, -scale), Vec3f_Muls(e2,  scale)));

  Metric_AddDrawImm(1, 2, 4);
  ids.begin(PrimitiveType::Quads);
  ids.vertex3(p0);
  ids.vertex3(p1);
  ids.vertex3(p2);
  ids.vertex3(p3);
  ids.end();
}

void Draw_Point (float x, float y) {
  ids.begin(PrimitiveType::Points);
  ids.vertex2(Vec2f_Create(x, y));
  ids.end();
}

void Draw_Point3 (float x, float y, float z) {
  ids.begin(PrimitiveType::Points);
  ids.vertex3(Vec3f_Create(x, y, z));
  ids.end();
}

void Draw_PointSize (float size) {
  ids.pointSize(size);
}

void Draw_Poly (Vec2f const* points, int count) {
  Metric_AddDrawImm(1, count - 2, count);
  ids.begin(PrimitiveType::Polygon);
  for (int i = 0; i < count; ++i)
    ids.vertex2(points[i]);
  ids.end();
}

void Draw_Poly3 (Vec3f const* points, int count) {
  Metric_AddDrawImm(1, count - 2, count);
  ids.begin(PrimitiveType::Polygon);
  for (int i = 0; i < count; ++i)
    ids.vertex3(points[i]);
  ids.end();
}

void Draw_Quad (Vec2f const* p1, Vec2f const* p2, Vec2f const* p3, Vec2f const* p4) {
  Metric_AddDrawImm(1, 2, 4);
  ids.begin(PrimitiveType::Quads);
  ids.texCoord(Vec2f_Create(0, 0)); ids.vertex2(*p1);
  ids.texCoord(Vec2f_Create(0, 1)); ids.vertex2(*p2);
  ids.texCoord(Vec2f_Create(1, 1)); ids.vertex2(*p3);
  ids.texCoord(Vec2f_Create(1, 0)); ids.vertex2(*p4);
  ids.end();
}

void Draw_Quad3 (Vec3f const* p1, Vec3f const* p2, Vec3f const* p3, Vec3f const* p4) {
  Metric_AddDrawImm(1, 2, 4);
  ids.begin(PrimitiveType::Quads);
  ids.texCoord(Vec2f_Create(0, 0)); ids.vertex3(*p1);
  ids.texCoord(Vec2f_Create(0, 1)); ids.vertex3(*p2);
  ids.texCoord(Vec2f_Create(1, 1)); ids.vertex3(*p3);
  ids.texCoord(Vec2f_Create(1, 0)); ids.vertex3(*p4);
  ids.end();
}

void Draw_Rect (float x1, float y1, float xs, float ys) {
  float x2 = x1 + xs;
  float y2 = y1 + ys;
  Metric_AddDrawImm(1, 2, 4);
  ids.begin(PrimitiveType::Quads);
  ids.texCoord(Vec2f_Create(0, 0)); ids.vertex2(Vec2f_Create(x1, y1));
  ids.texCoord(Vec2f_Create(0, 1)); ids.vertex2(Vec2f_Create(x1, y2));
  ids.texCoord(Vec2f_Create(1, 1)); ids.vertex2(Vec2f_Create(x2, y2));
  ids.texCoord(Vec2f_Create(1, 0)); ids.vertex2(Vec2f_Create(x2, y1));
  ids.end();
}

void Draw_SmoothLines (bool enabled) {
//  if (enabled) {
//    GLCALL(glEnable(GL_LINE_SMOOTH))
//    GLCALL(glHint(GL_LINE_SMOOTH_HINT, GL_NICEST))
//  } else {
//    GLCALL(glDisable(GL_LINE_SMOOTH))
//    GLCALL(glHint(GL_LINE_SMOOTH_HINT, GL_FASTEST))
//  }
}

void Draw_SmoothPoints (bool enabled) {
//  if (enabled) {
//    GLCALL(glEnable(GL_POINT_SMOOTH))
//    GLCALL(glHint(GL_POINT_SMOOTH_HINT, GL_NICEST))
//  } else {
//    GLCALL(glDisable(GL_POINT_SMOOTH))
//    GLCALL(glHint(GL_POINT_SMOOTH_HINT, GL_FASTEST))
//  }
}

inline static Vec3f Spherical (float r, float yaw, float pitch) {
  return Vec3f_Create(
    r * Sin(pitch) * Cos(yaw),
    r * Cos(pitch),
    r * Sin(pitch) * Sin(yaw));
}

/* TODO JP : Lazy creation of VBO / IBO & glDraw instead of immediate. */
void Draw_Sphere (Vec3f const* p, float r) {
  const size_t res = 7;
  const float fRes = float(res);

  /* First Row */ {
    Metric_AddDrawImm(res, res, res * 3);
    ids.begin(PrimitiveType::Triangles);
    float lastTheta = float(res - 1) / fRes * Tau;
    float phi = 1.0f / fRes * Pi;
    Vec3f tc = Vec3f_Add(*p, Spherical(r, 0, 0));
    for (size_t iTheta = 0; iTheta < res; iTheta++) {
      float theta = float(iTheta) / fRes * Tau;
      Vec3f br = Vec3f_Add(*p, Spherical(r, lastTheta, phi));
      Vec3f bl = Vec3f_Add(*p, Spherical(r, theta, phi));
      ids.vertex3(br);
      ids.vertex3(tc);
      ids.vertex3(bl);
      lastTheta = theta;
    }
    ids.end();
  }

  /* Middle Rows */ {
    Metric_AddDrawImm(res - 2, 2 * (res - 2), 4 * (res - 2));
    ids.begin(PrimitiveType::Quads);
    float lastPhi = 1.0f / fRes * Pi;
    float lastTheta = float(res - 1) / fRes * Tau;

    for (size_t iPhi = 2; iPhi < res; iPhi++) {
      float phi = float(iPhi) / fRes * Pi;
      for (size_t iTheta = 0; iTheta < res; iTheta++) {
        float theta = float(iTheta) / fRes * Tau;
        Vec3f br = Vec3f_Add(*p, Spherical(r, lastTheta, phi));
        Vec3f tr = Vec3f_Add(*p, Spherical(r, lastTheta, lastPhi));
        Vec3f tl = Vec3f_Add(*p, Spherical(r, theta, lastPhi));
        Vec3f bl = Vec3f_Add(*p, Spherical(r, theta, phi));
        ids.vertex3(br);
        ids.vertex3(tr);
        ids.vertex3(tl);
        ids.vertex3(bl);
        lastTheta = theta;
      }
      lastPhi = phi;
    }
    ids.end();
  }

  /* Bottom Row */ {
    Metric_AddDrawImm(res, res, res * 3);
    ids.begin(PrimitiveType::Triangles);
    float lastTheta = float(res - 1) / fRes * Tau;
    float phi = float(res - 1) / fRes * Pi;
    Vec3f bc = Vec3f_Add(*p, Spherical(r, 0, Pi));

    for (size_t iTheta = 0; iTheta < res; iTheta++) {
      float theta = float(iTheta) / fRes * Tau;
      Vec3f tr = Vec3f_Add(*p, Spherical(r, lastTheta, phi));
      Vec3f tl = Vec3f_Add(*p, Spherical(r, theta, phi));
      ids.vertex3(tr);
      ids.vertex3(tl);
      ids.vertex3(bc);
      lastTheta = theta;
    }
    ids.end();
  }
}

void Draw_Tri (Vec2f const* v1, Vec2f const* v2, Vec2f const* v3) {
  Metric_AddDrawImm(1, 1, 3);
  ids.begin(PrimitiveType::Triangles);
  ids.texCoord(Vec2f_Create(0, 0)); ids.vertex2(*v1);
  ids.texCoord(Vec2f_Create(0, 1)); ids.vertex2(*v2);
  ids.texCoord(Vec2f_Create(1, 1)); ids.vertex2(*v3);
  ids.end();
}

void Draw_Tri3 (Vec3f const* v1, Vec3f const* v2, Vec3f const* v3) {
  Metric_AddDrawImm(1, 1, 3);
  ids.begin(PrimitiveType::Triangles);
  ids.texCoord(Vec2f_Create(0, 0)); ids.vertex3(*v1);
  ids.texCoord(Vec2f_Create(0, 1)); ids.vertex3(*v2);
  ids.texCoord(Vec2f_Create(1, 1)); ids.vertex3(*v3);
  ids.end();
}

bool Draw_Init(Window* w) {
  ids.createResources(w);
  return true;
}

void Draw_Free() {
  ids.free();
}
