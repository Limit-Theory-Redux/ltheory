#include fragment
#include math

#include imm
#include shape_params

/* The bendable stroke (`Shape::Stroke`, render/stroke.rs): a line `len`
   pixels long that leaves the origin along +x and turns towards +y by `curv`
   radians per pixel (0 = straight, `curv * len` = 2 pi = a full circle).
   `uv` is the pixel's position in that frame, in pixels (the quad's corners
   carry it, so it interpolates exactly).

   The distance to the stroke goes through one look curve, the one the other
   `ui/*` shapes use with fixed constants, here read from the shape parameter
   block: a solid core with a soft edge plus a glow, times `boost`. The
   output is premultiplied (`PreMultAlpha`): the light is added, and alpha
   covers what is behind (`shade` 0 = additive neon, 1 = normal alpha), with
   an optional dark halo. */

#define len (imm_p.x)
#define curv (imm_p.y)
#define halfWidth (imm_p.z)
#define caps (imm_p.w)
#define dashLen (imm_q.x)
#define gapLen (imm_q.y)
#define dashPhase (imm_q.z)
#define lookOffset (imm_q.w)
#define fadeCentre (imm_r.x)
#define fadeVisible (imm_r.y)
#define fadeGone (imm_r.z)
#define fadePeriod (imm_r.w)
#define color imm_color

/* The look, in the order of `STROKE_LOOK` (render/stroke.rs). */
#define core (look0.x)
#define coreWidth (look0.y)
#define glow (look0.z)
#define glowWidth (look0.w)
#define boost (look1.x)
#define shade (look1.y)
#define halo (look1.z)
#define haloWidth (look1.w)

float segmentDistance(vec2 p, vec2 a, vec2 b) {
  vec2 ab = b - a;
  float t = clamp(dot(p - a, ab) / max(dot(ab, ab), 1e-12), 0.0, 1.0);
  return length(p - a - t * ab);
}

void main() {
  vec2 p = uv;
  float k = abs(curv);
  float side = curv < 0.0 ? -1.0 : 1.0;

  /* How far along the stroke the pixel is (`s`, unclamped: negative before
     the start) and how far from its middle line (`n`). For a curved stroke
     the centre is at (0, 1 / curv); both formulas stay exact as the
     curvature goes to 0. */
  float s = p.x;
  if (k > 1e-7) {
    float phi = atan(k * p.x, 1.0 - curv * p.y);
    /* Centre the angle's range on the stroke's middle, so the far side is
       split evenly between both ends. */
    if (phi < 0.5 * k * len - PI) {
      phi += TAU;
    }
    s = phi / k;
  }
  float n = abs((k * dot(p, p) - 2.0 * side * p.y) / (length(curv * p - vec2(0.0, 1.0)) + 1.0));

  /* `caps` below 0: round ends. Otherwise flat ends whose corners are
     rounded by `caps` pixels: the outline of a stroke that much thinner and
     shorter at both ends, grown back by it. */
  bool roundEnds = caps < 0.0;
  float corner = roundEnds ? 0.0 : min(caps, halfWidth);
  float s0 = corner;
  float s1 = len - corner;
  if (s1 < s0) {
    s0 = 0.5 * len;
    s1 = s0;
  }
  float w = halfWidth - corner;

  /* Signed distance to the outline: within the length it is the distance
     across; beyond an end, to the end's flat cap or round cap. */
  float d = n - w;
  if (s < s0 || s > s1) {
    float t = curv * (s < s0 ? s0 : s1);
    float h = sin(0.5 * t);
    vec2 e = k > 1e-7 ? vec2(sin(t), 2.0 * h * h) / curv : vec2(s < s0 ? s0 : s1, 0.0);
    vec2 across = vec2(-sin(t), cos(t));
    d = roundEnds ? length(p - e) - w : segmentDistance(p, e - w * across, e + w * across);
  }
  d -= corner;

  /* Dashes: the distance to the nearest dash along the middle line (negative
     inside one), as a box with the same rounded corners (flat caps) or a
     capsule (round caps). */
  if (gapLen > 0.0) {
    float period = dashLen + gapLen;
    float m = mod(s + dashPhase, period);
    float along = m < dashLen ? -min(m, dashLen - m) : min(m - dashLen, period - m);
    float dash;
    if (roundEnds) {
      dash = length(vec2(max(along, 0.0), n)) - halfWidth;
    } else {
      float c = min(corner, 0.5 * dashLen);
      vec2 q = vec2(along + c, n - halfWidth + c);
      dash = length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - c;
    }
    d = max(d, dash);
  }

  /* Fade along the length, around a point; wrapped on a closed circle. */
  float fade = 1.0;
  if (fadeGone > 0.0) {
    float x = s - fadeCentre;
    if (fadePeriod > 0.0) {
      x = mod(x + 0.5 * fadePeriod, fadePeriod) - 0.5 * fadePeriod;
    }
    fade = 1.0 - smoothstep(fadeVisible, fadeGone, abs(x));
  }

  int at = int(lookOffset + 0.5);
  vec4 look0 = shapeParams[at];
  vec4 look1 = shapeParams[at + 1];

  float outside = max(d, 0.0);
  float a = core * exp(-max(0.0, d - 0.5) / max(coreWidth, 1e-3));
  a += glow * exp(-pow(outside / max(glowWidth, 1e-3), 0.75));
  a *= color.a * fade;

  float cover = clamp(shade * a, 0.0, 1.0);
  float dark = clamp(halo * color.a * fade * (1.0 - smoothstep(0.0, max(haloWidth, 1e-3), outside)), 0.0, 1.0);
  outColor = vec4(boost * color.rgb * a, 1.0 - (1.0 - cover) * (1.0 - dark));
}
