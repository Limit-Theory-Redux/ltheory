use std::collections::HashMap;
use std::iter;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use parley::fontique::{
    Blob, Collection, CollectionOptions, FallbackKey, FontInfoOverride, FontStyle, FontWeight,
    FontWidth, GenericFamily, Script, ScriptExt, SourceCache,
};
use parley::{FontContext, LayoutContext};
use swash::scale::ScaleContext;
use swash::{FontRef, StringId};
use tracing::warn;

use crate::render::Color;
use crate::system::{Resource, ResourceType};

/// Family that covers the characters the other game fonts lack: parley falls
/// back to it for every script, so text never depends on the fonts of the
/// operating system.
const FALLBACK_FAMILY: &str = "DejaVu Sans";
/// Family for the generic `monospace` family.
const MONOSPACE_FAMILY: &str = "DejaVu Sans Mono";

/// A font file registered with the text context.
#[derive(Clone)]
pub struct RegisteredFont {
    /// Contents of the font file, shared with parley's font collection.
    pub blob: Blob<u8>,
    /// The family that holds exactly this file: the file name without its
    /// extension (`Exo2Bold`), or the font's own family if that is the same
    /// name.
    pub family: String,
    /// The file's own width, style and weight: asking `family` for these
    /// selects this file even when the family has more files.
    pub width: FontWidth,
    pub style: FontStyle,
    pub weight: FontWeight,
}

/// Persistent per application cache of text related information: fonts, layout and scale contexts.
///
/// The font collection holds only the game fonts (`res/font`), never the
/// operating system's, so text looks the same on every platform. Every font
/// file is registered twice: under the family the font itself declares
/// (`Exo 2`, so a weight picks the matching file of the family) and under its
/// file name (`Exo2Bold`, the name `Font.Load` uses).
pub struct TextContext {
    pub font: FontContext,
    pub layout: LayoutContext<Color>,
    pub scale: ScaleContext,
    /// Registered font files by lowercase file name without extension.
    files: HashMap<String, RegisteredFont>,
}

impl Default for TextContext {
    fn default() -> Self {
        Self::with_font_dirs(default_font_dirs())
    }
}

impl TextContext {
    /// A context with every `ttf`/`otf` file of `dirs` registered (folders
    /// that don't exist are skipped) and `DejaVu Sans` as the fallback.
    pub fn with_font_dirs(dirs: impl IntoIterator<Item = PathBuf>) -> Self {
        let mut ctx = Self {
            font: FontContext {
                collection: Collection::new(CollectionOptions {
                    shared: false,
                    system_fonts: false,
                }),
                source_cache: SourceCache::default(),
            },
            layout: LayoutContext::new(),
            scale: ScaleContext::new(),
            files: HashMap::new(),
        };

        for dir in dirs {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            let mut paths: Vec<PathBuf> = entries
                .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                .filter(|path| is_font_file(path))
                .collect();
            // Registration order decides the order of a family's fonts.
            paths.sort();

            for path in paths {
                ctx.register_font_file(&path);
            }
        }

        ctx.set_fallbacks();
        ctx
    }

    /// Register the font file at `path` (once per file name) and return it.
    /// `None` if the file can't be read or holds no font.
    pub fn register_font_file(&mut self, path: &Path) -> Option<RegisteredFont> {
        let name = path.file_stem()?.to_str()?;
        if let Some(font) = self.files.get(&name.to_lowercase()) {
            return Some(font.clone());
        }

        match std::fs::read(path) {
            Ok(data) => self.register_font(name, data),
            Err(err) => {
                warn!("Cannot read font file {path:?}: {err}");
                None
            }
        }
    }

    /// Register a font from the contents of its file, under its own family
    /// and under `name` (the file name without extension). Registering a name
    /// again returns the first registration.
    pub fn register_font(&mut self, name: &str, data: Vec<u8>) -> Option<RegisteredFont> {
        let key = name.to_lowercase();
        if let Some(font) = self.files.get(&key) {
            return Some(font.clone());
        }

        let style = italic_by_name(&data).then_some(FontStyle::Italic);
        let blob = Blob::new(Arc::new(data));
        let collection = &mut self.font.collection;
        let registered = collection.register_fonts(
            blob.clone(),
            Some(FontInfoOverride {
                style,
                ..Default::default()
            }),
        );
        let Some((family_id, info)) = registered
            .iter()
            .find_map(|(id, fonts)| fonts.first().map(|info| (*id, info.clone())))
        else {
            warn!("Font <{name}> holds no font that can be used");
            return None;
        };

        let own_family = collection.family_name(family_id).unwrap_or_default();
        let family = if own_family.eq_ignore_ascii_case(name) {
            own_family.to_string()
        } else {
            collection.register_fonts(
                blob.clone(),
                Some(FontInfoOverride {
                    family_name: Some(name),
                    style,
                    ..Default::default()
                }),
            );
            name.to_string()
        };

        let font = RegisteredFont {
            blob,
            family,
            width: info.width(),
            style: info.style(),
            weight: info.weight(),
        };
        self.files.insert(key, font.clone());
        FONT_GENERATION.fetch_add(1, Ordering::Relaxed);
        Some(font)
    }

    /// Make `DejaVu Sans` the fallback of every script and the generic
    /// `sans-serif` family, and `DejaVu Sans Mono` the generic `monospace`.
    fn set_fallbacks(&mut self) {
        let collection = &mut self.font.collection;

        if let Some(sans) = collection.family_id(FALLBACK_FAMILY) {
            collection.set_generic_families(GenericFamily::SansSerif, iter::once(sans));
            for (script, _) in Script::all_samples() {
                collection.set_fallbacks(FallbackKey::new(*script, None), iter::once(sans));
            }
        } else if !self.files.is_empty() {
            warn!("Fallback font family <{FALLBACK_FAMILY}> not found in the font folders");
        }

        if let Some(mono) = collection.family_id(MONOSPACE_FAMILY) {
            collection.set_generic_families(GenericFamily::Monospace, iter::once(mono));
        }
    }
}

/// Whether a font whose style flags say upright calls itself italic or
/// oblique in its (typographic) subfamily name. The Unageo italics do: without
/// this they would count as upright and replace the upright files of their
/// family.
fn italic_by_name(data: &[u8]) -> bool {
    let Some(font) = FontRef::from_index(data, 0) else {
        return false;
    };
    if font.attributes().style() != swash::Style::Normal {
        return false;
    }

    [StringId::TypographicSubFamily, StringId::SubFamily]
        .into_iter()
        .filter_map(|id| font.localized_strings().find_by_id(id, None))
        .any(|name| {
            let name = name.chars().collect::<String>().to_lowercase();
            name.contains("italic") || name.contains("oblique")
        })
}

fn is_font_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("ttf") || ext.eq_ignore_ascii_case("otf"))
}

/// The game's font folders. Unit tests run from the crate's folder, so they
/// use the repository's `res/font` directly.
fn default_font_dirs() -> Vec<PathBuf> {
    if cfg!(test) {
        vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../res/font")]
    } else {
        Resource::get_folders(ResourceType::Font)
    }
}

pub static TEXT_CTX: LazyLock<Mutex<TextContext>> = LazyLock::new(Default::default);

/// Counts font registrations. A registered font can change which font a
/// character falls back to, so text laid out before it may be stale.
static FONT_GENERATION: AtomicU64 = AtomicU64::new(0);

/// The number of fonts registered so far (in any text context), readable
/// without locking [`TEXT_CTX`].
pub fn font_generation() -> u64 {
    FONT_GENERATION.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use parley::style::{FontFamily, StyleProperty};
    use parley::{FontWeight, Layout, PositionedLayoutItem};

    use super::*;

    fn res_font_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../res/font")
    }

    /// The font file parley picks for the first glyph run of `text`.
    fn picked_font(
        ctx: &mut TextContext,
        family: &str,
        weight: Option<f32>,
        text: &str,
    ) -> Blob<u8> {
        let TextContext { font, layout, .. } = ctx;
        let mut builder = layout.ranged_builder(font, text, 1.0, false);
        builder.push_default(FontFamily::from(family));
        builder.push_default(StyleProperty::FontSize(16.0));
        if let Some(weight) = weight {
            builder.push_default(StyleProperty::FontWeight(FontWeight::new(weight)));
        }
        let mut layout: Layout<Color> = builder.build(text);
        layout.break_all_lines(None);

        let line = layout.lines().next().expect("a line");
        let item = line.items().next().expect("a glyph run");
        let PositionedLayoutItem::GlyphRun(run) = item else {
            panic!("expected a glyph run");
        };
        run.run().font().data.clone()
    }

    fn file(ctx: &mut TextContext, name: &str) -> Blob<u8> {
        ctx.register_font_file(&res_font_dir().join(format!("{name}.ttf")))
            .expect("registered")
            .blob
    }

    #[test]
    fn file_names_and_families_find_the_game_fonts() {
        let mut ctx = TextContext::with_font_dirs([res_font_dir()]);
        let exo2_bold = file(&mut ctx, "Exo2Bold");
        let exo2_semi_bold = file(&mut ctx, "Exo2SemiBold");

        assert_eq!(
            picked_font(&mut ctx, "Exo2Bold", None, "Credits").id(),
            exo2_bold.id(),
            "the file name is a family of its own"
        );
        assert_eq!(
            picked_font(&mut ctx, "Exo2Bold", Some(600.0), "Credits").id(),
            exo2_bold.id(),
            "a file-name family has only that file"
        );
        assert_eq!(
            picked_font(&mut ctx, "Exo 2", Some(600.0), "Credits").id(),
            exo2_semi_bold.id(),
            "the font's own family picks the file by weight"
        );
    }

    #[test]
    fn italic_files_without_style_flags_count_as_italic() {
        let mut ctx = TextContext::with_font_dirs([res_font_dir()]);
        let regular = file(&mut ctx, "Unageo-Regular");
        let italic = ctx
            .register_font_file(&res_font_dir().join("Unageo-Regular-Italic.ttf"))
            .expect("registered");
        assert_eq!(italic.style, FontStyle::Italic);

        assert_eq!(
            picked_font(&mut ctx, "Unageo", None, "Haron").id(),
            regular.id(),
            "the family's upright file is the default"
        );
    }

    #[test]
    fn dejavu_sans_is_the_only_fallback() {
        let mut ctx = TextContext::with_font_dirs([res_font_dir()]);
        let dejavu = file(&mut ctx, "DejaVuSans");

        assert_eq!(
            picked_font(&mut ctx, "NoSuchFamily", None, "abc").id(),
            dejavu.id(),
            "an unknown family falls back to DejaVu Sans, not a system font"
        );
        // Exo 2 has no Hebrew; DejaVu Sans has.
        assert_eq!(
            picked_font(&mut ctx, "Exo2", None, "\u{5e9}\u{5dc}\u{5d5}\u{5dd}").id(),
            dejavu.id(),
            "characters the font lacks come from DejaVu Sans"
        );
    }

    #[test]
    fn no_system_fonts() {
        let mut ctx = TextContext::with_font_dirs(Vec::new());
        assert_eq!(ctx.font.collection.family_names().count(), 0);
    }
}
