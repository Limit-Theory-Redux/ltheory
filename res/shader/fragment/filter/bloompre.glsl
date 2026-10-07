#include fragment
#include math
#include color

#group 3
uniform sampler2D src;

void main() {
  vec3 c = texture(src, uv).xyz;
  float a = lum(c);
  // c *= (1.0 - exp(-lum(c))) / lum(c);
  outColor = vec4(c, a);
}
