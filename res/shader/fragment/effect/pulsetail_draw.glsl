// Draw-block variant of pulsetail.glsl for scene passes: the color (xyz) and
// alpha (w) come from drawUser[0] of the group-2 DrawBlock.
#include fragment
#include math

#include draw_block

#define color (drawUser[0].xyz)
#define alpha (drawUser[0].w)

void main() {
  float u = uv.x;
  float v = uv.y;
  float a = 0.0;
  v = 1.0 + v;

  float iv = 1.0 - v;
  u = max(0.0, abs(u) - 0.010);
  v = saturate(v);
  a += 1.5 * exp(-sqrt(128.0 * u)) * v;
  a *= exp(-8.0 * iv);
  a *= 1.0 - exp(-pow2(32.0 * iv));
  a *= 4.0;
  vec3 c = color;
  c *= c / avg(c);
  outColor = vec4(a * alpha * c, 1.0);
  FRAGMENT_CORRECT_DEPTH;
}
