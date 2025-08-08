use std::env;
use std::path::PathBuf;
use std::process::Command;

fn clang_builtin_include_path() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("clang")
        .args(&["-print-resource-dir"])
        .output()?;

    if output.status.success() {
        let resource_dir = String::from_utf8(output.stdout)?;
        let resource_dir = resource_dir.trim();
        Ok(format!("{}/include", resource_dir))
    } else {
        Err("Failed to get clang resource directory".into())
    }
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let c_files = [
        "c/fake_exit.c",
        "c/fake_io.c",
        "c/fake_kill.c",
        "c/fake_stub.c",
        "c/fake_sysconf.c",
        "c/fake_clock.c",
        "c/fake_entropy.c",
        "c/memalign.c",
    ];

    let mut build = cc::Build::new();

    build
        .flag("-fPIC")
        .flag("-nostdinc")
        .flag("-ffreestanding")
        .flag("-fno-builtin")
        .flag("-Wall")
        .flag("-Wextra")
        .flag("-mrdrnd")
        .opt_level(2)
        .compiler("clang");

    let Ok(builtin_include) = clang_builtin_include_path() else {
        panic!("Could not determine clang builtin include path");
    };

    build.include(&builtin_include);
    build.include("third-party/libc/include");
    build.define("_GNU_SOURCE1", "1");
    build.define("_POSIX_MONOTONIC_CLOCK", "1");
    build.define("_POSIX_C_SOURCE", "200809L");

    for file in &c_files {
        build.file(file);
        println!("cargo:rerun-if-changed={}", file);
    }

    build.compile("fakesys");

    println!("cargo:rustc-link-lib=static=fakesys");
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-search=native=third-party/libcxx/lib/x86_64-unknown-none");
    println!(
        "cargo:rustc-link-search=native=third-party/libcxx/lib/clang/21/lib/x86_64-unknown-none"
    );
    println!("cargo:rustc-link-search=native=third-party/libc/lib");

    println!("cargo:rustc-link-lib=c");
    println!("cargo:rustc-link-lib=m");
    println!("cargo:rustc-link-lib=c++");
    println!("cargo:rustc-link-lib=c++abi");
    println!("cargo:rustc-link-lib=unwind");
    println!("cargo:rustc-link-lib=clang_rt.builtins");

    println!("cargo:rustc-link-arg=-e");
    println!("cargo:rustc-link-arg=entrypoint");

    println!("cargo:rerun-if-changed=build.rs");
}
