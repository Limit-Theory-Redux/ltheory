#include vertex

void main() {
  VS_BEGIN;
  pos = vertex_position;
  normal = vertex_normal;
  flogz = 1.0;
  gl_Position = vec4(vertex_position.xy * 0.75, 0.5, 1.0);
}
