extern crate napi_build;

fn main() {
    napi_build::setup();
    println!("cargo:rustc-link-arg-tests=-Wl,--unresolved-symbols=ignore-all");
}
