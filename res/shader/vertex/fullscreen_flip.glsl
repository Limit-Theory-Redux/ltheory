#include vertex

/* `fullscreen`, flipped vertically: the quad covers the viewport with uv.y = 0
   at the bottom of a y-down target, the way `Draw.Rect(0, h, w, -h)` did. Used
   to present a texture into the window. */

void main() {
  uv = vertex_uv.xy;
  pos = vertex_position.xyz;
  vec2 p = vec2(vertex_position.x, 1.0 - vertex_position.y);
  gl_Position = mProjUI * (mWorldViewUI * vec4(p * ubo_viewport.zw, 0.0, 1.0));
}
