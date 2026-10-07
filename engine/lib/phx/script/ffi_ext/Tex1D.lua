local libphx = require('libphx').lib

-- Tex1D is now executor-owned (ResourceId, not a raw GL handle); every
-- GL-touching method takes the current Renderer as an explicit argument
-- (see doc/engine/render-thread.md). Inject the global `Renderer` set by
-- SetEngine so call sites don't change.
-- The optional `desc` of `Create`: `{ mips = true | <levels>, usage = <TexUsage bits> }`.
-- `mips = true` allocates the full mip chain; no `mips` is a single level, no
-- `usage` is the default for the texture kind.
local function mipsOf(desc)
    if desc.mips == true then return 0 end
    return desc.mips or 1
end

function onDef_Tex1D(t, mt)
    t.Create = function(size, format, desc)
        local _instance
        if desc then
            _instance = libphx.Tex1D_CreateDesc(Renderer, size, format, mipsOf(desc), desc.usage or 0)
        else
            _instance = libphx.Tex1D_Create(Renderer, size, format)
        end
        return Core.ManagedObject(_instance, libphx.Tex1D_Free)
    end
end

function onDef_Tex1D_t(t, mt)
    -- `tex:view()` is the whole texture (level 0 as an attachment);
    -- `tex:view{ baseMip = 1, mipCount = 3 }` restricts the levels sampled
    -- (`mipCount` 0 = all remaining).
    local view = mt.__index.view
    mt.__index.view = function(self, opts)
        local v = view(self)
        if opts then
            return v:mips(opts.baseMip or 0, opts.mipCount or 0)
        end
        return v
    end

    mt.__index.genMipmap = function(self)
        libphx.Tex1D_GenMipmap(self, Renderer)
    end

    mt.__index.getDataBytes = function(self, pf, df)
        local _instance = libphx.Tex1D_GetDataBytes(self, Renderer, pf, df)
        return Core.ManagedObject(_instance, libphx.Bytes_Free)
    end

    mt.__index.setDataBytes = function(self, data, pf, df)
        libphx.Tex1D_SetDataBytes(self, Renderer, data, pf, df)
    end

    mt.__index.setTexel = function(self, x, red, green, blue, alpha)
        libphx.Tex1D_SetTexel(self, Renderer, x, red, green, blue, alpha)
    end
end
