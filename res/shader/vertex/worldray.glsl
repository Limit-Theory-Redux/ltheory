#include vertex

out vec3 worldOrigin;
out vec3 worldDir;

void main () {
  // Camera-relative: origin is always 0, direction is rotation-only (no
  // translation) - avoids precision loss from reconstructing a world-space
  // point far from the origin and subtracting it back out.
  worldOrigin = vec3(0.0);

  vec4 p2 = mProjInv * vec4(vertex_position.xy, 1.0, 1.0);
  p2 /= p2.w;
  worldDir = mat3(mViewInv) * p2.xyz;

  gl_Position = vec4(vertex_position.xyz, 1.0);
  uv = vertex_uv.xy;
}
