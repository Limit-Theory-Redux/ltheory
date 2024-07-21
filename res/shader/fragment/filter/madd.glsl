in vec2 uv;
out vec4 outColor;

uniform sampler2D src;
uniform vec4 add;
uniform vec4 mult;

void main() {
  outColor = mult * texture(src, uv) + add;
}
