local Test = require('States.Application')
local UIBuilder = require('Systems.Overlay.UIBuilder')
local Text = require('Types.UI.Text')
local Spacer = require('Types.UI.Spacer')
local Button = require('Types.UI.Button')

local rng = RNG.FromTime()

local useRenderer = true
local testGroup = rng:choose({ "X", "Y" })

local time = 0
local debugTestShowInS = 1
local function testCallback()
    print("Executed Callback")
    Test.callbackTest = true
    time = 0
end

local function testCallback2()
    print("Executed Callback 2")
    Test.callbackTest = true
    time = 0
end

local function testCallback3()
    print("Executed Callback 3")
    Test.callbackTest = true
    time = 0
end

-- remove later
local testContainer = function ()
    return {
        padding = { 10, 10 },
        align = { 0.5, 0.5 },
        group = testGroup,
        contents = {
            [1] = Text:new { font = "Exo2Bold", size = 14, color = { r = 1, g = 1, b = 1, a = 1 },
                text = "Hello World!" },
            [2] = Spacer:new { size = 16 },
            [3] = Text:new { font = "Exo2Bold", size = 12, color = { r = 1, g = .4, b = .4, a = 1 }, text = "Hey!" },
            [4] = Button:new { title = "Button", callback = testCallback }
        }
    }
end

-- remove later
local testContainer2 = function ()
    return {
        align = { 0.5, 0.5 },
        padding = { 10, 10 },
        group = testGroup,
        contents = {
            [1] = Text:new { font = "Exo2Bold", size = 14, color = { r = 1, g = 1, b = 1, a = 1 },
                text = "Hello World 2!" },
            [2] = Spacer:new { size = 16 },
            [3] = Text:new { font = "Exo2Bold", size = 12, color = { r = 1, g = .4, b = .4, a = 1 }, text = "Hey 2!" },
            [4] = Button:new { title = "Button", callback = testCallback2 }
        }
    }
end

-- remove later
local testContainer3 = function ()
    return {
        align = { 0.5, 0.5 },
        padding = { 10, 10 },
        group = testGroup,
        contents = {
            [1] = Text:new { font = "Exo2Bold", size = 14, color = { r = 1, g = 1, b = 1, a = 1 },
                text = "Hello World 3!" },
            [2] = Spacer:new { size = 16 },
            [3] = Text:new { font = "Exo2Bold", size = 12, color = { r = 1, g = .4, b = .4, a = 1 }, text = "Hey 3!" },
            [4] = Button:new { title = "Button", callback = testCallback3 }
        }
    }
end

local function createWindow()
    testGroup = rng:choose({ "X", "Y" })

    local testWindow = UIBuilder:buildWindow {
        title = "UI Builder Test",
        group = testGroup,
        canClose = true,
        containers = {
            testContainer(),
            testContainer2(),
            testContainer3()
        }
    }

    Test.page[testWindow.guid] = testWindow
end

local createWindowContainer = function ()
    return {
        align = { 0.5, 0.5 },
        padding = { 10, 10 },
        group = testGroup,
        contents = {
            [1] = Button:new { title = "Create Window", callback = createWindow }
        }
    }
end

function Test:onInit()
    self.renderer = Renderer()

    self.page = {}

    local uiBuilderWindow = UIBuilder:buildWindow {
        title = "UI Builder Test Tools",
        group = rng:choose({ "X", "Y" }),
        containers = {
            createWindowContainer()
        }
    }

    Test.page[uiBuilderWindow.guid] = uiBuilderWindow
end

function Test:onInput()
end

function Test:onUpdate(dt)
    if self.callbackTest then
        time = time + dt
    end

    HmGui.Begin(self.resX, self.resY, InputInstance)
    for guid, window in pairs(self.page) do
        if window.close then
            self.page[guid] = nil
            goto skip
        end

        window.render()
        ::skip::
    end
    HmGui.End(InputInstance)
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
