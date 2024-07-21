#include fragment
#include math
#include color

uniform sampler2D src;

void main() {
  vec3 c = texture(src, uv).xyz;
  float a = 1.0 + avg(c);
  outColor = vec4(c, a);
}
