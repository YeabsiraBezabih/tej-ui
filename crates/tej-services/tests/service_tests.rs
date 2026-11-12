use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tej_services::{
    Clipboard, CredentialStore, FileDialog, Modifiers, Shortcut, ShortcutManager, TrayIcon,
};

#[test]
fn test_clipboard_text_and_custom_mime() {
    let cb = Clipboard::new();
    assert_eq!(cb.get_text().unwrap(), None);

    cb.set_text("Copied Linear Issue ID").unwrap();
    assert_eq!(
        cb.get_text().unwrap().as_deref(),
        Some("Copied Linear Issue ID")
    );

    // Custom MIME
    let payload = vec![0xDE, 0xAD, 0xBE, 0xEF];
    cb.set_custom("application/x-tej-node", payload.clone())
        .unwrap();
    assert_eq!(
        cb.get_custom("application/x-tej-node").unwrap(),
        Some(payload)
    );

    cb.clear().unwrap();
    assert_eq!(cb.get_text().unwrap(), None);
}

#[test]
fn test_file_dialog_builder() {
    let dialog = FileDialog::new()
        .title("Open Project")
        .filter("Rust Files", &["rs"])
        .filter("All Files", &["*"]);

    assert_eq!(dialog.title.as_deref(), Some("Open Project"));
    assert_eq!(dialog.filters.len(), 2);
    assert_eq!(dialog.filters[0].extensions, vec!["rs"]);
}

#[test]
fn test_shortcut_manager_dispatch() {
    let manager = ShortcutManager::new();
    let fired = Arc::new(AtomicBool::new(false));

    let fired_clone = Arc::clone(&fired);
    let shortcut = Shortcut::new(Modifiers::CMD, "K");
    manager
        .register(shortcut.clone(), move || {
            fired_clone.store(true, Ordering::SeqCst);
        })
        .unwrap();

    // Matching shortcut triggers handler
    let dispatched = manager.dispatch(&shortcut);
    assert!(dispatched);
    assert!(fired.load(Ordering::SeqCst));

    // Non-matching shortcut returns false
    let other = Shortcut::new(Modifiers::CTRL, "P");
    assert!(!manager.dispatch(&other));
}

#[test]
fn test_secure_credential_store() {
    let store = CredentialStore::new();
    assert_eq!(store.get_secret("linear-api", "user_1").unwrap(), None);

    store
        .save_secret("linear-api", "user_1", "lin_api_key_secret_123")
        .unwrap();
    assert_eq!(
        store.get_secret("linear-api", "user_1").unwrap().as_deref(),
        Some("lin_api_key_secret_123")
    );

    let deleted = store.delete_secret("linear-api", "user_1").unwrap();
    assert!(deleted);
    assert_eq!(store.get_secret("linear-api", "user_1").unwrap(), None);
}

#[test]
fn test_tray_icon_configuration() {
    let mut tray = TrayIcon::new("Linear Sync")
        .tooltip("Linear local daemon")
        .badge(5);

    assert_eq!(tray.title.as_deref(), Some("Linear Sync"));
    assert_eq!(tray.badge_count, Some(5));
    assert!(tray.is_visible);

    tray.set_visible(false).unwrap();
    assert!(!tray.is_visible);
}
