#ifndef PHX_SoundDef
#define PHX_SoundDef

#include "Common.h"
#include "RefCounted.h"
#include "fmod.h"

struct SoundDesc {
  RefCounted;
  FMOD_SOUND* handle;
  cstr        name;
  cstr        path;
};

typedef uint8 SoundState;
#define SoundState_Null     0
#define SoundState_Loading  1
#define SoundState_Paused   2
#define SoundState_Playing  3
#define SoundState_Finished 4
#define SoundState_Freed    5

struct Sound {
  SoundDesc*    desc;
  FMOD_CHANNEL* handle;
  SoundState    state;
  Vec3f const*  autoPos;
  Vec3f const*  autoVel;
  bool          freeOnFinish;
};

#endif
