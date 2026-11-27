# Getting Started with `tej-ui`

This guide walks through installing `tej-ui`, choosing feature flags, building your first interactive application, and running headless UI tests.

## Prerequisites

- **Rust**: `1.85.0` or newer (Edition 2024). The workspace pins `1.85.0` in [`rust-toolchain.toml`](../../rust-toolchain.toml).
- **Linux System Libraries** (required by `minifb` when building on Linux):
  ```bash
  sudo apt-get update && sudo apt-get install -y libxkbcommon-dev libx11-dev libwayland-dev pkg-config
  ```
- **macOS / Windows**: Standard Xcode Command Line Tools (macOS) or MSVC Build Tools (Windows). Platform dependencies are automatically target-gated in `tej-ui/Cargo.toml`.

## Installation

Create a new binary crate and add `tej-ui`:

```bash
cargo new hello-tej
cd hello-tej
```

In `Cargo.toml`:

```toml
[package]
name = "hello-tej"
version = "0.1.0"
edition = "2024"

[dependencies]
tej-ui = "0.1.0"
```

### Selecting Feature Flags

By default, `tej-ui` enables `widgets`, `a11y`, `animation`, and `icons`.

- **Minimal core only** (signals, static views, layout, text, HAL):
  ```toml
  tej-ui = { version = "0.1.0", default-features = false }
  ```
- **With `tej-db` zero-copy database sync**:
  ```toml
  tej-ui = { version = "0.1.0", features = ["db-sync"] }
  tej-db = "0.1.0"
  ```
- **Everything enabled** (adds `services`, `3d`, `db-sync`, `devtools`):
  ```toml
  tej-ui = { version = "0.1.0", features = ["full"] }
  ```

## Your First Application

Replace `src/main.rs` with a reactive counter app using `tej_ui::prelude::*`:

```rust
use tej_ui::prelude::*;

fn reactive_text(memo: Memo<String>) -> Text {
    let sig = Signal::new(memo.untracked_get());
    let _effect = create_effect(move || sig.set(memo.get()));
    Text::reactive(sig)
}

fn counter_view() -> impl View {
    let count = Signal::new(0i32);
    let label = Memo::new(move || format!("Count: {}", count.get()));

    VStack::new((
        Text::new("tej-ui Counter"),
        reactive_text(label),
        HStack::new((
            Button::new(Text::new("-1"))
                .variant(ButtonVariant::Outline)
                .on_click(move || count.update(|c| *c -= 1)),
            Button::new(Text::new("+1"))
                .variant(ButtonVariant::Default)
                .on_click(move || count.update(|c| *c += 1)),
            Button::new(Text::new("Reset"))
                .variant(ButtonVariant::Ghost)
                .on_click(move || count.set(0)),
        ))
        .spacing(8),
    ))
    .padding(24)
    .spacing(16)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new(counter_view()).window(WindowConfig {
        title: "Hello tej-ui".into(),
        size: Size::new(420.0, 260.0),
        ..Default::default()
    });

    app.run()
}
```

Run it:

```bash
cargo run
```

## Headless Rendering & Automated Testing

`App::run_headless_frame()` executes the full build → layout → paint pipeline without blocking on an interactive event loop, which is ideal for CI smoke tests:

```rust
let state = App::new(counter_view())
    .window(WindowConfig {
        title: "Headless Test".into(),
        size: Size::new(800.0, 600.0),
        ..Default::default()
    })
    .run_headless_frame();
```

For unit-testing views and inspecting rendered trees, `tej-ui-view` also exports `TestHarness` and `TestNode` in `tej_ui::prelude::*`.

## Next Steps

- Learn how signals, memos, and batching work in the **[Reactivity Guide](Reactivity-Guide.md)**.
- Explore Flexbox, Grid, and design tokens in **[Layout and Styling](Layout-and-Styling.md)**.
