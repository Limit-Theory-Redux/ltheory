#include fragment
#include math

uniform vec3 axis;
uniform vec3 color;
uniform float alpha;

void main() {
  float u = max(0.0, abs(uv.x) - 0.008);
  float v = saturate(1.0 + uv.y);
  float plume = exp(-sqrt(160.0 * u)) * v;
  float taper = exp(-7.0 * (1.0 - v));
  vec3 hot = mix(color, vec3(1.0, 0.55, 0.18), 0.35);
  hot *= hot / avg(hot);
  outColor = vec4(5.0 * plume * taper * alpha * hot, 1.0);
  FRAGMENT_CORRECT_DEPTH;
}
