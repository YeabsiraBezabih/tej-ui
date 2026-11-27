//! Project scaffolding generator for new tej-ui applications.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Templates generated when running `tej-ui new <project_name>`.
#[derive(Debug, Clone, Default)]
pub struct ProjectScaffolder;

impl ProjectScaffolder {
    /// Generates the manifest and source files for a new tej-ui app.
    #[must_use]
    pub fn generate_files(project_name: &str) -> HashMap<String, String> {
        let mut files = HashMap::new();

        let cargo_toml = format!(
            r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
tej-ui = "0.1.0"
"#
        );

        let main_rs = format!(
            r#"use tej_ui::prelude::*;

fn app_view() -> impl View {{
    VStack::new((
        Text::new("Hello from tej-ui!"),
        Button::new(Text::new("Click me")).on_click(|| {{
            println!("Button pressed");
        }}),
    ))
}}

fn main() -> Result<(), Box<dyn std::error::Error>> {{
    App::new(app_view())
        .window(WindowConfig {{
            title: String::from("{project_name}"),
            size: Size::new(800.0, 600.0),
            ..WindowConfig::default()
        }})
        .run()
}}
"#
        );

        files.insert("Cargo.toml".to_string(), cargo_toml);
        files.insert("src/main.rs".to_string(), main_rs);

        files
    }

    /// Validates that `project_name` is safe to use as a directory and Cargo package name.
    pub fn validate_project_name(project_name: &str) -> Result<(), String> {
        if project_name.is_empty() {
            return Err("project name cannot be empty".to_string());
        }
        if project_name.contains('/') || project_name.contains('\\') || project_name.contains("..")
        {
            return Err(format!(
                "project name '{project_name}' must not contain path separators or '..'"
            ));
        }
        if !project_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(format!(
                "project name '{project_name}' may only contain ASCII letters, digits, '-', and '_'"
            ));
        }
        Ok(())
    }

    /// Writes the scaffolded project files into `target_dir/<project_name>/`.
    ///
    /// Returns the sorted list of created file paths.
    pub fn write_to_disk(project_name: &str, parent_dir: &Path) -> io::Result<Vec<PathBuf>> {
        Self::validate_project_name(project_name)
            .map_err(|msg| io::Error::new(io::ErrorKind::InvalidInput, msg))?;

        let project_root = parent_dir.join(project_name);
        let files = Self::generate_files(project_name);
        let mut created = Vec::with_capacity(files.len());

        for (rel_path, content) in files {
            let full_path = project_root.join(&rel_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&full_path, content)?;
            created.push(full_path);
        }

        created.sort();
        Ok(created)
    }
}
