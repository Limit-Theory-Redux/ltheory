#include common
#include view_block

in vec2 uv;
in vec3 pos;
in vec3 normal;
in vec3 vertNormal;
in vec3 vertPos;
in float flogz;

layout (location = 0) out vec4 outColor;

uniform mat4 mWorldIT;

uniform vec3 starColor;

#define FRAGMENT_CORRECT_DEPTH                                                 \
  gl_FragDepth = log2(flogz) * (0.5 * Fcoef);

// Environment maps (group 0, units 0 and 1): set with `Renderer:setEnvironment`.
// Keep these last: a `#group` applies to the declarations that follow it.
#group 0
uniform samplerCube envMap;
uniform samplerCube irMap;
