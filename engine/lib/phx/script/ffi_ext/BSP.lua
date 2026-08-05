local ffi = require('ffi')
local libphx = require('libphx').lib

-- BSP_Create now also takes the current Renderer as an explicit argument
-- (it eagerly loads a debug-draw shader - see ai/multithreaded_rendering.md).
function onDef_BSP(t, mt)
    t.Create = function(...)
        local e = libphx.Mesh_Validate(...)
        if e ~= 0 then
            Log.Warn('BSP Incoming Mesh Error:')
            libphx.Error_Print(e)
        end
        local result = libphx.BSP_Create(Renderer, ...)
        return result
    end
end

-- These now take the current Renderer as an explicit argument (see
-- ai/multithreaded_rendering.md); inject the global `Renderer` set by
-- SetEngine so call sites don't change.
function onDef_BSP_t(t, mt)
    mt.__index.drawNode = function(self, nodeRef, color)
        ffi.gc(nodeRef, nil)
        libphx.BSP_DrawNode(self, Renderer, nodeRef, color)
    end

    mt.__index.drawNodeSplit = function(self, nodeRef)
        ffi.gc(nodeRef, nil)
        libphx.BSP_DrawNodeSplit(self, Renderer, nodeRef)
    end

    mt.__index.drawLineSegment = function(self, lineSegment, eye)
        libphx.BSP_DrawLineSegment(self, Renderer, lineSegment, eye)
    end

    mt.__index.drawSphere = function(self, sphere)
        libphx.BSP_DrawSphere(self, Renderer, sphere)
    end
end
