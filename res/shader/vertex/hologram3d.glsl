#include common
#include view_block
#include vertex_base

/* Trails of the 3D system map: world-space geometry (camera-relative) drawn
   with the map's own view and projection, not the pass camera's. `ui/trail3d`
   reads the color from the varying. */

flat out vec4 imm_color;

#group 2
layout(std140) uniform Params {
  mat4 trailView;
  mat4 trailProj;
  vec4 trailColor;
};

void main() {
  uv = vertex_uv;
  imm_color = trailColor;
  gl_Position = trailProj * (trailView * vec4(vertex_position, 1.0));
}
