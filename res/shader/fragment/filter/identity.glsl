#include fragment

uniform sampler2D src;

void main() {
  gl_FragColor = texture2D(src, uv);
}
