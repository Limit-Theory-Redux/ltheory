#include fragment
#include math

#include imm

#define radius (imm_p.x)
#define size (imm_p.yz)
#define color imm_color

void main() {
  vec2 uvp = abs(size * (uv - 0.5));
  float r = length(uvp);

  float alpha = 0.0;

  float d = max(0.0, r - radius) / (size.x);
  alpha += 0.8 * exp(-pow(32.0 * d, 4.0));

  vec3 c = 2.0 * color.xyz;
  outColor = alpha * color.w * vec4(c.xyz, 1.0);
}
