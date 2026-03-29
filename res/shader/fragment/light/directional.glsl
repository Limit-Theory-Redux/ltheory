#include fragment
#include deferred
#include math
#include pbr

#autovar vec3 eye

in vec3 worldOrigin;
in vec3 worldDir;

uniform vec3 lightDir;    // normalized direction FROM light TO scene (toward objects)
uniform vec3 lightColor;  // light color and intensity

uniform sampler2D texNormalMat;
uniform sampler2D texDepth;

void main () {
  vec4 normalMat = texture(texNormalMat, uv);
  float depth = texture(texDepth, uv).x;
  vec3 N = decodeNormal(normalMat.xy);
  float rough = normalMat.z;
  float mat = normalMat.w;

  vec3 p = worldOrigin + depth * normalize(worldDir);
  vec3 L = -lightDir;  // direction toward light

  vec3 light = vec3(0.0);

  if (mat == Material_Diffuse) {
    float NdotL = max(0.0, dot(N, L));
    light += lightColor * NdotL;
  }

  else if (mat == Material_Metal) {
    float NdotL = max(0.0, dot(N, L));
    light += lightColor * NdotL;  // simple diffuse for now, cookTorrance can be added later
  }

  outColor = vec4(light, 1.0);
}
