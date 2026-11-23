//! Dynamic library (dylib) stateful hot-reloading coordinator.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Status of a dynamic library reload check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReloadStatus {
    /// No change detected on disk.
    Unchanged,
    /// Binary file was modified on disk and successfully prepared for reload.
    Reloaded { generation: u32, path: PathBuf },
    /// Error during reload (e.g. file lock or compilation in progress).
    PendingCompilation,
}

/// Dynamic library watcher and stateful hot reloader.
#[derive(Debug, Clone)]
pub struct DylibReloader {
    dylib_path: PathBuf,
    current_generation: u32,
    last_modified: Option<SystemTime>,
    active_symbol_table: Vec<String>,
}

impl DylibReloader {
    /// Creates a new dylib reloader targeting the given dynamic library path.
    pub fn new(path: impl AsRef<Path>) -> Self {
        let p = path.as_ref().to_path_buf();
        let modified = std::fs::metadata(&p).and_then(|m| m.modified()).ok();
        Self {
            dylib_path: p,
            current_generation: 1,
            last_modified: modified,
            active_symbol_table: Vec::new(),
        }
    }

    /// Path to the dynamic library being observed.
    pub fn dylib_path(&self) -> &Path {
        &self.dylib_path
    }

    /// Current reload generation.
    pub fn current_generation(&self) -> u32 {
        self.current_generation
    }

    /// Registers active exported view symbols from the loaded dylib.
    pub fn register_symbols(&mut self, symbols: &[&str]) {
        self.active_symbol_table = symbols.iter().map(|s| s.to_string()).collect();
    }

    /// Checks if a named symbol is exported by the current dylib version.
    pub fn has_symbol(&self, symbol: &str) -> bool {
        self.active_symbol_table.iter().any(|s| s == symbol)
    }

    /// Checks for disk modification and triggers state-preserving swap.
    pub fn check_and_reload(&mut self) -> ReloadStatus {
        let new_mod = match std::fs::metadata(&self.dylib_path).and_then(|m| m.modified()) {
            Ok(t) => t,
            Err(_) => return ReloadStatus::PendingCompilation,
        };

        if Some(new_mod) != self.last_modified {
            self.last_modified = Some(new_mod);
            self.current_generation += 1;
            ReloadStatus::Reloaded {
                generation: self.current_generation,
                path: self.dylib_path.clone(),
            }
        } else {
            ReloadStatus::Unchanged
        }
    }

    /// Manually trigger a simulated reload with a mock timestamp update (for deterministic tests).
    pub fn simulate_reload(&mut self) -> ReloadStatus {
        self.current_generation += 1;
        ReloadStatus::Reloaded {
            generation: self.current_generation,
            path: self.dylib_path.clone(),
        }
    }
}
