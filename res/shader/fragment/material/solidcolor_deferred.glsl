#include fragment
#include deferred

uniform vec3 color;

void main() {
  setAlbedo(color);
  setNormal(normalize(normal));
  setRoughness(0.5);
  setMaterial(Material_Diffuse);
  setDepth();
  outColor.w = 1.0;
  FRAGMENT_CORRECT_DEPTH;
}
