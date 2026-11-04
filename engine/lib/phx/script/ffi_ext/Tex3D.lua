local libphx = require('libphx').lib

-- These now take the current Renderer as an explicit argument (see
-- doc/engine/render-thread.md); inject the global `Renderer` set by
-- SetEngine so call sites don't change.
-- The optional `desc` of `Create`: `{ mips = true | <levels>, usage = <TexUsage bits> }`.
-- `mips = true` allocates the full mip chain; no `mips` is a single level, no
-- `usage` is the default for the texture kind.
local function mipsOf(desc)
    if desc.mips == true then return 0 end
    return desc.mips or 1
end

function onDef_Tex3D(t, mt)
    t.Create = function(sx, sy, sz, format, desc)
        local _instance
        if desc then
            _instance = libphx.Tex3D_CreateDesc(Renderer, sx, sy, sz, format, mipsOf(desc), desc.usage or 0)
        else
            _instance = libphx.Tex3D_Create(Renderer, sx, sy, sz, format)
        end
        return Core.ManagedObject(_instance, libphx.Tex3D_Free)
    end
end

function onDef_Tex3D_t(t, mt)
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
        libphx.Tex3D_GenMipmap(self, Renderer)
    end

    mt.__index.getDataBytes = function(self, pf, df)
        local _instance = libphx.Tex3D_GetDataBytes(self, Renderer, pf, df)
        return Core.ManagedObject(_instance, libphx.Bytes_Free)
    end

    mt.__index.setDataBytes = function(self, data, pf, df)
        libphx.Tex3D_SetDataBytes(self, Renderer, data, pf, df)
    end
end
