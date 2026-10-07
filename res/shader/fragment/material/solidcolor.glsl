#include fragment

#group 1
layout(std140) uniform MaterialParams {
  vec3 color;
};

void main() {
  outColor = vec4(color, 1.0);
  FRAGMENT_CORRECT_DEPTH;
}
