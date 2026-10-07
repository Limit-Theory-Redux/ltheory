#include fragment

#group 2
layout(std140) uniform Params {
  vec4 color;
};

void main() {
  outColor = color;
}
