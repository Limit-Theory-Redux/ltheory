use indexmap::IndexMap;
use parley::{
    context::RangedBuilder,
    style::{FontStack, FontStretch, FontStyle, FontWeight, StyleProperty},
};

use internal::ConvertIntoString;

use crate::render::Color;

#[derive(Default, Clone, PartialEq)]
pub struct TextStyle {
    style_properties: IndexMap<TextStylePropertyId, TextStyleProperty>,
}

#[luajit_ffi_gen::luajit_ffi]
impl TextStyle {
    #[bind(name = "Create")]
    pub fn new() -> Self {
        Self::default()
    }

    /// Font family list in CSS format.
    pub fn set_font_family(&mut self, family: &str) {
        self.style_properties[&TextStylePropertyId::FontFamily] =
            TextStyleProperty::FontFamily(family.into());
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.style_properties[&TextStylePropertyId::FontSize] = TextStyleProperty::FontSize(size);
    }

    /// Visual width of a font-- a relative change from the normal aspect
    /// ratio, typically in the range 0.5 to 2.0.
    pub fn set_font_stretch(&mut self, stretch: f32) {
        self.style_properties[&TextStylePropertyId::FontStretch] =
            TextStyleProperty::FontStretch(FontStretch::from_ratio(stretch));
    }

    /// Specify whether font italic or normal.
    pub fn set_font_italic(&mut self, italic: bool) {
        self.style_properties[&TextStylePropertyId::FontStyle] = if italic {
            TextStyleProperty::FontStyle(FontStyle::Italic)
        } else {
            TextStyleProperty::FontStyle(FontStyle::Normal)
        };
    }

    /// Visual weight class of a font, typically on a scale from 1.0 to 1000.0.
    pub fn set_font_weight(&mut self, weight: f32) {
        self.style_properties[&TextStylePropertyId::FontWeight] =
            TextStyleProperty::FontWeight(FontWeight::new(weight));
    }

    pub fn set_locale(&mut self, locale: Option<&str>) {
        self.style_properties[&TextStylePropertyId::Locale] =
            TextStyleProperty::Locale(locale.map(|l| l.into()));
    }

    /// Brush for rendering text.
    pub fn set_brush(&mut self, color: &Color) {
        self.style_properties[&TextStylePropertyId::Brush] = TextStyleProperty::Brush(*color);
    }

    /// Underline decoration.
    pub fn set_underline(&mut self, underline: bool) {
        self.style_properties[&TextStylePropertyId::Underline] =
            TextStyleProperty::Underline(underline);
    }

    /// Offset of the underline decoration.
    pub fn set_underline_offset(&mut self, offset: Option<f32>) {
        self.style_properties[&TextStylePropertyId::UnderlineOffset] =
            TextStyleProperty::UnderlineOffset(offset);
    }

    /// Size of the underline decoration.
    pub fn set_underline_size(&mut self, size: Option<f32>) {
        self.style_properties[&TextStylePropertyId::UnderlineSize] =
            TextStyleProperty::UnderlineSize(size);
    }

    /// Brush for rendering the underline decoration.
    pub fn set_underline_brush(&mut self, color: Option<&Color>) {
        self.style_properties[&TextStylePropertyId::UnderlineBrush] =
            TextStyleProperty::UnderlineBrush(color.map(|c| *c));
    }

    /// Strikethrough decoration.
    pub fn set_strikethrough(&mut self, strikethrough: bool) {
        self.style_properties[&TextStylePropertyId::Strikethrough] =
            TextStyleProperty::Strikethrough(strikethrough);
    }

    /// Offset of the strikethrough decoration.
    pub fn set_strikethrough_offset(&mut self, offset: Option<f32>) {
        self.style_properties[&TextStylePropertyId::StrikethroughOffset] =
            TextStyleProperty::StrikethroughOffset(offset);
    }

    /// Size of the strikethrough decoration.
    pub fn set_strikethrough_size(&mut self, size: Option<f32>) {
        self.style_properties[&TextStylePropertyId::StrikethroughSize] =
            TextStyleProperty::StrikethroughSize(size);
    }

    /// Brush for rendering the strikethrough decoration.
    pub fn set_strikethrough_brush(&mut self, color: Option<&Color>) {
        self.style_properties[&TextStylePropertyId::StrikethroughBrush] =
            TextStyleProperty::StrikethroughBrush(color.map(|c| *c));
    }

    /// Line height multiplier.
    pub fn set_line_height(&mut self, height: f32) {
        self.style_properties[&TextStylePropertyId::LineHeight] =
            TextStyleProperty::LineHeight(height);
    }

    /// Extra spacing between words.
    pub fn set_word_spacing(&mut self, size: f32) {
        self.style_properties[&TextStylePropertyId::WordSpacing] =
            TextStyleProperty::WordSpacing(size);
    }

    /// Extra spacing between letters.
    pub fn set_letter_spacing(&mut self, size: f32) {
        self.style_properties[&TextStylePropertyId::LetterSpacing] =
            TextStyleProperty::LetterSpacing(size);
    }
}

impl TextStyle {
    pub fn apply_default<'a>(&'a self, builder: &mut RangedBuilder<'a, Color, &str>) {
        for (_, property) in &self.style_properties {
            builder.push_default(&property.as_parley());
        }
    }

    pub fn apply_to_section<'a>(
        &'a self,
        builder: &mut RangedBuilder<'a, Color, &str>,
        start: usize,
        end: usize,
    ) {
        for (_, property) in &self.style_properties {
            builder.push(&property.as_parley(), start..end);
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

/// Properties that define a style.
/// Lifetimeless equivalent of [`parley::style::StyleProperty`] enum.
#[derive(Clone, PartialEq, Debug)]
enum TextStyleProperty {
    FontFamily(String),
    FontSize(f32),
    FontStretch(FontStretch),
    FontStyle(FontStyle),
    FontWeight(FontWeight),
    // FontVariations(FontSettings<'a, FontVariation>),
    // FontFeatures(FontSettings<'a, FontFeature>),
    Locale(Option<String>),
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
    fn as_parley<'a>(&'a self) -> StyleProperty<'a, Color> {
        match self {
            Self::FontFamily(family) => {
                StyleProperty::FontStack(FontStack::Source(family.as_str()))
            }
            Self::FontSize(size) => StyleProperty::FontSize(*size),
            Self::FontStretch(stretch) => StyleProperty::FontStretch(*stretch),
            Self::FontStyle(style) => StyleProperty::FontStyle(*style),
            Self::FontWeight(weight) => StyleProperty::FontWeight(*weight),
            Self::Locale(locale) => StyleProperty::Locale(locale.as_ref().map(|l| l.as_str())),
            Self::Brush(brush) => StyleProperty::Brush(*brush),
            Self::Underline(underline) => StyleProperty::Underline(*underline),
            Self::UnderlineOffset(offset) => StyleProperty::UnderlineOffset(*offset),
            Self::UnderlineSize(size) => StyleProperty::UnderlineSize(*size),
            Self::UnderlineBrush(brush) => StyleProperty::UnderlineBrush(*brush),
            Self::Strikethrough(strikethrough) => StyleProperty::Strikethrough(*strikethrough),
            Self::StrikethroughOffset(offset) => StyleProperty::StrikethroughOffset(*offset),
            Self::StrikethroughSize(size) => StyleProperty::StrikethroughSize(*size),
            Self::StrikethroughBrush(brush) => StyleProperty::StrikethroughBrush(*brush),
            Self::LineHeight(height) => StyleProperty::LineHeight(*height),
            Self::WordSpacing(spacing) => StyleProperty::WordSpacing(*spacing),
            Self::LetterSpacing(spacing) => StyleProperty::LetterSpacing(*spacing),
        }
    }
}
