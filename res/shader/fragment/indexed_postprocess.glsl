#include fragment

uniform sampler2D src;

void main() {
  vec4 sampleColor = texture(src, uv);
  outColor = vec4(1.0 - sampleColor.rgb, 1.0);
}
