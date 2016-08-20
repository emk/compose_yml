// This is a custom build script which allows us to pretend to support
// real, programmatic Rust macros even when building under stable Rust,
// thanks to serde_codegen, which relies on Syntex to parse and transform
// Rust code before the compiler sees it.  But when we build under nightly
// builds, we let the compiler do its thing normally.
//
// Copied from https://serde.rs/codegen-hybrid.html and adapted to support
// multiple input files, because we have a ton of data structures.

#[cfg(feature = "serde_codegen")]
fn main() {
    extern crate glob;
    extern crate serde_codegen;

    use std::env;
    use std::path::Path;

    let out_dir = env::var_os("OUT_DIR").unwrap();

    // Switch to our `src` directory so that we have the right base for our
    // globs, and so that we won't need to strip `src/` off every path.
    env::set_current_dir("src").unwrap();

    for entry in glob::glob("**/*.in.rs").expect("Failed to read glob pattern") {
        match entry {
            Ok(src) => {
                let mut dst = Path::new(&out_dir).join(&src);

                // Change ".in.rs" to ".rs".
                dst.set_file_name(src.file_stem().expect("Failed to get file stem"));
                dst.set_extension("rs");

                serde_codegen::expand(&src, &dst).unwrap();
            }
            Err(e) => {
                panic!("Error globbing: {}", e);
            }
        }
    }
}

#[cfg(not(feature = "serde_codegen"))]
fn main() {
    // do nothing
}
