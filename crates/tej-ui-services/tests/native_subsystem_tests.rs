//! Verification tests for native OS subsystems: ClipboardProvider, FileDialogProvider, and SecureKeyStore.

use std::path::PathBuf;
use std::sync::Arc;
use tej_ui_services::{
    BiometricAuthResult, BiometricPromptConfig, BiometricVerifier, Clipboard, ClipboardProvider,
    FileDialog, SecureKeyStore,
};

struct StaticBiometricVerifier(BiometricAuthResult);

impl BiometricVerifier for StaticBiometricVerifier {
    fn authenticate(&self, _prompt: &BiometricPromptConfig) -> BiometricAuthResult {
        self.0
    }
}

#[test]
fn test_clipboard_provider_rich_formats() {
    let cb: Box<dyn ClipboardProvider> = Box::new(Clipboard::new());
    assert_eq!(cb.formats(), [] as [std::string::String; 0]);

    // HTML with plain text fallback
    cb.set_html("<h1>Headline</h1>", Some("Headline"))
        .expect("write html");
    assert_eq!(
        cb.get_html().expect("read html"),
        Some("<h1>Headline</h1>".to_string())
    );
    assert_eq!(
        cb.get_text().expect("read fallback text"),
        Some("Headline".to_string())
    );

    // Image RGBA
    let test_pixels = vec![255, 0, 0, 255, 0, 255, 0, 255];
    cb.set_image(2, 1, test_pixels.clone())
        .expect("write image");
    let (w, h, bytes) = cb.get_image().expect("read image").expect("image exists");
    assert_eq!(w, 2);
    assert_eq!(h, 1);
    assert_eq!(bytes, test_pixels);

    let formats = cb.formats();
    assert!(formats.contains(&"text/plain".to_string()));
    assert!(formats.contains(&"text/html".to_string()));
    assert!(formats.contains(&"image/png".to_string()));

    cb.clear().expect("clear clipboard");
    assert_eq!(cb.formats(), [] as [std::string::String; 0]);
}

#[test]
fn test_file_dialog_provider_multi_file_and_parent() {
    let base_dir = PathBuf::from("/tmp/tej_project");
    let dialog = FileDialog::new()
        .title("Select Source Files")
        .initial_directory(&base_dir)
        .default_name("main.rs")
        .allow_multiple()
        .filter("Rust", &["rs"]);

    assert!(dialog.allows_multiple);
    assert_eq!(
        dialog.pick_file().expect("pick single file"),
        Some(base_dir.join("main.rs"))
    );

    let files = dialog.pick_files().expect("pick multiple files");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0], base_dir);

    let save_target = dialog.save_file().expect("save file");
    assert_eq!(save_target, Some(base_dir.join("main.rs")));
}

#[test]
fn test_secure_keystore_biometrics_and_memory_wipe() {
    let mut store = SecureKeyStore::new();

    // Store without biometrics
    store.store_secret(
        "app_token",
        b"tok_live_12345",
        tej_ui_services::SecretProtection::Standard,
    );
    assert_eq!(
        store.retrieve_secret("app_token", None),
        Some(b"tok_live_12345".as_slice())
    );

    // Store with biometrics
    store.store_secret(
        "vault_master",
        b"super_secret_master_key",
        tej_ui_services::SecretProtection::BiometricRequired,
    );

    // Access fails closed without a configured verifier
    assert_eq!(store.retrieve_secret("vault_master", None), None);

    // Access fails when verifier returns UserCancelled
    store.set_verifier(Arc::new(StaticBiometricVerifier(
        BiometricAuthResult::UserCancelled,
    )));
    assert_eq!(store.retrieve_secret("vault_master", None), None);

    // Access succeeds when verifier returns Success
    store.set_verifier(Arc::new(StaticBiometricVerifier(
        BiometricAuthResult::Success,
    )));
    let prompt = BiometricPromptConfig::new("Unlock Vault");
    assert_eq!(
        store.retrieve_secret("vault_master", Some(&prompt)),
        Some(b"super_secret_master_key".as_slice())
    );

    // Secure volatile wipe of process memory
    store.wipe_memory();
    assert_eq!(store.retrieve_secret("app_token", None), None);
    assert_eq!(store.retrieve_secret("vault_master", Some(&prompt)), None);
}
