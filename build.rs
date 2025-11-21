use std::env;
use std::path::PathBuf;

fn main() {
    let mut build = cc::Build::new();

    build
        .cpp(true)
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-O3")
        .include("typthon-core/runtime/cpp/include")
        .file("typthon-core/runtime/cpp/src/core/types.cpp")
        .file("typthon-core/runtime/cpp/src/ffi/ffi.cpp");

    // Architecture-specific SIMD optimizations
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    match target_arch.as_str() {
        "x86_64" | "x86" => {
            build.flag_if_supported("-march=native");
            build.flag_if_supported("-mavx2");
        }
        "aarch64" | "arm" => {
            build.flag_if_supported("-march=native");
        }
        _ => {}
    }

    // Platform-specific optimizations
    if cfg!(target_os = "macos") {
        build.flag("-stdlib=libc++");
    }

    build.compile("typthon_cpp");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search=native={}", out_path.display());
    println!("cargo:rustc-link-lib=static=typthon_cpp");

    // Link C++ stdlib
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=c++");
    } else {
        println!("cargo:rustc-link-lib=stdc++");
    }

    println!("cargo:rerun-if-changed=typthon-core/runtime/cpp/include/typthon/core/types.hpp");
    println!("cargo:rerun-if-changed=typthon-core/runtime/cpp/src/core/types.cpp");
    println!("cargo:rerun-if-changed=typthon-core/runtime/cpp/include/typthon/ffi/ffi.hpp");
    println!("cargo:rerun-if-changed=typthon-core/runtime/cpp/src/ffi/ffi.cpp");
    println!("cargo:rerun-if-changed=typthon-core/runtime/cpp/include/typthon/typthon.hpp");
}

