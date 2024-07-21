#include fragment

uniform float radius;
uniform vec2 size;
uniform vec4 color;
uniform bool glow;

void main() {
  vec2 uvp = uv - 0.5;
  float r = length(size * uvp);
  float alpha = 0.0;
  float d = abs(r - radius);
  alpha += 0.3 * exp(-max(0.0, d - 0.5));
  alpha += 0.4 * exp(-pow(0.6 * d, 0.9));
  vec3 c = 0.7 * color.xyz;
  outColor = alpha * color.w * vec4(c.xyz, 1.0);
}
