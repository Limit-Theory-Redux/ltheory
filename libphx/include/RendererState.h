#ifndef PHX_RendererState
#define PHX_RendererState

// TODO: Have a "current" render state which draw calls within the engine are based off. Similar to how GL
// TODO: "current context" works.
// TODO:
// TODO: This current window is updated when we start a new frame for a particular Window.

#include "Graphics/GraphicsEngineOpenGL/interface/EngineFactoryOpenGL.h"
#include "Graphics/GraphicsEngine/interface/GraphicsTypes.h"
#include "Common/interface/RefCntAutoPtr.hpp"

struct RendererState {
    Diligent::RefCntAutoPtr<Diligent::IRenderDevice> device;
    Diligent::RefCntAutoPtr<Diligent::IDeviceContext> immediateContext;
    Diligent::RefCntAutoPtr<Diligent::ISwapChain> swapChain;
};

#endif