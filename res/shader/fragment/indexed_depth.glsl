#include fragment

uniform vec4 color;

void main() {
  // Constant colors make depth ordering observable without material or lighting.
  outColor = color;
}
