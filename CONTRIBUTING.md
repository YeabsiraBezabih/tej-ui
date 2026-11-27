# Contributing to `tej-ui`

Thank you for your interest in contributing to `tej-ui`! This guide covers setting up your development environment, coding standards, quality gates, and how to submit changes.

## Development Setup

- **Rust Toolchain**: `tej-ui` targets **Rust Edition 2024** with an MSRV of **1.85.0** (pinned in `rust-toolchain.toml`).
- **Required Components**: `rustfmt` and `clippy`.
- **Linux System Libraries** (for `minifb` / windowing on Linux):
  ```bash
  sudo apt-get update && sudo apt-get install -y libxkbcommon-dev libx11-dev libwayland-dev pkg-config
  ```

Clone the repository and verify the workspace builds:

```bash
git clone https://github.com/YeabsiraBezabih/tej-ui.git
cd tej-ui
cargo test --workspace
```

## Mandatory Quality Gates

Before opening a pull request, run all four local checks:

```bash
# 1. Formatting
cargo fmt --all -- --check

# 2. Linting (workspace enables clippy::all and clippy::pedantic)
cargo clippy --workspace --all-targets -- -D warnings

# 3. Tests (default and full feature sets)
cargo test --workspace
cargo test -p tej-ui --all-features
cargo check -p tej-ui --no-default-features

# 4. Packaging check
cargo package --list -p tej-ui-core
```

## Workspace Guidelines

1. **Crate Boundaries**: Keep crates strictly layered according to the dependency DAG (see [`docs/wiki/Architecture-and-Rendering.md`](docs/wiki/Architecture-and-Rendering.md)). Avoid circular dependencies and keep `tej-ui-core` and `tej-ui-layout` lean.
2. **Target Gating**: Never pull OS-specific dependencies (such as `minifb`, Win32, or AppKit) unconditionally in cross-platform crates. Use `[target.'cfg(...)'.dependencies]` in `Cargo.toml` and matching `#[cfg(...)]` attributes in Rust code.
3. **Safety & `unsafe`**: Workspace lints enforce `unsafe_op_in_unsafe_fn = "warn"`. Every `unsafe` block must include a concise `// SAFETY:` comment explaining the invariants upheld.
4. **Honest Benchmarks**: Benchmark harnesses must report only genuinely measured values. Never hardcode, clamp, or extrapolate competitor figures.
5. **Documentation & Examples**: Public APIs should include doc comments and runnable examples where appropriate. Keep `README.md` and `docs/wiki/` synchronized with API changes.

## Pull Request Process

1. Fork the repository and create a focused feature or bugfix branch from `main`.
2. Add unit or integration tests covering your change.
3. Update `CHANGELOG.md` under `[Unreleased]` for user-visible changes.
4. Open a pull request describing the motivation, architectural trade-offs, and verification steps.

## Licensing

By submitting a contribution, you agree that your work will be dual-licensed under **MIT OR Apache-2.0**, matching the workspace license.
