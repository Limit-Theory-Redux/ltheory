--- HmGui layout: how the Lua components size and align themselves. In every
--- sample the dark band is the parent and the blue box is the component.
--- Run: `./bin/ltr.exe HmGuiLayout` (capture with LTHEORY_CAPTURE).
local Test = require('States.Application')

function Test:getTitle() return 'HmGui Layout' end

local white = Color(1.0, 1.0, 1.0, 1.0)
local dim = Color(0.6, 0.65, 0.7, 1.0)
local bandColor = Color(0.10, 0.11, 0.14, 1.0)
local boxColor = Color(0.14, 0.41, 0.87, 0.85)
local backdrop = Color(0.02, 0.025, 0.04, 1.0)

local BAND_WIDTH, BAND_HEIGHT = 520, 64

local function label(text)
    return UIComponent.Text { text = text, font = "Unageo-Medium", size = 14, color = white }
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
end

function Test:onInput() end

--- A sample row: the caption, then the band (the parent) with the component in it.
function Test:sample(caption, component)
    Gui:beginHorizontalContainer()
    Gui:setSpacing(16)
    Gui:setChildrenVerticalAlignment(AlignVertical.Center)

    Gui:text(caption, self.font, dim)
    Gui:setFixedWidth(260)

    Gui:beginStackContainer()
    Gui:setFixedSize(BAND_WIDTH, BAND_HEIGHT)
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
        self:sample(sample[1], sample[2])
    end
    Gui:endContainer()
end

function Test:buildGui()
    Gui:beginGui(self.resX, self.resY)

    Gui:beginVerticalContainer()
    Gui:setPadding(30, 20)
    Gui:setSpacing(24)

    self:section('UIComponent.Container sizes', self.containerSizes)

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
