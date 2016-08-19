// Compiler plugins only work with Rust nightly builds, not with stable
// compilers.  We want to work with both.
#![cfg_attr(feature = "serde_macros", feature(plugin, custom_derive))]
#![cfg_attr(feature = "serde_macros", plugin(serde_macros))]

extern crate serde;
extern crate serde_yaml;

// This code is run if we have a nightly build of Rust, and hence compiler
// plugins.
#[cfg(feature = "serde_macros")]
include!("serde_types.in.rs");

// This code is run if we have a stable build of Rust.  `serde_types.rs` is
// generated from `serde_types.in.rs` by `build.rs` at build time.
#[cfg(feature = "serde_codegen")]
include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));

#[test]
fn it_serializes_as_yaml() {
    let point = Point { x: 1, y: 2 };
    let s = serde_yaml::to_string(&point).unwrap();
    assert_eq!(s, "---\n\"x\": 1\n\"y\": 2");
}
