/* -- Instanced Color Fragment Shader ------------------------------------------
   Simple color shader that uses the per-instance color from vertex shader.
----------------------------------------------------------------------------- */

#include common

in vec2 uv;
in vec3 pos;
in vec3 normal;
in vec3 vertNormal;
in vec3 vertPos;
in float flogz;
in vec4 vertColor; // From instance data

layout (location = 0) out vec4 outColor;

uniform vec3 eye;

#define FRAGMENT_CORRECT_DEPTH                                                 \
  gl_FragDepth = log2(flogz) * (0.5 * Fcoef);

void main() {
  FRAGMENT_CORRECT_DEPTH
  outColor = vertColor;
}
