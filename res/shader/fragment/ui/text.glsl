#include fragment
#include imm

#define color imm_color

/* The glyph atlas: one R8 page per font, gamma 1/1.8 coverage in red. */
#group 3
uniform sampler2D glyph;

void main() {
  float alpha = sqrt(texture(glyph, uv).r);
  vec3 c = color.xyz;
  outColor = alpha * color.w * vec4(c, 1.0);
}
