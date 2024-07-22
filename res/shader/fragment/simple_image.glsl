#include fragment

uniform sampler2D image;

void main() {
  outColor = texture(image, uv);
}
