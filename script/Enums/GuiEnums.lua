-- Register script defined properties
-- Enums.Gui.TextColor = Gui:registerPropertyVec4("text.color", Vec4(1, 1, 1, 1))

-- Register the IDs of element styles
-- Enums.Gui.Styles.MyButtonStyle = Gui:getStyleId("my_button")

Enums.Gui = {
    Styles = {
        MainMenuContent         = Gui:getStyleId("main-menu-content"),
        DialogButtons           = Gui:getStyleId("dialog-buttons"),
        SeedWindow              = Gui:getStyleId("seed-window"),
        SeedWindowContent       = Gui:getStyleId("seed-window-content"),
        SettingsWindow          = Gui:getStyleId("settings-window"),
        SettingsWindowContent   = Gui:getStyleId("settings-window-content"),
        FlightModeWindow        = Gui:getStyleId("flight-mode-window"),
        FlightModeWindowContent = Gui:getStyleId("flight-mode-window-content"),
    }
}
