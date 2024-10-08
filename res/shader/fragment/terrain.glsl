#version 330

in vec2 uv;
in vec3 pos;

out vec4 outColor;

uniform sampler2D texDiffuse;
uniform vec3 eye;

vec3 emix(vec3 a, vec3 b, float t) {
  return log(mix(exp(a), exp(b), t));
}

void main() {
  float dist = length(eye - pos);
  float freq = 1.0 / pow(dist, 0.75);
  float l2 = log2(freq) + 32.0;
  float f1 = pow(floor(l2), 2.0);
  float f2 = pow(floor(l2) + 1.0, 2.0);

  vec3 c = emix(
    texture(texDiffuse, f1 * uv).xyz,
    texture(texDiffuse, f2 * uv).xyz, fract(l2));

  c = sqrt(c * texture(texDiffuse, 4.0 * uv).xyz);
  c *= 0.5;

  outColor = vec4(c, 1.0);
}
