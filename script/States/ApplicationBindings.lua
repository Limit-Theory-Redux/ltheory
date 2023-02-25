local self = {
  Escape           = Button.Keyboard.Escape,
  Reload           = Button.Keyboard.F5,
  ProfilerToggle   = Button.Keyboard.F10,
  ToggleFullscreen = Button.Keyboard.F11,
  Screenshot       = Button.Keyboard.F12,
  SystemMap        = Button.Keyboard.Tab,
  NewBackground    = Button.Keyboard.B,
  TimeAccel        = Button.Keyboard.H,
  ToggleMetrics    = Button.Keyboard.K,
  MoveTo           = Button.Keyboard.M,
  ToggleHUD        = Button.Keyboard.V,
  ToggleWireframe  = Button.Keyboard.W, -- does nothing
  ScoreNebulaBad   = Button.Keyboard.Minus, -- does nothing
  ScoreNebulaGood  = Button.Keyboard.Equals, -- does nothing
  Exit             = Button.System.Exit, -- Modifier.Ctrl + Button.W or Modifier.Alt + Button.Q
}

return self
