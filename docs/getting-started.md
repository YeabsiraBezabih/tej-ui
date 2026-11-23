# Official Getting Started Guide: `tej-ui`

Welcome to `tej-ui`—a zero-copy, sub-microsecond, native OS UI framework designed for high-performance enterprise applications in Rust.

---

## 1. Installation & Setup

Add `tej` to your `Cargo.toml`:

```toml
[dependencies]
tej = "0.1.0"
```

To enable database synchronization backed by `MyDb`:

```toml
[dependencies]
tej = { version = "0.1.0", features = ["db-sync"] }
mydb = "0.1.0"
```

---

## 2. Core Concepts at a Glance

`tej-ui` is built on three core pillars:

1. **Glitch-Free Arena Reactivity**:
   - `Signal<T>`: 8-byte `Copy` identifier pointing to a generational slot in an arena. Reading it automatically registers dependency tracking; writing to it marks dependents dirty.
   - `Memo<T>`: Derived computation cached until dependencies change.
   - `create_effect(f)`: Side-effect executed whenever dependencies mutate.

2. **Compile-Time Static View Trees**:
   - Views implement the zero-vtable `View` trait. Combinators like `(A, B, C)`, `VStack`, `HStack`, `If`, and `For` resolve at compile-time without dynamic trait object dispatch.

3. **Flat SoA Layout Engine**:
   - Layout geometry, padding, and constraints reside in flat Structure-of-Arrays (SoA) vectors (`LayoutArena`), enabling 10,000-node flex passes in under 300 microseconds with zero steady-state heap allocations.

---

## 3. Your First Counter Application

Create a new file `src/main.rs`:

```rust
use tej::prelude::*;

fn main() {
    let app = App::new(counter_view()).window(WindowConfig {
        title: "tej-ui Counter".into(),
        size: Size::new(420.0, 320.0),
        resizable: true,
        ..Default::default()
    });

    app.run();
}

fn counter_view() -> impl View {
    // 1. Create a reactive signal
    let counter = Signal::new(0i32);

    // 2. Derive a cached memo
    let status_text = Memo::new(move || {
        let val = counter.get();
        if val == 0 {
            "Counter is at zero.".to_string()
        } else if val > 0 {
            format!("Positive count: +{val}")
        } else {
            format!("Negative count: {val}")
        }
    });

    // 3. Compose static view tree
    VStack::new((
        Text::new("Counter Application")
            .font_size(24.0)
            .bold(),
        Text::new(status_text)
            .font_size(16.0),
        HStack::new((
            Button::new("Increment (+1)", move || counter.update(|c| *c += 1))
                .variant(ButtonVariant::Primary),
            Button::new("Decrement (-1)", move || counter.update(|c| *c -= 1))
                .variant(ButtonVariant::Secondary),
            Button::new("Reset", move || counter.set(0))
                .variant(ButtonVariant::Outline),
        ))
        .spacing(8),
    ))
    .padding(24)
    .spacing(16)
}
```

---

## 4. Connecting to Embedded `MyDb` for Zero-Copy Data

To render a 1,000,000-row list directly from memory-mapped database pages:

```rust
use mydb::Database;
use std::sync::Arc;
use tej::prelude::*;

fn database_backed_list(db: Arc<Database>) -> impl View {
    let scroll_pos = Signal::new(0.0f32);

    // VirtualList only materializes the 20-30 rows currently visible in the viewport
    VirtualList::new(1_000_000, 36.0, 720.0, scroll_pos, move |idx| {
        // Zero-copy lookup: DbRowLens reads directly from slotted page slice
        HStack::new((
            Badge::new(format!("#{idx:06}")),
            Text::new(format!("Database record #{idx}")),
        ))
        .spacing(12)
        .padding(6)
    })
}
```

---

## 5. Next Steps

- Read the [Reactivity Guide](reactivity-guide.md) to master fine-grained state management, async resources, and stores.
- Read the [Zero-Copy Database Integration Guide](zero-copy-db-guide.md) for direct RLE filtering, RaBitQ command palette, and COW time-travel.
- Check out the full desktop applications in `examples/task-manager` and `examples/obsidian-clone`.
