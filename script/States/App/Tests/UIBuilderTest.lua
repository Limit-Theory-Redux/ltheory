local Test = require('States.Application')
local UIBuilder = require('UI.UIBuilder')

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
local testContainer = function()
    return {
        padding = { 10, 10 },
        align = { 0.5, 0.5 },
        group = testGroup,
        contents = {
            [1] = UIComponent.Text { font = "Exo2Bold", size = 14, color = { r = 1, g = 1, b = 1, a = 1 },
                text = "Hello World!" },
            [2] = UIComponent.Spacer { size = 16 },
            [3] = UIComponent.Text { font = "Exo2Bold", size = 12, color = { r = 1, g = .4, b = .4, a = 1 }, text = "Hey!" },
            [4] = UIComponent.Button { title = "Button", callback = testCallback }
        }
    }
end

-- remove later
local testContainer2 = function()
    return {
        align = { 0.5, 0.5 },
        padding = { 10, 10 },
        group = testGroup,
        contents = {
            [1] = UIComponent.Text { font = "Exo2Bold", size = 14, color = { r = 1, g = 1, b = 1, a = 1 },
                text = "Hello World 2!" },
            [2] = UIComponent.Spacer { size = 16 },
            [3] = UIComponent.Text { font = "Exo2Bold", size = 12, color = { r = 1, g = .4, b = .4, a = 1 }, text = "Hey 2!" },
            [4] = UIComponent.Button { title = "Button", callback = testCallback2 }
        }
    }
end

-- remove later
local testContainer3 = function()
    return {
        align = { 0.5, 0.5 },
        padding = { 10, 10 },
        group = testGroup,
        contents = {
            [1] = UIComponent.Text { font = "Exo2Bold", size = 14, color = { r = 1, g = 1, b = 1, a = 1 },
                text = "Hello World 3!" },
            [2] = UIComponent.Spacer { size = 16 },
            [3] = UIComponent.Text { font = "Exo2Bold", size = 12, color = { r = 1, g = .4, b = .4, a = 1 }, text = "Hey 3!" },
            [4] = UIComponent.Button { title = "Button", callback = testCallback3 }
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

    UIBuilder:addWindowToPage {
        page = "TestPage",
        window = testWindow
    }
end

local createWindowContainer = function()
    return {
        align = { 0.5, 0.5 },
        padding = { 10, 10 },
        group = testGroup,
        contents = {
            [1] = UIComponent.Button { title = "Create Window", callback = createWindow }
        }
    }
end

function Test:onInit()
    self.renderer = Renderer()

    --Init UIBuilder

    UIBuilder:buildPage {
        name = "TestPage"
    }

    local uiBuilderWindow = UIBuilder:buildWindow {
        title = "UI Builder Test Tools",
        group = rng:choose({ "X", "Y" }),
        containers = {
            createWindowContainer()
        }
    }

    UIBuilder:addWindowToPage {
        page = "TestPage",
        window = uiBuilderWindow
    }

    UIBuilder:setCurrentPage("TestPage")
end

function Test:onInput()
end

function Test:onUpdate(dt)
    if self.callbackTest then
        time = time + dt
    end

    Gui:beginGui(self.resX, self.resY, InputInstance)
    UIBuilder:update()
    Gui:endGui(InputInstance)
end

function Test:onDraw()
    if useRenderer then
        self.renderer:start(self.resX, self.resY)
        Viewport.Push(0, 0, self.resX, self.resY, true)
        Gui:draw()

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
        Gui:draw()
    end
end

return Test
