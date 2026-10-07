#include fragment

#group 3
uniform sampler2D src;

void main() {
  outColor = texture(src, uv * 4.006);
}
