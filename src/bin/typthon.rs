use typthon::{TypeChecker, TypeContext, parse_module};
use std::sync::Arc;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct Config {
    files: Vec<PathBuf>,
    strict: bool,
    no_color: bool,
}

impl Config {
    fn from_args() -> Result<Self, String> {
        let args: Vec<String> = std::env::args().collect();

        if args.len() < 2 {
            return Err(Self::usage(&args[0]));
        }

        let mut files = Vec::new();
        let mut strict = false;
        let mut no_color = false;

        for arg in &args[1..] {
            match arg.as_str() {
                "--help" | "-h" => return Err(Self::usage(&args[0])),
                "--strict" => strict = true,
                "--no-color" => no_color = false,
                path if !path.starts_with("--") => files.push(PathBuf::from(path)),
                opt => return Err(format!("Unknown option: {}\n\n{}", opt, Self::usage(&args[0]))),
            }
        }

        if files.is_empty() {
            return Err("No files specified".to_string());
        }

        Ok(Self { files, strict, no_color })
    }

    fn usage(prog: &str) -> String {
        format!(
            "Typthon - Advanced Type Checker for Python\n\n\
            USAGE:\n    {} [OPTIONS] <files...>\n\n\
            OPTIONS:\n    \
            -h, --help      Print help information\n    \
            --strict        Enable strict type checking\n    \
            --no-color      Disable colored output\n\n\
            EXAMPLES:\n    \
            {} script.py\n    \
            {} --strict src/**/*.py\n    \
            {} --no-color myfile.py",
            prog, prog, prog, prog
        )
    }
}

fn print_errors(errors: &[String], file: &PathBuf, config: &Config) {
    if errors.is_empty() {
        return;
    }

    let file_display = file.display();

    for error in errors {
        if config.no_color {
            eprintln!("{}:{}", file_display, error);
        } else {
            eprintln!("\x1b[31m{}:{}\x1b[0m", file_display, error);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = match Config::from_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let ctx = Arc::new(TypeContext::new());
    let mut checker = TypeChecker::with_context(ctx.clone());

    let mut total_errors = 0;

    for file in &config.files {
        let source = match fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading {}: {}", file.display(), e);
                continue;
            }
        };

        let ast = match parse_module(&source) {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("Parse error in {}: {}", file.display(), e);
                total_errors += 1;
                continue;
            }
        };

        let errors = checker.check(&ast);
        let error_strs: Vec<String> = errors.iter().map(|e| e.to_string()).collect();

        total_errors += error_strs.len();
        print_errors(&error_strs, file, &config);
    }

    if total_errors > 0 {
        eprintln!("\nFound {} error(s)", total_errors);
        std::process::exit(1);
    } else {
        if !config.no_color {
            println!("\x1b[32m✓ All checks passed\x1b[0m");
        } else {
            println!("✓ All checks passed");
        }
    }

    Ok(())
}

