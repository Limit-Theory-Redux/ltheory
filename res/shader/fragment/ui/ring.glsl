#include fragment

#include imm

#define radius (imm_p.x)
#define size (imm_p.yz)
#define glow (imm_q.x)
#define color imm_color

void main() {
  vec2 uvp = uv - 0.5;
  float r = length(size * uvp);
  float alpha = 0.0;
  float d = abs(r - radius);
  alpha += 0.3 * exp(-max(0.0, d - 0.5));
  alpha += 0.4 * exp(-pow(0.2 * d, 0.75));
  if (glow != 0.0) {
    alpha += 0.4 * exp(-pow(0.2 * d, 0.75));
  }
  vec3 c = 2.0 * color.xyz;
  outColor = alpha * color.w * vec4(c.xyz, 1.0);
}
