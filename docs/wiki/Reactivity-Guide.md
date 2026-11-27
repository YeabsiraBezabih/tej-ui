# Reactivity Guide (`tej-ui-core`)

`tej-ui` uses a **generational arena reactive graph** inspired by fine-grained signal runtimes. Instead of cloning `Arc<Mutex<T>>` handles or diffing a Virtual DOM, reactive state is accessed through lightweight 8-byte `Copy` handles (`Signal<T>`, `ReadSignal<T>`, `WriteSignal<T>`, `Memo<T>`).

## 1. Signals (`Signal<T>`)

A `Signal<T>` stores its value inside the runtime's generational `SlotMap` arena and exposes a `Copy` handle that can be moved into multiple closures without `.clone()`:

```rust
use tej_ui::prelude::*;

let count = Signal::new(0i32);

// Read and track dependency (when inside a Memo or Effect)
assert_eq!(count.get(), 0);

// Overwrite value and notify dependents
count.set(10);

// Mutate in place
count.update(|c| *c += 5);
assert_eq!(count.get(), 15);

// Read without registering a dependency
let snapshot = count.untracked_get();
```

### Splitting Read and Write Capabilities

Use `Signal::split` to pass read-only handles to child views while keeping write access private:

```rust
let (read_count, write_count) = count.split();
write_count.set(42);
assert_eq!(read_count.get(), 42);
```

## 2. Derived Values (`Memo<T>`)

`Memo<T>` computes a derived value and automatically tracks every `Signal` or `Memo` read via `.get()` inside its closure. Because `tej-ui-core` evaluates updates in topological DAG order, diamond dependencies never observe intermediate "glitches":

```rust
let price = Signal::new(100.0_f64);
let qty = Signal::new(2_u32);

let subtotal = Memo::new(move || price.get() * f64::from(qty.get()));
let formatted = Memo::new(move || format!("${:.2}", subtotal.get()));

assert_eq!(formatted.get(), "$200.00");
qty.set(3);
assert_eq!(formatted.get(), "$300.00");
```

### Binding `Memo<String>` to `Text::reactive`

`Text::reactive` accepts a `Signal<String>`. To bind a derived `Memo<String>` to `Text::reactive`, bridge it with `create_effect`:

```rust
fn reactive_text(memo: Memo<String>) -> Text {
    let sig = Signal::new(memo.untracked_get());
    let _sync = create_effect(move || sig.set(memo.get()));
    Text::reactive(sig)
}
```

> **Avoid one-shot snapshots**: Never write `Text::reactive(Signal::new(format!("{}", count.get())))` inside a view builder — that reads `count` once at construction time and creates an isolated signal that will never update.

## 3. Side Effects (`Effect` & `create_effect`)

`create_effect` runs its closure immediately to subscribe to any tracked signals and re-runs whenever those dependencies change. Effects remain active until explicitly disposed via `effect.dispose()`:

```rust
let theme_dark = Signal::new(true);
let effect = create_effect(move || {
    println!("Dark mode is now: {}", theme_dark.get());
});

theme_dark.set(false); // re-runs effect
effect.dispose();      // unsubscribes from DAG
```

## 4. Batching Updates (`batch`)

When updating multiple related signals together (such as a telemetry tick or form reset), wrap mutations in `batch(|| { ... })` so downstream memos and effects execute once after all writes settle:

```rust
batch(|| {
    price.set(125.0);
    qty.set(4);
});
// `subtotal` and `formatted` recompute once for the combined update.
```

## 5. Structured State: `Store` and `Resource`

- **`Store<S, A>`**: Reducer-driven state container (`Store::new(initial_state, reducer)`) suitable for complex state machines and predictable action dispatch.
- **`Resource<T>`**: Represents asynchronous or deferred data loading with explicit `ResourceState` transitions (`Loading`, `Ready`, `Error`), usable with `Resource::new` or `Resource::new_with_source(source_signal, fetcher)`.

## 6. Context Propagation

Pass shared services or themes down a subtree without prop drilling using `provide_context`, `use_context`, `with_context`, or the `ContextProvider` view wrapper:

```rust
provide_context(Theme::dark());
if let Some(theme) = use_context::<Theme>() {
    let _bg = theme.colors.background;
}
```
