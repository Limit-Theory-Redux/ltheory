#version 330

in vec2 uv;

out vec4 outColor;

uniform int sDim;
uniform float radius;
uniform sampler2D sPointBuffer;
uniform sampler2D sNormalBuffer;
uniform sampler2D vPointBuffer;
uniform sampler2D vNormalBuffer;

void main() {
  vec3 p = texture(vPointBuffer, uv).xyz;
  vec3 n = texture(vNormalBuffer, uv).xyz;

  float total = 0.0;
  for (int y = 0; y < sDim; ++y) {
    float v = (float(y) + 0.5) / float(sDim);
    for (int x = 0; x < sDim; ++x) {
      float u = (float(x) + 0.5) / float(sDim);

      vec4 sp = texture(sPointBuffer, vec2(u, v));
      vec4 sn = texture(sNormalBuffer, vec2(u, v));
      float area = sp.w;

      vec3 r = sp.xyz - p;
      float d = dot(r, r) + 1e-16;
      r *= inversesqrt(d);
      d /= radius;

      float value = 1.0 - inversesqrt(area / (d * d) + 1.0);
      value *= clamp(-dot(r, sn.xyz), 0.0, 1.0);
      value *= clamp(4.0 * dot(r, n), 0.0, 1.0);
      total += value;
    }
  }

  outColor.x = exp(-2.0 * sqrt(total));
}
