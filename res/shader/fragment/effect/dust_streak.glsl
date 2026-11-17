// Space dust streaks (SpaceDustEntity, paired with vertex/dust_streak.glsl):
// a soft line, brightest where the speck is now and dimmer towards where it
// was, in the layer's colour, added to the picture.
#include fragment
#include math

#include draw_block

#define dustColor (drawUser[3].xyz)

in float fade;

void main() {
  float across = exp(-pow2(1.8 * uv.y));
  float tailDim = mix(1.0, 0.35, uv.x);
  outColor = vec4(dustColor * (fade * across * tailDim), 1.0);
  FRAGMENT_CORRECT_DEPTH;
}
