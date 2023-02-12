#ifndef PHX_Memory
#define PHX_Memory

#include "Common.h"

#include <stdlib.h>
#include <string.h>

#define MemNew(x)             ((x*)MemAlloc(sizeof(x)))
#define MemNewZero(x)         ((x*)MemAllocZero(sizeof(x)))
#define MemNewArray(x, s)     ((x*)MemAlloc(sizeof(x) * (s)))
#define MemNewArrayZero(x, s) ((x*)MemAllocZero(sizeof(x) * (s)))

/* Exported versions for applications that need to ensure usage of the same
 * memory allocator as libphx. */
PHX_API void*  Memory_Alloc    (size_t);
PHX_API void*  Memory_Calloc   (size_t n, size_t size);
PHX_API void   Memory_Free     (void* ptr);
PHX_API void   Memory_MemCopy  (void* dst, void const* src, size_t size);
PHX_API void   Memory_MemMove  (void* dst, void const* src, size_t size);
PHX_API void*  Memory_Realloc  (void* ptr, size_t newSize);

/* -------------------------------------------------------------------------- */

static inline void* MemAlloc (size_t size) {
  return malloc(size);
}

static inline void* MemAllocZero (size_t size) {
  return calloc(1, size);
}

static inline void MemCpy (void* dst, void const* src, size_t size) {
  memcpy(dst, src, size);
}

static inline void MemMove (void* dst, void const* src, size_t size) {
  memmove(dst, src, size);
}

static inline void MemFree (void const* ptr) {
  free((void*)ptr);
}

static inline void* MemRealloc (void* ptr, size_t newSize) {
  return realloc(ptr, newSize);
}

static inline void MemSet (void* dst, int value, size_t size) {
  memset(dst, value, size);
}

static inline void MemZero (void* dst, size_t size) {
  memset(dst, 0, size);
}

#endif
