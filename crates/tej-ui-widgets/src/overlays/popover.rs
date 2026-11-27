use crate::overlays::collision::{
    CollisionPlacementOptions, OverlayPlacementResult, PopoverAlignment,
    compute_collision_placement,
};
use std::fmt::Debug;
use std::sync::Arc;
use tej_ui_core::Signal;
use tej_ui_layout::{Rect, Size};
use tej_ui_view::View;

/// Positioning alignment for floating popovers.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PopoverPlacement {
    #[default]
    Bottom,
    Top,
    Left,
    Right,
}

/// Floating popover overlay anchored to an interactive target element.
#[derive(Clone, Debug)]
pub struct Popover<Target: View, Content: View> {
    is_open: Signal<bool>,
    placement: PopoverPlacement,
    alignment: PopoverAlignment,
    offset: f32,
    viewport_padding: f32,
    auto_flip: bool,
    viewport: Option<Rect>,
    anchor_bounds: Option<Rect>,
    overlay_size: Option<Size>,
    target: Target,
    content: Content,
}

impl<Target: View, Content: View> Popover<Target, Content> {
    /// Create a new popover anchored to `target`.
    pub fn new(is_open: Signal<bool>, target: Target, content: Content) -> Self {
        Self {
            is_open,
            placement: PopoverPlacement::Bottom,
            alignment: PopoverAlignment::Center,
            offset: 8.0,
            viewport_padding: 8.0,
            auto_flip: true,
            viewport: None,
            anchor_bounds: None,
            overlay_size: None,
            target,
            content,
        }
    }

    /// Set preferred popover placement relative to anchor.
    pub fn placement(mut self, placement: PopoverPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Set cross-axis alignment relative to anchor.
    #[must_use]
    pub const fn alignment(mut self, alignment: PopoverAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Set main-axis gap between anchor and popover.
    #[must_use]
    pub const fn offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    /// Set minimum clearance from viewport edges.
    #[must_use]
    pub const fn viewport_padding(mut self, padding: f32) -> Self {
        self.viewport_padding = padding;
        self
    }

    /// Enable or disable collision-aware auto-flipping.
    #[must_use]
    pub const fn auto_flip(mut self, auto_flip: bool) -> Self {
        self.auto_flip = auto_flip;
        self
    }

    /// Configure explicit viewport bounds used for collision detection and flipping.
    #[must_use]
    pub const fn viewport(mut self, viewport: Rect) -> Self {
        self.viewport = Some(viewport);
        self
    }

    /// Configure explicit anchor bounds for pre-layout collision resolution.
    #[must_use]
    pub const fn anchor_bounds(mut self, anchor: Rect) -> Self {
        self.anchor_bounds = Some(anchor);
        self
    }

    /// Configure explicit overlay size hint for collision calculation.
    #[must_use]
    pub const fn overlay_size(mut self, size: Size) -> Self {
        self.overlay_size = Some(size);
        self
    }

    /// Build `CollisionPlacementOptions` from this popover's configuration.
    #[must_use]
    pub const fn collision_options(&self) -> CollisionPlacementOptions {
        CollisionPlacementOptions {
            preferred_placement: self.placement,
            alignment: self.alignment,
            offset: self.offset,
            viewport_padding: self.viewport_padding,
            auto_flip: self.auto_flip,
        }
    }

    /// Compute collision-aware placement for the popover relative to `anchor` within `viewport`.
    #[must_use]
    pub fn resolve_placement(
        &self,
        anchor: Rect,
        overlay_size: Size,
        viewport: Rect,
    ) -> OverlayPlacementResult {
        compute_collision_placement(anchor, overlay_size, viewport, self.collision_options())
    }
}

/// State of a mounted Popover.
#[derive(Debug)]
pub struct PopoverState<T: Debug, C: Debug> {
    pub is_open: bool,
    pub target_state: T,
    pub content_state: Option<C>,
    pub node: u32,
    pub target_node: u32,
    pub content_node: Option<u32>,
    pub resolved_placement: PopoverPlacement,
    pub is_flipped: bool,
    pub content_rect: Option<Rect>,
}

impl<Target: View, Content: View> View for Popover<Target, Content> {
    type State = PopoverState<Target::State, Content::State>;

    fn build(&self) -> Self::State {
        let open = self.is_open.get();
        PopoverState {
            is_open: open,
            target_state: self.target.build(),
            content_state: if open {
                Some(self.content.build())
            } else {
                None
            },
            node: 0,
            target_node: 0,
            content_node: None,
            resolved_placement: self.placement,
            is_flipped: false,
            content_rect: None,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        self.target.rebuild(&mut state.target_state);
        let open = self.is_open.get();
        if open != state.is_open {
            state.is_open = open;
            state.content_state = if open {
                Some(self.content.build())
            } else {
                None
            };
        } else if let Some(cs) = state.content_state.as_mut() {
            self.content.rebuild(cs);
        }
    }

    fn patch(&self, state: &mut Self::State, dirty_bits: u64) {
        self.target.patch(&mut state.target_state, dirty_bits);
        let open = self.is_open.get();
        if open != state.is_open {
            self.rebuild(state);
        } else if let Some(cs) = state.content_state.as_mut() {
            self.content.patch(cs, dirty_bits);
        }
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        // Compute collision-aware placement if anchor & viewport are known before layout
        let effective_placement =
            if let (Some(anchor), Some(viewport)) = (self.anchor_bounds, self.viewport) {
                let overlay_sz = self.overlay_size.unwrap_or(Size::new(200.0, 120.0));
                let res = self.resolve_placement(anchor, overlay_sz, viewport);
                state.resolved_placement = res.actual_placement;
                state.is_flipped = res.is_flipped;
                state.content_rect = Some(res.rect);
                res.actual_placement
            } else {
                state.resolved_placement = self.placement;
                state.is_flipped = false;
                self.placement
            };

        let direction = match effective_placement {
            PopoverPlacement::Bottom | PopoverPlacement::Top => {
                tej_ui_layout::FlexDirection::Column
            }
            PopoverPlacement::Left | PopoverPlacement::Right => tej_ui_layout::FlexDirection::Row,
        };
        let root_style = tej_ui_layout::LayoutStyle {
            direction,
            spacing: self.offset,
            ..Default::default()
        };
        let root = arena.allocate_node(root_style);
        state.node = root;

        let tn = self.target.layout(&mut state.target_state, arena);
        state.target_node = tn;

        if state.is_open {
            if let Some(cs) = state.content_state.as_mut() {
                let cn = self.content.layout(cs, arena);
                state.content_node = Some(cn);
                match effective_placement {
                    PopoverPlacement::Top | PopoverPlacement::Left => {
                        arena.add_child(root, cn);
                        arena.add_child(root, tn);
                    }
                    PopoverPlacement::Bottom | PopoverPlacement::Right => {
                        arena.add_child(root, tn);
                        arena.add_child(root, cn);
                    }
                }
            } else {
                arena.add_child(root, tn);
            }
        } else {
            arena.add_child(root, tn);
            state.content_node = None;
        }

        root
    }

    fn paint(&self, state: &Self::State, ctx: &mut tej_ui_view::PaintContext<'_>) {
        self.target.paint(&state.target_state, ctx);
        if state.is_open {
            let anchor = self
                .anchor_bounds
                .or_else(|| {
                    ctx.arena
                        .computed_bounds
                        .get(state.target_node as usize)
                        .copied()
                })
                .unwrap_or(Rect::ZERO);

            let content_bounds = state
                .content_node
                .and_then(|cn| ctx.arena.computed_bounds.get(cn as usize).copied());

            let overlay_size = self.overlay_size.unwrap_or_else(|| {
                content_bounds
                    .map(|r| Size::new(r.size.width.max(120.0), r.size.height.max(40.0)))
                    .unwrap_or(Size::new(200.0, 120.0))
            });

            let viewport = self
                .viewport
                .or_else(|| ctx.arena.computed_bounds.first().copied())
                .unwrap_or(Rect::new(0.0, 0.0, 1280.0, 720.0));

            let placement_res = self.resolve_placement(anchor, overlay_size, viewport);
            let panel = placement_res.rect;

            // Paint collision-placed floating popover surface with border
            ctx.fill_rounded_rect(panel, ctx.theme.radii.md, ctx.theme.colors.border);
            let inner = Rect::new(
                panel.origin.x + 1.0,
                panel.origin.y + 1.0,
                (panel.size.width - 2.0).max(0.0),
                (panel.size.height - 2.0).max(0.0),
            );
            ctx.fill_rounded_rect(inner, ctx.theme.radii.md, ctx.theme.colors.popover);

            if let Some(cs) = state.content_state.as_ref() {
                self.content.paint(cs, ctx);
            }
        }
    }

    fn register_events(&self, state: &Self::State, dispatcher: &mut tej_ui_view::EventDispatcher) {
        self.target.register_events(&state.target_state, dispatcher);
        if state.is_open {
            let is_open_esc = self.is_open;
            dispatcher.register_key_down(
                state.node,
                Arc::new(move |key_code| {
                    if key_code == 27 {
                        is_open_esc.set(false);
                    }
                }),
            );
            if let Some(cs) = state.content_state.as_ref() {
                self.content.register_events(cs, dispatcher);
            }
        }
    }
}
