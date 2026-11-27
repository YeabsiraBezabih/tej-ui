//! Comprehensive unit tests for `tej-ui-cli` scaffolding template fixes and disk writing.

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tej_ui_cli::ProjectScaffolder;

#[test]
fn test_scaffold_template_uses_valid_tej_ui_dependency_and_apis() {
    let files = ProjectScaffolder::generate_files("demo-app");

    let cargo_toml = files.get("Cargo.toml").expect("Cargo.toml generated");
    assert!(cargo_toml.contains("name = \"demo-app\""));
    assert!(cargo_toml.contains("tej-ui = \"0.1.0\""));
    assert!(!cargo_toml.contains("\ntej = "));

    let main_rs = files.get("src/main.rs").expect("src/main.rs generated");
    assert!(main_rs.contains("use tej_ui::prelude::*;"));
    assert!(main_rs.contains("Text::new(\"Hello from tej-ui!\")"));
    assert!(main_rs.contains("Button::new(Text::new(\"Click me\")).on_click("));
    assert!(main_rs.contains("App::new(app_view())"));
    assert!(main_rs.contains("WindowConfig"));
}

#[test]
fn test_scaffold_validate_project_name_rejects_traversal() {
    assert!(ProjectScaffolder::validate_project_name("valid_app-1").is_ok());
    assert!(ProjectScaffolder::validate_project_name("").is_err());
    assert!(ProjectScaffolder::validate_project_name("../escape").is_err());
    assert!(ProjectScaffolder::validate_project_name("nested/path").is_err());
    assert!(ProjectScaffolder::validate_project_name("bad name!").is_err());
}

#[test]
fn test_scaffold_write_to_disk_creates_project_files() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temp_root = std::env::temp_dir().join(format!("tej_ui_cli_test_{nonce}"));
    fs::create_dir_all(&temp_root).expect("create temp dir");

    let created = ProjectScaffolder::write_to_disk("sample-app", &temp_root)
        .expect("write scaffolded project to disk");
    assert_eq!(created.len(), 2);

    let project_dir = temp_root.join("sample-app");
    let cargo_toml = fs::read_to_string(project_dir.join("Cargo.toml")).expect("read Cargo.toml");
    let main_rs =
        fs::read_to_string(project_dir.join("src").join("main.rs")).expect("read main.rs");

    assert!(cargo_toml.contains("tej-ui = \"0.1.0\""));
    assert!(main_rs.contains("App::new(app_view())"));

    let _ = fs::remove_dir_all(&temp_root);
}
