/* -- Instanced Rendering Include ----------------------------------------------
   Provides instance attributes for GPU instancing.

   Instance data layout (80 bytes per instance):
     - attributes 4-7: mat4 model matrix (4 vec4 columns, 64 bytes)
     - attribute 8: vec4 color (16 bytes)

   Use mInstance instead of mWorld for the model matrix.
   Use instanceColor for per-instance color.
----------------------------------------------------------------------------- */

#include common
#include camera_ubo

in vec3 vertex_position;
in vec3 vertex_normal;
in vec2 vertex_uv;
in vec3 vertex_color;

// Instance attributes (set with glVertexAttribDivisor = 1)
in vec4 instance_matrix_col0; // layout(location = 4)
in vec4 instance_matrix_col1; // layout(location = 5)
in vec4 instance_matrix_col2; // layout(location = 6)
in vec4 instance_matrix_col3; // layout(location = 7)
in vec4 instance_color;       // layout(location = 8)

out vec2 uv;
out vec3 pos;
out vec3 normal;
out vec3 vertNormal;
out vec3 vertPos;
out float flogz;
out vec4 vertColor;

// Construct instance model matrix from column vectors
mat4 getInstanceMatrix() {
  return mat4(
    instance_matrix_col0,
    instance_matrix_col1,
    instance_matrix_col2,
    instance_matrix_col3
  );
}

// Get inverse-transpose for normal transformation
mat3 getInstanceMatrixIT() {
  mat4 m = getInstanceMatrix();
  mat3 m3 = mat3(m);
  return transpose(inverse(m3));
}

#define VS_INSTANCED_BEGIN                                                     \
  uv = vertex_uv;                                                              \
  vertPos = vertex_position;                                                   \
  vertNormal = vertex_normal;                                                  \
  vertColor = instance_color;                                                  \
  mat4 mInstance = getInstanceMatrix();                                        \
  mat3 mInstanceIT = getInstanceMatrixIT();                                    \

#define VS_INSTANCED_END                                                       \
  gl_Position = logDepth(gl_Position);

vec4 logDepth(vec4 p) {
  p.z = log2(max(1e-6, 1.0 + abs(p.w))) * Fcoef - 1.0;
  p.z *= p.w;
  flogz = 1.0 + p.w;
  return p;
}
