//! Minimal example showing runtime usage

fn main() {
    // Initialize runtime
    typthon_runtime::typthon_runtime_init();

    println!("Runtime initialized successfully");

    // Cleanup
    typthon_runtime::typthon_runtime_cleanup();
}

