#include vertex

void main() {
  VS_BEGIN;
  normal = normalize((mWorld * vec4(vertex_normal, 0.0)).xyz);
  vec4 worldPos = mWorld * vec4(vertex_position, 1.0);
  pos = worldPos.xyz;
  uv = vertex_uv;
  gl_Position = mProj * mView * worldPos;
}
