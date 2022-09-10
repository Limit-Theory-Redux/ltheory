#include "PhxMemory.h"
#include "OpenGL.h"
#include "SDLext.h"
#include "Vec2.h"
#include "Viewport.h"
#include "Window.h"
#include "WindowMode.h"
#include "Draw.h"

struct Window {
  SDL_Window* handle;
  WindowMode mode;
  RendererState rs;
};

static Window* currentWindow = nullptr;

Window* Window_Create (cstr title, int x, int y, int sx, int sy, WindowMode mode) {
  Window* self = MemNew(Window);
  self->handle = SDL_CreateWindow(title, x, y, sx, sy, mode);

  SDL_SysWMinfo wmi;
  SDL_VERSION(&wmi.version);
  if (!SDL_GetWindowWMInfo(self->handle, &wmi)) {
    Fatal("Failed to create OpenGL immediateContext for window");
  }

  // Initialize renderer.
  Diligent::SwapChainDesc swapChainDesc;

  Diligent::EngineGLCreateInfo engineCI;
  engineCI.Window.pNSView = wmi.info.cocoa.window;

  // TODO: Initialize other renderers.
  auto factory = Diligent::GetEngineFactoryOpenGL();
  factory->CreateDeviceAndSwapChainGL(
      engineCI,
      &self->rs.device,
      &self->rs.immediateContext,
      swapChainDesc,
      &self->rs.swapChain);

  self->mode = mode;
  OpenGL_Init();
  Draw_Init();
  return self;
}

void Window_Free (Window* self) {
  Draw_Free();

  self->rs.swapChain.Release();
  self->rs.immediateContext.Release();
  self->rs.device.Release();

  SDL_DestroyWindow(self->handle);
  MemFree(self);
}

void Window_BeginDraw (Window* self) {
  currentWindow = self;

  Vec2i size;
  Window_GetSize(self, &size);
  Viewport_Push(0, 0, size.x, size.y, true);
}

void Window_EndDraw (Window* self) {
  Viewport_Pop();

  currentWindow = nullptr;
}

void Window_GetSize (Window* self, Vec2i* out) {
  SDL_GetWindowSize(self->handle, &out->x, &out->y);
}

void Window_GetPosition (Window* self, Vec2i* out) {
  SDL_GetWindowPosition(self->handle, &out->x, &out->y);
}

cstr Window_GetTitle (Window* self) {
  return SDL_GetWindowTitle(self->handle);
}

void Window_SetFullscreen (Window* self, bool fs) {
  SDL_SetWindowFullscreen(self->handle, fs ? WindowMode_Fullscreen : 0);
}

void Window_SetPosition (Window* self, WindowPos x, WindowPos y) {
  SDL_SetWindowPosition(self->handle, x, y);
}

void Window_SetSize (Window* self, int sx, int sy) {
  SDL_SetWindowSize(self->handle, sx, sy);
}

void Window_SetTitle (Window* self, cstr title) {
  SDL_SetWindowTitle(self->handle, title);
}

void Window_SetVsync (Window*, bool vsync) {
  SDL_GL_SetSwapInterval(vsync ? 1 : 0);
}

void Window_ToggleFullscreen (Window* self) {
  if (self->mode & WindowMode_Fullscreen)
    SDL_SetWindowFullscreen(self->handle, 0);
  else
    SDL_SetWindowFullscreen(self->handle, WindowMode_Fullscreen);
  self->mode ^= WindowMode_Fullscreen;
}

void Window_Hide (Window* self) {
  SDL_HideWindow(self->handle);
}

void Window_Show (Window* self) {
  SDL_ShowWindow(self->handle);
}

RendererState* Window_GetRS (Window* self) {
  return &self->rs;
}

RendererState* Window_GetCurrentRS () {
  return currentWindow != nullptr ? &currentWindow->rs : nullptr;
}
