#include common
#include view_block

/* The debug and backdrop vertex of the immediate batcher (`Imm3DVertex`):
   camera-relative position, uv and color, with the scene's logarithmic depth. */

in vec3 vertex_position;
in vec2 vertex_uv;
in vec4 vertex_color;

out vec2 uv;
out vec3 pos;
out float flogz;
flat out vec4 imm_color;

void main() {
  uv = vertex_uv;
  pos = vertex_position;
  imm_color = vertex_color;
  vec4 p = mProj * (mView * vec4(vertex_position, 1.0));
  p.z = log2(max(1e-6, 1.0 + abs(p.w))) * Fcoef - 1.0;
  p.z *= p.w;
  flogz = 1.0 + p.w;
  gl_Position = p;
}
