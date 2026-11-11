# Agent Development Rules & Engineering Standards

These rules govern all AI agent and developer contributions to the `tej-ui` codebase. They ensure high modularity, zero-copy performance, authentic native OS feel, glitch-free reactivity, and rigorous verification across the workspace.

---

## 1. Modularity & File Size Limits

- **Maximum 500 Lines Per File**:
  - No source file (`.rs`) should exceed **500 lines** unless strictly necessary (e.g., generated tables or comprehensive macro definitions).
  - Any file approaching 400 lines must be proactively decomposed into focused submodules.

- **Dedicated Files for Independent Concepts**:
  - Every distinct concept (e.g., a specific signal variant, reactive node arena, static view combinator, layout bridge, platform surface compositor, text shaper, or database cursor) **must have its own dedicated file and module**.
  - As structures or traits expand, extract helper logic and state containers into independent files. Do not accumulate disparate concerns in a single file.

- **Slim Root Facades (`lib.rs` / `mod.rs`)**:
  - `lib.rs` and `mod.rs` files must serve solely as public facades: declaring modules, re-exporting key types, and documenting architecture.
  - Do not place heavy implementations or large `impl` blocks directly inside `lib.rs` or `mod.rs`.

- **Unit Test Placement**:
  - Unit test suites exceeding 80–100 lines must be extracted into a separate `tests.rs` submodule or into dedicated integration test files under `tests/` to keep production code clean.

---

## 2. Modular Workspace Architecture

Like the modular ACID DBMS, `tej-ui` is split into clean, decoupled crates under `crates/` so that every layer can be independently scaled, tested, or swapped:

- **`tej-core`**: Generational `SlotMap` arena, 8-byte `Copy` `Signal<T>`, dependency tracking graph, dirty bitsets, reactive epochs.
- **`tej-view`**: Compile-time static view tree traits (`View`), tuple combinators `(A, B, ...)`, control flow primitives (`If`, `For`), zero-vtable dispatch.
- **`tej-layout`**: Data-oriented flat SoA widget storage, `Taffy` integration, zero-allocation Flexbox & CSS Grid passes.
- **`tej-text`**: System font discovery, text shaping, zero-copy string slice measurement (`Parley`, `Fontique`, or platform DirectWrite/CoreText).
- **`tej-hal`**: Hardware Abstraction Layer & OS Compositor traits (`PlatformWindow`, `NativeControl`, `SurfaceCompositor`, `DirtyRects`, event loop integration).
- **`tej-platform-*`**: Platform-specific native shells (`tej-platform-linux` with GTK4/Wayland/X11, `tej-platform-windows` with Win32/DirectComposition/Direct2D, `tej-platform-macos` with AppKit/CoreAnimation, `tej-platform-web` with WASM/DOM).
- **`tej-a11y`**: Semantic accessibility tree bridge (`AccessKit`) exposing view nodes and native controls to platform assistive tech (VoiceOver, Narrator, Orca, TalkBack).
- **`tej-db-sync`**: Zero-copy database-driven UI state integration, reactive range queries (`use_db_query`), WAL/commit listeners, and virtualized mmap list cursors connecting directly to the ACID DBMS.
- **`tej`**: Ergonomic top-level umbrella crate providing the unified `prelude`, widgets, and builder macros.

---

## 3. Mandatory Verification & Quality Gates

On **every code change**, all three gates must be executed and confirmed passing:

1. **Formatting**:
   ```bash
   cargo fmt --all -- --check
   ```
   No formatting deviations are permitted.

2. **Clippy (Zero Warnings)**:
   ```bash
   cargo clippy --all-targets --all-features
   ```
   Must pass with **zero warnings**. Workspace clippy lints are configured strictly (`missing_debug_implementations`, `unsafe_op_in_unsafe_fn`, `max-fn-params-bools = 0`). Do not silence lints with `#[allow(...)]` without documented justification.

3. **Test Suite**:
   ```bash
   cargo test -j 2
   ```
   All unit tests, integration tests, reactivity property tests, and doc tests must pass.

---

## 4. Test Discipline & Regression Prevention

- **Test Accompaniment**:
  - Every bug fix, feature addition, or refactoring **must** be accompanied by updated or new tests.
  - Reactive edge cases (diamond dependencies, dynamic branches, nested scopes, unmount cleanups) must have explicit unit tests.
- **Never Weaken Tests**:
  - Never delete, comment out, or ignore (`#[ignore]`) existing test assertions to bypass failures. A failing test signals either a real defect or an intentional semantic update requiring documented changes.

---

## 5. UI, Reactivity & Storage Invariants

- **Glitch-Free Reactivity**:
  - Signal propagation must be glitch-free and topologically consistent. Dependent computations must never observe intermediate stale states.
- **Zero-Allocation Steady-State**:
  - Steady-state frame layout, reactive dependency queries, and widget mutation must not allocate heap memory (`Vec::new()`, `Box::new()`, `String::clone()`). Flat arenas and reuse pools must be used.
- **Zero-Copy Database Integration**:
  - Virtual lists and database-backed views must read directly from memory-mapped page slices (`&str`, `&[u8]`) within the database's read transaction snapshot without cloning rows into UI memory.
- **Authentic Native Look & Feel**:
  - Chrome, text inputs, dialogs, and scrolling must utilize Tier 1 native OS controls or compositors wherever available to provide authentic IME composition, accessibility, kinetic scroll inertia, and system theme synchronization.
- **Dirty-Region Rasterization**:
  - Rendering must clip to dirty bounding boxes. Never blit or invalidate a full 4K frame buffer if only a fine-grained label or status icon changed.

---

## 6. Performance & Systems Idioms

- **Data-Oriented Storage (ECS-Lite)**:
  - Layout rectangles, dirty bitfields, and style flags must be stored in flat Structure-of-Arrays (`SoA`) vectors to maximize CPU cache locality.
- **Memory Ordering & Concurrency**:
  - Ensure lockless signal reads where applicable, using explicit atomics (`Ordering::Acquire`, `Ordering::Release`). Document lock hierarchy for any cross-thread dispatch.
- **Explicit Error Handling**:
  - Library crates (`tej-*`) must never call `.unwrap()`, `.expect()`, or `panic!()` in runtime paths.
  - Return typed results using `thiserror`-derived `UiError` or `Result<T, UiError>`.
- **Unsafe Code Guidelines**:
  - Confine `unsafe` strictly to OS FFI boundaries, raw window handles, and memory-mapped page slices.
  - Every `unsafe` block **must** be preceded by a `// SAFETY:` comment explaining why all platform and memory invariants hold.
