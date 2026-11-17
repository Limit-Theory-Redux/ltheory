local Container = {}
Container.__index = Container

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponent
---@field render function|nil

---@class UIComponentContainer: UIComponent
---@field visible boolean
---@field align table<AlignHorizontal, AlignVertical>
---@field childrenAlign table<AlignHorizontal, AlignVertical>
---@field width number|string|nil `"fill"` (the default: the parent's width), `"fit"` (as wide as the contents), a number of pixels (fixed), or a percentage of the parent's width such as `"50%"`
---@field height number|string|nil the same for the height
---@field scale number|nil a UI scale for the container and everything in it (Gui:setScale): their sizes, paddings, margins and font sizes are design pixels, this many screen pixels each
---@field widthInLayout number
---@field heightInLayout number
---@field padding { paddingX: number, paddingY: number }|nil
---@field margin { marginX: number, marginY: number }|nil
---@field spacing number
---@field layoutType GuiLayoutType
---@field color UIComponentContainerColors
---@field render fun(self: UIComponentContainer)
---@field contents table
---@field showContainer boolean

---@class UIComponentContainerConstructor
---@field visible boolean|nil
---@field align table<AlignHorizontal, AlignVertical>
---@field childrenAlign table<AlignHorizontal, AlignVertical>
---@field width number|string|nil `"fill"` (the default: the parent's width), `"fit"` (as wide as the contents), a number of pixels (fixed), or a percentage of the parent's width such as `"50%"`
---@field height number|string|nil the same for the height
---@field scale number|nil a UI scale for the container and everything in it (Gui:setScale): their sizes, paddings, margins and font sizes are design pixels, this many screen pixels each
---@field widthInLayout number
---@field heightInLayout number
---@field padding { paddingX: number, paddingY: number }|nil
---@field margin { marginX: number, marginY: number }|nil
---@field spacing number
---@field layoutType GuiLayoutType
---@field color UIComponentContainerColors
---@field contents table
---@field showContainer boolean

---@class UIComponentContainerColors
---@field background Color|nil

---What a `width` or `height` asks for: "fit" (no size set: as big as the
---contents), "fixed" with pixels, or "percent" with a percentage of the parent
---("fill" and nil are 100 %).
---@param size number|string|nil
---@param name string "width" or "height", for the error message
---@return string kind
---@return number|nil value
local function sizeOf(size, name)
    if size == nil or size == "fill" then
        return "percent", 100
    elseif size == "fit" then
        return "fit", nil
    elseif type(size) == "number" then
        return "fixed", size
    end

    local percent = type(size) == "string" and tonumber(size:match("^%s*([%d%.]+)%s*%%%s*$"))
    if not percent then
        error(string.format('UIComponent.Container: %s must be "fill", "fit", a number of pixels or a percentage such as "50%%", not %s',
            name, tostring(size)))
    end
    return "percent", percent
end

---returns a container object
---@param args UIComponentContainerConstructor
---@return UIComponentContainer|nil
function Container:new(args)
    if not args then
        return
    end

    local newContainer = {}
    newContainer.state = UICore.ComponentState {
        visible = args.visible,
        align = args.align or { AlignHorizontal.Default, AlignVertical.Default },
        childrenAlign = args.childrenAlign or { AlignHorizontal.Default, AlignVertical.Default },
        padding = args.padding or { 0, 0 },
        margin = args.margin or { 0, 0 },
        spacing = args.spacing,
        width = args.width,
        height = args.height,
        widthInLayout = args.widthInLayout,
        heightInLayout = args.heightInLayout,
        layoutType = args.layoutType or GuiLayoutType.Horizontal,
        scale = args.scale,
        contents = args.contents,
        color = {
            background = args.color and args.color.background
        },
        showContainer = args.showContainer or function() return GameState.debug.metricsEnabled end,
        showContainerColor = Color((math.random() + math.random(50, 99)) / 100, (math.random() + math.random(50, 99)) / 100, (math.random() + math.random(50, 99)) / 100, .4)
    }

    newContainer.render = function(self)
        if not self.state.visible() then
            return
        end

        Gui:beginContainer(self.state.layoutType())

        if self.state.scale then
            Gui:setScale(self.state.scale()) -- before the children: layers begun in it take it too
        end

        if self.state.color().background then
            Gui:setBackgroundColor(self.state.color().background)
        end

        if self.state.showContainer() then
            Gui:setBorderColor(self.state.showContainerColor())
            Gui:setBorderWidth(1)
        end

        Gui:setAlignment(self.state.align()[1], self.state.align()[2])
        Gui:setChildrenAlignment(self.state.childrenAlign()[1], self.state.childrenAlign()[2])
        Gui:setPadding(self.state.padding()[1], self.state.padding()[2])
        Gui:setMargin(self.state.margin()[1], self.state.margin()[2])

        if self.state.spacing then
            Gui:setSpacing(self.state.spacing())
        end

        local widthKind, width = sizeOf(self.state.width and self.state.width(), "width")
        if widthKind == "fixed" then
            Gui:setFixedWidth(width)
        elseif widthKind == "percent" then
            Gui:setPercentWidth(width)
        end

        local heightKind, height = sizeOf(self.state.height and self.state.height(), "height")
        if heightKind == "fixed" then
            Gui:setFixedHeight(height)
        elseif heightKind == "percent" then
            Gui:setPercentHeight(height)
        end

        if #self.state.contents() > 1 then
            for _, component in ipairs(self.state.contents()) do
                if type(component) == "function" then
                    component = component() -- dynamic components
                end
                component:render()
            end
        elseif #self.state.contents() == 1 then
            local component = self.state.contents()[1]

            if not component then
                return
            end

            if type(component) == "function" then
                component = component() -- dynamic components
            end
            component:render()
        end -- this allows containers without any content

        Gui:endContainer()
    end

    return newContainer
end

setmetatable(Container, meta)

-- Add to global UIComponent table
---@type UIComponentContainerConstructor
UIComponent.Container = Container

return Container
