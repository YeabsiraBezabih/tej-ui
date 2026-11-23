//! Project scaffolding generator for new tej-ui applications.

use std::collections::HashMap;

/// Templates generated when running `tej new <project_name>`.
#[derive(Debug, Clone, Default)]
pub struct ProjectScaffolder;

impl ProjectScaffolder {
    /// Generates the manifest and source files for a new tej-ui app.
    pub fn generate_files(project_name: &str) -> HashMap<String, String> {
        let mut files = HashMap::new();

        let cargo_toml = format!(
            r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
tej = "0.1"
"#
        );

        let main_rs = r#"use tej_ui::prelude::*;

fn app_view() -> impl View {
    VStack::new((
        Text::new("Hello from tej-ui!").bold(),
        Button::new("Click me", || {
            println!("Button pressed");
        }),
    ))
    .padding(20.0)
}

fn main() {
    App::new()
        .title("My App")
        .size(800.0, 600.0)
        .run(app_view);
}
"#
        .to_string();

        files.insert("Cargo.toml".to_string(), cargo_toml);
        files.insert("src/main.rs".to_string(), main_rs);

        files
    }
}
