#include common
#include view_block
#include vertex_base

/* Asteroid dots of the system map: a mesh of quads whose four vertices share a
   world position (x, y) and carry the corner in uv. The quad becomes a dot of
   `dotSize` pixels around the position on screen. `ui/mappoints` reads the
   color from the varying. */

flat out vec4 imm_color;

#group 2
layout(std140) uniform Params {
  vec4 mapGeom;    // xy: screen position of the belt/ring center, zw: viewport size
  vec4 mapParams;  // x: zoom factor, y: dot half-size in pixels
  vec4 dotColor;
};

void main() {
  uv = vertex_uv;
  imm_color = dotColor;

  vec2 worldPos = vertex_position.xy;
  vec2 screenPos = mapGeom.xy + worldPos * mapParams.x;

  // Add pixel-sized offset based on UV corner
  vec2 cornerOffset = (vertex_uv - 0.5) * 2.0 * mapParams.y;
  screenPos += cornerOffset;

  vec2 ndc = (screenPos / mapGeom.zw) * 2.0 - 1.0;
  ndc.y = -ndc.y;
  gl_Position = vec4(ndc, 0.0, 1.0);
}
