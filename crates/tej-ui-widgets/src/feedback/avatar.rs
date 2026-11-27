//! User profile avatar with initials fallback (shadcn `Avatar`).

use tej_ui_layout::{Dimension, LayoutArena, LayoutStyle, Point, Rect, Size};
use tej_ui_view::{PaintContext, View};

/// Retained state for a mounted Avatar.
#[derive(Debug, Default)]
pub struct AvatarState {
    pub bounds: Rect,
    pub initials: String,
    pub node: u32,
}

/// Circular user profile avatar image with fallback text initials.
#[derive(Debug, Clone)]
pub struct Avatar {
    pub name: String,
    pub image_url: Option<String>,
    pub size: f32,
}

impl Avatar {
    /// Creates an avatar from a user name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            image_url: None,
            size: 40.0,
        }
    }

    /// Sets optional remote or local image source.
    #[must_use]
    pub fn image(mut self, url: impl Into<String>) -> Self {
        self.image_url = Some(url.into());
        self
    }

    /// Sets circular diameter.
    #[must_use]
    pub const fn diameter(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Extracts two-letter initials from name.
    #[must_use]
    pub fn initials(&self) -> String {
        let parts: Vec<&str> = self.name.split_whitespace().collect();
        match parts.as_slice() {
            [first, second, ..] => {
                let c1 = first.chars().next().unwrap_or('?');
                let c2 = second.chars().next().unwrap_or('?');
                format!("{}{}", c1.to_uppercase(), c2.to_uppercase())
            }
            [single] => single.chars().take(2).collect::<String>().to_uppercase(),
            [] => "?".to_string(),
        }
    }
}

impl View for Avatar {
    type State = AvatarState;

    fn build(&self) -> Self::State {
        AvatarState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(self.size, self.size)),
            initials: self.initials(),
            node: 0,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.initials = self.initials();
        state.bounds.size = Size::new(self.size, self.size);
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut LayoutArena) -> u32 {
        let style = LayoutStyle {
            width: Dimension::Points(self.size),
            height: Dimension::Points(self.size),
            ..Default::default()
        };
        let node = arena.allocate_node(style);
        state.node = node;
        node
    }

    fn paint(&self, state: &Self::State, ctx: &mut PaintContext<'_>) {
        let bounds = ctx
            .arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(state.bounds);

        let diameter = bounds.size.width.min(bounds.size.height);
        let radius = diameter * 0.5;
        let circle = Rect::new(bounds.origin.x, bounds.origin.y, diameter, diameter);

        // Outer subtle border ring + circular avatar fill
        ctx.fill_rounded_rect(circle, radius, ctx.theme.colors.border);
        let inner = Rect::new(
            circle.origin.x + 1.0,
            circle.origin.y + 1.0,
            (diameter - 2.0).max(0.0),
            (diameter - 2.0).max(0.0),
        );
        ctx.fill_rounded_rect(inner, (radius - 1.0).max(0.0), ctx.theme.colors.secondary);

        // Centered initials text
        let font_size = (diameter * 0.4).clamp(10.0, 24.0);
        let approx_text_w = (state.initials.chars().count() as f32) * font_size * 0.6;
        let text_pt = Point::new(
            circle.origin.x + (diameter - approx_text_w) * 0.5,
            circle.origin.y + (diameter - font_size) * 0.5,
        );
        ctx.draw_text(
            &state.initials,
            text_pt,
            font_size,
            ctx.theme.colors.secondary_foreground,
        );
    }
}
