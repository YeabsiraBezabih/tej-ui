# Publishing and Contributing

This page documents the crates.io release process, topological publish order, CI/CD workflow, and contributor verification steps for `tej-ui`.

## 1. Pre-Publish Verification Checklist

Before tagging a release:

1. **Ensure `tej-db` & `tej-db-core` are live on crates.io** at the version referenced in root `Cargo.toml` (`0.1.0`).
2. **Run all quality gates locally**:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo test -p tej-ui --all-features
   cargo check -p tej-ui --no-default-features
   ```
3. **Verify documentation builds cleanly**:
   ```bash
   RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
   ```
4. **Verify packaging manifests**:
   ```bash
   cargo package --list -p tej-ui-core
   ```
5. **Confirm clean git state** (`git status --porcelain` is empty) and update `CHANGELOG.md`.

## 2. Topological Crates.io Publish Order (19 Crates)

Crates.io indexes newly published versions asynchronously. Publish in strict bottom-up dependency order, pausing ~30 seconds between tiers so the crates.io index propagates:

```bash
#!/usr/bin/env bash
set -euo pipefail

# Tier 1: Zero internal dependencies
cargo publish -p tej-ui-core
cargo publish -p tej-ui-layout
sleep 30

# Tier 2: Depends on Tier 1
cargo publish -p tej-ui-hal
cargo publish -p tej-ui-a11y
cargo publish -p tej-ui-services
cargo publish -p tej-ui-cli
sleep 30

# Tier 3: Depends on Tier 1-2
cargo publish -p tej-ui-text
cargo publish -p tej-ui-view
cargo publish -p tej-ui-animation
cargo publish -p tej-ui-platform-linux
cargo publish -p tej-ui-platform-windows
cargo publish -p tej-ui-platform-macos
cargo publish -p tej-ui-platform-web
sleep 30

# Tier 4: Depends on Tier 1-3
cargo publish -p tej-ui-icons
cargo publish -p tej-ui-devtools
cargo publish -p tej-ui-3d
cargo publish -p tej-ui-db-sync
sleep 30

# Tier 5: Depends on Tier 1-4
cargo publish -p tej-ui-widgets
sleep 30

# Tier 6: Umbrella crate
cargo publish -p tej-ui
```

Private workspace crates (`tej-ui-benches`, `tej-ui-3d-benches`, `examples/*`, `benches`, `tests`) all set `publish = false` and are never uploaded.

## 3. Continuous Integration (`.github/workflows/ci.yml`)

Every push and pull request to `main` runs five GitHub Actions jobs pinned to Rust `1.85.0`:

- **Rustfmt**: `cargo fmt --all -- --check`
- **Clippy**: `cargo clippy --workspace --all-targets -- -D warnings` (enforcing `clippy::all` and `clippy::pedantic`)
- **Cross-OS Test Matrix**: `cargo test --workspace` on `ubuntu-latest`, `macos-latest`, and `windows-latest`
- **Feature Matrix**: `--no-default-features`, `--all-features` on `tej-ui`, and `--no-default-features` on `tej-ui-hal`
- **Crates.io Package Dry-Run**: `cargo package --list` across all 19 publishable crates

## 4. Contributing & Security

- Follow [`CONTRIBUTING.md`](../../CONTRIBUTING.md) for branch workflow, `unsafe` invariants, and honest benchmarking rules.
- Report vulnerabilities privately per [`SECURITY.md`](../../SECURITY.md).
