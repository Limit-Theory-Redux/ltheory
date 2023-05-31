local id = 1

local Entity = class(function (self)
  self.id = id
  self.handlers = {}
  self.visibleMesh = nil
  id = id + 1
end)

function Entity:delete ()
  self.deleted = true
end

function Entity:register (eventType, handler)
  if not self.handlers[eventType] then self.handlers[eventType] = {} end
  insert(self.handlers[eventType], handler)
--if eventType == Event.Debug then
--printf("Entity:register() - '%s' eventType = %s, handler = %s", self:getName(), eventType, handler)
--end
end

function Entity:send (event)
  if self.handlers[event.type] then
    for i, v in ipairs(self.handlers[event.type]) do
--if event.type == Event.Debug then
--printf("Entity:send() - '%s' eventType = %s, context = %s", self:getName(), event.type, event.context)
--end
      v(self, event)
    end
  end

  -- Respond to the contents of a broadcasted message if applicable
  if event.type == Event.Broadcast then
    self:send(event.event)
  end
end

function Entity:unregister (eventType, handler)
  assert(self.handlers[eventType])
  for i, v in ipairs(self.handlers[eventType]) do
    if v == handler then
      remove(self.handlers[eventType], i)
      break
    end
  end
end

return Entity
