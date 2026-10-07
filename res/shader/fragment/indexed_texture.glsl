#include fragment

// A material-style texture: group 1, bound through a bind group.
#group 1
uniform sampler2D tex;

void main() {
  outColor = texture(tex, uv);
}
