--- Named looks for strokes: how a stroke's light is drawn (core, glow,
--- shade, dark halo), defined once under a name and picked per widget.
---
---   UICore.Look.define('hudNeon', { glow = 0.5, glowWidth = 6, shade = 0 })
---   UIComponent.Arc { size = 320, thickness = 2, start = 225, sweep = 90, look = 'hudNeon' }
---   Gui:arc(color, 2, 225, 90); Gui:setStrokeLook('hudNeon')
---
--- Values not given take their defaults, which draw a plain line in its
--- colour:
---   core      = 1    strength of the solid core
---   coreWidth = 0.5  how soft its edge is, in pixels
---   glow      = 0    strength of the glow around it
---   glowWidth = 4    reach of the glow, in pixels
---   boost     = 1    multiplies the colour of the light
---   shade     = 1    how much it covers what is behind: 0 only adds light
---                    (neon), 1 blends like normal alpha
---   halo      = 0    a dark halo that darkens what is behind (1 = black)
---   haloWidth = 8    how far the dark halo reaches, in pixels
---
--- Looks live in the engine's Gui and outlive frames: define them once.
--- Change one and every stroke that uses it follows. A widget can override
--- single values on top of its look (`lookParams` on the components,
--- `Gui:setStrokeLookParam`). The built-in looks `annulus`, `ring`, `wedge`,
--- `line` and their `*Glow` twins are what those shape calls draw with;
--- redefining one restyles all of them.
local Look = {}

---@param name string
---@param values table<string, number>|nil the values that differ from the defaults
function Look.define(name, values)
    Gui:defineLook(name)
    for param, value in pairs(values or {}) do
        Gui:setLookParam(name, param, value)
    end
end

UICore.Look = Look

return Look
