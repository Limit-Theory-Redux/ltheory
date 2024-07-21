#include fragment

uniform vec4 color;
uniform sampler2D icon;

void main() {
  vec3 c = color.xyz;
  float alpha = texture(icon, uv).w;
  outColor = alpha * color.w * vec4(c, 1.0);
}
