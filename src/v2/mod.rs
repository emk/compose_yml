//! Support for the `docker-compose.yml` version 2 file format.

use regex::Regex;
use serde;
use serde::de::{self, Deserialize, Deserializer, SeqVisitor, Visitor};
use serde::ser::{Serialize, Serializer};
use serde_yaml;
use std::borrow::ToOwned;
use std::collections::BTreeMap;
use std::convert::Into;
use std::default::Default;
use std::fs;
use std::fmt;
use std::io;
use std::marker::PhantomData;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::result;
use std::str::FromStr;
use void::Void;

use self::helpers::*;
use self::env_file::EnvFile;
pub use self::git_url::GitUrl;
pub use self::interpolation::{InterpolationError, RawOr, raw, escape, value,
                              InterpolateAll, Environment, OsEnvironment};
pub use self::merge_override::MergeOverride;
pub use self::mode_enum::*;
use self::string_or_struct::*;
use self::true_or_struct::*;

// Re-export errors here so that people can use them by including `use
// compose_yml::v2`.
pub use errors::*;

mod helpers;
mod env_file;
mod git_url;
#[macro_use]
mod interpolation;
mod string_or_struct;
mod true_or_struct;
#[macro_use]
mod merge_override;
mod mode_enum;
#[macro_use]
mod derive;

macro_rules! assert_roundtrip {
    ( $ty:ty, $yaml:expr ) => {
        {
            let yaml: &str = $yaml;
            let data: $ty = serde_yaml::from_str(&yaml).unwrap();
            let yaml2 = serde_yaml::to_string(&data).unwrap();
            assert_eq!(normalize_yaml(yaml), normalize_yaml(&yaml2));
        }
    }
}

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
        include!(concat!($basename, ".in.rs"));

        // This code is run if we have a stable build of Rust.  The
        // `$preprocessed` file is generated from `$original` by `build.rs`
        // at build time.
        #[cfg(feature = "serde_codegen")]
        include!(concat!(env!("OUT_DIR"), "/v2/", $basename, ".rs"));
    };
}

// Support types.
serde_include!("aliased_name");
serde_include!("command_line");
serde_include!("memory_size");
serde_include!("permissions");
serde_include!("host_mapping");
serde_include!("image");

// Basic file structure.
serde_include!("file");
serde_include!("service");
serde_include!("network");

// Service-related types.
serde_include!("build");
serde_include!("context");
serde_include!("extends");
serde_include!("logging");
serde_include!("network_interface");
serde_include!("port_mapping");
serde_include!("volume_mount");
serde_include!("volumes_from");

// Network-related types.
serde_include!("external_network");
