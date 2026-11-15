#include fragment
#include math

#include imm

#define r1 (imm_p.x)
#define r2 (imm_p.y)
#define to (imm_p.z)
#define tw (imm_p.w)
#define size (imm_q.xy)
#define color imm_color

const float bevel = 2.0;

void main() {
  float Tau = radians(360.0);
  vec2 p = size * (uv - 0.5);
  float r = length(p);
  vec2 dir = vec2(cos(Tau * to), sin(Tau * to));
  float rd = abs(r - 0.5 * (r1 + r2)) - 0.5 * (r2 - r1);
  float td = 0.5 * size.x * (acos(dot(dir, normalize(vec2(uv.x, 1.0-uv.y) - 0.5))) - 0.5 * Tau * tw);
  float d = max(0.0, length(max(vec2(0.0), vec2(rd, td) + bevel)) - bevel);

  float alpha = 0.0;
  alpha += 0.5 * exp(-max(0.0, d - 0.5));
  alpha += 0.4 * exp(-pow(0.3 * d, 0.75));
  vec3 c = 2.0 * color.xyz;
  outColor = alpha * color.w * vec4(c, 1.0);
}
