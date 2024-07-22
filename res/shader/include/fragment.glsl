#include common

in vec2 uv;
in vec3 pos;
in vec3 normal;
in vec3 vertNormal;
in vec3 vertPos;
in float flogz;

layout (location = 0) out vec4 outColor;

uniform vec3 eye;
uniform mat4 mWorldIT;

uniform samplerCube envMap;
uniform samplerCube irMap;
uniform vec3 starColor;
uniform vec3 starDir;

#define FRAGMENT_CORRECT_DEPTH                                                 \
  gl_FragDepth = log2(flogz) * (0.5 * Fcoef);
