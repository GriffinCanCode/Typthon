use typthon::frontend;

fn main() {
    if let Err(e) = frontend::cli_main() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

