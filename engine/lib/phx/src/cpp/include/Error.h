#ifndef PHX_Error
#define PHX_Error

#include "Common.h"

/* --- Error -------------------------------------------------------------------
 *
 *   A compact error status encoding.
 *
 *    high <---                                               ---> low
 *
 *           2 bytes               byte               byte
 *     [ operand location ]  [ operand type ]   [ failure type ]
 *
 *
 *   For example:
 *     0x00000101 -> received null pointer
 *     0x00080102 -> received invalid path
 *     0x00040404 -> output buffer overflow
 *     0x00000220 -> NaN detected in intermediate computation
 *     0x00200120 -> NaN detected in vertex normal of input
 *
 *   TODO : Do this in a better way, such that error information can still be
 *          concatenated/built-up from sub-functions and subjected to partial
 *          queries, but without the limitations of a bitfield.
 *
 * -------------------------------------------------------------------------- */

#define Error_None         0x00000000

#define Error_Null         0x00000001
#define Error_Invalid      0x00000002
#define Error_Overflow     0x00000004
#define Error_Underflow    0x00000008
#define Error_Empty        0x00000010
#define Error_NaN          0x00000020
#define Error_Degenerate   0x00000040
#define Error_BadCount     0x00000080

#define Error_Input        0x00000100
#define Error_Intermediate 0x00000200
#define Error_Output       0x00000400

#define Error_Stack        0x00010000
#define Error_Heap         0x00020000
#define Error_Buffer       0x00040000
#define Error_Path         0x00080000
#define Error_Index        0x00100000
#define Error_Vertex       0x00200000
#define Error_VertPos      0x00400000
#define Error_VertNorm     0x00800000
#define Error_VertUV       0x01000000

PHX_API void  Error_Print  (Error);

#endif
