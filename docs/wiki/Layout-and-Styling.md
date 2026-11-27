# Layout and Styling (`tej-ui-layout` & `tej-ui-view`)

`tej-ui` separates geometric layout calculation (`tej-ui-layout`) from visual presentation and CSS styling (`tej-ui-view`).

## 1. Flat Structure-of-Arrays (SoA) Layout Engine

Instead of heap-allocating pointer-linked tree nodes, `LayoutArena` stores styles, parent/child indices, and computed `Rect` bounds in contiguous parallel vectors:

- **`LayoutArena`**: Allocates integer node handles (`u32`) and stores `LayoutStyle` and `computed_bounds`.
- **`LayoutEngine::compute_layout(&mut arena, root, viewport_size)`**: Solves Flexbox and CSS Grid constraints in a cache-friendly pass and populates `arena.computed_bounds`.
- **Supported Layout Primitives**:
  - `DisplayMode` (Flex, Grid, Block, None)
  - `FlexDirection`, `FlexWrap`, `JustifyContent`, `AlignItems`, `JustifyItems`, `JustifySelf`, `PlaceContent`
  - `GridAutoFlow`
  - `Dimension` (`Auto`, `Points(f32)`, `Percent(f32)`), `Size`, `Point`, `Rect`, `Insets`
  - Logical box properties: `LogicalSize`, `LogicalInsets`, `LogicalCornerRadii`
  - Scroll & responsive primitives: `ScrollSnapType`, `ScrollSnapAlign`, `ScrollSnapStop`, `OverscrollBehavior`, `ScrollbarGutter`, `ScrollbarWidth`, `Breakpoint`, `ContainerQuery`, `AnchorPositionArea`, `ResizeMode`.

## 2. Stack & Grid View Containers

At the view layer, compose layouts declaratively with static tuple containers:

```rust
use tej_ui::prelude::*;

fn dashboard_layout() -> impl View {
    VStack::new((
        HStack::new((
            Text::new("Overview"),
            Badge::new("Live"),
        ))
        .spacing(12),
        Grid::new((
            Card::new("Metric A").description("Primary KPI"),
            Card::new("Metric B").description("Secondary KPI"),
        )),
    ))
    .padding(20)
    .spacing(16)
}
```

- `VStack::new(children).spacing(u32).padding(u32)` lays out children vertically.
- `HStack::new(children).spacing(u32).padding(u32)` lays out children horizontally.
- `Grid::new(children)` arranges children in a 2D CSS Grid container.

## 3. Styling with `StyleProps`, `ViewExt`, and `CssStyleSheet`

`tej-ui-view` implements a comprehensive CSS property model (`TOTAL_CSS_PROPERTIES` tracks the supported property set) via `StyleProps`, `Styled`, `CssStyleSheet`, and fluent `ViewExt` modifiers:

- **Visual Properties**: Background/foreground `Color`, border radii, `Shadow` elevations, `CursorIcon`, `TextStyle`, `TextWrap`, `TextUnderlineStyle`, `FontVariantNumeric`, `CaretStyle`.
- **Vector / SVG Paint Styling**: `SvgPaintStyle`, `FillRule`, `StrokeLineCap`, `StrokeLineJoin`, `VectorEffect`.
- **Custom Vector Drawing**: Use `Canvas::new(move |compositor: &mut dyn SurfaceCompositor, bounds: Rect| { ... })` for custom charts, waveforms, or retained 2D graphics.

## 4. Design Tokens & Theming

`Theme` bundles semantic design tokens that widgets automatically read during `paint`:

- `ColorTokens` (`background`, `foreground`, accent, muted, destructive, border)
- `TypographyTokens` (`body`, headings, monospace scales)
- `SpacingTokens`
- `RadiiTokens`
- `ElevationTokens`

Switch or scope themes using `provide_theme(theme)` and read the active theme with `use_theme()`:

```rust
use tej_ui::prelude::*;

provide_theme(Theme::dark());
let active = use_theme();
let bg = active.colors.background;
```
