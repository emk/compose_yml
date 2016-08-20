//! Support for the `docker-compose.yml` version 2 file format.

use regex::Regex;
use serde::Error;
use serde::de::{self, Deserialize, Deserializer, MapVisitor, Visitor};
use serde::ser::{self, Serialize, Serializer};
#[cfg(test)] use serde_yaml;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A macro for including another source file directly into this one,
/// without defining a normal submodule, and with support for preprocessing
/// the source code using serde_codegen if necessary.
///
/// We generate as much of our (de)serialization code as possible using
/// serde, either in `serde_macros` mode (with nightly Rust) or in
/// `serde_codegen` mode called by `build.rs` (with stable Rust).
macro_rules! serde_include {
    ( $basename:expr ) => {
        // This code is run if we have a nightly build of Rust, and hence
        // compiler plugins.
        #[cfg(feature = "serde_macros")]
        include!(concat!("v2/", $basename, ".in.rs"));

        // This code is run if we have a stable build of Rust.  The
        // `$preprocessed` file is generated from `$original` by `build.rs`
        // at build time.
        #[cfg(feature = "serde_codegen")]
        include!(concat!(env!("OUT_DIR"), "/v2/", $basename, ".rs"));
    };
}

// Basic file structure.
serde_include!("file");
serde_include!("service");

// Service-related types.
serde_include!("build");
serde_include!("context");
