use crate::frontend::parser::parse_module;
use crate::analysis::checker::TypeChecker;
use crate::core::types::TypeContext;
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CliConfig {
    pub paths: Vec<PathBuf>,
    pub recursive: bool,
    pub strict: bool,
    pub max_errors: usize,
    pub show_suggestions: bool,
    pub color: bool,
    pub parallel: bool,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            paths: Vec::new(),
            recursive: true,
            strict: false,
            max_errors: 100,
            show_suggestions: true,
            color: true,
            parallel: true,
        }
    }
}

pub struct Cli {
    config: CliConfig,
}

impl Cli {
    pub fn new(config: CliConfig) -> Self {
        Self { config }
    }

    pub fn run(&self) -> Result<i32, String> {
        if self.config.paths.is_empty() {
            return Err("No paths specified".to_string());
        }

        let mut all_errors = Vec::new();
        let mut checked_files = 0;

        for path in &self.config.paths {
            if path.is_file() {
                checked_files += 1;
                if let Err(errors) = self.check_file(path) {
                    all_errors.extend(errors);
                }
            } else if path.is_dir() {
                let (count, errors) = self.check_directory(path)?;
                checked_files += count;
                all_errors.extend(errors);
            } else {
                return Err(format!("Path not found: {}", path.display()));
            }
        }

        self.print_summary(checked_files, &all_errors);

        Ok(if all_errors.is_empty() { 0 } else { 1 })
    }

    fn check_file(&self, path: &Path) -> Result<(), Vec<String>> {
        let source = fs::read_to_string(path)
            .map_err(|e| vec![format!("Failed to read {}: {}", path.display(), e)])?;

        let ast = parse_module(&source)
            .map_err(|e| vec![format!("Parse error in {}: {}", path.display(), e)])?;

        let ctx = Arc::new(TypeContext::new());
        let mut checker = TypeChecker::with_context(ctx);
        let errors = checker.check(&ast);

        if !errors.is_empty() {
            let error_messages: Vec<String> = errors.iter()
                .map(|e| format!("{}:{}", path.display(), e))
                .collect();
            return Err(error_messages);
        }

        Ok(())
    }

    fn check_directory(&self, dir: &Path) -> Result<(usize, Vec<String>), String> {
        let mut count = 0;
        let mut all_errors = Vec::new();

        for entry in glob::glob(&format!("{}/**/*.py", dir.display()))
            .map_err(|e| format!("Glob pattern error: {}", e))? {

            match entry {
                Ok(path) => {
                    if path.is_file() {
                        count += 1;
                        if let Err(errors) = self.check_file(&path) {
                            all_errors.extend(errors);
                            if all_errors.len() >= self.config.max_errors {
                                break;
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Error accessing path: {}", e),
            }
        }

        Ok((count, all_errors))
    }

    fn print_summary(&self, file_count: usize, errors: &[String]) {
        if errors.is_empty() {
            self.print_success(&format!("✓ Checked {} files, no errors found", file_count));
        } else {
            for error in errors {
                self.print_error(error);
            }
            eprintln!();
            self.print_error(&format!(
                "Found {} error{} in {} file{}",
                errors.len(),
                if errors.len() == 1 { "" } else { "s" },
                file_count,
                if file_count == 1 { "" } else { "s" }
            ));
        }
    }

    fn print_error(&self, msg: &str) {
        if self.config.color {
            eprintln!("\x1b[31m{}\x1b[0m", msg);
        } else {
            eprintln!("{}", msg);
        }
    }

    fn print_success(&self, msg: &str) {
        if self.config.color {
            println!("\x1b[32m{}\x1b[0m", msg);
        } else {
            println!("{}", msg);
        }
    }
}

pub fn parse_args() -> Result<CliConfig, String> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        return Err(format!(
            "Usage: {} [OPTIONS] <path>...\n\nOptions:\n  \
             --no-recursive  Don't check subdirectories\n  \
             --strict        Enable strict mode\n  \
             --no-color      Disable colored output\n  \
             --max-errors N  Maximum errors to report (default: 100)",
            args[0]
        ));
    }

    let mut config = CliConfig::default();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--no-recursive" => config.recursive = false,
            "--strict" => config.strict = true,
            "--no-color" => config.color = false,
            "--no-parallel" => config.parallel = false,
            "--max-errors" => {
                i += 1;
                if i >= args.len() {
                    return Err("--max-errors requires an argument".to_string());
                }
                config.max_errors = args[i].parse()
                    .map_err(|_| "Invalid value for --max-errors".to_string())?;
            }
            arg if arg.starts_with("--") => {
                return Err(format!("Unknown option: {}", arg));
            }
            path => {
                config.paths.push(PathBuf::from(path));
            }
        }
        i += 1;
    }

    if config.paths.is_empty() {
        return Err("No paths specified".to_string());
    }

    Ok(config)
}

/// Entry point for CLI binary
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_args()?;
    let cli = Cli::new(config);
    let exit_code = cli.run()?;
    std::process::exit(exit_code);
}

