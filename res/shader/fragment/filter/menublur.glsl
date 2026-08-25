#include fragment

uniform sampler2D src;
uniform sampler2D srcBlur;
uniform float strength;

// Blends the sharp scene with a blurred downsampled copy. Used behind
// UI menus (with the Darken overlay) to keep text readable over bright
// starfields; strength 0 < s < 1 mixes toward the blurred copy.
void main() {
  vec3 c = mix(texture(src, uv).xyz, texture(srcBlur, uv).xyz, strength);
  outColor = vec4(c, 1.0);
}
