#version 330

in vec2 uv;
in vec3 pos;
in vec3 vertPos;

out vec4 outColor;

void main() {
  vec3 dir = normalize(vertPos);
  vec3 c = vec3(0.0, 0.0, 0.0);
  c = mix(
    vec3(0.8, 0.5 + 0.5 * abs(dir.y), 0.5),
    vec3(0.5, 0.7, 1.0),
    abs(dir.y));

  if (dir.y < 0.0) c = vec3(0.1, 0.1, 0.1);

  float d = acos(dot(dir, normalize(vec3(1, 1, 0))));
  c += exp(-24.0 * d * d) * vec3(1.0, 0.5, 0.1);

  outColor = vec4(c, 1.0);
  gl_FragDepth = 1.0;
}
