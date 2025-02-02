local Socket = Class("Socket", function(self, type, pos, external)
    self.type = type
    self.pos = pos
    self.external = external
    self.child = nil
end)

return Socket
