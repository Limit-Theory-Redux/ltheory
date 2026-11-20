// Flat color of a mesh drawn in a scene pass: the rgba color is drawUser[0] of
// the group-2 DrawBlock.
#include fragment
#include draw_block

void main() {
  outColor = drawUser[0];
  FRAGMENT_CORRECT_DEPTH;
}
