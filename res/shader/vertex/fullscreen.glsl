#include vertex

/* Draws the built-in unit quad (`pass:drawFullscreen`) over the whole viewport.
   The quad's positions are [0,1]^2; scaling them by the viewport size and going
   through the UI projection reproduces `Draw.Rect(0, 0, w, h)` exactly (y-down
   on the backbuffer, y-up in textures, uv.y = 0 at the top/bottom accordingly). */

void main() {
  uv = vertex_uv.xy;
  pos = vertex_position.xyz;
  gl_Position = mProjUI * (mWorldViewUI * vec4(vertex_position.xy * ubo_viewport.zw, 0.0, 1.0));
}
