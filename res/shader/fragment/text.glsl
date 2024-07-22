#include fragment

uniform vec3 color;

void main() {
  outColor = vec4(color, uv.x);
}
