local Test = require('States.Application')

local Todo = class(function(self, name, done)
    self.name = name
    self.children = {}
    self.done = done or false
end)

function Todo:add(child)
    insert(self.children, child)
    return self
end

function Todo:show()
    self.done = Gui:checkbox(self.name, self.done)
    if #self.children > 0 then
        Gui:beginVerticalContainer()
        Gui:setPaddingLeft(12)
        for i = 1, #self.children do
            self.children[i]:show()
        end
        Gui:endContainer()
    end
end

local todo = Todo("Limit Theory")
    :add(Todo("Audio"))
    :add(Todo("Game Mechanics")
        :add(Todo("Combat")
            :add(Todo("Consumable Munitions"))
            :add(Todo("Energy Grid"))
            :add(Todo("Firing Groups"))
        )
        :add(Todo("Command"))
        :add(Todo("Construction"))
        :add(Todo("Diplomacy"))
        :add(Todo("Exploration"))
        :add(Todo("Extraction"))
        :add(Todo("Freelancing"))
        :add(Todo("Management"))
        :add(Todo("Manufacturing"))
        :add(Todo("Navigation"))
        :add(Todo("Research"))
        :add(Todo("Trade"))
    )
    :add(Todo("Game Objects"))
    :add(Todo("Game Simulation"))
    :add(Todo("Graphics"))
    :add(Todo("User Interface"))

function Test:onInit()
    self.bg = Tex2D.Load('./screenshot/wp2.png')
    self.renderer = Renderer()
end

function Test:showTodo()
    Gui:beginWindow("HmGui Todo List", InputInstance)
    Gui:beginScroll(512)
    todo:show()
    Gui:endScroll(InputInstance)
    Gui:endWindow()
    Gui:setAlign(0.5, 0.5)
end

function Test:onUpdate(dt)
    Gui:beginGui(self.resX, self.resY, InputInstance)
    Gui:image(self.bg)
    self:showTodo()
    Gui:endGui(InputInstance)
end

function Test:onDraw()
    self.renderer:start(self.resX, self.resY)
    self.renderer:stop()
    self.renderer:startUI()
    Viewport.Push(0, 0, self.resX, self.resY, true)
    Gui:draw()
    Viewport.Pop()
    self.renderer:stopUI()
    self.renderer:present(0, 0, self.resX, self.resY)
end

return Test
