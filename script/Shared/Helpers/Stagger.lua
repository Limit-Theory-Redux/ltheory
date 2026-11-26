-- The share of a period that timers started together are put off by, from an id, so the work of many
-- of them spreads over frames instead of landing in one. Design: doc/script/nav/navigation.md

---@class Stagger
local Stagger = {}

-- The golden ratio conjugate spreads consecutive ids evenly over the period, however many there are.
local STEP = 0.6180339887498949

---@param id integer
---@return number In [0, 1)
function Stagger.of(id)
    return (id * STEP) % 1
end

return Stagger
