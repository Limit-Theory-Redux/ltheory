#include common
#include view_block

/* The UI vertex of the immediate batcher (`Imm2DVertex`, render/gpu/imm.rs):
   pixel position, uv, color and two parameter vectors. The `ui/*` fragment
   shaders read the shape parameters from the flat varyings below (see
   `include/imm.glsl`) instead of uniforms, so every primitive of a batch
   shares one draw. */

in vec2 vertex_position;
in vec2 vertex_uv;
in vec4 vertex_color;
in vec4 imm_params;
in vec4 imm_params2;

out vec2 uv;
out vec3 pos;
flat out vec4 imm_color;
flat out vec4 imm_p;
flat out vec4 imm_q;

void main() {
  uv = vertex_uv;
  pos = vec3(vertex_position, 0.0);
  imm_color = vertex_color;
  imm_p = imm_params;
  imm_q = imm_params2;
  gl_Position = mProjUI * (mWorldViewUI * vec4(vertex_position, 0.0, 1.0));
}
