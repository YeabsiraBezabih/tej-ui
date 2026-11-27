# Security Policy

## Supported Versions

| Version | Supported |
| :--- | :--- |
| `0.1.x` | Yes |

## Reporting a Vulnerability

Please **do not** open a public GitHub issue for security vulnerabilities.

Instead, report security issues privately via one of the following channels:
1. **GitHub Security Advisories** (preferred): Open a private advisory on `https://github.com/YeabsiraBezabih/tej-ui/security/advisories/new`.
2. **Email**: Send details to `ets1389@aastu.edu.et` with `[SECURITY] tej-ui` in the subject line.

Please include:
- Affected crate(s) and version(s)
- Target OS / architecture and enabled Cargo feature flags
- Minimal reproduction steps or proof-of-concept
- Potential impact assessment

You can expect an initial acknowledgment within **72 hours** and a remediation timeline once the issue is confirmed.

## Crate Security Boundaries

`tej-ui` is a modular workspace; security-sensitive boundaries are isolated to specific crates:

| Crate | Security Boundary & Threat Model |
| :--- | :--- |
| `tej-ui-core` / `tej-ui-layout` / `tej-ui-view` | Pure safe Rust reactive arena, layout engine, and view tree. No direct OS I/O or network access. |
| `tej-ui-hal` & `tej-ui-platform-*` | Interfaces with GPU drivers (`wgpu`), raw window handles (`raw-window-handle`), and OS windowing APIs (`minifb` on Linux). Treat untrusted window/input events defensively. |
| `tej-ui-db-sync` | Bridges directly to `tej-db` memory-mapped storage (`DbRowLens`). Callers must ensure database files are trusted or validated at the application layer; bounds checks are enforced on slice reads. |
| `tej-ui-services` | Interfaces with OS clipboard, file dialogs, drag-and-drop payloads, and credential storage (`CredentialStore`). Treat clipboard and drop payloads as untrusted input; never log credential secrets. |
| `tej-ui-devtools` | Includes dynamic library hot-reloading (`HotReloadHost`) and runtime DAG inspection. **Do not enable the `devtools` feature in untrusted production builds**, as loading dynamic libraries executes arbitrary native code. |
| `tej-ui-3d` | Parses external geometry assets (`.obj` files via `parse_obj` / `load_obj_from_file`). Treat untrusted 3D asset files with appropriate size limits to prevent excessive memory allocation. |
| `tej-ui-text` | Parses system fonts (`fontdue`, `fontique`, `swash`) and arbitrary Unicode strings (`unicode-bidi`). Keep font-shaping dependencies updated via `cargo audit` / Dependabot. |

## Dependency Auditing

Contributors and maintainers should periodically run `cargo audit` (via `rustsec`) to detect advisories in transitive dependencies.
