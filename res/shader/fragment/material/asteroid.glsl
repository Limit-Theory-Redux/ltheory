#include fragment
#include deferred
#include gamma
#include fdm
#include color
#include math
#include fog
#include draw_block

#define scale (drawScale.x)

#group 1
uniform sampler2D texDiffuse;

void main() {
  vec3 N = normalize(normal);
  vec3 V = normalize(pos - eye);
  float fdmLo, fdmHi, fdmT;
  getFDMParams(fdmLo, fdmHi, fdmT);
  vec3 fdmPos = scale * vertPos.xyz;
  vec4 fdmColor = mix(
    texture(texDiffuse, triplanarCoords(fdmLo * fdmPos)),
    texture(texDiffuse, triplanarCoords(fdmHi * fdmPos)),
    fdmT);
  vec3 c = linear(fdmColor.xyz);
  c *= radians(360.0);
  c *= uv.x;
  c *= c;
  // c *= textureLod(envMap, N, 9.0).xyz;
  // c = applyFog(c, V);

  FRAGMENT_CORRECT_DEPTH;

  setAlbedo(c);
  setAlpha(1.0);
  setDepth();
  setNormal(N);
  setRoughness(1.0);
  setMaterial(Material_Diffuse);
}
