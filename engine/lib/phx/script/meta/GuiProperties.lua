---@meta

---@enum GuiProperties
GuiProperties = {
    ---Config name:
    ---opacity
    ---Type: f32. Default value: 0
    Opacity = 0,
    ---Config name:
    ---border-color
    ---Type: Color. Default value: Color.TRANSPARENT
    BorderColor = 1,
    ---Config name:
    ---background-color
    ---Type: Color. Default value: Color.TRANSPARENT
    BackgroundColor = 2,
    ---Config name:
    ---highlight-color
    ---Type: Color. Default value: Color.TRANSPARENT
    HighlightColor = 3,
    ---Config name:
    ---text.font
    ---Type: Font. Default value: Font("Rajdhani", 14)
    TextFont = 4,
    ---Config name:
    ---text.color
    ---Type: Color. Default value: White
    TextColor = 5,
    ---Config name:
    ---container.clip
    ---Type: boolean. Default value: true
    ContainerClip = 6,
    ---Config name:
    ---container.spacing
    ---Type: f32. Default value: 6
    ContainerSpacing = 7,
    ---Config name:
    ---container.color-frame
    ---Type: Color. Default value: Color(0.1, 0.1, 0.1, 0.5)
    ContainerColorFrame = 8,
    ---Config name:
    ---container.color-primary
    ---Type: Color. Default value: Color(0.1, 0.5, 1.0, 1.0)
    ContainerColorPrimary = 9,
    ---Config name:
    ---button.rect.opacity
    ---Type: f32. Default value: 0.5
    ButtonRectOpacity = 10,
    ---Config name:
    ---button.rect.border-color
    ---Type: Color. Default value: Color.TRANSPARENT. Maps to: BorderColor
    ButtonRectBorderColor = 11,
    ---Config name:
    ---button.rect.background-color
    ---Type: Color. Default value: Color(0.15, 0.15, 0.15, 0.8). Maps to: BackgroundColor
    ButtonRectBackgroundColor = 12,
    ---Config name:
    ---button.rect.highlight-color
    ---Type: Color. Default value: Color(0.1, 0.5, 1.0, 1.0). Maps to: HighlightColor
    ButtonRectHighlightColor = 13,
    ---Config name:
    ---button.text.opacity
    ---Type: f32. Default value: 0.5
    ButtonTextOpacity = 14,
    ---Config name:
    ---button.text.background-color
    ---Type: Color. Default value: Color::TRANSPARENT. Maps to: BackgroundColor
    ButtonTextBackgroundColor = 15,
    ---Config name:
    ---button.text.highlight-color
    ---Type: Color. Default value: Color::TRANSPARENT. Maps to: HighlightColor
    ButtonTextHighlightColor = 16,
    ---Config name:
    ---button.text.font
    ---Type: Font. Default value: Font("Rajdhani", 14). Maps to: TextFont
    ButtonTextFont = 17,
    ---Config name:
    ---button.text.color
    ---Type: Color. Default value: White. Maps to: TextColor
    ButtonTextColor = 18,
    ---Config name:
    ---checkbox.rect.opacity
    ---Type: f32. Default value: 0.5
    CheckboxRectOpacity = 19,
    ---Config name:
    ---checkbox.rect.border-color
    ---Type: Color. Default value: Color.TRANSPARENT. Maps to: BorderColor
    CheckboxRectBorderColor = 20,
    ---Config name:
    ---checkbox.rect.background-color
    ---Type: Color. Default value: Color(0.3, 0.3, 0.3, 0.5). Maps to: BackgroundColor
    CheckboxRectBackgroundColor = 21,
    ---Config name:
    ---checkbox.rect.highlight-color
    ---Type: Color. Default value: Color(0.3, 0.3, 0.3, 1.0). Maps to: HighlightColor
    CheckboxRectHighlightColor = 22,
    ---Config name:
    ---checkbox.text.opacity
    ---Type: f32. Default value: 0.5
    CheckboxTextOpacity = 23,
    ---Config name:
    ---checkbox.text.background-color
    ---Type: Color. Default value: Color::TRANSPARENT. Maps to: BackgroundColor
    CheckboxTextBackgroundColor = 24,
    ---Config name:
    ---checkbox.text.highlight-color
    ---Type: Color. Default value: Color::TRANSPARENT. Maps to: HighlightColor
    CheckboxTextHighlightColor = 25,
    ---Config name:
    ---checkbox.text.font
    ---Type: Font. Default value: Font("Rajdhani", 14). Maps to: TextFont
    CheckboxTextFont = 26,
    ---Config name:
    ---checkbox.text.color
    ---Type: Color. Default value: White. Maps to: TextColor
    CheckboxTextColor = 27,
    ---Config name:
    ---checkbox.click-area.border-color
    ---Type: Color. Default value: Color(0.1, 0.1, 0.1, 0.5). Maps to: BorderColor
    CheckboxClickAreaBorderColor = 28,
    ---Config name:
    ---checkbox.click-area.background-color
    ---Type: Color. Default value: Color::TRANSPARENT. Maps to: BackgroundColor
    CheckboxClickAreaBackgroundColor = 29,
    ---Config name:
    ---checkbox.click-area.highlight-color
    ---Type: Color. Default value: Color::TRANSPARENT. Maps to: HighlightColor
    CheckboxClickAreaHighlightColor = 30,
    ---Config name:
    ---checkbox.click-area.selected-color
    ---Type: Color. Default value: Color(0.1, 0.5, 1.0, 1.0)
    CheckboxClickAreaSelectedColor = 31,
    ---Config name:
    ---scroll-area.hscroll.show
    ---Type: boolean. Default value: true
    ScrollAreaHScrollShow = 32,
    ---Config name:
    ---scroll-area.vscroll.show
    ---Type: boolean. Default value: true
    ScrollAreaVScrollShow = 33,
    ---Config name:
    ---scroll-area.scrollbar.length
    ---Type: f32. Default value: 4
    ScrollAreaScrollbarLength = 34,
    ---Config name:
    ---scroll-area.scrollbar.background-color
    ---Type: Color. Default value: Color(0.3, 0.3, 0.3, 0.3)
    ScrollAreaScrollbarBackgroundColor = 35,
    ---Config name:
    ---scroll-area.scroll-scale
    ---Type: f32. Default value: 20
    ScrollAreaScrollScale = 36,
    ---Config name:
    ---scroll-area.scrollbar.visibility-stable-time
    ---Type: u64. Default value: 400
    ScrollAreaScrollbarVisibilityStableTime = 37,
    ---Config name:
    ---scroll-area.scrollbar.visibility-fade-time
    ---Type: u64. Default value: 200
    ScrollAreaScrollbarVisibilityFadeTime = 38,
}

