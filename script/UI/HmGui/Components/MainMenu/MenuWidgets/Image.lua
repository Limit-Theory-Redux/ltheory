local MenuStyle = require("UI.HmGui.Styles.MenuStyle")
local Common = require("UI.HmGui.Components.MainMenu.MenuWidgets.Common")
local widget = Common.widget

---An image at a design position and width (height from the image's aspect).
---@param args { x: number, y: number, w: number, image: Tex2D }
local function Image(args)
    local size = args.image:getSize()
    local h = args.w * size.y / size.x
    return widget(function(self, ctx)
        MenuStyle.beginAt(ctx, args.x, args.y, args.w, h)
        Gui:image(args.image)
        Gui:setFixedSize(args.w * ctx.s, h * ctx.s)
        Gui:endContainer()
    end)
end

-- Add to the global UIComponent table
UIComponent.MenuImage = Image

return Image
