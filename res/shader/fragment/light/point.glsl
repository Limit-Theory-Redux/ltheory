#include fragment
#include deferred_read
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

  // Skip pixels with no geometry
  if (depth < 0.001) {
    outColor = vec4(0.0, 0.0, 0.0, 1.0);
    return;
  }

  vec3 p = worldOrigin + depth * normalize(worldDir);

  vec3 light = vec3(0.0);

  // Calculate light direction and distance
  vec3 L = lightPos - p;
  float dist = length(L);
  L = L / max(dist, kMinDistance); // normalize
  float attenuation = 1.0 / max(kMinDistance, dist);

  if (mat == Material_Diffuse) {
    float NdotL = saturate(dot(N, L));
    light = lightColor * attenuation * NdotL;
  }
  else if (mat == Material_Metal) {
    light = lightColor * attenuation * cookTorrance(L, p, N, rough, 1.0);
  }

  light *= kPointLightMult;

  outColor = vec4(light, 1.0);
}
