//! Enterprise localization (l10n) catalog, plural rules, and locale-aware message formatting.

use std::collections::HashMap;
use std::fmt::Debug;

/// Language and regional locale identifier.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Locale {
    code: String,
    language: String,
    region: Option<String>,
}

impl Locale {
    /// Create a locale from a BCP 47 string (e.g. "en-US", "fr", "ar-EG").
    #[must_use]
    pub fn new(code: impl Into<String>) -> Self {
        let code_str = code.into();
        let mut parts = code_str.split(&['-', '_'][..]);
        let lang = parts.next().unwrap_or("en").to_ascii_lowercase();
        let reg = parts.next().map(str::to_ascii_uppercase);
        Self {
            code: code_str,
            language: lang,
            region: reg,
        }
    }

    pub fn en_us() -> Self {
        Self::new("en-US")
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }

    pub fn code(&self) -> &str {
        &self.code
    }
}

/// Standard Unicode CLDR plural categories.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PluralCategory {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

impl PluralCategory {
    /// Determine the cardinal plural category for a number in the given locale.
    #[must_use]
    pub fn for_count(locale: &Locale, count: i64) -> Self {
        match locale.language() {
            "fr" => {
                if count.abs() == 0 || count.abs() == 1 {
                    Self::One
                } else {
                    Self::Other
                }
            }
            "ar" => match count.abs() {
                0 => Self::Zero,
                1 => Self::One,
                2 => Self::Two,
                3..=10 => Self::Few,
                11..=99 => Self::Many,
                _ => Self::Other,
            },
            "ja" | "zh" | "ko" | "vi" | "am" => Self::Other,
            _ => {
                if count.abs() == 1 {
                    Self::One
                } else {
                    Self::Other
                }
            }
        }
    }
}

/// Translation entry: either a direct template or a pluralized set of templates.
#[derive(Clone, Debug)]
pub enum MessageEntry {
    Exact(String),
    Plural(HashMap<PluralCategory, String>),
}

/// Localized message string catalog.
#[derive(Clone, Debug, Default)]
pub struct MessageCatalog {
    messages: HashMap<String, MessageEntry>,
}

impl MessageCatalog {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, key: impl Into<String>, template: impl Into<String>) {
        self.messages
            .insert(key.into(), MessageEntry::Exact(template.into()));
    }

    pub fn add_plural<I>(&mut self, key: impl Into<String>, forms: I)
    where
        I: IntoIterator<Item = (PluralCategory, &'static str)>,
    {
        let mut map = HashMap::new();
        for (cat, tmpl) in forms {
            map.insert(cat, tmpl.to_string());
        }
        self.messages.insert(key.into(), MessageEntry::Plural(map));
    }
}

/// Central localization manager resolving translations, plurals, and locale-specific formatters.
#[derive(Clone, Debug)]
pub struct L10nManager {
    current_locale: Locale,
    fallback_locale: Locale,
    catalogs: HashMap<Locale, MessageCatalog>,
}

impl Default for L10nManager {
    fn default() -> Self {
        Self::new(Locale::en_us())
    }
}

impl L10nManager {
    #[must_use]
    pub fn new(current_locale: Locale) -> Self {
        Self {
            fallback_locale: Locale::en_us(),
            current_locale,
            catalogs: HashMap::new(),
        }
    }

    pub fn set_locale(&mut self, locale: Locale) {
        self.current_locale = locale;
    }

    pub fn set_catalog(&mut self, locale: Locale, catalog: MessageCatalog) {
        self.catalogs.insert(locale, catalog);
    }

    /// Format a localized message replacing `{param}` tokens.
    #[must_use]
    pub fn message(&self, key: &str, params: &[(&str, &str)]) -> String {
        let template = self.resolve_template(key, None);
        interpolate(&template, params)
    }

    /// Format a pluralized localized message according to CLDR plural rules.
    #[must_use]
    pub fn plural(&self, key: &str, count: i64, params: &[(&str, &str)]) -> String {
        let cat = PluralCategory::for_count(&self.current_locale, count);
        let template = self.resolve_template(key, Some(cat));

        let count_str = count.to_string();
        let mut extended_params = Vec::with_capacity(params.len() + 1);
        extended_params.push(("count", count_str.as_str()));
        extended_params.extend_from_slice(params);

        interpolate(&template, &extended_params)
    }

    /// Format currency with symbol placement according to locale standards.
    #[must_use]
    pub fn format_currency(&self, amount: f64, currency_code: &str) -> String {
        let formatted_num = self.format_number(amount, 2);
        match currency_code {
            "USD" => format!("${formatted_num}"),
            "EUR" => {
                if self.current_locale.language() == "fr" || self.current_locale.language() == "de"
                {
                    format!("{formatted_num} €")
                } else {
                    format!("€{formatted_num}")
                }
            }
            "GBP" => format!("£{formatted_num}"),
            other => format!("{other} {formatted_num}"),
        }
    }

    /// Format a number with thousands separators and decimals.
    #[must_use]
    pub fn format_number(&self, number: f64, decimals: usize) -> String {
        let is_euro = matches!(self.current_locale.language(), "de" | "fr" | "it" | "es");
        let thousands_sep = if is_euro { '.' } else { ',' };
        let decimal_sep = if is_euro { ',' } else { '.' };

        let int_part = number.trunc().abs() as u64;
        let frac_part = (number.abs().fract() * 10f64.powi(decimals as i32)).round() as u64;

        let int_str = int_part.to_string();
        let mut formatted_int = String::new();
        let len = int_str.len();

        for (i, ch) in int_str.chars().enumerate() {
            if i > 0 && (len - i).is_multiple_of(3) {
                formatted_int.push(thousands_sep);
            }
            formatted_int.push(ch);
        }

        if number < 0.0 {
            formatted_int.insert(0, '-');
        }

        if decimals > 0 {
            format!("{formatted_int}{decimal_sep}{frac_part:0decimals$}")
        } else {
            formatted_int
        }
    }

    fn resolve_template(&self, key: &str, category: Option<PluralCategory>) -> String {
        let entry = self
            .catalogs
            .get(&self.current_locale)
            .and_then(|c| c.messages.get(key))
            .or_else(|| {
                self.catalogs
                    .get(&self.fallback_locale)
                    .and_then(|c| c.messages.get(key))
            });

        match entry {
            Some(MessageEntry::Exact(tmpl)) => tmpl.clone(),
            Some(MessageEntry::Plural(map)) => {
                if let Some(cat) = category {
                    map.get(&cat)
                        .or_else(|| map.get(&PluralCategory::Other))
                        .cloned()
                        .unwrap_or_else(|| key.to_string())
                } else {
                    map.get(&PluralCategory::Other)
                        .cloned()
                        .unwrap_or_else(|| key.to_string())
                }
            }
            None => key.to_string(),
        }
    }
}

fn interpolate(template: &str, params: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (k, v) in params {
        let needle = format!("{{{k}}}");
        out = out.replace(&needle, v);
    }
    out
}
