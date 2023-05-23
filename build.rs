use std::env;

fn main() {
    let host = env::var("HOST").unwrap();
    let target = env::var("TARGET").unwrap();
    let is_apple = host.contains("apple") && target.contains("apple");
    let is_linux = host.contains("linux") && target.contains("linux");

    if is_apple {
        println!("cargo:rustc-link-lib=z");
        println!("cargo:rustc-link-lib=uv");
        println!("cargo:rustc-link-lib=ssl");
        println!("cargo:rustc-link-lib=crypto");
        println!("cargo:rustc-link-lib=c++");
    } else if is_linux {
        println!("cargo:rustc-link-arg=/usr/lib/gcc/aarch64-linux-gnu/11/libstdc++.a");
        println!("cargo:rustc-link-arg=/usr/lib/aarch64-linux-gnu/libcrypto.a");
        println!("cargo:rustc-link-arg=/usr/lib/aarch64-linux-gnu/libssl.a");
        println!("cargo:rustc-link-arg=/usr/lib/aarch64-linux-gnu/libuv_a.a");
        println!("cargo:rustc-link-arg=/usr/lib/aarch64-linux-gnu/libz.a");
    } else {
        panic!("Only macOS and Linux are currently supported")
    }
}
