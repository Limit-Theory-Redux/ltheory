use std::ops::Range;

use indexmap::IndexMap;
use parley::RangedBuilder;
use parley::fontique::Language;
use parley::style::{FontFamily, FontStyle, FontWeight, FontWidth, LineHeight, StyleProperty};
use tracing::warn;

use crate::render::Color;

/// Collection of the text properties.
#[derive(Default, Clone, PartialEq)]
pub struct TextStyle {
    style_properties: IndexMap<TextStylePropertyId, TextStyleProperty>,
}

/// Contains collection of different text styling properties.
#[luajit_ffi_gen::luajit_ffi]
impl TextStyle {
    #[bind(name = "Create")]
    pub fn new() -> Self {
        Self::default()
    }

    /// Font family list in CSS format.
    pub fn set_font_family(&mut self, family: &str) {
        self.style_properties.insert(
            TextStylePropertyId::FontFamily,
            TextStyleProperty::FontFamily(family.into()),
        );
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.style_properties.insert(
            TextStylePropertyId::FontSize,
            TextStyleProperty::FontSize(size),
        );
    }

    /// Visual width of a font-- a relative change from the normal aspect
    /// ratio, typically in the range 0.5 to 2.0.
    pub fn set_font_stretch(&mut self, stretch: f32) {
        self.style_properties.insert(
            TextStylePropertyId::FontStretch,
            TextStyleProperty::FontStretch(FontWidth::from_ratio(stretch)),
        );
    }

    /// Specify whether font italic or normal.
    pub fn set_font_italic(&mut self, italic: bool) {
        self.style_properties.insert(
            TextStylePropertyId::FontStyle,
            if italic {
                TextStyleProperty::FontStyle(FontStyle::Italic)
            } else {
                TextStyleProperty::FontStyle(FontStyle::Normal)
            },
        );
    }

    /// Visual weight class of a font, typically on a scale from 1.0 to 1000.0.
    pub fn set_font_weight(&mut self, weight: f32) {
        self.style_properties.insert(
            TextStylePropertyId::FontWeight,
            TextStyleProperty::FontWeight(FontWeight::new(weight)),
        );
    }

    /// BCP 47 language tag (`en-US`); no tag or an invalid one means no locale.
    pub fn set_locale(&mut self, locale: Option<&str>) {
        let language = locale.and_then(|tag| {
            Language::parse(tag)
                .inspect_err(|err| warn!("Ignoring invalid locale <{tag}>: {err:?}"))
                .ok()
        });
        self.style_properties.insert(
            TextStylePropertyId::Locale,
            TextStyleProperty::Locale(language),
        );
    }

    /// Brush for rendering text.
    pub fn set_brush(&mut self, color: &Color) {
        self.style_properties
            .insert(TextStylePropertyId::Brush, TextStyleProperty::Brush(*color));
    }

    /// Underline decoration.
    pub fn set_underline(&mut self, underline: bool) {
        self.style_properties.insert(
            TextStylePropertyId::Underline,
            TextStyleProperty::Underline(underline),
        );
    }

    /// Offset of the underline decoration.
    pub fn set_underline_offset(&mut self, offset: f32) {
        self.style_properties.insert(
            TextStylePropertyId::UnderlineOffset,
            TextStyleProperty::UnderlineOffset(Some(offset)),
        );
    }

    /// Size of the underline decoration.
    pub fn set_underline_size(&mut self, size: f32) {
        self.style_properties.insert(
            TextStylePropertyId::UnderlineSize,
            TextStyleProperty::UnderlineSize(Some(size)),
        );
    }

    /// Brush for rendering the underline decoration.
    pub fn set_underline_brush(&mut self, color: Option<&Color>) {
        self.style_properties.insert(
            TextStylePropertyId::UnderlineBrush,
            TextStyleProperty::UnderlineBrush(color.copied()),
        );
    }

    /// Strikethrough decoration.
    pub fn set_strikethrough(&mut self, strikethrough: bool) {
        self.style_properties.insert(
            TextStylePropertyId::Strikethrough,
            TextStyleProperty::Strikethrough(strikethrough),
        );
    }

    /// Offset of the strikethrough decoration.
    pub fn set_strikethrough_offset(&mut self, offset: f32) {
        self.style_properties.insert(
            TextStylePropertyId::StrikethroughOffset,
            TextStyleProperty::StrikethroughOffset(Some(offset)),
        );
    }

    /// Size of the strikethrough decoration.
    pub fn set_strikethrough_size(&mut self, size: f32) {
        self.style_properties.insert(
            TextStylePropertyId::StrikethroughSize,
            TextStyleProperty::StrikethroughSize(Some(size)),
        );
    }

    /// Brush for rendering the strikethrough decoration.
    pub fn set_strikethrough_brush(&mut self, color: Option<&Color>) {
        self.style_properties.insert(
            TextStylePropertyId::StrikethroughBrush,
            TextStyleProperty::StrikethroughBrush(color.copied()),
        );
    }

    /// Line height as a multiple of the font's own line height (ascent +
    /// descent + line gap).
    pub fn set_line_height(&mut self, height: f32) {
        self.style_properties.insert(
            TextStylePropertyId::LineHeight,
            TextStyleProperty::LineHeight(height),
        );
    }

    /// Extra spacing between words.
    pub fn set_word_spacing(&mut self, size: f32) {
        self.style_properties.insert(
            TextStylePropertyId::WordSpacing,
            TextStyleProperty::WordSpacing(size),
        );
    }

    /// Extra spacing between letters.
    pub fn set_letter_spacing(&mut self, size: f32) {
        self.style_properties.insert(
            TextStylePropertyId::LetterSpacing,
            TextStyleProperty::LetterSpacing(size),
        );
    }
}

impl TextStyle {
    /// Apply default text styling to the text layout builder.
    pub fn apply_default<'a>(&'a self, builder: &mut RangedBuilder<'a, Color>) {
        for (_, property) in &self.style_properties {
            builder.push_default(property.as_parley());
        }
    }

    /// Apply text styling to the range of text into the text layout builder.
    pub fn apply_to_section<'a>(
        &'a self,
        builder: &mut RangedBuilder<'a, Color>,
        range: &Range<usize>,
    ) {
        for (_, property) in &self.style_properties {
            builder.push(property.as_parley(), range.start..range.end);
        }
    }
}

/// Unique ids of the [`TextStyleProperty`] enum variants.
#[derive(Clone, PartialEq, Eq, Hash)]
enum TextStylePropertyId {
    FontFamily,
    FontSize,
    FontStretch,
    FontStyle,
    FontWeight,
    // FontVariations,
    // FontFeatures,
    Locale,
    Brush,
    Underline,
    UnderlineOffset,
    UnderlineSize,
    UnderlineBrush,
    Strikethrough,
    StrikethroughOffset,
    StrikethroughSize,
    StrikethroughBrush,
    LineHeight,
    WordSpacing,
    LetterSpacing,
}

/// Properties that define a text style.
/// Equivalent of [`parley::style::StyleProperty`] enum without lifetime parameter.
#[derive(Clone, PartialEq, Debug)]
enum TextStyleProperty {
    FontFamily(String),
    FontSize(f32),
    FontStretch(FontWidth),
    FontStyle(FontStyle),
    FontWeight(FontWeight),
    // FontVariations(FontSettings<'a, FontVariation>),
    // FontFeatures(FontSettings<'a, FontFeature>),
    Locale(Option<Language>),
    Brush(Color),
    Underline(bool),
    UnderlineOffset(Option<f32>),
    UnderlineSize(Option<f32>),
    UnderlineBrush(Option<Color>),
    Strikethrough(bool),
    StrikethroughOffset(Option<f32>),
    StrikethroughSize(Option<f32>),
    StrikethroughBrush(Option<Color>),
    LineHeight(f32),
    WordSpacing(f32),
    LetterSpacing(f32),
}

impl TextStyleProperty {
    fn as_parley(&self) -> StyleProperty<'_, Color> {
        match self {
            Self::FontFamily(family) => {
                StyleProperty::FontFamily(FontFamily::from(family.as_str()))
            }
            Self::FontSize(size) => StyleProperty::FontSize(*size),
            Self::FontStretch(stretch) => StyleProperty::FontWidth(*stretch),
            Self::FontStyle(style) => StyleProperty::FontStyle(*style),
            Self::FontWeight(weight) => StyleProperty::FontWeight(*weight),
            Self::Locale(locale) => StyleProperty::Locale(*locale),
            Self::Brush(brush) => StyleProperty::Brush(*brush),
            Self::Underline(underline) => StyleProperty::Underline(*underline),
            Self::UnderlineOffset(offset) => StyleProperty::UnderlineOffset(*offset),
            Self::UnderlineSize(size) => StyleProperty::UnderlineSize(*size),
            Self::UnderlineBrush(brush) => StyleProperty::UnderlineBrush(*brush),
            Self::Strikethrough(strikethrough) => StyleProperty::Strikethrough(*strikethrough),
            Self::StrikethroughOffset(offset) => StyleProperty::StrikethroughOffset(*offset),
            Self::StrikethroughSize(size) => StyleProperty::StrikethroughSize(*size),
            Self::StrikethroughBrush(brush) => StyleProperty::StrikethroughBrush(*brush),
            Self::LineHeight(height) => {
                StyleProperty::LineHeight(LineHeight::MetricsRelative(*height))
            }
            Self::WordSpacing(spacing) => StyleProperty::WordSpacing(*spacing),
            Self::LetterSpacing(spacing) => StyleProperty::LetterSpacing(*spacing),
        }
    }
}
