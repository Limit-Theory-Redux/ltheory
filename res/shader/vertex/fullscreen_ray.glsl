#include common
#include view_block
#include vertex_base

/* `fullscreen_ndc` plus the camera-relative world ray of each pixel
   (deferred lighting). */

out vec3 worldOrigin;
out vec3 worldDir;

void main() {
  vec2 ndc = 2.0 * vertex_position.xy - 1.0;

  // Camera-relative: origin is always 0, direction is rotation-only (no
  // translation) - avoids precision loss from reconstructing a world-space
  // point far from the origin and subtracting it back out.
  worldOrigin = vec3(0.0);

  vec4 p2 = mProjInv * vec4(ndc, 1.0, 1.0);
  p2 /= p2.w;
  worldDir = mat3(mViewInv) * p2.xyz;

  gl_Position = vec4(ndc, 0.0, 1.0);
  uv = vertex_uv.xy;
}
