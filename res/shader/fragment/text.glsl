#include "fragment.glsl"

uniform vec3 color;

void main() {
  fragColor = vec4(color, uv.x);
}
