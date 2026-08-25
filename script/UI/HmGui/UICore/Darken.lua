--- Darken — reusable full-screen translucent black overlay for UI menus.
--- Draw it inside a Gui:beginGui/endGui pass, before the contents that
--- should be dimmed, to keep text readable over bright 3D scenes.
local Darken = {}

local DEFAULT_ALPHA = 0.45

--- Draw a full-window translucent black rect. Purely additive to the
--- current UI pass; call it once per frame before the menu's windows.
---@param alpha number|nil Opacity in 0..1 (default 0.45). 0 = no-op.
function Darken:draw(alpha)
    alpha = alpha == nil and DEFAULT_ALPHA or alpha
    if alpha <= 0 then
        return
    end
    Gui:rect()
    Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)
    Gui:setBackgroundColor(Color(0, 0, 0, alpha))
end

return Darken
