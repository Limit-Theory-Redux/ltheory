#include fragment

uniform vec4 color;

void main() {
  float alpha = uv.x; // per-vertex fade encoded in UV.x
  outColor = vec4(color.rgb, color.a * alpha * alpha); // quadratic falloff
}
