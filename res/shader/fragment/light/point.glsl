#include fragment
#include deferred
#include math
#include pbr
#include light_ubo

in vec3 worldOrigin;
in vec3 worldDir;

uniform sampler2D texNormalMat;
uniform sampler2D texDepth;

const float kMinDistance = 0.0001;
const float kPointLightMult = 16.0;

void main () {
  vec4 normalMat = texture(texNormalMat, uv);
  float depth = texture(texDepth, uv).x;
  vec3 N = decodeNormal(normalMat.xy);
  float rough = normalMat.z;
  float mat = normalMat.w;

  // A cleared deferred pixel has no valid world-space surface. Reject it before
  // reconstructing p so point lights cannot paint halos into empty space.
  if (depth <= kMinDistance) {
    outColor = vec4(0.0);
    return;
  }

  vec3 p = worldOrigin + depth * normalize(worldDir);
  vec3 L = lightPos - p;
  float distanceToLight = length(L);
  float Lmag = 1.0 / max(kMinDistance, distanceToLight);
  float radiusAttenuation = lightRadius > 0.0
    ? saturate(1.0 - distanceToLight / lightRadius)
    : 1.0;

  vec3 light = vec3(0.0);
  if (mat == Material_Metal) {
    light += lightColor * Lmag * cookTorrance(L * Lmag, p, N, rough, 1.0)
      * radiusAttenuation * radiusAttenuation;
  }
  else if (mat != Material_NoShade) {
    // Diffuse, ice, and future shaded environment materials all receive the
    // same bounded point-light irradiance. Only explicit NoShade is excluded.
    light += lightColor * Lmag * saturate(dot(N, L * Lmag))
      * radiusAttenuation * radiusAttenuation;
  }

  light *= kPointLightMult;

  outColor = vec4(light, 1.0);
}
