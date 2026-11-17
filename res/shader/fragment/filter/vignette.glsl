#version 330

in vec2 uv;
out vec4 outColor;

#group 2
layout(std140) uniform Params {
    float hardness;
    float strength;
    float roundStrength; // a round darkening on top (speed): 0 = none, 1 = black edges
    float aspect;        // screen width / height, for the round one
};
#group 3
uniform sampler2D src;

void main() {
  float a = 1.0;
  vec2 uvp = vec2(1.0, 1.0) - 2.0 * abs(vec2(0.5, 0.5) - uv);
  a *= 1.0 - strength * exp(-hardness * uvp.x);
  a *= 1.0 - strength * exp(-hardness * uvp.y);

  // Round: 0 in the centre, 1 in the corners, darkening from about halfway out
  vec2 d = (uv - vec2(0.5)) * vec2(aspect, 1.0);
  float r = length(d) / length(vec2(0.5 * aspect, 0.5));
  a *= 1.0 - roundStrength * smoothstep(0.35, 1.0, r);

  vec4 c = texture(src, uv);
  c.xyz *= a;
  outColor = c;
}
