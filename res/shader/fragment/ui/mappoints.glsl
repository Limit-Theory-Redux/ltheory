#include fragment

uniform vec4 color;

void main() {
  // Circle: discard outside radius
  vec2 d = uv - 0.5;
  float r = dot(d, d);
  if (r > 0.25) discard;

  // Soft edge
  float alpha = 1.0 - smoothstep(0.15, 0.25, r);
  outColor = vec4(color.xyz, color.w * alpha);
}
