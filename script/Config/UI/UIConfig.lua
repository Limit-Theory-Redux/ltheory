Config.ui.general                        = {
    defaultControl                   = "Background",
    controlBarHeight                 = 48,
    hudStyle                         = 1,
    sensorsDisplayed                 = false,
    cursorSmooth                     = "cursor/cursor1-small",
    cursorSimple                     = "cursor/simple_cursor",
    cursor                           = "cursor/simple_cursor",
    cursorStyle                      = 1,
    cursorX                          = 1,
    cursorY                          = 1,

    -- Trackers
    showTrackers                     = true,
    maxTrackingRange                 = 500000,
    trackerBracketingRenderDistances = {
        Planet   = math.huge,
        Asteroid = 25000,
        Jumpgate = 50000,
        Station  = math.huge,
        Ship     = 25000,
        Colony   = 200000,
    },
    trackerObjectOcclusion           = 0.1
}