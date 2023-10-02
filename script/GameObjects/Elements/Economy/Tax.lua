local Entity = require('GameObjects.Entity')

function Entity:addTax(percentage)
    self.taxRate = math.max(percentage, 0)
end

function Entity:getTax()
    return self.taxRate or 0
end

function Entity:hasTax()
    if self.taxRate and self.taxRate > 0 then
        return true
    else
        return false
    end
end

function Entity:removeTax()
    self.taxRate = nil
end
