#include common
#include view_block
#include vertex_base

/* Post-processing, generation and present: `pass:drawFullscreen` draws the
   built-in unit quad (positions and uv in [0,1]^2) over the whole viewport.
   The positions are mapped straight to NDC, so the target's orientation never
   matters: uv.y = 0 is the bottom row of whatever the pass renders to (a
   texture row 0, or the bottom of the window), which is what a y-up
   `Draw.Rect(0, 0, w, h)` in a texture pass and the flipped
   `Draw.Rect(0, h, w, -h)` into the window both gave. */

void main() {
  uv = vertex_uv.xy;
  pos = vertex_position.xyz;
  gl_Position = vec4(2.0 * vertex_position.xy - 1.0, 0.0, 1.0);
}
