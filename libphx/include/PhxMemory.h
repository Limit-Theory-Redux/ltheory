#ifndef PHX_Memory
#define PHX_Memory

#include "Common.h"

#include <cstdlib>
#include <cstring>
#include <new>
#include <utility>

inline void*  MemAlloc      (size_t size);
inline void*  MemAllocZero  (size_t size);
inline void   MemCpy        (void* dst, void const* src, size_t size);
inline void   MemMove       (void* dst, void const* src, size_t size);
inline void   MemFree       (void const* ptr);
inline void*  MemRealloc    (void* ptr, size_t newSize);
inline void   MemSet        (void* dst, int value, size_t size);
inline void   MemZero       (void* dst, size_t size);

template <typename T, typename... Args>
T* MemNewImpl(Args&& ...args) {
  return new (MemAlloc(sizeof(T))) T(std::forward<Args>(args)...);
}

template <typename T>
T* MemNewArrayImpl(size_t s) {
  T* memory = (T*)MemAlloc(sizeof(T) * s);
  for (size_t i = 0; i < s; ++i) {
    new (memory + i) T();
  }
  return memory;
}

#define MemNew(T, ...)        (MemNewImpl<T>(__VA_ARGS__))
#define MemNewZero(T)         ((T*)MemAllocZero(sizeof(T)))
#define MemNewArray(T, s)     (MemNewArrayImpl<T>(s))
#define MemNewArrayZero(T, s) ((T*)MemAllocZero(sizeof(T) * (s)))

/* Exported versions for applications that need to ensure usage of the same
 * memory allocator as libphx. */
PHX_API void*  Memory_Alloc    (size_t);
PHX_API void*  Memory_Calloc   (size_t n, size_t size);
PHX_API void   Memory_Free     (void* ptr);
PHX_API void   Memory_MemCopy  (void* dst, void const* src, size_t size);
PHX_API void   Memory_MemMove  (void* dst, void const* src, size_t size);
PHX_API void*  Memory_Realloc  (void* ptr, size_t newSize);

/* -------------------------------------------------------------------------- */

inline void* MemAlloc (size_t size) {
  return malloc(size);
}

inline void* MemAllocZero (size_t size) {
  return calloc(1, size);
}

inline void MemCpy (void* dst, void const* src, size_t size) {
  memcpy(dst, src, size);
}

inline void MemMove (void* dst, void const* src, size_t size) {
  memmove(dst, src, size);
}

template <typename T>
inline void MemFree (T* ptr) {
  ptr->~T();
  free((void*)ptr);
}

template <>
inline void MemFree<void> (void* ptr) {
  free(ptr);
}

inline void* MemRealloc (void* ptr, size_t newSize) {
  return realloc(ptr, newSize);
}

inline void MemSet (void* dst, int value, size_t size) {
  memset(dst, value, size);
}

inline void MemZero (void* dst, size_t size) {
  memset(dst, 0, size);
}

#endif
