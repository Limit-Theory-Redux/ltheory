local ffi = require('ffi')
local libphx = require('libphx').lib

-- Hand-written export (the FFI generator cannot return raw pointers): the
-- group-2 block of the next draw, in uniform ring staging.
ffi.cdef [[
    uint8* RenderPass_Alloc(RenderPass const*, Renderer* r, uint32 size);
]]

-- A `RenderPass` records pass commands on the current Renderer (see
-- doc/engine/render-api-v2.md); inject the global `Renderer` set by
-- SetEngine so call sites don't pass it.
function onDef_RenderPass_t(t, mt)
    local index = mt.__index

    index.finish = function(self)
        libphx.RenderPass_Finish(self, Renderer)
    end

    index.setPipeline = function(self, pipeline)
        libphx.RenderPass_SetPipeline(self, Renderer, pipeline)
    end

    --- `pass:setInputs(view0, sampler0, view1, sampler1, ...)`: group 3
    --- textures (up to 4), bound together before the next draw.
    index.setInputs = function(self, ...)
        local n = select('#', ...)
        for i = 1, n, 2 do
            local view, sampler = select(i, ...)
            libphx.RenderPass_SetInput(self, Renderer, (i - 1) / 2, view, sampler)
        end
    end

    index.setInput = function(self, slot, view, sampler)
        libphx.RenderPass_SetInput(self, Renderer, slot, view, sampler)
    end

    index.clearInput = function(self, slot)
        libphx.RenderPass_ClearInput(self, Renderer, slot)
    end

    index.setBindGroup = function(self, group, bindGroup)
        libphx.RenderPass_SetBindGroup(self, Renderer, group, bindGroup)
    end

    index.drawMesh = function(self, mesh)
        libphx.RenderPass_DrawMesh(self, Renderer, mesh)
    end

    index.drawFullscreen = function(self)
        libphx.RenderPass_DrawFullscreen(self, Renderer)
    end

    index.setViewport = function(self, x, y, width, height)
        libphx.RenderPass_SetViewport(self, Renderer, x, y, width, height)
    end

    index.setScissor = function(self, x, y, width, height)
        libphx.RenderPass_SetScissor(self, Renderer, x, y, width, height)
    end

    index.clearScissor = function(self)
        libphx.RenderPass_ClearScissor(self, Renderer)
    end

    index.setUiTransform = function(self, transform)
        libphx.RenderPass_SetUiTransform(self, Renderer, transform)
    end

    --- `pass:alloc(T)`: a zeroed `T*` (T from `shader:blockType(name)`) that
    --- becomes the group-2 block of the next draw. Write its fields, then
    --- draw; the pointer is stale once the draw is recorded.
    local pointerTypes = setmetatable({}, { __mode = 'k' })
    index.alloc = function(self, T)
        local pointerType = pointerTypes[T]
        if not pointerType then
            pointerType = ffi.typeof('$ *', T)
            pointerTypes[T] = pointerType
        end
        return ffi.cast(pointerType, libphx.RenderPass_Alloc(self, Renderer, ffi.sizeof(T)))
    end
end
