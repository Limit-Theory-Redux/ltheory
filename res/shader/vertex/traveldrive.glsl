#include vertex

out vec3 objPos;

uniform float effectScale;

void main() {
  VS_BEGIN

  // Inflate mesh along normals to create energy shell around ship
  vec3 inflated = vertex_position + vertex_normal * effectScale;

  normal = normalize((mWorldIT * vec4(vertex_normal, 0)).xyz);
  vec4 v = vec4(inflated, 1.0);
  objPos = v.xyz;
  vec4 wp = mWorld * v;
  pos = wp.xyz;
  gl_Position = mProj * (mView * wp);
  VS_END
}
