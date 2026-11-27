# tej-ui

<div align="center">

**A Modular, Reactive, Zero-Copy Cross-Platform UI Framework for Rust**

[![Crates.io](https://img.shields.io/crates/v/tej-ui.svg)](https://crates.io/crates/tej-ui)
[![Docs.rs](https://docs.rs/tej-ui/badge.svg)](https://docs.rs/tej-ui)
[![MSRV](https://img.shields.io/badge/MSRV-1.88.0-blue.svg)](rust-toolchain.toml)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

*Generational Arena Signals • Flat SoA Layout Engine • Compile-Time Static View Trees • Zero-Copy `tej-db` Integration*

</div>

`tej-ui` is a modular Rust UI framework organized as a 19-crate workspace. It combines an 8-byte `Copy` reactive signal graph, a data-oriented Structure-of-Arrays (SoA) Flexbox/Grid layout engine, static compile-time view trees, optional `wgpu` GPU rendering, and direct zero-copy integration with [`tej-db`](https://github.com/YeabsiraBezabih/tej-db).

## Installation

Add `tej-ui` to your `Cargo.toml` (requires **Rust 1.88.0+** / Edition 2024):

```toml
[dependencies]
tej-ui = "0.1.0"
```

To enable zero-copy database synchronization with `tej-db`, 3D viewports, OS services, or runtime devtools, opt into the corresponding feature flags:

```toml
[dependencies]
tej-ui = { version = "0.1.0", features = ["db-sync", "services"] }
```

## Quickstart

```rust
use tej_ui::prelude::*;

fn counter_app() -> impl View {
    let count = Signal::new(0i32);

    // Derive a reactive string with Memo::new and bind it to Text::reactive
    let label_memo = Memo::new(move || format!("Current Count: {}", count.get()));
    let label_signal = Signal::new(label_memo.untracked_get());
    let _sync = create_effect(move || {
        label_signal.set(label_memo.get());
    });

    VStack::new((
        Text::new("Hello from tej-ui!"),
        Text::reactive(label_signal),
        HStack::new((
            Button::new(Text::new("Increment"))
                .on_click(move || count.update(|c| *c += 1)),
            Button::new(Text::new("Decrement"))
                .on_click(move || count.update(|c| *c -= 1)),
        ))
        .spacing(8),
    ))
    .padding(24)
    .spacing(16)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new(counter_app()).window(WindowConfig {
        title: "Counter Application".into(),
        size: Size::new(400.0, 300.0),
        ..Default::default()
    });

    app.run()
}
```

## Feature Flags

| Feature | Enabled by Default | Pulls In | Description |
| :--- | :---: | :--- | :--- |
| `widgets` | Yes | `tej-ui-widgets` | Enterprise widgets (`DataGrid`, `RichTextEditor`, `TreeView`, `FormController`, `CommandPalette`, `Modal`, `Tabs`, `Card`, `Progress`, etc.). |
| `a11y` | Yes | `tej-ui-a11y` | Semantic accessibility tree, `AccessKit` bridge, Web ARIA shadow adapter, and live announcements. |
| `animation` | Yes | `tej-ui-animation` | Spring physics (`AnimatedSpring`), `AnimatedValue`, `Lerp`, and `VsyncTicker`. |
| `icons` | Yes | `tej-ui-icons` | Zero-allocation vector icon primitives and Lucide-compatible icon sets. |
| `db-sync` | No | `tej-ui-db-sync` | Zero-copy `tej-db` ACID database integration (`DbRowLens`, `DirectRleFilter`, `SemanticPalette`, `CowTimeTravelStack`, `VirtualList`). |
| `services` | No | `tej-ui-services` | Clipboard, file dialogs, global shortcuts, system tray, drag-and-drop, and credential store. |
| `3d` | No | `tej-ui-3d` | Retained 3D viewport (`Canvas3D`), software rasterizer, OBJ loader, orbit camera, and meshlet streamer. |
| `devtools` | No | `tej-ui-devtools` | Reactive DAG inspector, dirty-rect HUD overlay, frame metrics, and dynamic hot-reload host. |
| `full` | No | All of the above | Enables `default` plus `services`, `3d`, `db-sync`, and `devtools`. |

Platform backends (`tej-ui-platform-linux`, `tej-ui-platform-windows`, `tej-ui-platform-macos`, `tej-ui-platform-web`) are target-gated automatically via `[target.'cfg(...)'.dependencies]` so OS-specific crates (such as `minifb` on Linux) are never compiled on other platforms.

## Workspace Crates (19 Publishable Crates)

| Crate | Status | Description |
| :--- | :--- | :--- |
| [`tej-ui`](crates/tej-ui) | ![Beta](https://img.shields.io/badge/status-Beta-yellow) | Umbrella facade crate providing `prelude::*`, `App`, and feature-gated re-exports. |
| [`tej-ui-core`](crates/tej-ui-core) | ![Production-Ready](https://img.shields.io/badge/status-Production--Ready-brightgreen) | Generational `SlotMap` arena, 8-byte `Copy` `Signal<T>`, `Memo<T>`, `Effect`, `Resource`, `Store`, and `batch`. |
| [`tej-ui-layout`](crates/tej-ui-layout) | ![Production-Ready](https://img.shields.io/badge/status-Production--Ready-brightgreen) | Flat Structure-of-Arrays (SoA) Flexbox and CSS Grid layout engine (`LayoutArena`, `LayoutEngine`). |
| [`tej-ui-view`](crates/tej-ui-view) | ![Production-Ready](https://img.shields.io/badge/status-Production--Ready-brightgreen) | Compile-time static `View` traits, tuple combinators, CSS stylesheet/tokens, theming, event dispatch, and core primitives. |
| [`tej-ui-hal`](crates/tej-ui-hal) | ![Beta](https://img.shields.io/badge/status-Beta-yellow) | Hardware abstraction layer, `SurfaceCompositor`, `PlatformWindow`, dirty-rect tracking, and optional `wgpu` GPU pipeline. |
| [`tej-ui-text`](crates/tej-ui-text) | ![Beta](https://img.shields.io/badge/status-Beta-yellow) | Unicode BiDi (UAX #9), system font discovery (`fontique`), shaping (`swash`), and measurement (`fontdue`). |
| [`tej-ui-widgets`](crates/tej-ui-widgets) | ![Beta](https://img.shields.io/badge/status-Beta-yellow) | High-level enterprise widget suite (`DataGrid`, `RichTextEditor`, `TreeView`, `FormController`, `CommandPalette`, overlays). |
| [`tej-ui-db-sync`](crates/tej-ui-db-sync) | ![Beta](https://img.shields.io/badge/status-Beta-yellow) | Zero-copy `tej-db` ACID database integration, mmap row lenses, direct RLE filters, RaBitQ semantic palette, and COW undo. |
| [`tej-ui-a11y`](crates/tej-ui-a11y) | ![Beta](https://img.shields.io/badge/status-Beta-yellow) | Accessibility semantic tree, `AccessKit` bridge, Web ARIA shadow adapter, and announcement queue. |
| [`tej-ui-animation`](crates/tej-ui-animation) | ![Beta](https://img.shields.io/badge/status-Beta-yellow) | Spring physics simulations, interruptible value transitions, and vsync pacing. |
| [`tej-ui-icons`](crates/tej-ui-icons) | ![Beta](https://img.shields.io/badge/status-Beta-yellow) | Vector icon geometry primitives and Lucide-compatible icon definitions. |
| [`tej-ui-platform-linux`](crates/tej-ui-platform-linux) | ![Beta](https://img.shields.io/badge/status-Beta-yellow) | Linux windowing and surface presentation backend (`minifb`) with Adwaita CSD helpers. |
| [`tej-ui-platform-windows`](crates/tej-ui-platform-windows) | ![Preview](https://img.shields.io/badge/status-Preview-orange) | Windows Win32, DirectComposition surface, and DWM backdrop backend. |
| [`tej-ui-platform-macos`](crates/tej-ui-platform-macos) | ![Preview](https://img.shields.io/badge/status-Preview-orange) | macOS AppKit windowing and Metal surface compositor backend. |
| [`tej-ui-platform-web`](crates/tej-ui-platform-web) | ![Preview](https://img.shields.io/badge/status-Preview-orange) | WebAssembly Canvas/DOM event bridge and surface compositor backend. |
| [`tej-ui-services`](crates/tej-ui-services) | ![Preview](https://img.shields.io/badge/status-Preview-orange) | Native OS clipboard, file dialogs, global shortcuts, tray icon, drag-and-drop, and credential store. |
| [`tej-ui-devtools`](crates/tej-ui-devtools) | ![Preview](https://img.shields.io/badge/status-Preview-orange) | Runtime reactive DAG inspector, dirty-rect HUD overlay, frame metrics, and hot-reload host. |
| [`tej-ui-3d`](crates/tej-ui-3d) | ![Preview](https://img.shields.io/badge/status-Preview-orange) | Retained 3D viewport (`Canvas3D`), software rasterizer, OBJ parser, orbit controls, and meshlet streamer. |
| [`tej-ui-cli`](crates/tej-ui-cli) | ![Preview](https://img.shields.io/badge/status-Preview-orange) | `tej-ui` CLI binary for project scaffolding and build workflows. |

## Zero-Copy `tej-db` Integration

`tej-ui` pairs natively with [`tej-db`](https://github.com/YeabsiraBezabih/tej-db) (`0.1.0` on crates.io) via the `db-sync` feature:

```toml
[dependencies]
tej-ui = { version = "0.1.0", features = ["db-sync"] }
tej-db = "0.1.0"
```

Key primitives in `tej_ui::prelude::*` when `db-sync` is enabled:
- **`DbRowLens`**: Borrows `&str` and fixed-width binary scalars directly from memory-mapped database pages without cloning strings onto the heap.
- **`VirtualList` & `DbVirtualWindow`**: Materializes only rows visible in the viewport, keeping UI memory proportional to the viewport rather than total dataset size.
- **`DirectRleFilter`**: Evaluates predicates directly over Run-Length Encoded (`RleColumn`) runs into a `FilterBitset`.
- **`SemanticPalette`**: Searches 64-bit 1-bit quantized (RaBitQ) command signatures via hardware popcount.
- **`CowTimeTravelStack`**: Tracks copy-on-write database root page IDs (8 bytes per commit) for instant undo/redo.

See [`examples/task-manager`](examples/task-manager) and the [Database Sync with Tej-DB Wiki](docs/wiki/Database-Sync-with-Tej-DB.md) for full examples.

## Developer Wiki

Explore the complete documentation in [`docs/wiki/`](docs/wiki/Home.md):

- [Wiki Home](docs/wiki/Home.md)
- [Getting Started](docs/wiki/Getting-Started.md)
- [Reactivity Guide](docs/wiki/Reactivity-Guide.md)
- [Layout and Styling](docs/wiki/Layout-and-Styling.md)
- [Widgets and Forms](docs/wiki/Widgets-and-Forms.md)
- [Database Sync with Tej-DB](docs/wiki/Database-Sync-with-Tej-DB.md)
- [Architecture and Rendering](docs/wiki/Architecture-and-Rendering.md)
- [Publishing and Contributing](docs/wiki/Publishing-and-Contributing.md)

## Running Examples & Verification

```bash
# Run headless verification on examples
cargo run -p telemetry-dashboard-example -- --test
cargo run -p task-manager-example -- --headless

# Quality gates
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Contributing & Security

- Read [`CONTRIBUTING.md`](CONTRIBUTING.md) for setup instructions and PR guidelines.
- Read [`SECURITY.md`](SECURITY.md) for vulnerability reporting and crate security boundaries.
- See [`CHANGELOG.md`](CHANGELOG.md) for release notes.

## License

Dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
