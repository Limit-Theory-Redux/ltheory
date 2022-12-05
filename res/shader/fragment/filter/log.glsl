#include "filter.glsl"

void main() {
  fragColor = log(vec4(1.0) + texture2D(src, uv));
}
