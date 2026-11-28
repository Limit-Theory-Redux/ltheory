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

#define age (drawUser[0].x)

// The fragment stage's `animSpeed` and front radius, and its reach
// (fragment/effect/explosion_volume_draw.glsl): the billboard covers the
// sphere the fire reaches at its age, with a margin for perspective.
const float animSpeed = 0.42;
const float QUAD_MARGIN = 1.08;

void main() {
  VS_BEGIN
  vec4 wp = vec4(vertPos + origin, 1.0);
  vec3 look = normalize(eye - wp.xyz);
  vec3 right = cross(look, up);
  float R = 1.15 * (1.0 - exp(-1.5 * animSpeed * age));
  float quad = min(1.46, 1.3 * R + 0.25) * QUAD_MARGIN;
  wp.xyz += size * quad * uv.x * right;
  wp.xyz += size * quad * uv.y * up;
  pos = wp.xyz;
  gl_Position = mProj * (mView * wp);
  VS_END
}
