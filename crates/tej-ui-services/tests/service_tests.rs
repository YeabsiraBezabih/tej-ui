use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tej_ui_services::{
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

#[test]
fn test_shortcut_chords_and_scoped_resolution() {
    use std::sync::atomic::AtomicUsize;
    use tej_ui_services::{ShortcutChord, ShortcutScope};

    let manager = ShortcutManager::new();
    let global_fired = Arc::new(AtomicUsize::new(0));
    let local_fired = Arc::new(AtomicUsize::new(0));
    let chord_fired = Arc::new(AtomicBool::new(false));

    let g_clone = Arc::clone(&global_fired);
    let s_save = Shortcut::primary("S");
    manager
        .register(s_save.clone(), move || {
            g_clone.fetch_add(1, Ordering::SeqCst);
        })
        .unwrap();

    let l_clone = Arc::clone(&local_fired);
    manager
        .register_scoped(ShortcutScope::Local(42), s_save.clone(), move || {
            l_clone.fetch_add(1, Ordering::SeqCst);
        })
        .unwrap();

    // 1. Dispatching without local context triggers global
    assert!(manager.dispatch(&s_save));
    assert_eq!(global_fired.load(Ordering::SeqCst), 1);
    assert_eq!(local_fired.load(Ordering::SeqCst), 0);

    // 2. Dispatching with active local context (42) overrides global
    assert!(manager.dispatch_scoped(&s_save, None, Some(42)));
    assert_eq!(global_fired.load(Ordering::SeqCst), 1);
    assert_eq!(local_fired.load(Ordering::SeqCst), 1);

    // 3. Chord sequence: Ctrl+K, Ctrl+S
    let k_stroke = Shortcut::new(Modifiers::CTRL, "K");
    let s_stroke = Shortcut::new(Modifiers::CTRL, "S");
    let c_clone = Arc::clone(&chord_fired);

    let chord = ShortcutChord::sequence(k_stroke.clone(), s_stroke.clone());
    manager
        .register(chord, move || {
            c_clone.store(true, Ordering::SeqCst);
        })
        .unwrap();

    // First stroke (Ctrl+K) begins pending chord
    assert!(manager.dispatch(&k_stroke));
    assert!(!chord_fired.load(Ordering::SeqCst));

    // Second stroke (Ctrl+S) completes chord
    assert!(manager.dispatch(&s_stroke));
    assert!(chord_fired.load(Ordering::SeqCst));
}

#[test]
fn test_workspace_geometry_and_state_persistence() {
    use std::collections::HashMap;
    use tej_ui_services::{TabState, WindowGeometry, WorkspaceLayout, WorkspaceStateStore};

    let mut drafts = HashMap::new();
    drafts.insert(
        "doc_1".to_string(),
        "fn main() { println!(\"Hello\"); }".to_string(),
    );

    let store = WorkspaceStateStore {
        window: WindowGeometry {
            x: 120.0,
            y: 80.0,
            width: 1440.0,
            height: 900.0,
            is_maximized: false,
            is_fullscreen: false,
            display_id: 1,
        },
        workspace: WorkspaceLayout {
            active_tab_index: 1,
            tabs: vec![
                TabState {
                    id: "tab_welcome".to_string(),
                    title: "Welcome".to_string(),
                    scroll_x: 0.0,
                    scroll_y: 0.0,
                    is_pinned: true,
                },
                TabState {
                    id: "tab_main_rs".to_string(),
                    title: "main.rs".to_string(),
                    scroll_x: 0.0,
                    scroll_y: 150.0,
                    is_pinned: false,
                },
            ],
            split_ratio: 0.25,
            drafts,
        },
    };

    // Serialize to JSON
    let json = store.to_json_string();
    assert!(json.contains("\"active_tab_index\": 1"));
    assert!(json.contains("\"width\": 1440"));
    assert!(json.contains("\"tab_main_rs\""));

    // Parse back
    let restored = WorkspaceStateStore::parse_json(&json).expect("Failed to parse JSON");
    assert_eq!(restored.window.x, 120.0);
    assert_eq!(restored.window.width, 1440.0);
    assert_eq!(restored.workspace.active_tab_index, 1);
    assert_eq!(restored.workspace.tabs.len(), 2);
    assert_eq!(restored.workspace.tabs[1].id, "tab_main_rs");
    assert_eq!(restored.workspace.tabs[1].scroll_y, 150.0);
    assert_eq!(
        restored.workspace.drafts.get("doc_1").map(|s| s.as_str()),
        Some("fn main() { println!(\"Hello\"); }")
    );

    // Test file atomic save and load
    let temp_dir = tempfile::tempdir().unwrap();
    let state_path = temp_dir.path().join("workspace_state.json");

    store.save_to_file(&state_path).unwrap();
    assert!(state_path.exists());

    let loaded = WorkspaceStateStore::load_from_file(&state_path).unwrap();
    assert_eq!(loaded.window.height, 900.0);
    assert_eq!(loaded.workspace.split_ratio, 0.25);
}

#[test]
fn test_l10n_manager_plurals_and_formatting() {
    use tej_ui_services::{L10nManager, Locale, MessageCatalog, PluralCategory};

    // 1. Message Catalog creation
    let mut en_catalog = MessageCatalog::new();
    en_catalog.add("welcome", "Welcome, {name}!");
    en_catalog.add_plural(
        "unread_emails",
        [
            (PluralCategory::One, "You have 1 unread email"),
            (PluralCategory::Other, "You have {count} unread emails"),
        ],
    );

    let mut fr_catalog = MessageCatalog::new();
    fr_catalog.add("welcome", "Bienvenue, {name}!");
    fr_catalog.add_plural(
        "unread_emails",
        [
            (PluralCategory::One, "Vous avez {count} message non lu"),
            (PluralCategory::Other, "Vous avez {count} messages non lus"),
        ],
    );

    let mut l10n = L10nManager::new(Locale::new("en-US"));
    l10n.set_catalog(Locale::new("en-US"), en_catalog);
    l10n.set_catalog(Locale::new("fr-FR"), fr_catalog);

    // 2. English message and plurals
    let msg_en = l10n.message("welcome", &[("name", "Alice")]);
    assert_eq!(msg_en, "Welcome, Alice!");

    let plural_en_1 = l10n.plural("unread_emails", 1, &[]);
    assert_eq!(plural_en_1, "You have 1 unread email");

    let plural_en_5 = l10n.plural("unread_emails", 5, &[]);
    assert_eq!(plural_en_5, "You have 5 unread emails");

    // 3. Switch to French
    l10n.set_locale(Locale::new("fr-FR"));
    let msg_fr = l10n.message("welcome", &[("name", "Alice")]);
    assert_eq!(msg_fr, "Bienvenue, Alice!");

    // In French, count 0 is in the "One" plural category!
    let plural_fr_0 = l10n.plural("unread_emails", 0, &[]);
    assert_eq!(plural_fr_0, "Vous avez 0 message non lu");

    let plural_fr_4 = l10n.plural("unread_emails", 4, &[]);
    assert_eq!(plural_fr_4, "Vous avez 4 messages non lus");

    // 4. Currency and number formatting
    assert_eq!(l10n.format_currency(12345.67, "USD"), "$12.345,67");
    assert_eq!(l10n.format_currency(12345.67, "EUR"), "12.345,67 €");

    l10n.set_locale(Locale::new("en-US"));
    assert_eq!(l10n.format_currency(12345.67, "USD"), "$12,345.67");
}

#[test]
fn test_dnd_protocol_and_mime_negotiation() {
    use std::path::PathBuf;
    use tej_ui_services::{DragPreview, DragSession, DropAction, DropPayload, MimeNegotiator};

    // 1. MIME OS payload parsing
    let plain_bytes = b"Hello from desktop file manager";
    let text_payload = MimeNegotiator::parse_os_payload("text/plain", plain_bytes).unwrap();
    assert_eq!(
        text_payload,
        DropPayload::Text("Hello from desktop file manager".to_string())
    );

    let uri_bytes = b"file:///home/user/document.pdf\nfile:///home/user/image.png\n";
    let file_payload = MimeNegotiator::parse_os_payload("text/uri-list", uri_bytes).unwrap();
    assert_eq!(
        file_payload,
        DropPayload::Files(vec![
            PathBuf::from("/home/user/document.pdf"),
            PathBuf::from("/home/user/image.png")
        ])
    );

    // 2. Drag Session with allowed actions and preview
    let preview = DragPreview::new("2 Files", 120.0, 40.0);
    let mut session = DragSession::new(
        101,
        Some(1),
        file_payload,
        vec![DropAction::Copy, DropAction::Move],
    )
    .with_preview(preview);

    assert_eq!(session.session_id(), 101);
    assert_eq!(session.source_window_id(), Some(1));
    assert_eq!(session.selected_action(), DropAction::None);

    // Action selection
    session.set_selected_action(DropAction::Copy);
    assert_eq!(session.selected_action(), DropAction::Copy);

    // Invalid action rejected
    session.set_selected_action(DropAction::Link);
    assert_eq!(session.selected_action(), DropAction::None);
}

#[test]
fn test_native_os_services_batch() {
    use tej_ui_services::{
        AppBadgeController, BiometricAuthResult, MenuBar, MenuItem, MenuItemRole,
        NotificationAction, NotificationBuilder, PageLayout, PageMargins, PaperSize,
        PdfPrintSpooler, SecureKeyStore, SharePayload, ShareSheetDispatcher, TaskbarState,
    };

    // 1. Menu Bar and Badges
    let mut menu_bar = MenuBar::new();
    let file_menu = vec![
        MenuItem::action("new_doc", "New Document"),
        MenuItem::Separator,
        MenuItem::role(MenuItemRole::Quit, "Quit"),
    ];
    menu_bar.add_menu("File", file_menu);
    assert!(menu_bar.find_item("new_doc").is_some());
    assert!(menu_bar.find_item("quit").is_some());

    let mut badge_ctrl = AppBadgeController::new();
    badge_ctrl.set_badge_count(5);
    assert_eq!(badge_ctrl.badge_label(), Some("5"));
    badge_ctrl.set_taskbar_state(TaskbarState::Determinate {
        progress_percent: 75,
    });
    assert_eq!(
        badge_ctrl.taskbar_state(),
        Some(&TaskbarState::Determinate {
            progress_percent: 75
        })
    );

    // 2. Hardware Enclave & Biometrics Keystore
    let mut keystore = SecureKeyStore::new();
    keystore.store_secret("cloud_token", b"token_12345", true);
    keystore.store_secret("public_pref", b"pref_val", false);

    // Accessing without biometrics returns None for protected key
    assert_eq!(keystore.retrieve_secret("cloud_token", None), None);
    // Accessing with successful biometric auth returns the secret
    assert_eq!(
        keystore.retrieve_secret("cloud_token", Some(BiometricAuthResult::Success)),
        Some(b"token_12345".as_slice())
    );
    // Non-protected key requires no auth
    assert_eq!(
        keystore.retrieve_secret("public_pref", None),
        Some(b"pref_val".as_slice())
    );

    // 3. Notifications & Share Sheet
    let notif = NotificationBuilder::new("n1", "Update Available", "tej-ui v2.0 is ready")
        .subtitle("Software Update")
        .action(NotificationAction::new("install_now", "Install Now"))
        .build();
    assert_eq!(notif.title, "Update Available");
    assert_eq!(notif.actions.len(), 1);

    let mut share = ShareSheetDispatcher::new();
    share.share(SharePayload::Url("https://tej-ui.dev".to_string()));
    assert_eq!(
        share.last_shared(),
        Some(&SharePayload::Url("https://tej-ui.dev".to_string()))
    );

    // 4. Print & PDF Pagination Spooler
    let layout = PageLayout::new(PaperSize::A4, PageMargins::STANDARD_HALF_INCH);
    let printable = layout.printable_area();
    assert!(printable.width() > 500.0);
    assert!(printable.height() > 700.0);

    let spooler = PdfPrintSpooler::new(layout);
    // Content spanning ~2.5 pages
    let pages = spooler.paginate(printable.height() * 2.5);
    assert_eq!(pages.len(), 3);
    assert_eq!(pages[0].page_number, 1);
    assert_eq!(pages[1].page_number, 2);
    assert_eq!(pages[2].page_number, 3);
}

#[test]
fn test_enterprise_domain_12_services() {
    use tej_ui_services::{OtaUpdater, ReleaseChannel, UpdateManifest, UpdateState};

    // 1. OTA Auto-Updater & Cryptographic Signature Verification
    let mut updater = OtaUpdater::new("1.0.0", ReleaseChannel::Stable, "abcdef0123456789");
    assert_eq!(updater.state(), UpdateState::Idle);

    let dummy_sig = "a".repeat(128); // 64-byte hex Ed25519 signature
    let dummy_sha = "b".repeat(64); // 32-byte hex SHA-256
    let valid_manifest = UpdateManifest {
        version: "1.1.0".to_string(),
        channel: ReleaseChannel::Stable,
        download_url: "https://updates.tej.dev/v1.1.0.tar.gz".to_string(),
        sha256_checksum: dummy_sha,
        ed25519_signature_hex: dummy_sig,
        is_critical_security_patch: true,
    };

    assert!(updater.verify_manifest(&valid_manifest));
    updater.transition_to(UpdateState::ReadyToInstall);
    assert_eq!(updater.state(), UpdateState::ReadyToInstall);

    // Rollback
    updater.trigger_rollback();
    assert_eq!(updater.state(), UpdateState::RollbackActive);
}
