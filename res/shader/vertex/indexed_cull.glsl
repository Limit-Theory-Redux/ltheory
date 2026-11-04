#include vertex

void main() {
  // The culling experiment keeps projection and depth constant. Only triangle
  // winding and the RenderState cull mode are allowed to affect visibility.
  gl_Position = vec4(vertex_position.xy * 0.75, 0.5, 1.0);
}
