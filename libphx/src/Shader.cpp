#include "ArrayList.h"
#include "Matrix.h"
#include "PhxMemory.h"
//#include "OpenGL.h"
#include "Profiler.h"
#include "RefCounted.h"
#include "Resource.h"
#include "Shader.h"
#include "ShaderState.h"
#include "ShaderVar.h"
#include "ShaderVarType.h"
#include "StrMap.h"
#include "PhxString.h"
#include "Tex1D.h"
#include "Tex2D.h"
#include "Tex3D.h"
#include "TexCube.h"
#include "Vec2.h"
#include "Vec3.h"
#include "Vec4.h"
#include "Window.h"

//#include "ShaderResourceBindingBase.hpp"

#include <unordered_map>
#include <string>
#include <vector>

/* TODO : Implement custom directives to mimic layout functionality from GL3+. */
/* TODO : Use glShaderSource's array functionality to implement include files
 *        elegantly. */

static cstr includePath = "include/";
static cstr versionString = "#version 120\n#define texture2DLod texture2D\n#define textureCubeLod textureCube\n";

struct ShaderVar {
  ShaderVarType type;
  cstr name;
  int index;
};

class UniformSet {
public:
    struct Uniform {
        uint32_t offset;
        uint32_t size;
    };

    std::vector<Uniform> uniforms;
    std::unordered_map<std::string, uint32_t> uniformIndices;

    Diligent::RefCntAutoPtr<Diligent::IBuffer> constantBuffer;

    void populate(Diligent::IShader* shader) {
    }
};

struct Shader {
  RefCounted;
  cstr name;
  Diligent::RefCntAutoPtr<Diligent::IShader> vs;
  Diligent::RefCntAutoPtr<Diligent::IShader> ps;
//  Diligent::RefCntAutoPtr<Diligent::IPipelineState> pso;
  uint texIndex;
  ArrayList(ShaderVar, vars);
};

static Shader* current = 0;
static StrMap* cache = 0;

static cstr GLSL_Load(cstr path, Shader*, bool);
static cstr GLSL_Preprocess(cstr code, Shader*);

static int GetUniformIndex (Shader* self, cstr name, bool mustSucceed = false) {
  return -1;
//  if (!self)
//    Fatal("GetUniformIndex: No shader is bound");
//  int index = glGetUniformLocation(self->program, name);
//  if (index == -1 && mustSucceed)
//    Fatal("GetUniformIndex: Shader <%s> has no variable <%s>", self->name, name);
//  return index;
}

static Diligent::IShader* CreateShader (cstr name, cstr src, uint32_t type) {
  Diligent::IShader* result = nullptr;
  Diligent::ShaderCreateInfo ShaderCI;
  ShaderCI.Desc.ShaderType = static_cast<Diligent::SHADER_TYPE>(type);
  ShaderCI.EntryPoint = "main";
  ShaderCI.Desc.Name = name;
  ShaderCI.Source = src;
  ShaderCI.SourceLanguage = Diligent::SHADER_SOURCE_LANGUAGE_GLSL;
  Window_GetCurrentRS()->device->CreateShader(ShaderCI, &result);
  if (result == nullptr) {
    Fatal("Failed to create shader %s", name);
  }
//  Diligent::ShaderResourceDesc desc;
////  desc.Type
//  result->GetResourceDesc(0, desc);
  return result;

//  uint self = glCreateShader(type);
//
//  cstr srcs[] = {
//    versionString,
//    src,
//  };
//
//  GLCALL(glShaderSource(self, 2, srcs, 0))
//  GLCALL(glCompileShader(self))
//
//  /* Check for compile errors. */ {
//    int status;
//    GLCALL(glGetShaderiv(self, GL_COMPILE_STATUS, &status))
//    if (status == GL_FALSE) {
//      int length;
//      GLCALL(glGetShaderiv(self, GL_INFO_LOG_LENGTH, &length))
//      char* infoLog = (char*)MemAllocZero(length + 1);
//      GLCALL(glGetShaderInfoLog(self, length, 0, infoLog))
//      Fatal("CreateShader: Failed to compile shader:\n%s", infoLog);
//    }
//  }
//  return self;
}

//static Diligent::IPipelineState* CreatePSO (Diligent::IShader* vs, Diligent::IShader* fs) {
//  Diligent::IPipelineState* pso = nullptr;
//
//  Diligent::PipelineStateDesc PSODesc;
//  Diligent::ShaderResourceVariableDesc Vars[] =
//      {
//          {Diligent::SHADER_TYPE_PIXEL, "g_Texture", Diligent::SHADER_RESOURCE_VARIABLE_TYPE_MUTABLE}
//      };
//  PSODesc.ResourceLayout.Variables    = Vars;
//  PSODesc.ResourceLayout.NumVariables = _countof(Vars);
//
//  // Define static sampler for g_Texture. Static samplers should be used whenever possible
//  Diligent::SamplerDesc SamLinearClampDesc( Diligent::FILTER_TYPE_LINEAR, Diligent::FILTER_TYPE_LINEAR, Diligent::FILTER_TYPE_LINEAR,
//                                  Diligent::TEXTURE_ADDRESS_CLAMP, Diligent::TEXTURE_ADDRESS_CLAMP, Diligent::TEXTURE_ADDRESS_CLAMP);
//  Diligent::StaticSamplerDesc StaticSamplers[] =
//      {
//          {Diligent::SHADER_TYPE_PIXEL, "g_Texture", SamLinearClampDesc}
//      };
//  PSODesc.ResourceLayout.StaticSamplers    = StaticSamplers;
//  PSODesc.ResourceLayout.NumStaticSamplers = _countof(StaticSamplers);
//
////  Diligent::GraphicsPipelineStateCreateInfo createInfo;
////  createInfo.GraphicsPipeline = PSODesc
//  Diligent::PipelineStateCreateInfo createInfo;
//  createInfo.PSODesc = PSODesc;
//  Window_GetCurrentRS()->device->CreatePipelineState(&createInfo, &pso);
//  pso->GetStaticShaderVariable(Diligent::SHADER_TYPE_VERTEX, "Constants")->Set(m_VSConstants);
//  return pso;
//  uint self = glCreateProgram();
//  GLCALL(glAttachShader(self, vs))
//  GLCALL(glAttachShader(self, fs))
//
//  /* TODO : Replace with custom directives. */ {
//    GLCALL(glBindAttribLocation(self, 0, "vertex_position"))
//    GLCALL(glBindAttribLocation(self, 1, "vertex_normal"))
//    GLCALL(glBindAttribLocation(self, 2, "vertex_uv"))
//  }
//
//  GLCALL(glLinkProgram(self))
//
//  /* Check for link errors. */ {
//    int status;
//    GLCALL(glGetProgramiv(self, GL_LINK_STATUS, &status))
//    if (status == GL_FALSE) {
//      int length;
//      GLCALL(glGetProgramiv(self, GL_INFO_LOG_LENGTH, &length))
//      char* infoLog = (char*)MemAllocZero(length + 1);
//      GLCALL(glGetProgramInfoLog(self, length, 0, infoLog))
//      Fatal("CreateGLProgram: Failed to link program:\n%s", infoLog);
//    }
//  }
//  return self;
//}

/* BUG : Cache does not contain information about custom preprocessor
 *       directives, hence cached shaders with custom directives do not work */
static cstr GLSL_Load (cstr name, Shader* self, bool addExtension = true) {
  cstr resName = addExtension ? StrAdd(name, ".glsl") : StrDup(name);
  cstr rawCode = Resource_LoadCstr(ResourceType_Shader, resName);
  cstr code = StrReplace(rawCode, "\r\n", "\n");
  cstr preprocessedCode = GLSL_Preprocess(code, self);
  StrFree(rawCode);
  StrFree(resName);
  return preprocessedCode;
//  if (!cache)
//    cache = StrMap_Create(16);
//  void* cached = StrMap_Get(cache, name);
//  if (cached)
//    return (cstr)cached;
//  cstr resName = StrAdd(name, ".glsl");
//  cstr rawCode = Resource_LoadCstr(ResourceType_Shader, resName);
//  cstr code = StrReplace(rawCode, "\r\n", "\n");
//  StrFree(rawCode);
//  StrFree(resName);
//  code = GLSL_Preprocess(code, self);
//  /* BUG : Disable GLSL caching until preprocessor cache works. */
//  // StrMap_Set(cache, name, (void*)code);
//  return code;
}

static cstr GLSL_Preprocess (cstr code, Shader* self) {
  cstr begin;

  /* Parse Includes. */
  while ((begin = StrFind(code, "#include")) != 0) {
    cstr nameStart = StrFind(begin, "\"");
    cstr nameEnd = StrFind(nameStart + 1, "\"");
    cstr name = StrSubStr(nameStart + 1, nameEnd);
    cstr path = StrAdd(includePath, name);
    cstr prev = code;
    code = StrSub(code, begin, nameEnd + 1, GLSL_Load(path, self, false));
    StrFree(prev);
    StrFree(path);
    StrFree(name);
  }

  /* Parse automatic ShaderVar stack bindings. */
  while ((begin = StrFind(code, "#autovar")) != 0) {
    cstr end = StrFind(begin, "\n");
    cstr line = StrSubStr(begin, end);
    char varType[32] = { 0 };
    char varName[32] = { 0 };

    if (sscanf(line, "#autovar %31s %31s", varType, varName) == 2) {
      ShaderVar var = { 0 };
      var.type = ShaderVarType_FromStr(varType);
      if (var.type == ShaderVarType_None)
        Fatal("GLSL_Preprocess: Unknown shader variable type <%s> "
              "in directive:\n  %s", varType, line);
      var.name = StrDup(varName);
      var.index = -1;
      ArrayList_Append(self->vars, var);
    } else {
      Fatal("GLSL_Preprocess: Failed to parse directive:\n  %s", line);
    }

    cstr prev = code;
    code = StrSub(code, begin, end, "");
    StrFree(prev);
    StrFree(line);
  }
  return code;
}

static void Shader_BindVariables (Shader* self) {
//  for (int i = 0; i < ArrayList_GetSize(self->vars); ++i) {
//    ShaderVar* var = ArrayList_GetPtr(self->vars, i);
//    var->index = glGetUniformLocation(self->program, var->name);
//    if (var->index < 0)
//      Warn("Shader_BindVariables: Automatic shader variable <%s> does not exist"
//           " in shader <%s>", var->name, self->name);
//  }
}

/* --- Creation ------------------------------------------------------------- */

Shader* Shader_Create (cstr vs, cstr fs) {
  Shader* self = MemNew(Shader);
  RefCounted_Init(self);
  ArrayList_Init(self->vars);
  vs = GLSL_Preprocess(StrReplace(vs, "\r\n", "\n"), self);
  fs = GLSL_Preprocess(StrReplace(fs, "\r\n", "\n"), self);
  self->vs = CreateShader(StrFormat("[anonymous vs @ %p]", self), vs, Diligent::SHADER_TYPE_VERTEX);
  self->ps = CreateShader(StrFormat("[anonymous ps @ %p]", self), fs, Diligent::SHADER_TYPE_PIXEL);
//  self->program = CreatePSO(self->vs, self->ps);
  self->texIndex = 1;
  self->name = StrFormat("[anonymous shader @ %p]", self);
  StrFree(vs);
  StrFree(fs);
  Shader_BindVariables(self);
  return self;
}

Shader* Shader_Load (cstr vName, cstr fName) {
  Shader* self = MemNew(Shader);
  RefCounted_Init(self);
  ArrayList_Init(self->vars);
  cstr vs = GLSL_Load(vName, self);
  cstr ps = GLSL_Load(fName, self);
  self->vs = CreateShader(vName, vs, Diligent::SHADER_TYPE_VERTEX);
  self->ps = CreateShader(fName, ps, Diligent::SHADER_TYPE_PIXEL);
//  self->program = CreateGLProgram(self->vs, self->ps);
  self->texIndex = 1;
  self->name = StrFormat("[vs: %s , ps: %s]", vName, fName);
  Shader_BindVariables(self);
  return self;
}

void Shader_Acquire (Shader* self) {
  RefCounted_Acquire(self);
}

void Shader_Free (Shader* self) {
  RefCounted_Free(self) {
    self->ps.Release();
    self->vs.Release();
    ArrayList_Free(self->vars);
    StrFree(self->name);
    MemFree(self);
  }
}

ShaderState* Shader_ToShaderState (Shader* self) {
  return ShaderState_Create(self);
}

/* --- Usage ---------------------------------------------------------------- */

void Shader_Start (Shader* self) {
  FRAME_BEGIN;
  // TODO: Create PipelineStateManager which dynamically builds PSOs on demand.
//  GLCALL(glUseProgram(self->program))
  current = self;
  self->texIndex = 1;

  /* Fetch & bind automatic variables from the shader var stack. */
  for (int i = 0; i < ArrayList_GetSize(self->vars); ++i) {
    ShaderVar* var = ArrayList_GetPtr(self->vars, i);
    if (var->index < 0) continue;
    void* pValue = ShaderVar_Get(var->name, var->type);
    if (!pValue)
      Fatal("Shader_Start: Shader variable stack does not contain variable <%s>", var->name);

    switch (var->type) {
      case ShaderVarType_Float: {
        Shader_ISetFloat(var->index, *(float*)pValue);
        break; }
      case ShaderVarType_Float2: {
        Vec2f value = *(Vec2f*)pValue;
        Shader_ISetFloat2(var->index, value.x, value.y);
        break; }
      case ShaderVarType_Float3: {
        Vec3f value = *(Vec3f*)pValue;
        Shader_ISetFloat3(var->index, value.x, value.y, value.z);
        break; }
      case ShaderVarType_Float4: {
        Vec4f value = *(Vec4f*)pValue;
        Shader_ISetFloat4(var->index, value.x, value.y, value.z, value.w);
        break; }
      case ShaderVarType_Int: {
        Shader_ISetInt(var->index, *(int*)pValue);
        break; }
      case ShaderVarType_Int2: {
        Vec2i value = *(Vec2i*)pValue;
        Shader_ISetInt2(var->index, value.x, value.y);
        break; }
      case ShaderVarType_Int3: {
        Vec3i value = *(Vec3i*)pValue;
        Shader_ISetInt3(var->index, value.x, value.y, value.z);
        break; }
      case ShaderVarType_Int4: {
        Vec4i value = *(Vec4i*)pValue;
        Shader_ISetInt4(var->index, value.x, value.y, value.z, value.w);
        break; }
      case ShaderVarType_Matrix: {
        Shader_ISetMatrix(var->index, *(Matrix**)pValue);
        break; }
      case ShaderVarType_Tex1D: {
        Shader_ISetTex1D(var->index, *(Tex1D**)pValue);
        break; }
      case ShaderVarType_Tex2D: {
        Shader_ISetTex2D(var->index, *(Tex2D**)pValue);
        break; }
      case ShaderVarType_Tex3D: {
        Shader_ISetTex3D(var->index, *(Tex3D**)pValue);
        break; }
      case ShaderVarType_TexCube: {
        Shader_ISetTexCube(var->index, *(TexCube**)pValue);
        break; }
    }
  }
  FRAME_END;
}

void Shader_Stop (Shader*) {
//  GLCALL(glUseProgram(0))
  current = nullptr;
}

static void ShaderCache_FreeElem (cstr, void* data) {
  free(data);
}

void Shader_ClearCache () {
  if (cache) {
    StrMap_FreeEx(cache, ShaderCache_FreeElem);
    cache = 0;
  }
}

int Shader_GetVariable (Shader* self, cstr name) {
  return -1;
//  int index = glGetUniformLocation(self->program, name);
//  if (index == -1)
//    Fatal("Shader_GetVariable: Shader <%s> has no variable <%s>", self->name, name);
//  return index;
}

bool Shader_HasVariable (Shader* self, cstr name) {
  return false;
//  return glGetUniformLocation(self->program, name) > -1;
}

/* --- Variable Binding ----------------------------------------------------- */

void Shader_ResetTexIndex () {
  current->texIndex = 1;
}

void Shader_SetFloat (cstr name, float value) {
  Shader_ISetFloat(GetUniformIndex(current, name), value);
}

void Shader_SetFloat2 (cstr name, float x, float y) {
  Shader_ISetFloat2(GetUniformIndex(current, name), x, y);
}

void Shader_SetFloat3 (cstr name, float x, float y, float z) {
  Shader_ISetFloat3(GetUniformIndex(current, name), x, y, z);
}

void Shader_SetFloat4 (cstr name, float x, float y, float z, float w) {
  Shader_ISetFloat4(GetUniformIndex(current, name), x, y, z, w);
}

void Shader_SetInt (cstr name, int value) {
  Shader_ISetInt(GetUniformIndex(current, name), value);
}

void Shader_SetInt2 (cstr name, int x, int y) {
  Shader_ISetInt2(GetUniformIndex(current, name), x, y);
}

void Shader_SetInt3 (cstr name, int x, int y, int z) {
  Shader_ISetInt3(GetUniformIndex(current, name), x, y, z);
}

void Shader_SetInt4 (cstr name, int x, int y, int z, int w) {
  Shader_ISetInt4(GetUniformIndex(current, name), x, y, z, w);
}

void Shader_SetMatrix (cstr name, Matrix* value) {
  Shader_ISetMatrix(GetUniformIndex(current, name), value);
}

void Shader_SetMatrixT (cstr name, Matrix* value) {
  Shader_ISetMatrixT(GetUniformIndex(current, name), value);
}

void Shader_SetTex1D (cstr name, Tex1D* value) {
  Shader_ISetTex1D(GetUniformIndex(current, name), value);
}

void Shader_SetTex2D (cstr name, Tex2D* value) {
  Shader_ISetTex2D(GetUniformIndex(current, name), value);
}

void Shader_SetTex3D (cstr name, Tex3D* value) {
  Shader_ISetTex3D(GetUniformIndex(current, name), value);
}

void Shader_SetTexCube (cstr name, TexCube* value) {
  Shader_ISetTexCube(GetUniformIndex(current, name), value);
}

void Shader_ISetFloat (int index, float value) {
//  GLCALL(glUniform1f(index, value))
}

void Shader_ISetFloat2 (int index, float x, float y) {
//  GLCALL(glUniform2f(index, x, y))
}

void Shader_ISetFloat3 (int index, float x, float y, float z) {
//  GLCALL(glUniform3f(index, x, y, z))
}

void Shader_ISetFloat4 (int index, float x, float y, float z, float w) {
//  GLCALL(glUniform4f(index, x, y, z, w))
}

void Shader_ISetInt (int index, int value) {
//  GLCALL(glUniform1i(index, value))
}

void Shader_ISetInt2 (int index, int x, int y) {
//  GLCALL(glUniform2i(index, x, y))
}

void Shader_ISetInt3 (int index, int x, int y, int z) {
//  GLCALL(glUniform3i(index, x, y, z))
}

void Shader_ISetInt4 (int index, int x, int y, int z, int w) {
//  GLCALL(glUniform4i(index, x, y, z, w))
}

void Shader_ISetMatrix (int index, Matrix* value) {
//  GLCALL(glUniformMatrix4fv(index, 1, true, (float*)value))
}

void Shader_ISetMatrixT (int index, Matrix* value) {
//  GLCALL(glUniformMatrix4fv(index, 1, false, (float*)value))
}

void Shader_ISetTex1D (int index, Tex1D* value) {
//  GLCALL(glUniform1i(index, current->texIndex))
//  GLCALL(glActiveTexture(GL_TEXTURE0 + current->texIndex++))
//  GLCALL(glBindTexture(GL_TEXTURE_1D, Tex1D_GetHandle(value)))
//  GLCALL(glActiveTexture(GL_TEXTURE0))
}

void Shader_ISetTex2D (int index, Tex2D* value) {
//  GLCALL(glUniform1i(index, current->texIndex))
//  GLCALL(glActiveTexture(GL_TEXTURE0 + current->texIndex++))
//  GLCALL(glBindTexture(GL_TEXTURE_2D, Tex2D_GetHandle(value)))
//  GLCALL(glActiveTexture(GL_TEXTURE0))
}

void Shader_ISetTex3D (int index, Tex3D* value) {
//  GLCALL(glUniform1i(index, current->texIndex))
//  GLCALL(glActiveTexture(GL_TEXTURE0 + current->texIndex++))
//  GLCALL(glBindTexture(GL_TEXTURE_3D, Tex3D_GetHandle(value)))
//  GLCALL(glActiveTexture(GL_TEXTURE0))
}

void Shader_ISetTexCube (int index, TexCube* value) {
//  GLCALL(glUniform1i(index, current->texIndex))
//  GLCALL(glActiveTexture(GL_TEXTURE0 + current->texIndex++))
//  GLCALL(glBindTexture(GL_TEXTURE_CUBE_MAP, TexCube_GetHandle(value)))
//  GLCALL(glActiveTexture(GL_TEXTURE0))
}
