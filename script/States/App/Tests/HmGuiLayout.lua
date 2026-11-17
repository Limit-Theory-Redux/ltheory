--- HmGui layout: how the Lua components size and align themselves. In every
--- sample the dark band is the parent and the blue box is the component.
--- Run: `./bin/ltr.exe HmGuiLayout` (capture with LTHEORY_CAPTURE).
local Test = require('States.Application')

function Test:getTitle() return 'HmGui Layout' end

local white = Color(1.0, 1.0, 1.0, 1.0)
local dim = Color(0.6, 0.65, 0.7, 1.0)
local bandColor = Color(0.10, 0.11, 0.14, 1.0)
local boxColor = Color(0.14, 0.41, 0.87, 0.85)
local boxColor2 = Color(0.30, 0.55, 0.95, 0.85)
local backdrop = Color(0.02, 0.025, 0.04, 1.0)

local BAND_WIDTH, BAND_HEIGHT = 520, 64

local function label(text, size)
    return UIComponent.Text { text = text, font = "Unageo-Medium", size = size or 14, color = white }
end

--- A Container sample: a blue box with its options as the label.
local function box(text, args)
    args.color = { background = boxColor }
    args.childrenAlign = { AlignHorizontal.Center, AlignVertical.Center }
    args.padding = { 10, 4 }
    args.contents = { label(text) }
    return UIComponent.Container(args)
end

function Test:onInit()
    self.font = Cache.Font('Unageo-Medium', 14)
    self.titleFont = Cache.Font('Exo2Bold', 22)

    self.containerSizes = {
        { 'width, height: "fill" (default)', box('fill', {}) },
        { 'width, height: "fit"', box('fit', { width = "fit", height = "fit" }) },
        { 'width = 240, height = 40 (pixels)', box('240 x 40', { width = 240, height = 40 }) },
        { 'width = "50%", height = "fit"', box('50 %', { width = "50%", height = "fit" }) },
        { 'width = "fit", height = "75%"', box('fit x 75 %', { width = "fit", height = "75%" }) },
    }

    -- Each band aligns its children left, so a component that follows its own
    -- `align` sits right; `textAlign` only places the title.
    local right = { AlignHorizontal.Right, AlignVertical.Center }
    local function noop() end
    self.componentAlign = {
        { 'Slider, align = Right', UIComponent.Slider {
            title = "Volume", size = Vec2f(220, 20), align = right, callback = noop } },
        { 'Switch, align = Right', UIComponent.Switch {
            title = "Bloom", size = Vec2f(50, 22), currentValue = true, align = right, callback = noop } },
        { 'ProgressBar, align = Right', UIComponent.ProgressBar {
            title = "Hull", size = Vec2f(220, 20), currentValue = 70, align = right } },
        { 'Slider, default align', UIComponent.Slider {
            title = "Volume", size = Vec2f(220, 20), callback = noop } },
    }

    -- Percent-sized children of a container that sizes to its content: it
    -- makes room for each child's content at its percentage, so equal shares
    -- stay equal and nothing is cut.
    local function share(text, size, color, width, height)
        return UIComponent.Container {
            width = width, height = height, color = { background = color },
            childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
            padding = { 8, 0 }, contents = { label(text, size) },
        }
    end
    self.percentShares = {
        { '3 rows of "33.333%", column "fit"', UIComponent.Container {
            width = 200, height = "fit", layoutType = GuiLayoutType.Vertical, contents = {
                share("small", 14, boxColor, "fill", "33.333%"),
                share("Large", 30, boxColor2, "fill", "33.333%"),
                share("medium", 20, boxColor, "fill", "33.333%"),
            } }, 150 },
        { '2 cells of "50%", row "fit"', UIComponent.Container {
            width = "fit", height = "fit", contents = {
                share("Hull", 14, boxColor, "50%", "fill"),
                share("Shield generator", 14, boxColor2, "50%", "fill"),
            } } },
    }

    -- Gui:text: line breaks and wrapping at a max width, lines placed by textAlign.
    local paragraph = "A text that is too long for one line breaks between its words, "
        .. "at the width it is given. A short line."
    local function wrapped(align)
        return UIComponent.Text { text = paragraph, font = "Unageo-Medium", size = 14, color = white,
            wrap = 300, textAlign = align }
    end
    self.textWrap = {
        { 'text with "\\n"', UIComponent.Text { text = "First line\nThe second, longer line\nThird",
            font = "Unageo-Medium", size = 14, color = white, textAlign = AlignHorizontal.Left }, 80 },
        { 'wrap = 300, textAlign = Left', wrapped(AlignHorizontal.Left), 84 },
        { 'wrap = 300 (centred)', wrapped(nil), 84 },
        { 'wrap = 300, textAlign = Right', wrapped(AlignHorizontal.Right), 84 },
    }
end

function Test:onInput() end

--- A sample row: the caption, then the band (the parent) with the component in it.
function Test:sample(caption, component, bandHeight)
    Gui:beginHorizontalContainer()
    Gui:setSpacing(16)
    Gui:setChildrenVerticalAlignment(AlignVertical.Center)

    Gui:text(caption, self.font, dim)
    Gui:setFixedWidth(260)

    Gui:beginStackContainer()
    Gui:setFixedSize(BAND_WIDTH, bandHeight or BAND_HEIGHT)
    Gui:setBackgroundColor(bandColor)
    Gui:setChildrenAlignment(AlignHorizontal.Left, AlignVertical.Center)
    component:render()
    Gui:endContainer()

    Gui:endContainer()
end

function Test:section(title, samples)
    Gui:beginVerticalContainer()
    Gui:setSpacing(10)
    Gui:text(title, self.titleFont, white)
    for _, sample in ipairs(samples) do
        self:sample(sample[1], sample[2], sample[3])
    end
    Gui:endContainer()
end

--- One small panel built with the same design pixels at different UI scales:
--- its size, padding, spacing, border, text and shape all follow the scale.
function Test:scaledPanel(scale, label)
    Gui:beginVerticalContainer()
    Gui:setScale(scale)
    Gui:setPadding(10, 8)
    Gui:setSpacing(6)
    Gui:setBackgroundColor(bandColor)
    Gui:setBorderWidth(1)
    Gui:setBorderColor(dim)
    Gui:setChildrenHorizontalAlignment(AlignHorizontal.Center)

    Gui:text(label or string.format('scale %g', scale), self.font, white)
    Gui:annulus(boxColor2, 6, false)
    Gui:setFixedSize(50, 50)
    Gui:rect()
    Gui:setFixedSize(80, 10)
    Gui:setBackgroundColor(boxColor)

    Gui:endContainer()
end

function Test:scaleSection()
    Gui:beginVerticalContainer()
    Gui:setSpacing(10)
    Gui:text('Gui:setScale (UIComponent.Container scale)', self.titleFont, white)

    Gui:beginHorizontalContainer()
    Gui:setSpacing(20)
    Gui:setChildrenVerticalAlignment(AlignVertical.Top)
    for _, scale in ipairs({ 0.75, 1, 1.5, 2 }) do
        self:scaledPanel(scale)
    end
    -- nested: a 1.5 panel inside a scale-2 container draws at 3
    Gui:beginStackContainer()
    Gui:setScale(2)
    self:scaledPanel(1.5, '1.5 in 2 = 3')
    Gui:endContainer()
    Gui:endContainer()

    Gui:endContainer()
end

function Test:buildGui()
    Gui:beginGui(self.resX, self.resY)

    Gui:beginHorizontalContainer()
    Gui:setPadding(30, 20)
    Gui:setSpacing(60)

    Gui:beginVerticalContainer()
    Gui:setSpacing(24)
    self:section('UIComponent.Container sizes', self.containerSizes)
    self:section('Components place themselves by align', self.componentAlign)
    Gui:endContainer()

    Gui:beginVerticalContainer()
    Gui:setSpacing(24)
    self:section('Percent sizes in a parent that fits its content', self.percentShares)
    self:scaleSection()
    self:section('Gui:text line breaks and wrapping', self.textWrap)
    Gui:endContainer()

    Gui:endContainer()

    Gui:endGui()
end

-- Built and drawn from the render event (`Application` never calls `onUpdate`).
function Test:onRender()
    self:buildGui()
    self:immediateUI(function()
        Imm.Rect(0, 0, self.resX, self.resY, backdrop)
        Gui:draw()
    end)
end

return Test
