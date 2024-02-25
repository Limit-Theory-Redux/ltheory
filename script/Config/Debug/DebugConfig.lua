Config.debug                     = {
    metricsEnabled     = false,
    window             = true, -- Debug window visible by default at launch?
    windowSection      = nil,  -- Set to the name of a debug window section to
    -- collapse all others by default

    instantJobs        = false, -- set to true to speed up economic testing
    jobSpeed           = 10000, -- acceleration rate for instant jobs (in MineAt, DockAt)

    timeAccelFactor    = 10,    -- acceleration rate when holding "TimeAccel" input

    printConfig        = false,

    showMapActionLines = false
}

Config.debug.physics             = {
    drawWireframes         = false,
    drawBoundingBoxesLocal = false,
    drawBoundingBoxesworld = false,
}