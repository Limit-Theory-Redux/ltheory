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
    ---Default value: Font :: load ("Rajdhani" , 14)
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
    ---Config name: button.rect.opacity
    ---Default value: 0.5f32
    ---Maps to: Opacity
    ButtonRectOpacity = 8,
    ---Config name: button.rect.border-color
    ---Default value: Color :: TRANSPARENT
    ---Maps to: BorderColor
    ButtonRectBorderColor = 9,
    ---Config name: button.rect.background-color
    ---Default value: Color :: new (0.15 , 0.15 , 0.15 , 0.8)
    ---Maps to: BackgroundColor
    ButtonRectBackgroundColor = 10,
    ---Config name: button.rect.highlight-color
    ---Default value: Color :: new (0.1 , 0.5 , 1.0 , 1.0)
    ---Maps to: HighlightColor
    ButtonRectHighlightColor = 11,
    ---Config name: button.text.opacity
    ---Default value: 0.5f32
    ---Maps to: Opacity
    ButtonTextOpacity = 12,
    ---Config name: button.text.background-color
    ---Default value: Color :: TRANSPARENT
    ---Maps to: BackgroundColor
    ButtonTextBackgroundColor = 13,
    ---Config name: button.text.highlight-color
    ---Default value: Color :: TRANSPARENT
    ---Maps to: HighlightColor
    ButtonTextHighlightColor = 14,
    ---Config name: button.text.font
    ---Default value: Font :: load ("Rajdhani" , 14)
    ---Maps to: TextFont
    ButtonTextFont = 15,
    ---Config name: button.text.color
    ---Default value: Color :: WHITE
    ---Maps to: TextColor
    ButtonTextColor = 16,
    ---Config name: checkbox.rect.opacity
    ---Default value: 0.5f32
    ---Maps to: Opacity
    CheckboxRectOpacity = 17,
    ---Config name: checkbox.rect.border-color
    ---Default value: Color :: TRANSPARENT
    ---Maps to: BorderColor
    CheckboxRectBorderColor = 18,
    ---Config name: checkbox.rect.background-color
    ---Default value: Color :: new (0.3 , 0.3 , 0.3 , 0.5)
    ---Maps to: BackgroundColor
    CheckboxRectBackgroundColor = 19,
    ---Config name: checkbox.rect.highlight-color
    ---Default value: Color :: new (0.3 , 0.3 , 0.3 , 1.0)
    ---Maps to: HighlightColor
    CheckboxRectHighlightColor = 20,
    ---Config name: checkbox.text.opacity
    ---Default value: 0.5f32
    ---Maps to: Opacity
    CheckboxTextOpacity = 21,
    ---Config name: checkbox.text.background-color
    ---Default value: Color :: TRANSPARENT
    ---Maps to: BackgroundColor
    CheckboxTextBackgroundColor = 22,
    ---Config name: checkbox.text.highlight-color
    ---Default value: Color :: TRANSPARENT
    ---Maps to: HighlightColor
    CheckboxTextHighlightColor = 23,
    ---Config name: checkbox.text.font
    ---Default value: Font :: load ("Rajdhani" , 14)
    ---Maps to: TextFont
    CheckboxTextFont = 24,
    ---Config name: checkbox.text.color
    ---Default value: Color :: WHITE
    ---Maps to: TextColor
    CheckboxTextColor = 25,
    ---Config name: checkbox.click-area.border-color
    ---Default value: Color :: new (0.1 , 0.1 , 0.1 , 0.5)
    ---Maps to: BorderColor
    CheckboxClickAreaBorderColor = 26,
    ---Config name: checkbox.click-area.background-color
    ---Default value: Color :: TRANSPARENT
    ---Maps to: BackgroundColor
    CheckboxClickAreaBackgroundColor = 27,
    ---Config name: checkbox.click-area.highlight-color
    ---Default value: Color :: TRANSPARENT
    ---Maps to: HighlightColor
    CheckboxClickAreaHighlightColor = 28,
    ---Config name: checkbox.click-area.selected-color
    ---Default value: Color :: new (0.1 , 0.5 , 1.0 , 1.0)
    CheckboxClickAreaSelectedColor = 29,
    ---Config name: scroll-area.hscroll.show
    ---Default value: true
    ScrollAreaHScrollShow = 30,
    ---Config name: scroll-area.vscroll.show
    ---Default value: true
    ScrollAreaVScrollShow = 31,
    ---Config name: scroll-area.scroll-scale
    ---Default value: 20f32
    ScrollAreaScrollScale = 32,
    ---Config name: scroll-area.scrollbar.length
    ---Default value: 4f32
    ScrollAreaScrollbarLength = 33,
    ---Config name: scroll-area.scrollbar.background-color
    ---Default value: Color :: new (0.3 , 0.3 , 0.3 , 0.3)
    ScrollAreaScrollbarBackgroundColor = 34,
    ---Config name: scroll-area.scrollbar.visibility-fading
    ---Default value: true
    ScrollAreaScrollbarVisibilityFading = 35,
    ---Time in milliseconds for how long scrollbar is visible fading
    ---Config name: scroll-area.scrollbar.visibility-stable-time
    ---Default value: 400u64
    ScrollAreaScrollbarVisibilityStableTime = 36,
    ---Time in milliseconds for how long scrollbar is fading
    ---Config name: scroll-area.scrollbar.visibility-fade-time
    ---Default value: 200u64
    ScrollAreaScrollbarVisibilityFadeTime = 37,
    ---Config name: scroll-area.scrollbar.knob-color
    ---Default value: Color :: new (0.1 , 0.1 , 0.1 , 0.5)
    ScrollAreaScrollbarKnobColor = 38,
}

