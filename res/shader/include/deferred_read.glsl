#include math

// Material constants for G-buffer material ID channel
const float Material_SIZE       = 4.0;

const float Material_Diffuse = 0.0 / Material_SIZE;
const float Material_Metal   = 1.0 / Material_SIZE;
const float Material_Ice     = 2.0 / Material_SIZE;
const float Material_NoShade = 3.0 / Material_SIZE;

// Decode normal from G-buffer (2D encoded -> 3D world normal)
vec3 decodeNormal(vec2 n) {
  vec2 ang = 2.0 * n - 1.0;
  vec2 scth = vec2(sin(ang.x * PI), cos(ang.x * PI));
  vec2 scphi = vec2(sqrt(1.0 - ang.y * ang.y), ang.y);
  return vec3(scth.y * scphi.x, scth.x * scphi.x, scphi.y);
}
