#include fragment

uniform sampler2D src;

void main() {
  outColor = texture(src, uv);
}
