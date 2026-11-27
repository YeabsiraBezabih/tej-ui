//! Zero-extra-dependency CLI binary for `tej-ui` project scaffolding and diagnostics.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use tej_ui_cli::{ProjectScaffolder, TargetPlatform};

const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

const SUPPORTED_TARGETS: &[(&str, TargetPlatform)] = &[
    ("linux-x64", TargetPlatform::LinuxX64),
    ("linux-arm64", TargetPlatform::LinuxArm64),
    ("windows-x64", TargetPlatform::WindowsX64),
    ("macos-arm64", TargetPlatform::MacOsArm64),
    ("macos-x64", TargetPlatform::MacOsX64),
    ("wasm", TargetPlatform::Wasm),
    ("android", TargetPlatform::Android),
    ("ios", TargetPlatform::Ios),
];

fn print_usage() {
    println!(
        "tej-ui CLI v{CLI_VERSION}\n\
         \n\
         USAGE:\n\
         \ttej-ui new <name> [--dir <path>]    Scaffold a new tej-ui application\n\
         \ttej-ui check [path]                 Validate a tej-ui project directory\n\
         \ttej-ui info                         Show CLI version and supported targets\n\
         \ttej-ui --help | -h                  Show this help message\n\
         \ttej-ui --version | -V               Show CLI version"
    );
}

fn cmd_new(args: &[String]) -> ExitCode {
    let Some(project_name) = args.first() else {
        eprintln!("error: missing <name> argument for `tej-ui new`");
        eprintln!("usage: tej-ui new <name> [--dir <path>]");
        return ExitCode::FAILURE;
    };

    let mut parent_dir = match env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("error: failed to determine current working directory: {err}");
            return ExitCode::FAILURE;
        }
    };

    let mut idx = 1;
    while idx < args.len() {
        match args[idx].as_str() {
            "--dir" | "-d" => {
                let Some(dir_arg) = args.get(idx + 1) else {
                    eprintln!("error: `--dir` requires a directory path argument");
                    return ExitCode::FAILURE;
                };
                parent_dir = PathBuf::from(dir_arg);
                idx += 2;
            }
            unknown => {
                eprintln!("error: unexpected argument '{unknown}' for `tej-ui new`");
                return ExitCode::FAILURE;
            }
        }
    }

    match ProjectScaffolder::write_to_disk(project_name, &parent_dir) {
        Ok(created_files) => {
            println!(
                "Created new tej-ui project '{project_name}' in {}",
                parent_dir.join(project_name).display()
            );
            for path in created_files {
                println!("  wrote {}", path.display());
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: failed to scaffold project '{project_name}': {err}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_check(args: &[String]) -> ExitCode {
    let project_dir = match args.first() {
        Some(path) => PathBuf::from(path),
        None => match env::current_dir() {
            Ok(dir) => dir,
            Err(err) => {
                eprintln!("error: failed to determine current working directory: {err}");
                return ExitCode::FAILURE;
            }
        },
    };

    let cargo_toml = project_dir.join("Cargo.toml");
    let main_rs = project_dir.join("src").join("main.rs");

    if !cargo_toml.is_file() {
        eprintln!(
            "check failed: Cargo.toml not found in {}",
            project_dir.display()
        );
        return ExitCode::FAILURE;
    }

    let manifest = match fs::read_to_string(&cargo_toml) {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!(
                "check failed: unable to read {}: {err}",
                cargo_toml.display()
            );
            return ExitCode::FAILURE;
        }
    };

    let has_tej_ui = manifest.contains("tej-ui");
    let has_entry = main_rs.is_file() || project_dir.join("src").join("lib.rs").is_file();

    if !has_tej_ui {
        eprintln!(
            "check failed: {} does not reference `tej-ui`",
            cargo_toml.display()
        );
        return ExitCode::FAILURE;
    }

    if !has_entry {
        eprintln!(
            "check failed: neither src/main.rs nor src/lib.rs exists in {}",
            project_dir.display()
        );
        return ExitCode::FAILURE;
    }

    println!(
        "ok: valid tej-ui project detected at {}",
        Path::new(&project_dir).display()
    );
    ExitCode::SUCCESS
}

fn cmd_info() -> ExitCode {
    println!("tej-ui CLI v{CLI_VERSION}");
    println!("Edition: 2024");
    println!("Supported target platforms:");
    for (name, target) in SUPPORTED_TARGETS {
        println!("  {name:<14} {}", target.triple());
    }
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some(command) = args.first().map(String::as_str) else {
        print_usage();
        return ExitCode::SUCCESS;
    };

    match command {
        "new" => cmd_new(&args[1..]),
        "check" => cmd_check(&args[1..]),
        "info" => cmd_info(),
        "--help" | "-h" | "help" => {
            print_usage();
            ExitCode::SUCCESS
        }
        "--version" | "-V" => {
            println!("tej-ui {CLI_VERSION}");
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("error: unknown command '{other}'");
            print_usage();
            ExitCode::FAILURE
        }
    }
}
