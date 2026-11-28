//! Dynamic system font discovery, font family resolution, and shared font registry using `fontique`.

use crate::rasterizer::FontRasterizer;
use fontique::{
    Attributes, Blob, Collection, CollectionOptions, FontStyle, FontWeight, FontWidth,
    GenericFamily, QueryFamily, QueryFont, QueryStatus, SourceCache,
};
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use tej_ui_layout::Size;

/// Broad font family classification for registry lookup and fallback sizing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontFamilyKind {
    /// Proportional sans-serif UI font.
    SansSerif,
    /// Fixed-width monospace font.
    Monospace,
}

/// Font slant selection (normal upright vs. italic).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontSlant {
    /// Upright roman face.
    Normal,
    /// Italic or oblique face.
    Italic,
}

/// Type alias for cached font face entries keyed by family kind, weight bucket, and slant.
type RasterizerMap = HashMap<(FontFamilyKind, u16, FontSlant), (u64, Arc<FontRasterizer>)>;

/// Dynamic system font database discovering installed OS fonts and managing fallback chains.
pub struct SystemFontDb {
    collection: RwLock<Collection>,
    source_cache: RwLock<SourceCache>,
}

impl Default for SystemFontDb {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SystemFontDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemFontDb").finish()
    }
}

impl SystemFontDb {
    /// Construct a font database initialized with live host system fonts.
    #[must_use]
    pub fn new() -> Self {
        let options = CollectionOptions {
            shared: false,
            system_fonts: true,
        };
        let mut collection = Collection::new(options);
        collection.load_system_fonts();
        Self {
            collection: RwLock::new(collection),
            source_cache: RwLock::new(SourceCache::default()),
        }
    }

    /// Construct an empty font database without querying OS paths (ideal for hermetic tests).
    #[must_use]
    pub fn new_empty() -> Self {
        let options = CollectionOptions {
            shared: false,
            system_fonts: false,
        };
        Self {
            collection: RwLock::new(Collection::new(options)),
            source_cache: RwLock::new(SourceCache::default()),
        }
    }

    /// Register a font directly from in-memory TTF/OTF bytes.
    pub fn register_font_bytes(&self, bytes: Vec<u8>) -> bool {
        let mut coll = self.collection.write();
        let handle = Blob::new(Arc::new(bytes));
        let results = coll.register_fonts(handle, None);
        !results.is_empty()
    }

    /// Query the best matching font face for a family name, weight, and style.
    #[must_use]
    pub fn query_font(
        &self,
        family_name: &str,
        weight: u16,
        slant: FontSlant,
    ) -> Option<QueryFont> {
        let mut coll = self.collection.write();
        let mut cache = self.source_cache.write();
        let mut query = coll.query(&mut cache);

        let family = match family_name.to_lowercase().as_str() {
            "sans-serif" | "system-ui" => QueryFamily::Generic(GenericFamily::SansSerif),
            "serif" => QueryFamily::Generic(GenericFamily::Serif),
            "monospace" => QueryFamily::Generic(GenericFamily::Monospace),
            "cursive" => QueryFamily::Generic(GenericFamily::Cursive),
            "fantasy" => QueryFamily::Generic(GenericFamily::Fantasy),
            _ => QueryFamily::Named(family_name),
        };

        let style = match slant {
            FontSlant::Italic => FontStyle::Italic,
            FontSlant::Normal => FontStyle::Normal,
        };
        let attrs = Attributes::new(FontWidth::NORMAL, style, FontWeight::new(f32::from(weight)));

        query.set_families([family]);
        query.set_attributes(attrs);

        let mut matched = None;
        query.matches_with(|font: &QueryFont| {
            matched = Some(font.clone());
            QueryStatus::Stop
        });

        matched
    }

    /// Total number of font families discovered in this database.
    #[must_use]
    pub fn family_count(&self) -> usize {
        self.collection.write().family_names().count()
    }
}

/// Thread-safe shared registry of system [`FontRasterizer`] instances keyed by
/// `(monospace, weight_bucket, italic)`.
///
/// Both [`crate::TextMeasurer`] and GPU text renderers share [`FontRegistry::global()`] so layout
/// measurement and glyph rasterization use the exact same font faces and metrics.
pub struct FontRegistry {
    db: SystemFontDb,
    faces: Mutex<RasterizerMap>,
    next_font_id: AtomicU64,
}

impl Default for FontRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for FontRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontRegistry")
            .field("cached_faces", &self.faces.lock().len())
            .finish_non_exhaustive()
    }
}

impl FontRegistry {
    /// Return the process-wide shared [`FontRegistry`] initialized with host system fonts.
    #[must_use]
    pub fn global() -> &'static Self {
        static GLOBAL: OnceLock<FontRegistry> = OnceLock::new();
        GLOBAL.get_or_init(Self::new)
    }

    /// Create a new registry backed by live host system fonts.
    #[must_use]
    pub fn new() -> Self {
        Self {
            db: SystemFontDb::new(),
            faces: Mutex::new(HashMap::new()),
            next_font_id: AtomicU64::new(1),
        }
    }

    /// Create an empty registry without querying host OS fonts (useful for hermetic tests).
    #[must_use]
    pub fn new_empty() -> Self {
        Self {
            db: SystemFontDb::new_empty(),
            faces: Mutex::new(HashMap::new()),
            next_font_id: AtomicU64::new(1),
        }
    }

    /// Snap an arbitrary CSS-style numeric font weight to one of `400`, `600`, or `700`.
    #[must_use]
    pub const fn weight_bucket(weight: u16) -> u16 {
        if weight >= 650 {
            700
        } else if weight >= 500 {
            600
        } else {
            400
        }
    }

    fn try_query_face(
        &self,
        family: &str,
        weight: u16,
        slant: FontSlant,
    ) -> Option<FontRasterizer> {
        let font = self.db.query_font(family, weight, slant)?;
        FontRasterizer::from_bytes_with_index(font.blob.data(), font.index)
            .or_else(|_| FontRasterizer::from_bytes(font.blob.data()))
            .ok()
    }

    fn load_face(
        &self,
        family_kind: FontFamilyKind,
        weight_bucket: u16,
        slant: FontSlant,
    ) -> Option<FontRasterizer> {
        let family = match family_kind {
            FontFamilyKind::Monospace => "monospace",
            FontFamilyKind::SansSerif => "sans-serif",
        };
        if let Some(rast) = self.try_query_face(family, weight_bucket, slant) {
            return Some(rast);
        }
        if slant == FontSlant::Italic
            && let Some(rast) = self.try_query_face(family, weight_bucket, FontSlant::Normal)
        {
            return Some(rast);
        }
        if weight_bucket != 400
            && let Some(rast) = self.try_query_face(family, 400, FontSlant::Normal)
        {
            return Some(rast);
        }
        if family_kind == FontFamilyKind::Monospace
            && let Some(rast) = self.try_query_face("sans-serif", 400, FontSlant::Normal)
        {
            return Some(rast);
        }
        None
    }

    /// Resolve (or lazily load) a cached `(font_id, Arc<FontRasterizer>)` for the requested style.
    #[must_use]
    pub fn rasterizer(
        &self,
        family_kind: FontFamilyKind,
        weight: u16,
        slant: FontSlant,
    ) -> Option<(u64, Arc<FontRasterizer>)> {
        let bucket = Self::weight_bucket(weight);
        let key = (family_kind, bucket, slant);

        let mut faces = self.faces.lock();
        if let Some(entry) = faces.get(&key) {
            return Some(entry.clone());
        }

        let rast = self.load_face(family_kind, bucket, slant)?;
        let font_id = self.next_font_id.fetch_add(1, Ordering::Relaxed);
        let entry = (font_id, Arc::new(rast));
        faces.insert(key, entry.clone());
        Some(entry)
    }

    /// Fallback proportional/monospace heuristic measurement when no system font is available.
    #[must_use]
    pub fn fallback_measure_text(text: &str, font_size: f32, family_kind: FontFamilyKind) -> Size {
        let safe_size = if font_size.is_finite() && font_size > 0.0 {
            font_size
        } else {
            1.0
        };
        let ratio = match family_kind {
            FontFamilyKind::Monospace => 0.62,
            FontFamilyKind::SansSerif => 0.58,
        };
        let avg_width = safe_size * ratio;
        let line_height = safe_size * 1.35;

        let mut max_chars = 0usize;
        let mut cur_chars = 0usize;
        let mut lines = 1usize;

        for ch in text.chars() {
            if ch == '\n' {
                max_chars = max_chars.max(cur_chars);
                cur_chars = 0;
                lines += 1;
            } else {
                cur_chars += 1;
            }
        }
        max_chars = max_chars.max(cur_chars);

        let max_chars_f32 = f32::from(u16::try_from(max_chars).unwrap_or(u16::MAX));
        let lines_f32 = f32::from(u16::try_from(lines).unwrap_or(u16::MAX));
        Size::new(max_chars_f32 * avg_width, lines_f32 * line_height)
    }

    /// Measure `text` using the resolved system [`FontRasterizer`], falling back to calibrated
    /// character-width ratios only if no system font is installed.
    #[must_use]
    pub fn measure_text(
        &self,
        text: &str,
        font_size: f32,
        family_kind: FontFamilyKind,
        weight: u16,
    ) -> Size {
        if let Some((_, rast)) = self.rasterizer(family_kind, weight, FontSlant::Normal) {
            rast.measure_text(text, font_size)
        } else {
            Self::fallback_measure_text(text, font_size, family_kind)
        }
    }
}
