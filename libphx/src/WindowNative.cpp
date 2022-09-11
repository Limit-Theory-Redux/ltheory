#include "WindowNative.h"

#include <SDL_syswm.h>

bool PopulateNativeWindow(SDL_Window* window, Diligent::NativeWindow& out) {
    SDL_SysWMinfo wmi;
    SDL_VERSION(&wmi.version);
    if (!SDL_GetWindowWMInfo(window, &wmi)) {
        return false;
    }

#ifdef PLATFORM_MACOS
    out.pNSView = wmi.info.cocoa.window;
#elif PLATFORM_LINUX
    if (wmi.subsystem == SDL_SYSWM_X11) {
      out.pDisplay = wmi.info.x11.display;
      out.WindowId = wmi.info.x11.window;
//    } else {
//      out.pDisplay = wmi.info.wl.
    } else {
      return false;
    }
#endif
    return true;
}
