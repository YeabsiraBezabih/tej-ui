# `tej-ui` Developer Wiki

Welcome to the official Developer Wiki for **`tej-ui`** — a modular, reactive, zero-copy cross-platform UI framework for Rust (Edition 2024, MSRV 1.85.0).

## Navigation

| Guide | Topics Covered |
| :--- | :--- |
| **[Getting Started](Getting-Started.md)** | Prerequisites, installation, feature flags, your first interactive app, and headless testing. |
| **[Reactivity Guide](Reactivity-Guide.md)** | Generational arena `Signal<T>`, `ReadSignal`/`WriteSignal`, `Memo<T>`, `Effect`, `batch`, `Resource`, `Store`, and context propagation. |
| **[Layout and Styling](Layout-and-Styling.md)** | Flat SoA `LayoutArena` & `LayoutEngine`, Flexbox & CSS Grid, `Theme` design tokens, `StyleProps`, `CssStyleSheet`, and `ViewExt`. |
| **[Widgets and Forms](Widgets-and-Forms.md)** | Core view primitives, enterprise widgets (`DataGrid`, `RichTextEditor`, `TreeView`, `CommandPalette`, overlays), and `FormController` validation. |
| **[Database Sync with Tej-DB](Database-Sync-with-Tej-DB.md)** | Zero-copy `DbRowLens`, `VirtualList`, `DirectRleFilter`, `SemanticPalette` (RaBitQ), `CowTimeTravelStack`, and `DbSyncHook`. |
| **[Architecture and Rendering](Architecture-and-Rendering.md)** | 19-crate dependency DAG, static `View` trees, dirty-rect composition, `tej-ui-hal` (`wgpu` & software), text shaping, accessibility, and 3D viewports. |
| **[Publishing and Contributing](Publishing-and-Contributing.md)** | Topological crates.io publish order, pre-flight verification, CI/CD matrix, coding standards, and security boundaries. |

## Quick Links

- **Repository**: <https://github.com/YeabsiraBezabih/tej-ui>
- **Companion Database (`tej-db`)**: <https://github.com/YeabsiraBezabih/tej-db>
- **API Documentation**: <https://docs.rs/tej-ui>
- **Changelog**: [`CHANGELOG.md`](../../CHANGELOG.md)
- **Contributing Guide**: [`CONTRIBUTING.md`](../../CONTRIBUTING.md)
- **Security Policy**: [`SECURITY.md`](../../SECURITY.md)
