#ifndef PHX_Window
#define PHX_Window

#include "Common.h"

#include "Graphics/GraphicsEngineOpenGL/interface/EngineFactoryOpenGL.h"
#include "Graphics/GraphicsEngine/interface/GraphicsTypes.h"
#include "Common/interface/RefCntAutoPtr.hpp"

PHX_API Window*  Window_Create            (cstr title, WindowPos x, WindowPos y, int sx, int sy, WindowMode mode);
PHX_API void     Window_Free              (Window*);

PHX_API void     Window_BeginDraw         (Window*);
PHX_API void     Window_EndDraw           (Window*);

PHX_API void     Window_GetPosition       (Window*, Vec2i* out);
PHX_API void     Window_GetSize           (Window*, Vec2i* out);
PHX_API cstr     Window_GetTitle          (Window*);

PHX_API void     Window_SetFullscreen     (Window*, bool);
PHX_API void     Window_SetPosition       (Window*, WindowPos, WindowPos);
PHX_API void     Window_SetSize           (Window*, int, int);
PHX_API void     Window_SetTitle          (Window*, cstr);
PHX_API void     Window_SetVsync          (Window*, bool);

PHX_API void     Window_ToggleFullscreen  (Window*);

PHX_API void     Window_Hide              (Window*);
PHX_API void     Window_Show              (Window*);

// Internal renderer state information.
struct RendererState {
    Diligent::RefCntAutoPtr<Diligent::IRenderDevice>  device;
    Diligent::RefCntAutoPtr<Diligent::IDeviceContext> immediateContext;
    Diligent::RefCntAutoPtr<Diligent::ISwapChain>     swapChain;
};

PRIVATE RendererState* Window_GetRS        (Window*);
PRIVATE RendererState* Window_GetCurrentRS ();

#endif
