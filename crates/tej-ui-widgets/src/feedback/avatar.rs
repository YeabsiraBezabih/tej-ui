//! User profile avatar with initials fallback (shadcn `Avatar`).

use tej_ui_layout::{Point, Rect, Size};
use tej_ui_view::View;

/// Retained state for a mounted Avatar.
#[derive(Debug, Default)]
pub struct AvatarState {
    pub bounds: Rect,
    pub initials: String,
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
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.initials = self.initials();
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
