//! Support for the `docker-compose.yml` version 2 file format.

#[cfg(windows)]
use regex::Captures;
use regex::Regex;
use serde;
use serde::de::{self, Deserialize, Deserializer, SeqAccess, Visitor};
use serde::ser::{Serialize, Serializer};
use serde_yaml;
use std::borrow::ToOwned;
use std::collections::BTreeMap;
use std::convert::Into;
use std::default::Default;
#[cfg(test)]
use std::env;
use std::fs;
use std::fmt;
use std::io;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::result;
use std::str::FromStr;
use void::Void;

use self::helpers::*;
use self::env_file::EnvFile;
pub use self::git_url::GitUrl;
pub use self::interpolation::{RawOr, raw, escape, value, InterpolateAll, Environment,
                              OsEnvironment};
pub use self::merge_override::MergeOverride;
pub use self::mode_enum::*;
use self::string_or_struct::*;
use self::true_or_struct::*;
use self::validate::validate_file;

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
mod validate;

#[cfg(test)]
macro_rules! assert_roundtrip {
    ( $ty:ty, $yaml:expr ) => {
        {
            use serde_json;
            let yaml: &str = $yaml;
            let data: $ty = serde_yaml::from_str(&yaml).unwrap();
            let serialized = serde_json::to_value(&data).unwrap();
            let expected: serde_json::Value = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(serialized, expected);
        }
    }
}

// TODO: We use `include!` instead of submodules because of how `serde` used
// to work long ago. These should be converted to submodules.

// Support types.
include!("aliased_name.rs");
include!("command_line.rs");
include!("memory_size.rs");
include!("permissions.rs");
include!("host_mapping.rs");
include!("image.rs");

// Basic file structure.
include!("file.rs");
include!("service.rs");
include!("volume.rs");
include!("network.rs");

// Service-related types.
include!("build.rs");
include!("context.rs");
include!("extends.rs");
include!("logging.rs");
include!("network_interface.rs");
include!("port_mapping.rs");
include!("volume_mount.rs");
include!("volumes_from.rs");

// Network-related types.
include!("external_network.rs");
