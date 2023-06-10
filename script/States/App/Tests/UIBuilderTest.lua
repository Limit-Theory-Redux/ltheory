local Test = require('States.Application')
local UIBuilder = require('Systems.Overlay.UIBuilder')
local Text = require('Types.UI.Text')
local Spacer = require('Types.UI.Spacer')
local Button = require('Types.UI.Button')

local rng = RNG.FromTime()

local useRenderer = true

local time = 0
local debugTestShowInS = 2
function Test:testCallback()
    print("Executed Callback")
    Test.callbackTest = true
    time = 0
end

local function testCallback2()
    print("Executed Callback 2")
end

-- remove later
local testContainer = function() return {
    padding = { 10, 10 },
    align = { 0.5, 0.5 },
    group = "X",
    [1] = Text:new {
        font = "Exo2Bold",
        size = 14,
        color = { r = 1, g = 1, b = 1, a = 1 },
        text = "Hello World!"
    },
    [2] = Spacer:new {
        size = 16
    },
    [3] = Text:new {
        font = "Exo2Bold",
        size = 12,
        color = { r = 1, g = .4, b = .4, a = 1 },
        text = "Horizontal!"
    },
    [4] = Button:new {
        title = "A button",
        callback = Test.testCallback
    }
} end

-- remove later
local testContainer2 = function() return {
    align = { 0.5, 0.5 },
    padding = { 10, 10 },
    group = "Y",
    [1] = Text:new {
        font = "Exo2Bold",
        size = 14,
        color = { r = 1, g = 1, b = 1, a = 1 },
        text = "Hello World 2!"
    },
    [2] = Spacer:new {
        size = 16
    },
    [3] = Text:new {
        font = "Exo2Bold",
        size = 12,
        color = { r = 1, g = .4, b = .4, a = 1 },
        text = "Vertical!"
    },
    [4] = Button:new {
        title = "A button",
        callback = testCallback2
    }
} end

-- remove later
local testContainer3 = function() return {
    align = { 0.5, 0.5 },
    padding = { 10, 10 },
    group = "Y",
    [1] = Text:new {
        font = "Exo2Bold",
        size = 14,
        color = { r = 1, g = 1, b = 1, a = 1 },
        text = "Hello World 3!"
    },
    [2] = Spacer:new {
        size = 16
    },
    [3] = Text:new {
        font = "Exo2Bold",
        size = 12,
        color = { r = 1, g = .4, b = .4, a = 1 },
        text = "Vertical!"
    },
    [4] = Button:new {
        title = "A button",
        callback = Test.testCallback
    }
} end

function Test:onInit()
    --* Audio initializations *--
    Audio.Init()
    Audio.Set3DSettings(0.0, 10, 2);
    self.renderer = Renderer()

    self.page = {}
    local testWindow = UIBuilder:buildWindow {
        title = "UI Builder Test",
        group = "X",
        containers = {
            testContainer(),
            testContainer2(),
            testContainer3()
        }
    }

    table.insert(self.page, testWindow)
end

function Test:onInput()
end

function Test:onUpdate(dt)
    if self.callbackTest then
        time = time + dt
    end

    HmGui.Begin(self.resX, self.resY)
    for _, window in ipairs(self.page) do
        window.render()
    end
    HmGui.End()
end

function Test:onDraw()
    if useRenderer then
        self.renderer:start(self.resX, self.resY)
        Viewport.Push(0, 0, self.resX, self.resY, true)
        HmGui.Draw()

        if self.callbackTest then
            UI.DrawEx.TextAdditive(
                'NovaRound',
                'Executed Function',
                20,
                self.resX / 2 - 24, 128, 40, 20,
                1, .4, .4, 1,
                0.5, 0.5
            )

            if time >= debugTestShowInS then
                self.callbackTest = nil
                time = 0
            end
        end

        Viewport.Pop()
        self.renderer:stop()
        self.renderer:present(0, 0, self.resX, self.resY)
    else
        HmGui.Draw()
    end
end

return Test
