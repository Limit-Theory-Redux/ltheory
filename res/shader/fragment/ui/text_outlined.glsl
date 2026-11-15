#include fragment
#include imm

#define color imm_color
#define outlineColor imm_p
#define haloColor imm_q

/* The outlined glyph atlas (`Font::draw_outlined`): RGBA8 pages with the
   coverage in red (gamma 1/1.8, read like `text.glsl` reads it), the 1 px
   outline in green and the 3 px halo in blue. A string is drawn twice: first
   outline and halo (text alpha 0), then the text itself (outline and halo
   alpha 0), so no glyph's halo lands on a neighbour's text. */
#group 3
uniform sampler2D glyph;

void main() {
  vec3 t = texture(glyph, uv).rgb;
  float fill = sqrt(t.r) * color.w;
  float outline = t.g * outlineColor.w;
  float halo = t.b * haloColor.w;

  /* Back to front: the halo, the outline over it, the text on top. `ink` is
     the colour laid down so far, `clear` how much of what is behind still
     shows. The text lays down its colour times fill squared, as `text.glsl`
     does under the alpha blend, so it looks exactly like plain text. */
  vec3 ink = haloColor.xyz * halo;
  float clear = 1.0 - halo;
  ink = outlineColor.xyz * outline + ink * (1.0 - outline);
  clear *= 1.0 - outline;
  ink = color.xyz * fill * fill + ink * (1.0 - fill);
  clear *= 1.0 - fill;

  /* The alpha blend gives src.rgb * src.a + dst * (1 - src.a). */
  float a = 1.0 - clear;
  outColor = a > 0.0 ? vec4(ink / a, a) : vec4(0.0);
}
