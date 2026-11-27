# Architecture and Rendering

`tej-ui` is structured as a 19-crate workspace with a strict acyclic dependency graph (DAG). Each subsystem can be consumed independently or through the unified `tej-ui` umbrella crate.

## 1. Layered Workspace Architecture

```text
Tier 1 (Foundations):
  tej-ui-core       (Reactive arena, Signal, Memo, Effect, Resource, Store)
  tej-ui-layout     (Flat SoA LayoutArena, Flexbox & CSS Grid LayoutEngine)

Tier 2 (HAL, Text & Platform Backends):
  tej-ui-hal        (PlatformWindow, SurfaceCompositor, DirtyRectTracker, optional wgpu)
  tej-ui-text       (Unicode BiDi UAX #9, fontique, fontdue, swash)
  tej-ui-platform-{linux, windows, macos, web}

Tier 3 (View Tree, Accessibility, Animation & Services):
  tej-ui-view       (Static View trait, tuple combinators, CSS/Theme, EventDispatcher, primitives)
  tej-ui-a11y       (AccessKit bridge, Web ARIA shadow adapter, announcements)
  tej-ui-animation  (Spring physics, AnimatedValue, VsyncTicker)
  tej-ui-services   (Clipboard, dialogs, shortcuts, tray, DnD, credentials)

Tier 4 (Domain Subsystems):
  tej-ui-db-sync    (Zero-copy tej-db integration: DbRowLens, RLE filter, RaBitQ, COW undo)
  tej-ui-icons      (Vector icon geometry primitives)
  tej-ui-3d         (Canvas3D, software rasterizer, OBJ loader, Nanite meshlet streamer)
  tej-ui-devtools   (DAG inspector, dirty-rect HUD, frame metrics, hot-reload host)
  tej-ui-widgets    (Enterprise widget library & FormController)

Tier 5 (Facade & Tooling):
  tej-ui            (Umbrella crate, App runner, feature flags, unified prelude)
  tej-ui-cli        (`tej-ui` CLI binary for scaffolding and builds)
```

## 2. Frame Lifecycle (`App::run`)

On every frame, `App` orchestrates a four-stage pipeline:

1. **Event Poll & Dispatch**: `PlatformWindow::poll_events()` yields `UiEvent`s (resize, scroll, mouse, keyboard). `EventDispatcher` hit-tests against `LayoutArena::computed_bounds` and invokes registered handlers.
2. **Reactive Patch**: When signals mutate, `root_view.patch(&mut state, dirty_bits)` updates affected leaf widget state in place without reconstructing the entire view tree.
3. **Relayout (on demand)**: When structure or viewport dimensions change, `LayoutArena` clears and `LayoutEngine::compute_layout` recomputes bounds in a flat SoA pass.
4. **Dirty-Rect Paint & Present**: `DirtyRectTracker` supplies invalidated rectangles to `SurfaceCompositor::begin_frame(rects)`. `root_view.paint(&state, &mut paint_ctx)` draws into the surface, followed by `end_frame()` and `window.present(&surface)`.

## 3. Hardware Abstraction Layer (`tej-ui-hal`)

`tej-ui-hal` decouples rendering from OS windowing through two core traits:

- **`PlatformWindow`**: Window creation (`WindowConfig`), event polling, sizing, and frame presentation.
- **`SurfaceCompositor`**: 2D drawing primitives (`fill_rect`, text, vector paths) and dirty-region frame framing.

The `gpu` feature (enabled by default on `tej-ui-hal`) activates `wgpu`, `bytemuck`, `pollster`, and `lyon_*` vector tessellation. Disable it (`default-features = false`) for minimal software-only or headless environments.

## 4. Platform Target Gating

In `crates/tej-ui/Cargo.toml`, platform backends are gated by target configuration:

- `cfg(target_os = "linux")` → `tej-ui-platform-linux` (uses `minifb`)
- `cfg(target_os = "windows")` → `tej-ui-platform-windows`
- `cfg(target_os = "macos")` → `tej-ui-platform-macos`
- `cfg(target_arch = "wasm32")` → `tej-ui-platform-web`

This guarantees that Linux-specific dependencies such as `minifb` are never pulled when compiling for Windows, macOS, or WebAssembly.

## 5. Typography, Accessibility & 3D

- **Typography (`tej-ui-text`)**: Handles right-to-left and mixed-script text via `unicode-bidi`, system font enumeration via `fontique`, glyph shaping via `swash`, and rasterization/measurement via `fontdue`.
- **Accessibility (`tej-ui-a11y`)**: Maintains an `A11yTree` of `A11yNode`s and synchronizes with desktop assistive technologies via `AccessKitBridge` or web browsers via `WebAriaAdapter`.
- **3D Viewports (`tej-ui-3d`)**: Embeds `Canvas3D` inside standard 2D layouts with `PerspectiveCamera`, `OrbitControls`, OBJ loading (`parse_obj`), and meshlet streaming (`NaniteRenderEngine`, `NaniteStreamer`).
