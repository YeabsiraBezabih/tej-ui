# The Definitive `tej-ui` Developer Guide: Idiomatic Patterns & Production Best Practices

This guide establishes the official architectural standards, mental models, and production idioms for building high-performance, accessible, and glitch-free native applications with `tej-ui`.

---

## 1. The `tej-ui` Mental Model

Unlike immediate-mode frameworks (which redraw the whole world every frame) or Virtual DOM frameworks (which allocate and diff heap trees), `tej-ui` operates on three distinct, decoupled lifecycles:

1. **Mount (Compile-Time View Tree Construction)**:
   - Views implement the zero-vtable `View` trait. Combinators like `VStack`, `HStack`, `(A, B, C)`, and `If` resolve at compile-time into static structs.
   - **Rule**: Never construct dynamic `Box<dyn View>` allocations unless strictly necessary for polymorphic plugin boundaries.

2. **Reactivity (Fine-Grained Signal Propagation)**:
   - Signals (`Signal<T>`) are 8-byte `Copy` handles. Reading a signal inside a view automatically registers the containing sub-tree as a subscriber.
   - Mutating a signal (`signal.set(val)` or `signal.update(|v| ...)`) invalidates **only** the affected node in $O(1)$ time, bypassing full-tree diffing.

3. **Steady-State Paint (Dirty-Region Compositing)**:
   - When state changes, only the dirty bounding rectangles are pushed to the OS compositor or GPU surface. An idle application burns 0% CPU.

---

## 2. Reactive State Architecture

### A. Local Component State (`Signal<T>`)
Use `Signal::new()` for fine-grained scalar values, toggles, form fields, and selection cursors:

```rust
// GOOD: Granular, 8-byte Copy handles
let is_open = Signal::new(false);
let count = Signal::new(0usize);

// Mutate in-place without heap reallocation:
count.update(|c| *c += 1);
```

### B. Derived Computations (`Memo<T>`)
Use `Memo::new()` for expensive calculations, filtered lists, or formatted labels:

```rust
// GOOD: Automatically cached until `items` or `filter` changes
let filtered_items = Memo::new(move || {
    let query = search_query.get().to_lowercase();
    items.get().into_iter().filter(|item| item.contains(&query)).collect::<Vec<_>>()
});
```

### C. Dependency Injection (`ContextProvider` & `use_context`)
Avoid prop-drilling across deep component trees by providing typed contexts:

```rust
// Provider at application or window root:
ContextProvider::new(theme_signal, app_view)

// Consumer anywhere downstream:
let theme = use_context::<Signal<Theme>>().unwrap();
```

### D. Background & Async Computations (`WorkerBridge`)
Never block the UI thread with heavy CPU tasks (e.g. image processing, large JSON parsing, or cryptography):

```rust
let worker = WorkerBridge::spawn(|rx, tx| {
    while let Ok(task) = rx.recv() {
        let result = compute_heavy_task(task);
        tx.send(result).unwrap();
    }
});
```

---

## 3. Layout Best Practices (`tej-layout`)

`tej-layout` is a cache-coherent, flat Structure-of-Arrays (SoA) Flexbox and Grid engine. Follow these rules to guarantee zero-allocation layout passes:

### Rule 1: Use Fixed or Percentage Constraints
Avoid unconstrained nesting where parents and children both use `Dimension::Auto`:
```rust
// GOOD: Deterministic, cache-friendly sizing
HStack::new((
    Sidebar::new().width(240.0),
    ContentPane::new().flex_grow(1.0),
))
```

### Rule 2: Embrace Container Queries for Responsive UIs
Instead of querying the global window size, use container queries to make components adapt to their immediate parent container:
```rust
let mut style = LayoutStyle::default();
style.container_query = Some(ContainerQuery {
    min_width: Some(768.0),
    direction_override: Some(FlexDirection::Row),
    ..Default::default()
});
```

---

## 4. Virtualization & Zero-Copy Database State (`tej-db-sync`)

When displaying datasets exceeding 1,000 rows (up to 100,000,000 rows):

### Rule 1: Never Clone Database Rows into UI State
Do not convert database records into `Vec<Row>` heap structures:

```rust
// BAD: Duplicates database storage into UI heap (High RSS, slow cold start)
let rows: Vec<MyRow> = db.query_all(); 

// GOOD: Direct memory-mapped page-slice pinning via DbRowLens
let vlist = VirtualList::new(100_000_000, 36.0, 1080.0, scroll_pos, move |idx| {
    let rx = db.begin_read().unwrap();
    let lens = rx.get_row_lens(idx).unwrap();
    Text::new(lens.slice_str(0, 32).unwrap_or(""))
});
```

### Rule 2: Use Virtual Windowing
Only construct view nodes for rows currently visible in the active viewport:
```rust
let mut window = DbVirtualWindow::new(total_rows, row_height, viewport_height);
window.update_scroll(scroll_y, row_height, viewport_height);
for idx in window.start_index()..window.end_index() {
    // Render visible row
}
```

---

## 5. Typography, BiDi & Text Input (`tej-text`)

1. **Font Categories**: Use semantic categories (`SystemFont::Default`, `SystemFont::Monospace`, `SystemFont::Serif`) so the OS font fallback chain automatically discovers platform fonts (San Francisco on macOS, Segoe UI on Windows, Cantarell/Noto on Linux).
2. **BiDi & Complex Scripts**: When displaying mixed-direction text (Arabic, Hebrew, CJK, Ge'ez), `tej-text` automatically applies Unicode Bidirectional Algorithm (UAX #9) and HarfBuzz cluster shaping.
3. **IME Composition**: Text inputs integrate natively with platform Input Method Editors (IME) to support dead keys, Japanese kana-kanji, and Chinese pinyin composition without glitches.

---

## 6. Accessibility & Keyboard Navigation (`tej-a11y`)

Every production `tej-ui` application must adhere to WCAG 2.1 AA accessibility:

1. **Semantic Roles**: Assign appropriate accessibility roles (`Role::Button`, `Role::CheckBox`, `Role::Table`, `Role::TextInput`) to custom widgets.
2. **Keyboard Focus Ring**: Ensure all interactive controls respond to `Tab` and `Shift+Tab` traversals and render platform focus rings.
3. **Screen Reader Announcements**: Use `A11yAnnouncer::announce("Operation completed", LiveRegion::Polite)` for dynamic state changes.

---

## 7. Quality Gates & Testing Checklist

Before deploying any `tej-ui` application:

1. **Verify No Allocations during Steady-State Frame Ticks**:
   - Use `TrackingAllocator` to assert `heap_alloc_count == 0` during cursor blinks and hover states.
2. **Confirm Zero Warnings**:
   - `cargo clippy --all-targets --all-features` must pass with zero warnings.
3. **Format Compliance**:
   - `cargo fmt --all -- --check` must be clean.
4. **Integration Testing**:
   - Use `TestHarness` from `tej_view::testing` to simulate mouse clicks, keyboard shortcuts, and snapshot assertions headless without opening a physical window.
