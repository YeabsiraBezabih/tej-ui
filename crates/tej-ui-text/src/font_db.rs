//! Dynamic system font discovery, font family resolution, and fallback chains using `fontique`.

use fontique::{
    Attributes, Blob, Collection, CollectionOptions, FontStyle, FontWeight, FontWidth,
    GenericFamily, QueryFamily, QueryFont, QueryStatus, SourceCache,
};
use parking_lot::RwLock;
use std::sync::Arc;

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
    pub fn query_font(&self, family_name: &str, weight: u16, is_italic: bool) -> Option<QueryFont> {
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

        let style = if is_italic {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        };
        let attrs = Attributes::new(FontWidth::NORMAL, style, FontWeight::new(weight as f32));

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
