use crate::view::View;
use std::sync::OnceLock;
use tej_ui_core::Signal;
use tej_ui_hal::Color;
use tej_ui_text::{TextMeasurer, wrap_text};

static TEXT_MEASURER: OnceLock<TextMeasurer> = OnceLock::new();

fn shared_measurer() -> &'static TextMeasurer {
    TEXT_MEASURER.get_or_init(TextMeasurer::new)
}

/// Text content source: either a static string or a reactive `Signal<String>`.
#[derive(Debug, Clone)]
pub enum TextSource {
    Static(String),
    Reactive(Signal<String>),
}

/// Text widget displaying static or reactive string data with real text measurement
/// and optional multi-line word wrapping.
#[derive(Debug, Clone)]
pub struct Text {
    source: TextSource,
    color: Option<Color>,
    max_width: Option<f32>,
    font_size: f32,
    font_weight: u16,
    monospace: bool,
}

/// State retained by a mounted `Text` widget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextState {
    pub current_text: String,
    pub lines: Vec<String>,
    pub node: u32,
}

impl Text {
    /// Default body font size in logical pixels used for measurement when not overridden.
    pub const DEFAULT_FONT_SIZE: f32 = 14.0;

    /// Create a static text widget.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            source: TextSource::Static(text.into()),
            color: None,
            max_width: None,
            font_size: Self::DEFAULT_FONT_SIZE,
            font_weight: 400,
            monospace: false,
        }
    }

    /// Set foreground text color.
    #[must_use]
    pub const fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set maximum width constraint for multi-line word wrapping.
    #[must_use]
    pub const fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self
    }

    /// Set explicit font size in logical pixels.
    #[must_use]
    pub const fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    /// Set bold font weight (`700`).
    #[must_use]
    pub const fn bold(mut self) -> Self {
        self.font_weight = 700;
        self
    }

    fn resolve_paint_style(&self, ctx: &crate::paint::PaintContext<'_>) -> ResolvedTextPaintStyle {
        let inherited = ctx.current_text_style();
        let color = self
            .color
            .or_else(|| inherited.and_then(|s| s.color))
            .unwrap_or_else(|| ctx.current_foreground());
        let font_size = if (self.font_size - Self::DEFAULT_FONT_SIZE).abs() > f32::EPSILON {
            self.font_size
        } else {
            inherited
                .and_then(|s| s.font_size)
                .filter(|&sz| sz > 0.0)
                .unwrap_or(ctx.theme.typography.body.font_size)
        };
        let font_weight = if self.font_weight == 400 {
            inherited
                .and_then(|s| s.font_weight)
                .filter(|&w| w > 0)
                .unwrap_or(400)
        } else {
            self.font_weight
        };
        let monospace = self.monospace || inherited.and_then(|s| s.monospace).unwrap_or(false);
        let line_height = inherited
            .and_then(|s| s.line_height)
            .filter(|&lh| lh > 0.0)
            .unwrap_or(font_size * 1.35);
        ResolvedTextPaintStyle {
            color,
            font_size,
            font_weight,
            monospace,
            line_height,
            decoration: inherited.and_then(|s| s.text_decoration),
            underline_style: inherited.and_then(|s| s.text_underline),
            transform: inherited.and_then(|s| s.text_transform),
            align: inherited.and_then(|s| s.text_align),
            text_indent: inherited.and_then(|s| s.text_indent).unwrap_or(0.0),
            ellipsis: inherited.and_then(|s| s.text_overflow)
                == Some(crate::css::TextOverflow::Ellipsis),
        }
    }

    fn paint_text_line(
        ctx: &mut crate::paint::PaintContext<'_>,
        bounds: tej_ui_layout::Rect,
        style: &ResolvedTextPaintStyle,
        raw: &str,
        y: f32,
        line_pos: LinePosition,
    ) {
        let transformed;
        let mut text = match style.transform {
            Some(crate::css::TextTransform::Uppercase) => {
                transformed = raw.to_uppercase();
                transformed.as_str()
            }
            Some(crate::css::TextTransform::Lowercase) => {
                transformed = raw.to_lowercase();
                transformed.as_str()
            }
            _ => raw,
        };
        let ellipsized;
        let char_w = (style.font_size * 0.55).max(1.0);
        if style.ellipsis && bounds.size.width > 0.0 {
            let max_chars = (bounds.size.width / char_w).floor() as usize;
            if text.chars().count() > max_chars && max_chars > 3 {
                let keep: String = text.chars().take(max_chars - 3).collect();
                ellipsized = format!("{keep}...");
                text = ellipsized.as_str();
            }
        }
        let approx_w = f32::from(u16::try_from(text.len()).unwrap_or(u16::MAX)) * char_w;
        let indent = match line_pos {
            LinePosition::First => style.text_indent,
            LinePosition::Continuation => 0.0,
        };
        let x = match style.align {
            Some(crate::css::TextAlign::Center) if bounds.size.width > approx_w => {
                bounds.origin.x + (bounds.size.width - approx_w) * 0.5
            }
            Some(crate::css::TextAlign::Right | crate::css::TextAlign::End)
                if bounds.size.width > approx_w =>
            {
                bounds.origin.x + bounds.size.width - approx_w
            }
            _ => bounds.origin.x + indent,
        };
        let origin = tej_ui_layout::Point::new(x, y);
        let family = if style.monospace {
            tej_ui_hal::TextFamily::Monospace
        } else {
            tej_ui_hal::TextFamily::SansSerif
        };
        ctx.draw_text_styled(
            text,
            origin,
            style.font_size,
            style.font_weight,
            family,
            style.color,
        );
        let line_thickness = style
            .underline_style
            .filter(|u| u.thickness > 0.0)
            .map_or(1.0, |u| u.thickness);
        let underline_color = style
            .underline_style
            .and_then(|u| u.color)
            .unwrap_or(style.color);
        let underline_offset = style.underline_style.map_or(0.0, |u| u.offset);
        match style.decoration {
            Some(crate::css::TextDecorationLine::Underline) => {
                let line_y = y + style.font_size * 1.05 + underline_offset;
                ctx.draw_line(
                    tej_ui_layout::Point::new(x, line_y),
                    tej_ui_layout::Point::new(x + approx_w, line_y),
                    line_thickness,
                    underline_color,
                );
            }
            Some(crate::css::TextDecorationLine::LineThrough) => {
                let line_y = y + style.font_size * 0.55;
                ctx.draw_line(
                    tej_ui_layout::Point::new(x, line_y),
                    tej_ui_layout::Point::new(x + approx_w, line_y),
                    line_thickness,
                    style.color,
                );
            }
            Some(crate::css::TextDecorationLine::Overline) => {
                ctx.draw_line(
                    tej_ui_layout::Point::new(x, y),
                    tej_ui_layout::Point::new(x + approx_w, y),
                    line_thickness,
                    style.color,
                );
            }
            _ => {}
        }
    }
}

#[derive(Clone, Copy)]
enum LinePosition {
    First,
    Continuation,
}

struct ResolvedTextPaintStyle {
    color: tej_ui_hal::Color,
    font_size: f32,
    font_weight: u16,
    monospace: bool,
    line_height: f32,
    decoration: Option<crate::css::TextDecorationLine>,
    underline_style: Option<crate::typography_extended::TextUnderlineStyle>,
    transform: Option<crate::css::TextTransform>,
    align: Option<crate::css::TextAlign>,
    text_indent: f32,
    ellipsis: bool,
}

impl Text {
    /// Set semibold font weight (`600`).
    #[must_use]
    pub const fn semibold(mut self) -> Self {
        self.font_weight = 600;
        self
    }

    /// Set explicit CSS font weight (e.g. `400`, `500`, `600`, `700`).
    #[must_use]
    pub const fn weight(mut self, weight: u16) -> Self {
        self.font_weight = weight;
        self
    }

    /// Render text using the monospace font family.
    #[must_use]
    pub const fn monospace(mut self) -> Self {
        self.monospace = true;
        self
    }

    /// Create a reactive text widget bound to a signal.
    pub fn reactive(signal: Signal<String>) -> Self {
        Self {
            source: TextSource::Reactive(signal),
            color: None,
            max_width: None,
            font_size: Self::DEFAULT_FONT_SIZE,
            font_weight: 400,
            monospace: false,
        }
    }
}

impl View for Text {
    type State = TextState;

    fn build(&self) -> Self::State {
        let text = match &self.source {
            TextSource::Static(s) => s.clone(),
            TextSource::Reactive(sig) => sig.get(),
        };
        TextState {
            lines: vec![text.clone()],
            current_text: text,
            node: 0,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        match &self.source {
            TextSource::Static(s) => {
                if state.current_text != *s {
                    state.current_text.clone_from(s);
                }
            }
            TextSource::Reactive(sig) => {
                let new_val = sig.get();
                if state.current_text != new_val {
                    state.current_text = new_val;
                }
            }
        }
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        if let TextSource::Reactive(sig) = &self.source {
            let new_val = sig.get();
            if state.current_text != new_val {
                state.current_text = new_val;
            }
        }
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let measurer = shared_measurer();
        let (width, height, lines) = if let Some(max_w) = self.max_width {
            let wrapped = wrap_text(&state.current_text, max_w, self.font_size, measurer);
            (wrapped.width, wrapped.height, wrapped.lines)
        } else if state.current_text.contains('\n') {
            let wrapped = wrap_text(&state.current_text, f32::INFINITY, self.font_size, measurer);
            (wrapped.width, wrapped.height, wrapped.lines)
        } else {
            let family_kind = if self.monospace {
                tej_ui_text::FontFamilyKind::Monospace
            } else {
                tej_ui_text::FontFamilyKind::SansSerif
            };
            let size = measurer.measure_styled(
                &state.current_text,
                self.font_size,
                family_kind,
                self.font_weight,
            );
            (size.width, size.height, vec![state.current_text.clone()])
        };

        state.lines = lines;
        let style = tej_ui_layout::LayoutStyle {
            width: tej_ui_layout::Dimension::Points(width),
            height: tej_ui_layout::Dimension::Points(height),
            ..Default::default()
        };
        let node = arena.allocate_node(style);
        state.node = node;
        node
    }

    fn paint(&self, state: &Self::State, ctx: &mut crate::paint::PaintContext<'_>) {
        let bounds = ctx
            .arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(tej_ui_layout::Rect::ZERO);
        let style = self.resolve_paint_style(ctx);

        if state.lines.len() > 1 {
            for (idx, line) in state.lines.iter().enumerate() {
                let line_idx = f32::from(u16::try_from(idx).unwrap_or(u16::MAX));
                let pos = if idx == 0 {
                    LinePosition::First
                } else {
                    LinePosition::Continuation
                };
                Self::paint_text_line(
                    ctx,
                    bounds,
                    &style,
                    line,
                    bounds.origin.y + line_idx * style.line_height,
                    pos,
                );
            }
        } else {
            let line = state
                .lines
                .first()
                .map_or(state.current_text.as_str(), String::as_str);
            Self::paint_text_line(
                ctx,
                bounds,
                &style,
                line,
                bounds.origin.y,
                LinePosition::First,
            );
        }
    }

    fn accessibility(
        &self,
        state: &Self::State,
        arena: &tej_ui_layout::LayoutArena,
        tree: &mut tej_ui_a11y::A11yTree,
        parent: Option<tej_ui_a11y::A11yId>,
    ) {
        let id = tree.allocate_id();
        let bounds = arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(tej_ui_layout::Rect::ZERO);
        let node = tej_ui_a11y::A11yNode::new(id, tej_ui_a11y::A11yRole::Label)
            .with_label(state.current_text.clone())
            .with_bounds(bounds);
        tree.insert_node(node);
        if let Some(parent_id) = parent {
            if let Some(p) = tree.get_node_mut(parent_id) {
                p.children.push(id);
            }
            tree.mark_dirty(parent_id);
        }
    }
}
