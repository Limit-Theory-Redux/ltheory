#include common

in vec3 vertex_position;
in vec3 vertex_normal;
in vec2 vertex_uv;

out vec2 uv;
out vec3 pos;
out vec3 normal;
out vec3 vertNormal;
out vec3 vertPos;
out float flogz;

#group 2
layout(std140) uniform Params {
  mat4 mProj;
  mat4 mView;
  mat4 mWorld;
};

void main() {
  uv = vertex_uv;
  pos = vertex_position;
  normal = vertex_normal;
  vertNormal = vertex_normal;
  vertPos = vertex_position;
  flogz = 1.0;
  gl_Position = mProj * mView * mWorld * vec4(vertex_position, 1.0);
}
