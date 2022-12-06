#ifndef PHX_SDL
#define PHX_SDL

#ifdef UNIX
#include <SDL2/SDL.h>
#else
#include <sdl/SDL.h>
#endif
#ifdef main
#  undef main
#endif

#endif
