#include fragment
#include imm

#define color imm_color
#define outlineColor imm_p
#define haloColor imm_q
#define glowColor imm_r

/* The outlined glyph atlas (`Font::draw_with_outline`): RGBA8 pages with the
   coverage in red (gamma 1/1.8, read like `text.glsl` reads it), the 1 px
   outline in green, the 3 px halo in blue and the glow (the ink blurred) in
   alpha. A string is drawn in passes, each leaving the others' colours at
   alpha 0: the glow, then outline and halo, then the text itself, so no
   glyph's glow lands on a neighbour's halo and no halo on a neighbour's
   text. */
#group 3
uniform sampler2D glyph;

void main() {
  vec4 t = texture(glyph, uv);
  float fill = sqrt(t.r) * color.w;
  float outline = t.g * outlineColor.w;
  float halo = t.b * haloColor.w;
  float glow = t.a * glowColor.w;

  /* Back to front: the glow, the halo over it, the outline, the text on top.
     `ink` is the colour laid down so far, `clear` how much of what is behind
     still shows. The text lays down its colour times fill squared, as
     `text.glsl` does under the alpha blend, so it looks exactly like plain
     text. */
  vec3 ink = glowColor.xyz * glow;
  float clear = 1.0 - glow;
  ink = haloColor.xyz * halo + ink * (1.0 - halo);
  clear *= 1.0 - halo;
  ink = outlineColor.xyz * outline + ink * (1.0 - outline);
  clear *= 1.0 - outline;
  ink = color.xyz * fill * fill + ink * (1.0 - fill);
  clear *= 1.0 - fill;

  /* The alpha blend gives src.rgb * src.a + dst * (1 - src.a). */
  float a = 1.0 - clear;
  outColor = a > 0.0 ? vec4(ink / a, a) : vec4(0.0);
}
