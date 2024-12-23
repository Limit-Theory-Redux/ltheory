#version 330

in vec2 uv;
out vec4 outColor;

uniform sampler2D src;
uniform vec2 dir;
uniform vec2 size;
uniform int radius;
uniform float variance;

void main() {
  vec4 total = vec4(0.0);
  vec4 center = texture(src, uv);
  total += center;
  float tw = 1.0;
  float v = variance * variance;

  for (int i = 1; i <= radius; ++i) {
    float fi = float(i);
    float w = exp(-(fi * fi) / v);
    vec2 delta = fi * (dir / size);
    vec4 c0 = texture(src, uv + delta);
    vec4 c1 = texture(src, uv - delta);
    total += w * c0;
    total += w * c1;
    tw += 2.0 * w;
  }

  outColor = total / tw;
}
