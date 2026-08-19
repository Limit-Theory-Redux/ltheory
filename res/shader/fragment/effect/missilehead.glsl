#include fragment
#include color
#include math

uniform vec3 color;
uniform float alpha;

void main() {
  float r = length(uv);
  float core = exp(-sqrt(192.0 * r));
  float halo = exp(-sqrt(64.0 * r));
  vec3 hot = mix(color, vec3(1.0, 0.82, 0.42), 0.42);
  hot *= hot / avg(hot);
  outColor = vec4((2.6 * core + 0.7 * halo) * alpha * hot, 1.0);
  FRAGMENT_CORRECT_DEPTH;
}
