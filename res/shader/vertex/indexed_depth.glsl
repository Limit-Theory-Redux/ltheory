#include vertex

uniform float depthValue;

void main() {
  // Keep geometry and rasterization identical to IndexedGeometry. The only
  // added variable is an explicit clip-space depth value for the depth test.
  gl_Position = vec4(vertex_position.xy * 0.75, depthValue, 1.0);
}
