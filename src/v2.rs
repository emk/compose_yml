//! `docker-compose.yml` version 2 file format.

#[cfg(test)] use serde_yaml;

// This code is run if we have a nightly build of Rust, and hence compiler
// plugins.
#[cfg(feature = "serde_macros")]
include!("serde_types.in.rs");

// This code is run if we have a stable build of Rust.  `serde_types.rs` is
// generated from `serde_types.in.rs` by `build.rs` at build time.
#[cfg(feature = "serde_codegen")]
include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));
