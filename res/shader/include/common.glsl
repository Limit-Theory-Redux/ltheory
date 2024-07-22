#version 330

/* WARNING : Make sure to update Fcoef if farPlane is changed! */
const float farPlane  = 1.0e6;

/* NOTE : Fcoef = 2.0 / log2(farPlane + 1.0);
          AMD drivers complain due to non-constexpr if not folded. */
const float Fcoef = 0.10034332462;
