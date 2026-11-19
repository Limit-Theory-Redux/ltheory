---@alias NavRateClassId integer An `Enums.NavRateClass` value

-- How often a ship's avoidance and plan refresh run, from the most to the least: `High` ships near
-- stations and in dense traffic, `Normal` the rest, `Low` ships with nothing near. The periods of each
-- class are `NavConfig.rateClasses`. A lower value is a higher rate.
---@enum NavRateClass
Enums.NavRateClass = {
    High   = 1,
    Normal = 2,
    Low    = 3,
}
