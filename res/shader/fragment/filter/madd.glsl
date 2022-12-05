varying vec2 uv;

uniform sampler2D src;
uniform vec4 add;
uniform vec4 mult;

void main() {
  fragColor = mult * texture2D(src, uv) + add;
}
