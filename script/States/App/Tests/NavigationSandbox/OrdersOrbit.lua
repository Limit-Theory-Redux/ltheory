-- Ships flown by an Orbit order, which never ends: each circles a body that is itself on a circle, and
-- is judged on a circle of the radius the scenario names about the body, in the plane the scenario
-- names or, with none, the plane the order chose. Each starts outside the circle and is flown to it
-- by the order's plan first.
--   ship 1: the order names the plane, which is the xz plane;
--   ship 2: the order names none, so the plane is the one the ship is in when it gets to the circle.
-- The circle's radius is the order's stand-off, hundreds of kilometres, so a ship is judged after it
-- has turned a part of a turn: a whole one takes many minutes.
local NavigationSandbox = require("States.App.Tests.NavigationSandbox")

local ORBIT_RADIUS_M = 600000
local PART_OF_A_TURN = 0.3

return NavigationSandbox.ScenarioApp {
    name = "OrdersOrbit",
    timeLimit = 120,
    obstacles = {
        -- 1 and 2: on circles of their own, far from each other
        { radius = 100000, circle = { centre = { 0, 0, 0 }, radius = 2000000, periodS = 8000 } },
        { radius = 100000, circle = { centre = { 0, 0, 6000000 }, radius = 2000000, periodS = 8000 } },
    },
    ships = {
        {
            start = { 2000000, 0, -710000 },
            facing = { 0, 0, 1 },
            behaviour = {
                kind = "Orders",
                orders = {
                    {
                        kind = "Orbit", obstacle = 1, altitude = 6, normal = { 0, 1, 0 },
                        radiusM = ORBIT_RADIUS_M, turnRad = PART_OF_A_TURN,
                    },
                },
            },
        },
        {
            start = { 2000000, 0, 5290000 },
            facing = { 0, 0, 1 },
            behaviour = {
                kind = "Orders",
                orders = {
                    { kind = "Orbit", obstacle = 2, altitude = 6, radiusM = ORBIT_RADIUS_M, turnRad = PART_OF_A_TURN },
                },
            },
        },
    },
}
