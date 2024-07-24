--[[----------------------------------------------------------------------------
  Simple wrapper around namespace loading (for phx, util, etc.) Provides
  automatic loading of a module and assigning it to a global key. LoadInline
  additionally injects all of the module's symbols into global scope and warns
  about possible conflicts.

  The keySource table, which knows from which module global keys originated,
  could be used to provide additional debug information in the future.
----------------------------------------------------------------------------]]
--
local Log = require('Core.Util.Log')

local Namespace = {}

local keySource = {}

function Namespace.Inline(namespace, name)
    Namespace.Inject(_G, '_G', namespace, name)
end

function Namespace.Inject(dst, dstName, src, srcName)
    keySource[dst] = keySource[dst] or {}
    for k, v in pairs(src) do
        if type(v) ~= 'boolean' then
            -- We don't allow shadowing. If the type is there already, leave it.
            if not rawget(dst, k) then
                keySource[dst][k] = srcName
                rawset(dst, k, v)
            end
        end
    end
end

function Namespace.Load(path)
    local self = requireAll(path)
    rawset(_G, path, self)
    return self
end

function Namespace.LoadInline(path)
    local self = Namespace.Load(path)
    Namespace.Inject(_G, '_G', self, path)
    return self
end

function Namespace.LoadInject(dst, dstName, path, srcName)
    local self = Namespace.Load(path)
    Namespace.Inject(dst, dstName, self, srcName)
    return self
end

return Namespace
