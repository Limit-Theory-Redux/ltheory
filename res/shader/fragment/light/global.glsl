#include fragment
#include deferred_read
#include math

in vec3 worldOrigin;
in vec3 worldDir;

uniform vec3 lightColor;
uniform vec3 lightPos;

uniform sampler2D texNormalMat;
uniform sampler2D texDepth;

// Simple directional light calculation
void main () {
  vec4 normalMat = texture(texNormalMat, uv);
  float depth = texture(texDepth, uv).x;
  vec3 N = decodeNormal(normalMat.xy);
  float mat = normalMat.w;

  vec3 light = vec3(0.0);

  // Simple directional light from lightPos direction
  vec3 L = normalize(lightPos);
  float NdotL = max(0.0, dot(N, L));

  if (mat == Material_Diffuse) {
    // Diffuse: hemisphere lighting + directional
    vec3 up = vec3(0.0, 1.0, 0.0);
    float hemisphere = 0.5 + 0.5 * dot(N, up);
    vec3 ambient = vec3(0.15, 0.15, 0.18) * hemisphere;
    vec3 directional = lightColor * NdotL * 0.6;
    light = ambient + directional;
  }
  else if (mat == Material_Metal) {
    // Metal: stronger directional, reduced ambient
    vec3 ambient = vec3(0.05, 0.05, 0.06);
    vec3 directional = lightColor * NdotL * 0.8;
    light = ambient + directional;
  }
  else if (mat == Material_NoShade) {
    // Unlit: full brightness
    light = vec3(1.0);
  }

  outColor = vec4(light, 1.0);
}
