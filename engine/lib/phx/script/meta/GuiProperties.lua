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
    ---button.opacity
    ---Type: f32. Default value: 0.5
    ButtonOpacity = 10,
    ---Config name:
    ---button.background-color
    ---Type: Color. Default value: Color(0.15, 0.15, 0.15, 0.8). Maps to: BackgroundColor
    ButtonBackgroundColor = 11,
    ---Config name:
    ---button.highlight-color
    ---Type: Color. Default value: Color(0.1, 0.5, 1.0, 1.0). Maps to: HighlightColor
    ButtonHighlightColor = 12,
    ---Config name:
    ---checkbox.background-color
    ---Type: Color. Default value: Color(0.3, 0.3, 0.3, 0.5). Maps to: BackgroundColor
    CheckboxBackgroundColor = 13,
    ---Config name:
    ---checkbox.highlight-color
    ---Type: Color. Default value: Color(0.3, 0.3, 0.3, 1.0). Maps to: HighlightColor
    CheckboxHighlightColor = 14,
    ---Config name:
    ---checkbox.color-frame
    ---Type: Color. Default value: Color(0.1, 0.1, 0.1, 0.5)
    CheckboxColorFrame = 15,
    ---Config name:
    ---checkbox.color-primary
    ---Type: Color. Default value: Color(0.1, 0.5, 1.0, 1.0)
    CheckboxColorPrimary = 16,
    ---Config name:
    ---scroll-area.hscroll.show
    ---Type: boolean. Default value: true
    ScrollAreaHScrollShow = 17,
    ---Config name:
    ---scroll-area.vscroll.show
    ---Type: boolean. Default value: true
    ScrollAreaVScrollShow = 18,
    ---Config name:
    ---scroll-area.scrollbar.length
    ---Type: f32. Default value: 4
    ScrollAreaScrollbarLength = 19,
    ---Config name:
    ---scroll-area.scrollbar.background-color
    ---Type: Color. Default value: Color(0.3, 0.3, 0.3, 0.3)
    ScrollAreaScrollbarBackgroundColor = 20,
    ---Config name:
    ---scroll-area.scroll-scale
    ---Type: f32. Default value: 20
    ScrollAreaScrollScale = 21,
    ---Config name:
    ---scroll-area.scrollbar.visibility-stable-time
    ---Type: u64. Default value: 400
    ScrollAreaScrollbarVisibilityStableTime = 22,
    ---Config name:
    ---scroll-area.scrollbar.visibility-fade-time
    ---Type: u64. Default value: 200
    ScrollAreaScrollbarVisibilityFadeTime = 23,
}

