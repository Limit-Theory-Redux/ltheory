#include fragment

uniform float innerRadius;
uniform float outerRadius;
uniform vec2 size;
uniform vec4 color;

void main() {
  vec2 uvp = uv - 0.5;
  float r = length(size * uvp);

  // Soft edges (2px feather)
  float inner = smoothstep(innerRadius - 2.0, innerRadius + 2.0, r);
  float outer = 1.0 - smoothstep(outerRadius - 2.0, outerRadius + 2.0, r);

  float alpha = inner * outer;
  outColor = vec4(color.xyz, color.w * alpha);
}
