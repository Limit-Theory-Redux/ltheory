---@meta

---Core properties
---@enum GuiProperties
GuiProperties = {
    ---Config name: opacity
    ---Default value: 1.0f32
    Opacity = 0,
    ---Config name: border-color
    ---Default value: Color :: TRANSPARENT
    BorderColor = 1,
    ---Config name: background-color
    ---Default value: Color :: TRANSPARENT
    BackgroundColor = 2,
    ---Config name: highlight-color
    ---Default value: Color :: TRANSPARENT
    HighlightColor = 3,
    ---Config name: text.font
    ---Default value: Font :: load("Rajdhani", 14)
    TextFont = 4,
    ---Config name: text.color
    ---Default value: Color :: WHITE
    TextColor = 5,
    ---Config name: container.clip
    ---Default value: true
    ContainerClip = 6,
    ---Config name: container.spacing
    ---Default value: 6.0f32
    ContainerSpacing = 7,
    ---Config name: container.color-frame
    ---Default value: Color :: new(0.1, 0.1, 0.1, 0.5)
    ContainerColorFrame = 8,
    ---Config name: container.color-primary
    ---Default value: Color :: new(0.1, 0.5, 1.0, 1.0)
    ContainerColorPrimary = 9,
    ---Config name: button.rect.opacity
    ---Default value: 0.5f32
    ---Maps to: Opacity
    ButtonRectOpacity = 10,
    ---Config name: button.rect.border-color
    ---Default value: Color :: TRANSPARENT
    ---Maps to: BorderColor
    ButtonRectBorderColor = 11,
    ---Config name: button.rect.background-color
    ---Default value: Color :: new(0.15, 0.15, 0.15, 0.8)
    ---Maps to: BackgroundColor
    ButtonRectBackgroundColor = 12,
    ---Config name: button.rect.highlight-color
    ---Default value: Color :: new(0.1, 0.5, 1.0, 1.0)
    ---Maps to: HighlightColor
    ButtonRectHighlightColor = 13,
    ---Config name: button.text.opacity
    ---Default value: 0.5f32
    ---Maps to: Opacity
    ButtonTextOpacity = 14,
    ---Config name: button.text.background-color
    ---Default value: Color :: TRANSPARENT
    ---Maps to: BackgroundColor
    ButtonTextBackgroundColor = 15,
    ---Config name: button.text.highlight-color
    ---Default value: Color :: TRANSPARENT
    ---Maps to: HighlightColor
    ButtonTextHighlightColor = 16,
    ---Config name: button.text.font
    ---Default value: Font :: load("Rajdhani", 14)
    ---Maps to: TextFont
    ButtonTextFont = 17,
    ---Config name: button.text.color
    ---Default value: Color :: WHITE
    ---Maps to: TextColor
    ButtonTextColor = 18,
    ---Config name: checkbox.rect.opacity
    ---Default value: 0.5f32
    ---Maps to: Opacity
    CheckboxRectOpacity = 19,
    ---Config name: checkbox.rect.border-color
    ---Default value: Color :: TRANSPARENT
    ---Maps to: BorderColor
    CheckboxRectBorderColor = 20,
    ---Config name: checkbox.rect.background-color
    ---Default value: Color :: new(0.3, 0.3, 0.3, 0.5)
    ---Maps to: BackgroundColor
    CheckboxRectBackgroundColor = 21,
    ---Config name: checkbox.rect.highlight-color
    ---Default value: Color :: new(0.3, 0.3, 0.3, 1.0)
    ---Maps to: HighlightColor
    CheckboxRectHighlightColor = 22,
    ---Config name: checkbox.text.opacity
    ---Default value: 0.5f32
    ---Maps to: Opacity
    CheckboxTextOpacity = 23,
    ---Config name: checkbox.text.background-color
    ---Default value: Color :: TRANSPARENT
    ---Maps to: BackgroundColor
    CheckboxTextBackgroundColor = 24,
    ---Config name: checkbox.text.highlight-color
    ---Default value: Color :: TRANSPARENT
    ---Maps to: HighlightColor
    CheckboxTextHighlightColor = 25,
    ---Config name: checkbox.text.font
    ---Default value: Font :: load("Rajdhani", 14)
    ---Maps to: TextFont
    CheckboxTextFont = 26,
    ---Config name: checkbox.text.color
    ---Default value: Color :: WHITE
    ---Maps to: TextColor
    CheckboxTextColor = 27,
    ---Config name: checkbox.click-area.border-color
    ---Default value: Color :: new(0.1, 0.1, 0.1, 0.5)
    ---Maps to: BorderColor
    CheckboxClickAreaBorderColor = 28,
    ---Config name: checkbox.click-area.background-color
    ---Default value: Color :: TRANSPARENT
    ---Maps to: BackgroundColor
    CheckboxClickAreaBackgroundColor = 29,
    ---Config name: checkbox.click-area.highlight-color
    ---Default value: Color :: TRANSPARENT
    ---Maps to: HighlightColor
    CheckboxClickAreaHighlightColor = 30,
    ---Config name: checkbox.click-area.selected-color
    ---Default value: Color :: new(0.1, 0.5, 1.0, 1.0)
    CheckboxClickAreaSelectedColor = 31,
    ---Config name: scroll-area.hscroll.show
    ---Default value: true
    ScrollAreaHScrollShow = 32,
    ---Config name: scroll-area.vscroll.show
    ---Default value: true
    ScrollAreaVScrollShow = 33,
    ---Config name: scroll-area.scrollbar.length
    ---Default value: 4f32
    ScrollAreaScrollbarLength = 34,
    ---Config name: scroll-area.scrollbar.background-color
    ---Default value: Color :: new(0.3, 0.3, 0.3, 0.3)
    ScrollAreaScrollbarBackgroundColor = 35,
    ---Config name: scroll-area.scroll-scale
    ---Default value: 20f32
    ScrollAreaScrollScale = 36,
    ---Config name: scroll-area.scrollbar.visibility-stable-time
    ---Default value: 400u64
    ScrollAreaScrollbarVisibilityStableTime = 37,
    ---Config name: scroll-area.scrollbar.visibility-fade-time
    ---Default value: 200u64
    ScrollAreaScrollbarVisibilityFadeTime = 38,
}
