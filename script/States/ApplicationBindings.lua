local self = {
  Escape           = Button.Keyboard.Escape,
  Reload           = Button.Keyboard.F5,
  ToggleWireframe  = Button.Keyboard.Tab,
  ProfilerToggle   = Button.Keyboard.F10,
  ToggleFullscreen = Button.Keyboard.F11,
  Screenshot       = Button.Keyboard.F12,
  TimeAccel        = Button.Keyboard.H,
  ToggleMetrics    = Button.Keyboard.M,
  NewBackground    = Button.Keyboard.B,
  Exit             = Button.System.Exit, -- Modifier.Ctrl + Button.W or Modifier.Alt + Button.Q
  ScoreNebulaBad   = Button.Keyboard.Minus,
  ScoreNebulaGood  = Button.Keyboard.Equals,
}

return self
