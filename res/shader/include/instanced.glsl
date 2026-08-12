/* -- Instanced vertex attributes -----------------------------------------------
   Per-instance replacement for `vertex.glsl`'s uniform `mWorld`/`mWorldIT`:
   provides the same varyings and `VS_BEGIN`/`VS_END`-equivalent macros, but
   builds `mWorld`/`mWorldIT` from per-instance vertex attributes instead of
   reading them as uniforms - one instanced draw call covers every instance,
   so there is no single uniform value that would work for all of them.

   Attribute layout must match `InstanceData` (render_command.rs) and the
   VertexAttribPointer/Divisor setup in `command_executor_gl.rs`'s
   `cmd_draw_instanced_with_data`, and the attribute name -> location bindings
   in `command_executor_gl.rs`'s `create_shader` (`glBindAttribLocation`):
   locations 4-7 = mWorld matrix columns, location 8 = per-instance RGBA color.

   Usage: #include instanced   (in place of #include vertex)
----------------------------------------------------------------------------- */

#include common
#include camera_ubo

in vec3 vertex_position;
in vec3 vertex_normal;
in vec2 vertex_uv;
in vec3 vertex_color;

in vec4 instance_matrix_col0;
in vec4 instance_matrix_col1;
in vec4 instance_matrix_col2;
in vec4 instance_matrix_col3;
in vec4 instance_color;

out vec2 uv;
out vec3 pos;
out vec3 normal;
out vec3 vertNormal;
out vec3 vertPos;
out vec4 instColor;
out float flogz;

#define VS_INSTANCED_BEGIN                                                    \
  uv = vertex_uv;                                                             \
  vertPos = vertex_position;                                                  \
  vertNormal = vertex_normal;                                                 \
  instColor = instance_color;                                                 \
  mat4 mWorld = mat4(instance_matrix_col0, instance_matrix_col1,              \
                      instance_matrix_col2, instance_matrix_col3);            \
  mat4 mWorldIT = transpose(inverse(mWorld));

#define VS_INSTANCED_END                                                      \
  gl_Position = logDepth(gl_Position);

vec4 logDepth(vec4 p) {
  p.z = log2(max(1e-6, 1.0 + abs(p.w))) * Fcoef - 1.0;
  p.z *= p.w;
  flogz = 1.0 + p.w;
  return p;
}
