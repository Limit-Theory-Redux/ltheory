#include fragment

void main() {
  // Constant output isolates indexed rasterization from material and lighting.
  outColor = vec4(1.0, 0.125, 0.0, 1.0);
}
