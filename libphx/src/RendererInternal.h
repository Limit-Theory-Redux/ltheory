#pragma once

// Rendering utility functions.

#include "GraphicsTypes.h"
#include "RefCntAutoPtr.hpp"
#include "MapHelper.hpp"

#include <vector>

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