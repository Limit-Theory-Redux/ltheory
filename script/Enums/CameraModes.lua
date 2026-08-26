Enums.CameraMode = {
    FirstPerson = 1,  -- Cockpit view, locked to ship rotation
    Chase       = 2,  -- Close orbit behind ship
    Orbit       = 3,  -- Free orbit around ship
    Free        = 4,  -- Detached free-fly camera
    RTS         = 5,  -- Top-down strategy view
}

Enums.CameraModeNames = {
    [1] = "FirstPerson",
    [2] = "Chase",
    [3] = "Orbit",
    [4] = "Free",
    [5] = "RTS",
}

Enums.CameraModeCount = 5
