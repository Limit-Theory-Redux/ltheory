#include fragment

#include imm

#define innerRadius (imm_p.x)
#define outerRadius (imm_p.y)
#define size (imm_p.zw)
#define glow (imm_q.x)
#define color imm_color

void main() {
  vec2 uvp = uv - 0.5;
  float r = length(size * uvp);

  // Soft edges (2px feather); none inside a full disc (inner radius 0)
  float inner = innerRadius > 0.0 ? smoothstep(innerRadius - 2.0, innerRadius + 2.0, r) : 1.0;
  float outer = 1.0 - smoothstep(outerRadius - 2.0, outerRadius + 2.0, r);

  float alpha = inner * outer;

  if (glow != 0.0) {
    // `AnnulusGlow`: a soft halo around the band, like `RingGlow`.
    float d = max(0.0, max(innerRadius - r, r - outerRadius));
    alpha += 0.4 * exp(-pow(0.2 * d, 0.75));
  }

  // Premultiplied: `Annulus` blends with PreMultAlpha, `AnnulusGlow` adds.
  outColor = color.w * alpha * imm_fade(uv) * vec4(color.xyz, 1.0);
}
