// Space dust streaks (SpaceDustEntity, paired with vertex/dust_streak.glsl):
// a soft line (soft across and at both ends, like the old LT dust flecks),
// brightest where the speck is now and dimmer towards where it was, in the
// layer's colour, added to the picture.
#include fragment
#include math

#include draw_block

#define dustColor (drawUser[3].xyz)

in float fade;

void main() {
  float across = exp(-pow2(1.1 * uv.y));
  float ends = smoothstep(0.0, 0.15, uv.x) * smoothstep(1.0, 0.6, uv.x);
  float tailDim = mix(1.0, 0.35, uv.x);
  outColor = vec4(dustColor * (fade * across * ends * tailDim), 1.0);
  FRAGMENT_CORRECT_DEPTH;
}
