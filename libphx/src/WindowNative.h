#ifndef PHX_WindowNative
#define PHX_WindowNative

#include <SDL.h>
#include <Platforms/interface/NativeWindow.h>

// Populates Diligent::NativeWindow from SDL_Window. Returns true if succeeded, false otherwise.
bool PopulateNativeWindow(SDL_Window* window, Diligent::NativeWindow& out);

#endif
