#include vertex



void main() {
  uv = vertex_uv.xy;
  pos = vertex_position.xyz;
  gl_Position = mProjUI * (mWorldViewUI * vec4(vertex_position, 1));
}
