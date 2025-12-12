local Control = {}

---@param key integer
---@return table
function Control.Key(key)
    return { type = "Key", key = key }
end

---@param ... integer|table 
---@return table
function Control.Combo(...)
    return { type = "Combo", keys = { ... } }
end

---@param positive integer
---@param negative integer
---@return table
function Control.Pair(positive, negative)
    return { type = "Pair", positive = positive, negative = negative }
end

---@param button integer
---@return table
function Control.Axis(button)
    local bind = {
        type = "Axis",
        button = button,
        mult = 1.0,
        bias = 0.0,
        expn = 1.0,
    }
    
    function bind:invert()
        self.mult = -self.mult
        return self
    end
    
    function bind:setMult(value)
        self.mult = value
        return self
    end
    
    function bind:setBias(value)
        self.bias = value
        return self
    end
    
    function bind:setExpn(value)
        self.expn = value
        return self
    end
    
    return bind
end

---@return table
function Control.MouseX()
    local bind = { type = "MouseX", mult = 1.0 }
    function bind:invert() self.mult = -self.mult; return self end
    function bind:setMult(value) self.mult = value; return self end
    return bind
end

---@return table
function Control.MouseY()
    local bind = { type = "MouseY", mult = 1.0 }
    function bind:invert() self.mult = -self.mult; return self end
    function bind:setMult(value) self.mult = value; return self end
    return bind
end

---@return table
function Control.MouseDX()
    local bind = { type = "MouseDX", mult = 1.0 }
    function bind:invert() self.mult = -self.mult; return self end
    function bind:setMult(value) self.mult = value; return self end
    return bind
end

---@return table
function Control.MouseDY()
    local bind = { type = "MouseDY", mult = 1.0 }
    function bind:invert() self.mult = -self.mult; return self end
    function bind:setMult(value) self.mult = value; return self end
    return bind
end

---@return table
function Control.MouseWheel()
    local bind = { type = "MouseWheel", mult = 1.0 }
    function bind:invert() self.mult = -self.mult; return self end
    function bind:setMult(value) self.mult = value; return self end
    return bind
end

return Control