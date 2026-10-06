#include vertex

layout(location = 10) in uint instanceIndex;

void main() {
  VS_BEGIN;
  pos = vertex_position;
  normal = vertex_normal;
  flogz = 1.0;
  float x = (float(instanceIndex) - 1.0) * 0.65;
  gl_Position = vec4(vertex_position.xy * 0.30 + vec2(x, 0.0), 0.5, 1.0);
}
