// Draw-block variant of laserbolttail.glsl for scene passes: the color (xyz) and
// alpha (w) come from drawUser[0] of the group-2 DrawBlock.
#include fragment
#include math

#include draw_block

#define color (drawUser[0].xyz)
#define alpha (drawUser[0].w)

void main() {
  float u = max(0.0, abs(uv.x) - 0.006);
  float v = saturate(1.0 + uv.y);
  float core = exp(-sqrt(256.0 * u));
  float halo = exp(-sqrt(96.0 * u));
  float taper = exp(-7.0 * (1.0 - v));
  float edge = 1.0 - exp(-pow2(28.0 * (1.0 - v)));
  vec3 hot = mix(color, vec3(1.0, 0.88, 0.80), 0.72);
  hot *= hot / avg(hot);
  outColor = vec4(alpha * (1.15 * core + 0.24 * halo) * taper * edge * hot, 1.0);
  FRAGMENT_CORRECT_DEPTH;
}
