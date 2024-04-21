---@meta

---@enum GuiProperties
GuiProperties = {
    ---Config name:
    ---opacity
    ---Type: f32. Default value: 1
    Opacity = 0,
    ---Config name:
    ---background-color
    ---Type: Color. Default value: Color(0.1, 0.12, 0.13, 1.0)
    BackgroundColor = 1,
    ---Config name:
    ---highlight-color
    ---Type: Color. Default value: Color(0.1, 0.5, 1.0, 1.0)
    HighlightColor = 2,
    ---Config name:
    ---text.font
    ---Type: Font. Default value: Font("Rajdhani", 14)
    TextFont = 3,
    ---Config name:
    ---text.color
    ---Type: Color. Default value: White
    TextColor = 4,
    ---Config name:
    ---container.clip
    ---Type: boolean. Default value: true
    ContainerClip = 5,
    ---Config name:
    ---container.spacing
    ---Type: f32. Default value: 6
    ContainerSpacing = 6,
    ---Config name:
    ---container.color-frame
    ---Type: Color. Default value: Color(0.1, 0.1, 0.1, 0.5)
    ContainerColorFrame = 7,
    ---Config name:
    ---container.color-primary
    ---Type: Color. Default value: Color(0.1, 0.5, 1.0, 1.0)
    ContainerColorPrimary = 8,
    ---Config name:
    ---button.border-width
    ---Type: f32. Default value: 0
    ButtonBorderWidth = 9,
    ---Config name:
    ---button.text-color
    ---Type: Color. Default value: Wite. Maps to: TextColor
    ButtonTextColor = 10,
    ---Config name:
    ---button.opacity
    ---Type: f32. Default value: 0.5
    ButtonOpacity = 11,
    ---Config name:
    ---button.background-color
    ---Type: Color. Default value: Color(0.15, 0.15, 0.15, 0.8). Maps to: BackgroundColor
    ButtonBackgroundColor = 12,
    ---Config name:
    ---button.highlight-color
    ---Type: Color. Default value: Color(0.1, 0.5, 1.0, 1.0). Maps to: HighlightColor
    ButtonHighlightColor = 13,
    ---Config name:
    ---checkbox.background-color
    ---Type: Color. Default value: Color(0.3, 0.3, 0.3, 0.5). Maps to: BackgroundColor
    CheckboxBackgroundColor = 14,
    ---Config name:
    ---checkbox.highlight-color
    ---Type: Color. Default value: Color(0.3, 0.3, 0.3, 1.0). Maps to: HighlightColor
    CheckboxHighlightColor = 15,
    ---Config name:
    ---scroll-area.hscroll.show
    ---Type: boolean. Default value: true
    ScrollAreaHScrollShow = 16,
    ---Config name:
    ---scroll-area.vscroll.show
    ---Type: boolean. Default value: true
    ScrollAreaVScrollShow = 17,
    ---Config name:
    ---scroll-area.scrollbar.length
    ---Type: f32. Default value: 4
    ScrollAreaScrollbarLength = 18,
    ---Config name:
    ---scroll-area.scrollbar.background-color
    ---Type: Color. Default value: Color(0.3, 0.3, 0.3, 0.3)
    ScrollAreaScrollbarBackgroundColor = 19,
    ---Config name:
    ---scroll-area.scroll-scale
    ---Type: f32. Default value: 20
    ScrollAreaScrollScale = 20,
    ---Config name:
    ---scroll-area.scrollbar.visibility-stable-time
    ---Type: u64. Default value: 400
    ScrollAreaScrollbarVisibilityStableTime = 21,
    ---Config name:
    ---scroll-area.scrollbar.visibility-fade-time
    ---Type: u64. Default value: 200
    ScrollAreaScrollbarVisibilityFadeTime = 22,
}

