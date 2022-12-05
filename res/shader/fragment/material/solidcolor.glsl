#include "fragment.glsl"

uniform vec3 color;

void main() {
  fragColor = vec4(color, 1.0);
  FRAGMENT_CORRECT_DEPTH;
}
