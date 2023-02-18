local self = {
  Escape           = Button.Keyboard.Escape,
  Reload           = Button.Keyboard.F5,
  ProfilerToggle   = Button.Keyboard.F10,
  ToggleFullscreen = Button.Keyboard.F11,
  Screenshot       = Button.Keyboard.F12,
  SystemMap        = Button.Keyboard.Tab,
  TimeAccel        = Button.Keyboard.H,
  ToggleMetrics    = Button.Keyboard.M,
  NewBackground    = Button.Keyboard.B,
  ToggleWireframe  = Button.Keyboard.W, -- does nothing
  ScoreNebulaBad   = Button.Keyboard.Minus, -- does nothing
  ScoreNebulaGood  = Button.Keyboard.Equals, -- does nothing
  Exit             = Button.System.Exit, -- Modifier.Ctrl + Button.W or Modifier.Alt + Button.Q
}

return self
