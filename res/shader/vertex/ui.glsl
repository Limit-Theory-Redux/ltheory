#include vertex

uniform mat4 mWorldViewUI;
uniform mat4 mProjUI;

#autovar mat4 mWorldViewUI
#autovar mat4 mProjUI

void main() {
  uv = vertex_uv.xy;
  pos = vertex_position.xyz;
  gl_Position = mProjUI * (mWorldViewUI * vec4(vertex_position, 1));
}
