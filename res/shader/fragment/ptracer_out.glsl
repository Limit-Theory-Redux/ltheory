#version 330

in vec2 uv;
out vec4 outColor;

uniform sampler2D src;
uniform float iterations;

void main () {
  vec4 c4 = texture(src, uv);
  vec3 c = clamp(c4.xyz / max(1.0, c4.w), vec3(0.0), vec3(1.0));
  // c = 1.0 - exp(-c);
  c = pow(c, vec3(1.0 / 2.2));
  outColor = vec4(c, 1.0);
}
