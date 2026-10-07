#ifndef include_texturing
#define include_texturing

/* NOTE : Sampler-typed function parameters are deliberately avoided (not
          portable to wgpu/naga). These helpers compute blend weights and
          coordinates; call sites sample the texture directly, e.g.
            texture(tex, triplanarCoords(pos))
            triplanarBumpmap(texture(tex, pos.yz).xyz,
                             texture(tex, pos.zx).xyz,
                             texture(tex, pos.xy).xyz) */

vec3 triplanarBlend() {
  vec3 n = normalize(vertNormal);
  return n * n;
}

/* Projection coordinates along the dominant axis of the (squared) normal. */
vec2 triplanarCoords(vec3 pos) {
  vec3 blend = triplanarBlend();
  float maxBlend = max(blend.x, max(blend.y, blend.z));
  vec3 mask = vec3(step(maxBlend, blend.x),
                   step(maxBlend, blend.y),
                   step(maxBlend, blend.z));
  // vec3 ddx = dFdx(pos);
  // vec3 ddy = dFdy(pos);
  // vec2 dx = mask.x * ddx.yz + mask.y * ddx.xz + mask.z * ddx.xy;
  // vec2 dy = mask.x * ddy.yz + mask.y * ddy.xz + mask.z * ddy.xy;
  // float lod = max(0.0, 9.0 + 0.5 * log2(max(dot(dx, dx), dot(dy, dy))));
  return mask.x * pos.yz + mask.y * pos.xz + mask.z * pos.xy;
}

/* Blend three raw (unexpanded, 0..1) normal-map samples taken along the
   x (pos.yz), y (pos.zx) and z (pos.xy) projections. */
vec3 triplanarBumpmap(vec3 sx, vec3 sy, vec3 sz) {
  vec3 blend = triplanarBlend();
  vec3 tx = 2.0 * sx - 1.0;
  vec3 ty = 2.0 * sy - 1.0;
  vec3 tz = 2.0 * sz - 1.0;
  return blend.x * tx + blend.y * ty + blend.z * tz;
}

#endif
