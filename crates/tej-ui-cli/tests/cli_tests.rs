use tej_ui_cli::{BuildConfig, ProjectScaffolder, TargetPlatform};

#[test]
fn test_cli_target_parsing_and_cargo_args() {
    let target = TargetPlatform::parse("windows-x64").unwrap();
    assert_eq!(target, TargetPlatform::WindowsX64);
    assert_eq!(target.triple(), "x86_64-pc-windows-msvc");

    let config = BuildConfig::new(target).with_release(true);
    let args = config.cargo_args();
    assert_eq!(
        args,
        vec!["build", "--target", "x86_64-pc-windows-msvc", "--release"]
    );

    let wasm = TargetPlatform::parse("wasm").unwrap();
    assert_eq!(wasm.triple(), "wasm32-unknown-unknown");
}

#[test]
fn test_cli_project_scaffolding() {
    let files = ProjectScaffolder::generate_files("my-cool-app");
    assert!(files.contains_key("Cargo.toml"));
    assert!(files.contains_key("src/main.rs"));

    let cargo_toml = files.get("Cargo.toml").unwrap();
    assert!(cargo_toml.contains("name = \"my-cool-app\""));
    assert!(cargo_toml.contains("tej = \"0.1\""));

    let main_rs = files.get("src/main.rs").unwrap();
    assert!(main_rs.contains("use tej_ui::prelude::*;"));
    assert!(main_rs.contains("App::new()"));
}
