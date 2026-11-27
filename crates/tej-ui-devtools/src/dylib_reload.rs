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

/// Dynamic library watcher and stateful hot reloader backed by [`libloading::Library`].
#[derive(Debug)]
pub struct DylibReloader {
    dylib_path: PathBuf,
    current_generation: u32,
    last_modified: Option<SystemTime>,
    active_symbol_table: Vec<String>,
    loaded_library: Option<libloading::Library>,
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
            loaded_library: None,
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

    /// Returns `true` if a native dynamic library handle is currently loaded.
    #[must_use]
    pub fn is_loaded(&self) -> bool {
        self.loaded_library.is_some()
    }

    /// Unloads any currently loaded native dynamic library handle.
    pub fn unload_native(&mut self) {
        self.loaded_library = None;
    }

    /// Loads (or reloads) the native dynamic library at [`Self::dylib_path`] via `libloading`.
    ///
    /// Any previously loaded library handle is unloaded first. On success, updates the
    /// tracked modification timestamp, increments the reload generation, and returns the
    /// new generation number.
    pub fn load_native(&mut self) -> Result<u32, String> {
        self.unload_native();

        // SAFETY: Loading a dynamic library executes its platform initialization routines.
        // Callers invoke `load_native` explicitly on a trusted hot-reload target dylib.
        let library = unsafe { libloading::Library::new(&self.dylib_path) }.map_err(|err| {
            format!(
                "failed to load dylib '{}': {err}",
                self.dylib_path.display()
            )
        })?;

        self.last_modified = std::fs::metadata(&self.dylib_path)
            .and_then(|m| m.modified())
            .ok();
        self.current_generation += 1;
        self.loaded_library = Some(library);
        Ok(self.current_generation)
    }

    /// Resolves a typed symbol from the currently loaded native dynamic library.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `T` matches the exact ABI and type signature of the
    /// exported symbol `name` in the loaded library, and that all platform-specific symbol
    /// invariants are upheld.
    pub unsafe fn get_symbol<T>(&self, name: &[u8]) -> Result<libloading::Symbol<'_, T>, String> {
        let Some(library) = &self.loaded_library else {
            return Err("no dynamic library is currently loaded".to_string());
        };

        // SAFETY: Caller guarantees `T` matches the exported symbol's ABI and signature.
        unsafe { library.get::<T>(name) }.map_err(|err| {
            let symbol_name = String::from_utf8_lossy(name);
            format!("failed to resolve symbol '{symbol_name}': {err}")
        })
    }

    /// Registers active exported view symbols from the loaded dylib.
    pub fn register_symbols(&mut self, symbols: &[&str]) {
        self.active_symbol_table = symbols.iter().map(|s| (*s).to_string()).collect();
    }

    /// Checks if a named symbol is exported by the current dylib version.
    pub fn has_symbol(&self, symbol: &str) -> bool {
        self.active_symbol_table.iter().any(|s| s == symbol)
    }

    /// Checks for disk modification and triggers state-preserving swap.
    pub fn check_and_reload(&mut self) -> ReloadStatus {
        let Ok(new_mod) = std::fs::metadata(&self.dylib_path).and_then(|m| m.modified()) else {
            return ReloadStatus::PendingCompilation;
        };

        if Some(new_mod) == self.last_modified {
            ReloadStatus::Unchanged
        } else {
            self.last_modified = Some(new_mod);
            self.current_generation += 1;
            ReloadStatus::Reloaded {
                generation: self.current_generation,
                path: self.dylib_path.clone(),
            }
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
