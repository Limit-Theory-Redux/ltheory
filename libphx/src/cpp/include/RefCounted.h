#ifndef PHX_RefCounted
#define PHX_RefCounted

#ifdef __cplusplus
#define MUTABLE mutable
#else
#define MUTABLE
#endif

#define RefCounted               MUTABLE uint32 _refCount
#define RefCounted_Init(self)    (self)->_refCount = 1
#define RefCounted_Free(self)    if (self && (--((self)->_refCount) <= 0))
#define RefCounted_Acquire(self) (self)->_refCount++

#endif
