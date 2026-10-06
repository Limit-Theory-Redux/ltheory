#include fragment

uniform sampler2D tex;

void main() {
  outColor = texture(tex, uv);
}
