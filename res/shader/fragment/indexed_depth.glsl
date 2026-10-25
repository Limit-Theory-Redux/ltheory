#include fragment

// Same block as vertex/indexed_depth.glsl: both stages declare it.
#group 2
layout(std140) uniform Params {
  vec4 color;
  float depthValue;
};

void main() {
  // Constant colors make depth ordering observable without material or lighting.
  outColor = color;
}
