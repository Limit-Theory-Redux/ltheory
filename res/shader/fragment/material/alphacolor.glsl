#include "fragment.glsl"

uniform vec4 color;

void main() {
  fragColor = color;
  FRAGMENT_CORRECT_DEPTH;
}
