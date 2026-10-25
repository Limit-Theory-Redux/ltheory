#include fragment
#include imm

void main() {
  outColor = imm_color;
  FRAGMENT_CORRECT_DEPTH;
}
