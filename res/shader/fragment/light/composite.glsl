#include fragment
#include deferred
#include gamma

#autovar samplerCube irMap
#autovar samplerCube envMap
#autovar vec3 eye

varying vec3 worldOrigin;
varying vec3 worldDir;

uniform sampler2D texAlbedo;
uniform sampler2D texDepth;
uniform sampler2D texLighting;

void main() {
  vec3 albedo = texture2D(texAlbedo, uv).xyz;
  vec3 light = texture2D(texLighting, uv).xyz;
  float depth = texture2D(texDepth, uv).x;

  vec3 c = albedo * light;

  vec3 ambientColor = vec3(0.0025, 0.0025, 0.0025);
  vec3 ambientLighting = ambientColor * albedo;

  float fog = 1.0 - exp(-depth / 7000.0);
  float fogScale = 0.0675;

  fog *= fogScale;
  //fog *= 0.0;

  vec3 bg = linear(textureCubeLod(irMap, worldDir, 3.0 + 6.0 * (1.0 - fog)).xyz);
  c = mix(c, bg, fog);

  c += ambientLighting;

  gl_FragData[0] = vec4(c, 1.0);
}
