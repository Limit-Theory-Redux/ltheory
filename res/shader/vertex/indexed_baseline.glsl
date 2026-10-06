#include vertex

void main() {
  // Experiment 1 deliberately bypasses camera matrices and log-depth. The
  // mesh path is the only variable under test; every vertex projects into a
  // deterministic centered cube with valid WGPU clip-space depth.
  gl_Position = vec4(vertex_position.xy * 0.75, 0.5, 1.0);
}
