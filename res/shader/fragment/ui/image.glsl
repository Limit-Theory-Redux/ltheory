#include fragment
#include imm

#group 3
uniform sampler2D image;

void main() {
  outColor = texture(image, uv) * imm_color;
}
