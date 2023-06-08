#ifndef PHX_Math
#define PHX_Math

#include "Common.h"

#include <math.h>
#include <stdlib.h>

/* -- Float/Double ---------------------------------------------------------- */

#define Tau          6.28318531f
#define Pi           3.14159265f
#define Pi2          1.57079633f
#define Pi4          0.785398163f
#define Pi6          0.523598776f
#define F32_EPSILON  1.19209290e-07f
#define F64_EPSILON  2.2204460492503131e-16

inline bool   Approxf    (float a, float b);
inline float  Acosf      (float t);
inline float  Asinf      (float t);
inline float  Atanf      (float t);
inline float  Atan2f     (float y, float x);
inline float  Ceilf      (float t);
inline float  Cosf       (float t);
inline float  Degreesf   (float radians);
inline float  Expf       (float t);
inline float  Floorf     (float t);
inline float  Fractf     (float t);
inline float  Lerpf      (float a, float b, float t);
inline float  Logf       (float t);
inline float  Logbf      (float t, float b);
inline bool   NonZerof   (float t);
inline float  Powf       (float t, float p);
inline float  Pow2f      (float t);
inline float  Pow4f      (float t);
inline float  Pow8f      (float t);
inline float  Radiansf   (float degrees);
inline float  Roundf     (float t);
inline float  Roundpf    (float t, int places);
inline float  Saturatef  (float t);
inline float  SignPowf   (float t, float p);
inline float  Sinf       (float t);
inline float  Sqrtf      (float t);
inline float  Stepf      (float t, float edge);
inline float  Tanf       (float t);

/* -- Int/Float/Double  ----------------------------------------------------- */

inline int    Absi       (int t);
inline int    Clampi     (int t, int lower, int upper);
inline int    Clamp01i   (int t);
inline int    ClampUniti (int t);
inline int    Maxi       (int a, int b);
inline int    Mini       (int a, int b);
inline int    Modi       (int i, int m);
inline int    Signi      (int t);

/*
 *   Functions suffixed with 'Signed' are signed:
 *     FnSigned (x, ...)              Sign(x) * Fn(Abs(x), ...)
 *
 *   Functions marked with ! are specialized variants of another function.
 *   Specializations are provided only when doing so affords a significant
 *   performance benefit. Thus, functions marked with ! should be preferred
 *   over their generic variants when applicable:
 *
 *       Fn
 *     ! FnBlah  <-- Strictly more efficient than Fn
 *
 *     ExpMap (x, p)                  1.0 - Exp(-Pow(Abs(x), p))
 *   ! ExpMap1 (x)                    ExpMap(x, 1)
 *   ! ExpMap2 (x)                    ExpMap(x, 2)
 *
 *     PowSigned (x, p)               Sign(x) * Pow(Abs(x), p)
 *
 *     Sign(x)                        |  1  x > 0
 *                                    |  0  x = 0
 *                                    | -1  x < 0
 *
 */

PHX_API double  Math_Bezier3        (double x, double, double, double);
PHX_API double  Math_Bezier4        (double x, double, double, double, double);
PHX_API double  Math_Clamp          (double x, double a, double b);
PHX_API double  Math_Clamp01        (double x);
PHX_API double  Math_ClampSafe      (double x, double a, double b);
PHX_API double  Math_ClampUnit      (double x);
PHX_API double  Math_ExpMap         (double x, double p);
PHX_API double  Math_ExpMapSigned   (double x, double p);
PHX_API double  Math_ExpMap1        (double x);
PHX_API double  Math_ExpMap1Signed  (double x);
PHX_API double  Math_ExpMap2        (double x);
PHX_API double  Math_ExpMap2Signed  (double x);
PHX_API double  Math_PowSigned      (double x, double p);
PHX_API double  Math_Round          (double x);
PHX_API double  Math_Sign           (double x);

/* -------------------------------------------------------------------------- */

/* Units. */
inline float Degreesf (float radians) {
  return (180.0f / Pi) * radians;
}

inline double Degrees (double radians) {
  return (180.0 / Pi) * radians;
}

inline float Radiansf (float degrees) {
  return (Pi / 180.0f) * degrees;
}

inline double Radians (double degrees) {
  return (Pi / 180.0) * degrees;
}

/* General. */
inline int Absi (int t) {
  return t < 0 ? -t : t;
}

inline float Absf (float t) {
  return (float)fabs((double)t);
}

inline double Abs (double t) {
  return fabs(t);
}

inline bool Approxf (float a, float b) {
  return fabs(a - b) < 1e-4;
}

inline bool Approx (double a, double b) {
  return fabs(a - b) < 1e-4;
}

inline float Ceilf (float t) {
  return (float)ceil((double)t);
}

inline double Ceil (double t) {
  return ceil(t);
}

inline int Clampi (int t, int lower, int upper) {
  t = t > upper ? upper : t;
  t = t < lower ? lower : t;
  return t;
}

inline float Clampf (float t, float lower, float upper) {
  t = t > upper ? upper : t;
  t = t < lower ? lower : t;
  return t;
}

inline double Clamp (double t, double lower, double upper) {
  t = t > upper ? upper : t;
  t = t < lower ? lower : t;
  return t;
}

inline int Clamp01i (int t) {
  t = t > 1 ? 1 : t;
  t = t < 0 ? 0 : t;
  return t;
}

inline float Clamp01f (float t) {
  t = t > 1.0f ? 1.0f : t;
  t = t < 0.0f ? 0.0f : t;
  return t;
}

inline double Clamp01 (double t) {
  t = t > 1.0 ? 1.0 : t;
  t = t < 0.0 ? 0.0 : t;
  return t;
}

inline int ClampUniti (int t) {
  t = t >  1 ?  1 : t;
  t = t < -1 ? -1 : t;
  return t;
}

inline float ClampUnitf (float t) {
  t = t >  1.0f ?  1.0f : t;
  t = t < -1.0f ? -1.0f : t;
  return t;
}

inline double ClampUnit (double t) {
  t = t >  1.0 ?  1.0 : t;
  t = t < -1.0 ? -1.0 : t;
  return t;
}

inline float Expf (float t) {
  return (float)exp((double)t);
}

inline double Exp (double t) {
  return exp(t);
}

inline float Floorf (float t) {
  return (float)floor((double)t);
}

inline double Floor (double t) {
  return floor(t);
}

inline float Fractf (float t) {
  return t - Floor(t);
}

inline double Fract (double t) {
  return t - Floor(t);
}

inline float Lerpf (float a, float b, float t) {
  return a + t * (b - a);
}

inline double Lerp (double a, double b, double t) {
  return a + t * (b - a);
}

inline float Logf (float t) {
  return (float)log((double)t);
}

inline double Log (double t) {
  return log(t);
}

inline float Logbf (float t, float b) {
  return Log(t) / Log(b);
}

inline double Logb (double t, double b) {
  return Log(t) / Log(b);
}

inline uint32 Maxu (uint32 a, uint32 b) {
  return a > b ? a : b;
}

inline int Maxi (int a, int b) {
  return a > b ? a : b;
}

inline float Maxf (float a, float b) {
  return a > b ? a : b;
}

inline double Max (double a, double b) {
  return a > b ? a : b;
}

inline int Mini (int a, int b) {
  return a < b ? a : b;
}

inline float Minf (float a, float b) {
  return a < b ? a : b;
}

inline double Min (double a, double b) {
  return a < b ? a : b;
}

inline int Modi (int i, int m) {
  return i % m;
}

inline float Modf (float t, float m) {
  return (float)fmod((double)t, (double)m);
}

inline double Mod (double t, double m) {
  return fmod(t, m);
}

inline bool NonZerof (float t) {
  return Abs(t) > F32_EPSILON;
}

inline bool NonZero (double t) {
  return Abs(t) > F64_EPSILON;
}

inline float Powf (float t, float p) {
  return (float)pow((double)t, (double)p);
}

inline double Pow (double t, double p) {
  return pow(t, p);
}

inline float Pow2f (float t) {
  return t * t;
}

inline double Pow2 (double t) {
  return t * t;
}

inline float Pow4f (float t) {
  float t2 = t * t;
  return t2 * t2;
}

inline double Pow4 (double t) {
  double t2 = t * t;
  return t2 * t2;
}

inline float Pow8f (float t) {
  float t2 = t * t;
  float t4 = t2 * t2;
  return t4 * t4;
}

inline double Pow8 (double t) {
  double t2 = t * t;
  double t4 = t2 * t2;
  return t4 * t4;
}

inline float Roundf (float t) {
  return Floor(t + 0.5f);
}

inline double Round (double t) {
  return Floor(t + 0.5);
}

inline float Roundpf (float t, int places) {
  double factor = 10.0;
  for (int i = 0; i < places; ++i)
    factor *= 10.0;
  return (float)(Floor(factor * t + 0.5) / factor);
}

inline double Roundp (double t, int places) {
  double factor = 10.0;
  for (int i = 0; i < places; ++i)
    factor *= 10.0;
  return Floor(factor * t + 0.5) / factor;
}

inline float Saturatef (float t) {
  return t < 0.0f ? 0.0f :
         t > 1.0f ? 1.0f :
         t;
}

inline double Saturate (double t) {
  return t < 0.0 ? 0.0 :
         t > 1.0 ? 1.0 :
         t;
}

inline int Signi (int t) {
  return t  > 0 ? 1 :
         t == 0 ? 0 : -1;
}

inline float Signf (float t) {
  return t  > 0.0f ? 1.0f :
         t == 0.0f ? 0.0f : -1.0f;
}

inline double Sign (double t) {
  return t  > 0.0 ? 1.0 :
         t == 0.0 ? 0.0 : -1.0;
}

inline float SignPowf (float t, float p) {
  return Sign(t) * Pow(Abs(t), p);
}

inline double SignPow (double t, double p) {
  return Sign(t) * Pow(Abs(t), p);
}

inline float Stepf (float t, float edge) {
  return t < edge ? 0.0f : 1.0f;
}

inline double Step (double t, double edge) {
  return t < edge ? 0.0 : 1.0;
}

inline float Sqrtf (float t) {
  return (float)sqrt((double)t);
}

inline double Sqrt (double t) {
  return sqrt(t);
}

/* Trig. */
inline float Acosf (float t) {
  return (float)acos((double)t);
}

inline double Acos (double t) {
  return acos(t);
}

inline float Asinf (float t) {
  return (float)asin((double)t);
}

inline double Asin (double t) {
  return asin(t);
}

inline float Atanf (float t) {
  return (float)atan((double)t);
}

inline double Atan (double t) {
  return atan(t);
}

inline float Atan2f (float y, float x) {
  return (float)atan2((double)y, (double)x);
}

inline double Atan2 (double y, double x) {
  return atan2(y, x);
}

inline float Cosf (float t) {
  return (float)cos((double)t);
}

inline double Cos (double t) {
  return cos(t);
}

inline float Sinf (float t) {
  return (float)sin((double)t);
}

inline double Sin (double t) {
  return sin(t);
}

inline float Tanf (float t) {
  return (float)tan((double)t);
}

inline double Tan (double t) {
  return tan(t);
}

#endif
