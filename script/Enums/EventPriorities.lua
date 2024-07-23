---@enum EventPriority
Enums.EventPriority = {
    ["Lowest"] = -2147483648, -- i32:MIN
    ["VeryLow"] = -1000000000,
    ["Low"] = -500000000,
    ["BelowDefault"] = -100000000,
    ["Default"] = 0,
    ["AboveDefault"] = 100000000,
    ["High"] = 500000000,
    ["VeryHigh"] = 1000000000,
    ["Highest"] = 2147483647, -- i32:MAX
}
