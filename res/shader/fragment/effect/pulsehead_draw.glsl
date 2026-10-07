// Draw-block variant of pulsehead.glsl for scene passes: the color (xyz) and
// alpha (w) come from drawUser[0] of the group-2 DrawBlock.
#include fragment
#include color
#include math
#include draw_block

#define color (drawUser[0].xyz)
#define alpha (drawUser[0].w)

void main() {
  float r = length(uv);
  float a = 0.0;
  a += exp(-sqrt(256.0 * r));
  a += exp(-sqrt(128.0 * r));
  a *= 4.0;
  vec3 c = color;
  c *= c / avg(c);
  outColor = vec4(a * alpha * c, 1.0);
  FRAGMENT_CORRECT_DEPTH;
}
