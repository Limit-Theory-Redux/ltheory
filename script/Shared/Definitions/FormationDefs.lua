---The shapes ships fly in. A shape gives the slot of the n-th wingman (the leader is slot 0, at the
---origin) as (right, up, forward) in spacings, along the leader's own axes; `FormationSystem` turns
---spacings into metres by the size of the wing's ships. Shapes are data: a new one is an entry here.

---@class FormationShape
---@field id string
---@field name string
---@field slot fun(n: integer): number, number, number Right, up, forward, in spacings

---@param n integer
---@return integer rank, integer side The n-th wingman's distance from the leader's line, and its side (-1 left, 1 right)
local function alternate(n)
    return math.ceil(n / 2), (n % 2 == 1) and 1 or -1
end

---@type table<string, FormationShape>
local Formations = {
    Wedge = {
        id = "Wedge", name = "Wedge",
        slot = function(n)
            local rank, side = alternate(n)
            return side * rank, 0, -rank
        end,
    },
    LineAbreast = {
        id = "LineAbreast", name = "Line abreast",
        slot = function(n)
            local rank, side = alternate(n)
            return side * rank, 0, 0
        end,
    },
    Column = {
        id = "Column", name = "Column",
        slot = function(n) return 0, 0, -n end,
    },
    EchelonRight = {
        id = "EchelonRight", name = "Echelon right",
        slot = function(n) return n, 0, -n end,
    },
    Diamond = {
        id = "Diamond", name = "Diamond",
        slot = function(n)
            if n == 1 then return 1, 0, -1 elseif n == 2 then return -1, 0, -1 elseif n == 3 then return 0, 0, -2 end
            -- Beyond the diamond: a wedge behind it.
            local rank, side = alternate(n - 3)
            return side * rank, 0, -2 - rank
        end,
    },
}

return Formations
