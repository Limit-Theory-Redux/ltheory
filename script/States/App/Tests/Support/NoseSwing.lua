-- Measures how far a flown ship's nose swings out and back across a window of time, for the flight
-- tests: a nose that turns once stays near zero, one that turns away and returns does not.

local NoseSwing = {}

-- A flight's nose is judged over this many seconds around the ship's entry into a well.
NoseSwing.EDGE_WINDOW_S = 3
-- The most the nose may swing out and back across that window, in radians (about 9 degrees): a
-- swing a pilot does not see as a wrong turn.
NoseSwing.MAX_EDGE_SWING = 0.15

---The angle between two vectors in radians, zero for a vector of no length.
---@param a table { x, y, z }
---@param b table { x, y, z }
---@return number
function NoseSwing.angleBetween(a, b)
    local la = math.sqrt(a.x * a.x + a.y * a.y + a.z * a.z)
    local lb = math.sqrt(b.x * b.x + b.y * b.y + b.z * b.z)
    if la == 0 or lb == 0 then return 0 end
    local cosine = (a.x * b.x + a.y * b.y + a.z * b.z) / (la * lb)
    return math.acos(math.max(-1, math.min(1, cosine)))
end

---How far the nose swings out and back across a window of samples: the largest, over the window,
---of the smaller of its angles to the nose at the start and at the end. A nose that turns once,
---from one heading to another, stays near zero; one that turns away and returns is as large as the
---swing.
---@param samples table[] Samples with a `nose`, in time order
---@return number radians
function NoseSwing.excursion(samples)
    local first, last = samples[1].nose, samples[#samples].nose
    local excursion = 0
    for _, sample in ipairs(samples) do
        excursion = math.max(excursion,
            math.min(NoseSwing.angleBetween(sample.nose, first), NoseSwing.angleBetween(sample.nose, last)))
    end
    return excursion
end

---The samples within `window` seconds of `at`.
---@param samples table[] Samples with an `at` (seconds)
---@param at number Seconds
---@param window number Seconds
---@return table[]
function NoseSwing.around(samples, at, window)
    local inside = {}
    for _, sample in ipairs(samples) do
        if math.abs(sample.at - at) <= window then inside[#inside + 1] = sample end
    end
    return inside
end

return NoseSwing
