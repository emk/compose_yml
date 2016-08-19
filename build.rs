// This is a custom build script which allows us to pretend to support
// real, programmatic Rust macros even when building under stable Rust,
// thanks to serde_codegen, which relies on Syntex to parse and transform
// Rust code before the compiler sees it.  But when we build under nightly
// builds, we let the compiler do its thing normally.
//
// Copied from https://serde.rs/codegen-hybrid.html.

#[cfg(feature = "serde_codegen")]
fn main() {
    extern crate serde_codegen;

    use std::env;
    use std::path::Path;

    let out_dir = env::var_os("OUT_DIR").unwrap();

    let src = Path::new("src/serde_types.in.rs");
    let dst = Path::new(&out_dir).join("serde_types.rs");

    serde_codegen::expand(&src, &dst).unwrap();
}

#[cfg(not(feature = "serde_codegen"))]
fn main() {
    // do nothing
}
