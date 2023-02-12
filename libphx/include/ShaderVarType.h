#ifndef PHX_ShaderVarType
#define PHX_ShaderVarType

#include "Common.h"

#define ShaderVarType_None    0x0
#define ShaderVarType_BEGIN   0x1
#define ShaderVarType_Float   0x1
#define ShaderVarType_Float2  0x2
#define ShaderVarType_Float3  0x3
#define ShaderVarType_Float4  0x4
#define ShaderVarType_Int     0x5
#define ShaderVarType_Int2    0x6
#define ShaderVarType_Int3    0x7
#define ShaderVarType_Int4    0x8
#define ShaderVarType_Matrix  0x9
#define ShaderVarType_Tex1D   0xA
#define ShaderVarType_Tex2D   0xB
#define ShaderVarType_Tex3D   0xC
#define ShaderVarType_TexCube 0xD
#define ShaderVarType_END     0xD
#define ShaderVarType_SIZE    0xD

PHX_API ShaderVarType  ShaderVarType_FromStr      (cstr);
PHX_API cstr           ShaderVarType_GetGLSLName  (ShaderVarType);
PHX_API cstr           ShaderVarType_GetName      (ShaderVarType);
PHX_API int            ShaderVarType_GetSize      (ShaderVarType);

#endif
