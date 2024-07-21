#include fragment

uniform vec4 color;

void main() {
  outColor = color;
  FRAGMENT_CORRECT_DEPTH;
}
