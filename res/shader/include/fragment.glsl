#include common
#include camera_ubo

in vec2 uv;
in vec3 pos;
in vec3 normal;
in vec3 vertNormal;
in vec3 vertPos;
in float flogz;

layout (location = 0) out vec4 outColor;

// Per-object uniforms
uniform mat4 mWorldIT;

// Environment and lighting
uniform samplerCube envMap;
uniform samplerCube irMap;
uniform vec3 starColor;

#define FRAGMENT_CORRECT_DEPTH                                                 \
  gl_FragDepth = log2(flogz) * (0.5 * Fcoef);
