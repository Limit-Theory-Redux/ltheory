#include fragment

uniform vec3 color;

void main() {
  // This is the production solid-color material contract without the
  // log-depth write, isolating uniform/material output from depth behavior.
  outColor = vec4(color, 1.0);
}
