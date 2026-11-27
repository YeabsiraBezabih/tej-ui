# Changelog

All notable changes to `tej-ui` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-11-24

Initial crates.io release of the 19-crate `tej-ui` workspace.

### Added
- **Reactive Core (`tej-ui-core`)**: Generational `SlotMap` arena, 8-byte `Copy` `Signal<T>`, `ReadSignal<T>` / `WriteSignal<T>`, `Memo<T>`, `Effect`, `Resource`, `Store`, `batch`, and context propagation with glitch-free topological DAG scheduling.
- **Static View Tree & Styling (`tej-ui-view`)**: Compile-time static `View` trait, tuple combinators, CSS stylesheet and token engine (`Theme`, `StyleProps`, `CssStyleSheet`), event dispatcher, and core primitives (`VStack`, `HStack`, `Grid`, `Text`, `Button`, `Checkbox`, `Input`, `SearchBar`, `Canvas`, `If`, `For`).
- **Flat SoA Layout Engine (`tej-ui-layout`)**: Data-oriented Structure-of-Arrays (`LayoutArena`, `LayoutEngine`) supporting Flexbox and CSS Grid layout semantics.
- **Typography (`tej-ui-text`)**: Unicode Bidirectional Algorithm (UAX #9), system font discovery (`fontique`), glyph rasterization (`fontdue`), and shaping (`swash`).
- **Hardware Abstraction & Platform Backends (`tej-ui-hal`, `tej-ui-platform-linux`, `tej-ui-platform-windows`, `tej-ui-platform-macos`, `tej-ui-platform-web`)**: Dirty-rect tracking, monitor registry, optional `wgpu` GPU renderer (`gpu` feature), and target-gated platform backends.
- **Enterprise Widgets (`tej-ui-widgets`)**: `DataGrid`, `RichTextEditor`, `TreeView`, `FormController`, `CommandPalette`, `Modal`, `Sheet`, `Popover`, `Tooltip`, `Tabs`, `Accordion`, `Select`, `Slider`, `Switch`, `RadioGroup`, `Card`, `Badge`, `Alert`, `Progress`, `Pagination`, `Sidebar`, and `ScrollArea`.
- **Zero-Copy Database Sync (`tej-ui-db-sync`)**: Direct integration with `tej-db` (`0.1.0`) providing `DbRowLens`, `DirectRleFilter`, `SemanticPalette`, `CowTimeTravelStack`, `ReactiveDbQuery`, `DbSyncHook`, and `VirtualList`.
- **Accessibility, Animation, Icons, Services, Devtools, 3D & CLI**: `tej-ui-a11y` (AccessKit + Web ARIA), `tej-ui-animation`, `tej-ui-icons`, `tej-ui-services`, `tej-ui-devtools`, `tej-ui-3d`, and `tej-ui-cli` (`tej-ui` binary).
- **Governance, CI/CD & Developer Wiki**: Dual `MIT OR Apache-2.0` licenses, `rust-toolchain.toml` (MSRV 1.85.0), GitHub Actions CI matrix (Linux, macOS, Windows, feature matrix, packaging dry-run), `CONTRIBUTING.md`, `SECURITY.md`, and 8-page Developer Wiki in `docs/wiki/`.

### Changed & Hardened (Pre-Release Audit)
- Upgraded workspace resolver to `resolver = "3"` matching Rust Edition 2024 and MSRV `1.85.0`.
- Replaced local relative `tej-db` / `tej-db-core` path dependencies with published crates.io versions (`0.1.0`) and added `version = "0.1.0"` to all internal workspace dependencies.
- Added complete crates.io package metadata (`description`, `repository`, `homepage`, `documentation`, `readme`, `keywords`, `categories`) across all 19 publishable crates.
- Target-gated platform crates in `crates/tej-ui/Cargo.toml` so `tej-ui-platform-linux` (and `minifb`) are never pulled on Windows, macOS, or WebAssembly targets.
- Enabled `clippy::pedantic` (`warn`, priority `-1`) in `[workspace.lints.clippy]` so `clippy.toml` rules are actively enforced.
- Enabled `features = ["db-sync"]` in `examples/task-manager/Cargo.toml` and `benches/Cargo.toml`.
- Fixed reactivity in `examples/telemetry-dashboard/src/main.rs` by replacing one-shot `Signal::new(format!(...))` snapshots with derived `Memo::new` bindings.
- Removed hardcoded competitor footprint estimates and artificial startup clamping in `crates/tej-ui-benches/src/phase8_footprint.rs`; unmeasured competitors are now transparently reported as `0.0` (`N/A (unmeasured)`).
