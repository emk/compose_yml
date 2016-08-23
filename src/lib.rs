//! Support for reading and writing `docker-compose.yml` files.

#![warn(missing_docs)]

// Compiler plugins only work with Rust nightly builds, not with stable
// compilers.  We want to work with both.
#![cfg_attr(feature = "serde_macros", feature(plugin, custom_derive))]
#![cfg_attr(feature = "serde_macros", plugin(serde_macros))]

#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate serde;
extern crate serde_yaml;
extern crate void;

pub mod v2;
