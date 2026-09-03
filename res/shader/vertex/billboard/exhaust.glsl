#include vertex
#include math

// Exhaust-plume billboard: camera-facing quad anchored at the nozzle,
// oriented along the thrust axis (up), sized to cover nozzle radius and
// plume length (uniforms shared by name with the fragment stage). The
// quad always faces the camera, so the jet is visible from any angle;
// the raymarched cone in the fragment stage supplies the volume.
uniform vec3 origin;
uniform float size;
uniform vec3 up;
uniform float plumeLen;

void main() {
  VS_BEGIN
  vec4 wp = vec4(vertPos + origin, 1.0);
  vec3 look = normalize(eye - wp.xyz);
  vec3 right = normalize(cross(up, look));

  // uvs are in [-1, 1]: axial span 0..plumeLen (nozzle at 0), width
  // centered and wide enough for the widest cone section (3.6 x radius).
  float mid = plumeLen * 0.5;
  wp.xyz += up * (mid + mid * uv.y);
  wp.xyz += right * (size * 3.6 * uv.x);
  pos = wp.xyz;
  gl_Position = mProj * (mView * wp);
  VS_END
}
