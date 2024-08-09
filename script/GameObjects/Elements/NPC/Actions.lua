local Entity = require('GameObjects.Entity')

function Entity:addActions()
    assert(not self.actions)
    self.actions = {}
    self:register(OldEvent.Debug, Entity.debugActions)
    self:register(OldEvent.Update, Entity.updateActions)
end

function Entity:clearActions()
    assert(self.actions)
    for i = #self.actions, 1, -1 do
        self.actions[i]:onStop(self)
    end
    table.clear(self.actions)
end

function Entity:debugActions(state)
    if not self:isDestroyed() then
        local ctx = state.context
        ctx:text("Actions")
        ctx:indent()
        for i, v in ipairs(self.actions) do
            ctx:text("%d : %s", i, v:getName())
        end
        ctx:undent()
    end
end

function Entity:deleteAction(actionName)
    assert(self.actions)
    for i, action in ipairs(self.actions) do
        if action:getName() == actionName then
            action:onStop(self)
            remove(self.actions, i)
            break
        end
    end
end

function Entity:findAction(actionName)
    assert(self.actions)
    local actionRef = nil
    for _, action in ipairs(self.actions) do
        if action:getName() == actionName then
            actionRef = action
            break
        end
    end

    return actionRef
end

function Entity:getCurrentAction()
    assert(self.actions)
    return self.actions[#self.actions]
end

function Entity:hasActions()
    return self.actions ~= nil
end

function Entity:isIdle()
    assert(self.actions)
    return #self.actions == 0
end

function Entity:popAction()
    assert(self.actions)
    assert(#self.actions > 0, "Action stack underflow")
    self.actions[#self.actions]:onStop(self)
    remove(self.actions)
end

function Entity:pushAction(action)
    assert(self.actions)
    assert(#self.actions < 1024, "Action stack overflow")
    insert(self.actions, action)
    action:onStart(self)
end

function Entity:updateActions(state)
    if not GameState.paused then
        if #self.actions == 0 then return end
        Profiler.Begin('Actions.Update')
        for i, v in ipairs(self.actions) do
            v:onUpdatePassive(self, state.dt)
        end

        self.actions[#self.actions]:onUpdateActive(self, state.dt)
        Profiler.End()
    end
end
