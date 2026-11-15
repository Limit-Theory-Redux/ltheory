/* -- Immediate batcher varyings --------------------------------------------------
   Written by `vertex/imm2d.glsl` from the attributes of `Imm2DVertex`: the color
   and the three shape parameter vectors are the same for every vertex of a
   primitive (flat). Each `ui/*` fragment shader names the parameters it needs
   with the defines below the include; `imm_r` is the fade window (`imm_fade`).
----------------------------------------------------------------------------- */

flat in vec4 imm_color;
flat in vec4 imm_p;
flat in vec4 imm_q;
flat in vec4 imm_r;

/* The fade window of a round shape (`Renderer::imm_shape_faded`): 1 within
   `imm_r.y` radians of the direction `imm_r.x` (0 = right, counter-clockwise),
   0 beyond `imm_r.z`, smooth in between, measured from the quad's centre. 1
   everywhere when `imm_r.w` is 0 (no fade). */
float imm_fade(vec2 uv) {
  if (imm_r.w == 0.0) {
    return 1.0;
  }
  vec2 d = vec2(uv.x - 0.5, 0.5 - uv.y);
  float l = length(d);
  if (l <= 0.0) {
    return 1.0;
  }
  float c = clamp(dot(d / l, vec2(cos(imm_r.x), sin(imm_r.x))), -1.0, 1.0);
  return 1.0 - smoothstep(imm_r.y, imm_r.z, acos(c));
}
