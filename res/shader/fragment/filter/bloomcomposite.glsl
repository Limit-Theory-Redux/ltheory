#include fragment
#include math
#include gamma
#include color

uniform sampler2D src;
uniform sampler2D srcBlur;
uniform float intensity = 0.15;  // Default if not set

void main() {
  vec3 original = texture(src, uv).rgb;
  vec3 bloom = texture(srcBlur, uv).rgb;

  // Additive bloom with configurable intensity
  vec3 c = original + bloom * intensity;

  outColor = vec4(c, 1.0);
}
