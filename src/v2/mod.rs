//! Support for the `docker-compose.yml` version 2 file format.

// Re-export errors here so that people can use them by including `use
// compose_yml::v2`.
pub use crate::errors::*;

mod env_file;
mod git_url;
mod helpers;
#[macro_use]
mod interpolation;
mod string_or_struct;
mod true_or_struct;
#[macro_use]
mod merge_override;
mod mode_enum;
#[macro_use]
mod validate;

pub use git_url::GitUrl;
pub use interpolation::{escape, raw, value, Environment, RawOr};
pub use merge_override::MergeOverride;
pub use mode_enum::{IpcMode, NetworkMode, PidMode, RestartMode};

#[cfg(test)]
macro_rules! assert_roundtrip {
    ( $ty:ty, $yaml:expr ) => {{
        use serde_json;
        let yaml: &str = $yaml;
        let data: $ty = serde_yaml::from_str(&yaml).unwrap();
        let serialized = serde_json::to_value(&data).unwrap();
        let expected: serde_json::Value = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(serialized, expected);
    }};
}

macro_rules! derive_standard_impls_for {
    ($ty:ident, { $( $field:ident ),+ }) => {
        derive_interpolate_all_for!($ty, { $( $field ),+ });
        derive_merge_override_for!($ty, { $( $field ),+ });
    }
}

// TODO: We use `include!` instead of submodules because of how `serde` used
// to work long ago. These should be converted to submodules.

// Support types.
mod aliased_name;
mod command_line;
mod host_mapping;
mod image;
mod memory_size;
mod volume_modes;
mod permissions;

// Basic file structure.
mod file;
mod network;
mod service;
mod volume;

// Service-related types.
mod build;
mod context;
mod extends;
mod logging;
mod network_interface;
mod port_mapping;
mod ulimit;
mod volume_mount;
mod volumes_from;

// Network-related types.
mod external_network;

// Re-export from our child modules.
pub use aliased_name::*;
pub use build::*;
pub use command_line::*;
pub use context::*;
pub use extends::*;
pub use external_network::*;
pub use file::*;
pub use host_mapping::*;
pub use image::*;
pub use logging::*;
pub use memory_size::*;
pub use network::*;
pub use network_interface::*;
pub use permissions::*;
pub use port_mapping::*;
pub use service::*;
pub use ulimit::*;
pub use volume::*;
pub use volume_modes::*;
pub use volume_mount::*;
pub use volumes_from::*;

pub(self) mod common {
    pub(crate) use lazy_static::lazy_static;
    #[cfg(windows)]
    pub(crate) use regex::Captures;
    pub(crate) use regex::Regex;
    pub(crate) use serde;
    pub(crate) use serde::{Deserialize, Serialize, Serializer};
    pub(crate) use serde_yaml;
    pub(crate) use std::borrow::ToOwned;
    pub(crate) use std::collections::BTreeMap;
    pub(crate) use std::convert::Into;
    pub(crate) use std::default::Default;
    #[cfg(test)]
    pub(crate) use std::env;
    pub(crate) use std::fmt;
    pub(crate) use std::fs;
    pub(crate) use std::io;
    pub(crate) use std::net::IpAddr;
    pub(crate) use std::path::{Path, PathBuf};
    pub(crate) use std::result;
    pub(crate) use std::str::FromStr;
    pub(crate) use void::Void;

    pub(crate) use super::env_file::EnvFile;
    pub(crate) use super::helpers::{
        deserialize_item_or_list, deserialize_map_or_default_list,
        deserialize_map_or_key_value_list, deserialize_map_struct_or_null, is_false,
    };
    pub(crate) use super::interpolation::InterpolateAll;
    pub(crate) use super::string_or_struct::{
        deserialize_opt_string_or_struct, serialize_opt_string_or_struct,
        SerializeStringOrStruct,
    };
    pub(crate) use super::true_or_struct::{
        deserialize_opt_true_or_struct, serialize_opt_true_or_struct,
    };
    pub(crate) use super::validate::validate_file;

    pub(crate) use super::*;
}
