#include fragment

#include imm

#define innerRadius (imm_p.x)
#define outerRadius (imm_p.y)
#define size (imm_p.zw)
#define color imm_color

void main() {
  vec2 uvp = uv - 0.5;
  float r = length(size * uvp);

  // Soft edges (2px feather)
  float inner = smoothstep(innerRadius - 2.0, innerRadius + 2.0, r);
  float outer = 1.0 - smoothstep(outerRadius - 2.0, outerRadius + 2.0, r);

  float alpha = inner * outer;
  outColor = vec4(color.xyz, color.w * alpha);
}
