# Database Sync with `tej-db` (`tej-ui-db-sync`)

`tej-ui-db-sync` integrates `tej-ui` directly with the [`tej-db`](https://github.com/YeabsiraBezabih/tej-db) embedded ACID storage engine (`tej-db = "0.1.0"`, `tej-db-core = "0.1.0"` on crates.io).

Enable it via the `db-sync` feature flag:

```toml
[dependencies]
tej-ui = { version = "0.1.0", features = ["db-sync"] }
tej-db = "0.1.0"
```

## 1. Zero-Copy Row Lens (`DbRowLens`)

Rather than copying every database row into heap-allocated `String` and `Vec` structs, `DbRowLens` borrows key/value slices directly from a `tej-db` read transaction:

```rust
use tej_ui::prelude::*;

fn inspect_row(raw_key: &[u8], raw_value: &[u8]) {
    let lens = DbRowLens::new(raw_key, raw_value);
    if let Ok(key_text) = lens.key_str() {
        let _id = lens.read_u64_le(0);
        let _price = lens.read_f32_le(8);
        let _ = key_text;
    }
}
```

## 2. Virtualized Lists (`VirtualList` & `DbVirtualWindow`)

`VirtualList` only materializes views for rows currently intersecting the scroll viewport, keeping memory usage proportional to visible rows rather than total dataset size:

```rust
use tej_ui::prelude::*;

fn task_list(scroll_pos: Signal<f32>) -> impl View {
    VirtualList::new(1_000_000, 36.0, 720.0, scroll_pos, move |idx| {
        HStack::new((
            Checkbox::new(Signal::new(idx % 2 == 0)),
            Text::new(format!("Task #{idx:06}")),
        ))
        .spacing(12)
    })
}
```

Use `DbSyncHook` and `ReactiveDbQuery` to invalidate reactive UI queries when database write transactions commit.

## 3. Direct RLE Columnar Filtering (`DirectRleFilter`)

For status flags, tags, and low-cardinality columns stored as Run-Length Encoded runs (`RleColumn`), `DirectRleFilter` evaluates equality predicates directly over compressed runs into a `FilterBitset` without decompressing rows into RAM:

```rust
use tej_ui::prelude::*;

let mut status_col = RleColumn::new();
status_col.push_run(1u8, 500_000); // Active
status_col.push_run(2u8, 300_000); // Pending
status_col.push_run(3u8, 200_000); // Archived

let matches = DirectRleFilter::filter_equals(&status_col, 1u8);
assert_eq!(matches.total_set(), 500_000);
```

## 4. 1-Bit RaBitQ Semantic Palette (`SemanticPalette`)

`SemanticPalette` indexes actions using 64-bit 1-bit quantized signatures (8 bytes per item) and ranks nearest neighbors via hardware popcount (`u64::count_ones`):

```rust
use tej_ui::prelude::*;

let mut palette = SemanticPalette::new();
palette.register(1, "Create File", 0b1111_0000_1111_0000u64);
palette.register(2, "Open Folder", 0b1111_0000_0000_0000u64);

let top_matches = palette.search(0b1111_0000_1110_0000u64, 5);
```

## 5. Copy-On-Write Undo/Redo (`CowTimeTravelStack`)

Because `tej-db` uses append-only copy-on-write B+ Tree pages, each transaction commit produces a new 8-byte root page ID. `CowTimeTravelStack` stores only those 8-byte root IDs so undo/redo pivots state snapshots without serializing diffs:

```rust
use tej_ui::prelude::*;

let mut history = CowTimeTravelStack::new(initial_root_id);
history.record_commit(next_root_id);
if let Some(prev_root) = history.undo() {
    // Re-open read snapshot at `prev_root`
    let _ = prev_root;
}
```

## Complete Example

See [`examples/task-manager`](../../examples/task-manager) and run it headlessly with:

```bash
cargo run -p task-manager-example -- --headless
```
