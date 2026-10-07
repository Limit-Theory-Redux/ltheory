/* -- Immediate batcher varyings --------------------------------------------------
   Written by `vertex/imm2d.glsl` from the attributes of `Imm2DVertex`: the color
   and the two shape parameter vectors are the same for every vertex of a
   primitive (flat). Each `ui/*` fragment shader names the parameters it needs
   with the defines below the include.
----------------------------------------------------------------------------- */

flat in vec4 imm_color;
flat in vec4 imm_p;
flat in vec4 imm_q;
