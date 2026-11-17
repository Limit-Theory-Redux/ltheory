// Draw-block billboard of the volumetric explosion (paired with
// effect/explosion_volume_draw.glsl): the same slots as quadpos_draw.glsl -
// the camera-relative centre is drawUser[1].xyz, the size drawUser[1].w and
// the camera's up vector drawUser[2].xyz - but the quad half-extent
// overshoots `size` so turbulent plumes extending past the unit shell are
// never clipped by the geometry.
#include vertex_scene
#include math

#define origin (drawUser[1].xyz)
#define size (drawUser[1].w)
#define up (drawUser[2].xyz)

const float QUAD_SCALE = 1.7;

void main() {
  VS_BEGIN
  vec4 wp = vec4(vertPos + origin, 1.0);
  vec3 look = normalize(eye - wp.xyz);
  vec3 right = cross(look, up);
  wp.xyz += size * QUAD_SCALE * uv.x * right;
  wp.xyz += size * QUAD_SCALE * uv.y * up;
  pos = wp.xyz;
  gl_Position = mProj * (mView * wp);
  VS_END
}
