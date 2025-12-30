#include vertex

out vec3 worldOrigin;
out vec3 worldDir;

void main () {
  // Camera-relative rendering: camera is always at origin
  // This avoids float precision issues with large world coordinates
  worldOrigin = vec3(0.0);

  // Unproject screen position to view space, then transform by rotation only
  vec4 p2 = mProjInv * vec4(vertex_position.xy, 1.0, 1.0);
  p2 /= p2.w;
  // Use mat3 to extract rotation only (no translation)
  worldDir = mat3(mViewInv) * p2.xyz;

  gl_Position = vec4(vertex_position.xyz, 1.0);
  uv = vertex_uv.xy;
}
