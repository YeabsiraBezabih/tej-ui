//! Production dynamic library hot-reloading tests for `DylibReloader`.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use tej_ui_devtools::{DylibReloader, ReloadStatus};

fn unique_temp_path(filename: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "tej_ui_devtools_{}_{}_{}",
        std::process::id(),
        nanos,
        filename
    ))
}

#[test]
fn test_load_native_missing_dylib_returns_error_and_preserves_state() {
    let missing_path = unique_temp_path("nonexistent_plugin.so");
    let mut reloader = DylibReloader::new(&missing_path);

    assert!(!reloader.is_loaded());
    assert_eq!(reloader.current_generation(), 1);

    let err = reloader
        .load_native()
        .expect_err("loading a missing dylib should return an error");
    assert!(err.contains("failed to load dylib"));
    assert!(!reloader.is_loaded());
    assert_eq!(reloader.current_generation(), 1);

    // Resolving a symbol when nothing is loaded fails cleanly
    // SAFETY: No library is loaded; `get_symbol` returns `Err` before dereferencing anything.
    let sym_err = unsafe { reloader.get_symbol::<extern "C" fn()>(b"render_root\0") }
        .expect_err("symbol lookup without a loaded dylib must fail");
    assert!(sym_err.contains("no dynamic library is currently loaded"));
}

#[test]
fn test_load_native_invalid_dylib_file_returns_error() {
    let invalid_path = unique_temp_path("corrupt_plugin.so");
    fs::write(&invalid_path, b"not a valid ELF/Mach-O/PE shared library").expect("write temp file");

    let mut reloader = DylibReloader::new(&invalid_path);
    assert!(!reloader.is_loaded());
    assert_eq!(reloader.current_generation(), 1);

    let err = reloader
        .load_native()
        .expect_err("loading an invalid dylib file must fail");
    assert!(err.contains("failed to load dylib"));
    assert!(!reloader.is_loaded());
    assert_eq!(reloader.current_generation(), 1);

    let _ = fs::remove_file(&invalid_path);
}

#[test]
fn test_unload_lifecycle_and_generation_tracking() {
    let temp_path = unique_temp_path("lifecycle_target.so");
    let mut reloader = DylibReloader::new(&temp_path);

    assert!(!reloader.is_loaded());
    assert_eq!(reloader.current_generation(), 1);
    assert_eq!(
        reloader.check_and_reload(),
        ReloadStatus::PendingCompilation
    );

    // Unloading when nothing is loaded is a safe no-op
    reloader.unload_native();
    assert!(!reloader.is_loaded());
    assert_eq!(reloader.current_generation(), 1);

    // Creating the file on disk triggers a reload generation bump via mtime polling
    fs::write(&temp_path, b"placeholder").expect("write placeholder");
    let status = reloader.check_and_reload();
    assert_eq!(
        status,
        ReloadStatus::Reloaded {
            generation: 2,
            path: temp_path.clone(),
        }
    );
    assert_eq!(reloader.current_generation(), 2);
    assert_eq!(reloader.check_and_reload(), ReloadStatus::Unchanged);

    // Simulated reload increments generation without altering loaded state
    let sim_status = reloader.simulate_reload();
    assert_eq!(
        sim_status,
        ReloadStatus::Reloaded {
            generation: 3,
            path: temp_path.clone(),
        }
    );
    assert_eq!(reloader.current_generation(), 3);
    assert!(!reloader.is_loaded());

    // Unload again preserves generation tracking
    reloader.unload_native();
    assert!(!reloader.is_loaded());
    assert_eq!(reloader.current_generation(), 3);

    let _ = fs::remove_file(&temp_path);
}
