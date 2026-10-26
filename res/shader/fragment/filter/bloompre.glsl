#include fragment
#include math
#include color

uniform sampler2D src;
uniform vec2 size;

void main() {
  vec2 d = 1.0 / size;
  vec3 c = 0.25 * (texture(src, uv + vec2(-d.x, -d.y)).xyz +
                   texture(src, uv + vec2( d.x, -d.y)).xyz +
                   texture(src, uv + vec2(-d.x,  d.y)).xyz +
                   texture(src, uv + vec2( d.x,  d.y)).xyz);
  float a = lum(c);
  // c *= (1.0 - exp(-lum(c))) / lum(c);
  outColor = vec4(c, a);
}
