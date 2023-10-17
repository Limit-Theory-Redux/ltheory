local Entity = require('GameObjects.Entity')

function Entity:addVisibleMesh(mesh, material)
    assert(not self.mesh)
    assert(mesh)
    assert(material)
    self.mesh = mesh
    self.material = material
    self:setRenderVisibleMesh(true)
    self:register(Event.Update, Entity.handleMeshCulling)
end

function Entity:setRenderVisibleMesh(enabled, cullingLock)
    self.cullingLock = cullingLock
    if enabled and not self.visibleMesh then
        self:register(Event.Render, Entity.renderVisibleMesh)

        -- also show children
        if self.children and #self.children > 0 then
            for _, l_Child in ipairs(self.children) do
                if not l_Child.visibleMesh then
                    l_Child:register(Event.Render, l_Child.renderVisibleMesh)
                    l_Child.visibleMesh = true
                end
            end
        end
        self.visibleMesh = true
    elseif not enabled and self.visibleMesh then
        self:unregister(Event.Render, Entity.renderVisibleMesh)

        -- also hide children
        if self.children and #self.children > 0 then
            for _, l_Child in ipairs(self.children) do
                if l_Child.visibleMesh then
                    l_Child:unregister(Event.Render, l_Child.renderVisibleMesh)
                    l_Child.visibleMesh = false
                end
            end
        end
        self.visibleMesh = false
    end
end

function Entity:handleMeshCulling()
    -- culling
    if GameState:GetCurrentState() == Enums.GameStates.InGame and GameState.player.currentShip then
        local objectType = Config:getObjectInfo("object_types", self:getType())
        local distanceToEntity = self:getPos():distance(GameState.player.currentShip:getPos())

        -- Cull Entities (temp: without subtypes until that works properly)
        if objectType then
            --Log.Debug("%s: %s, %s", self.visibleMesh, distanceToEntity, self:getName())
            local renderDistance = GameState.render.renderDistances[objectType]

            if renderDistance then
                if not self.visibleMesh and not self.cullingLock and distanceToEntity < renderDistance then
                    self:setRenderVisibleMesh(true)
                elseif self.visibleMesh and not self.cullingLock and distanceToEntity > renderDistance then
                    self:setRenderVisibleMesh(false)
                end
            end
        end
    end
end

function Entity:renderVisibleMesh(state)
    if state.mode == BlendMode.Disabled then
        --Log.Debug("Entity:renderVisibleMesh() - self = %s", self:getName())
        self.material:start()
        self.material:setState(self)
        self.mesh:draw()
        self.material:stop()
        self.visibleMesh = true
    end
end
