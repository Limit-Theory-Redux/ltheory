// Draw-block variant of laserbolthead.glsl for scene passes: the color (xyz) and
// alpha (w) come from drawUser[0] of the group-2 DrawBlock.
#include fragment
#include color
#include math

#include draw_block

#define color (drawUser[0].xyz)
#define alpha (drawUser[0].w)

void main() {
  float r = length(uv);
  float core = exp(-sqrt(512.0 * r));
  float halo = exp(-sqrt(144.0 * r));
  vec3 hot = mix(color, vec3(1.0, 0.92, 0.86), 0.82);
  hot *= hot / avg(hot);
  outColor = vec4(alpha * (3.5 * core + 0.55 * halo) * hot, 1.0);
  FRAGMENT_CORRECT_DEPTH;
}
