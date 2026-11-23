//! AppKit window chrome styling and macOS titlebar customization.

/// AppKit window chrome and titlebar presentation styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacosWindowStyle {
    /// Standard titled macOS window with traffic light buttons.
    #[default]
    Titled,
    /// Modern edge-to-edge content view (`NSWindowStyleMaskFullSizeContentView`) with integrated titlebar.
    FullSizeContentView,
    /// Borderless floating panel or HUD window.
    Borderless,
}

/// macOS window titlebar and vibrancy configuration.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MacosWindowAttributes {
    /// Window chrome style.
    pub style: MacosWindowStyle,
    /// Whether the titlebar is transparent and blends into the view.
    pub titlebar_transparent: bool,
    /// Whether window title text is hidden (traffic lights retained).
    pub title_hidden: bool,
    /// Whether the window participates in Spaces and Expose.
    pub collection_behavior_transient: bool,
}

impl MacosWindowAttributes {
    /// Creates a modern Linear-style window with integrated titlebar and transparent background.
    #[must_use]
    pub const fn modern_flat() -> Self {
        Self {
            style: MacosWindowStyle::FullSizeContentView,
            titlebar_transparent: true,
            title_hidden: true,
            collection_behavior_transient: false,
        }
    }
}
