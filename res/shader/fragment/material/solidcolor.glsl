#include fragment
#include deferred

#group 1
layout(std140) uniform MaterialParams {
  vec3 color;
};

void main() {
  // Unlit, and every G-buffer output defined: a shader that writes only outColor leaves the
  // other attachments of an MRT pass undefined (GL drivers copy the color into them, wgpu
  // leaves them alone), which used to make debug boxes black on GL.
  setAlbedo(color);
  setAlpha(1.0);
  setMaterial(Material_NoShade);
  setDepth();
  FRAGMENT_CORRECT_DEPTH;
}
