//! Support for reading and writing `docker-compose.yml` files.
//!
//! The `docker-compose.yml` format is suprisingly complex, with support
//! for multiple ways of representing the same data.  This library attemps
//! to provide a single, consistent and type-safe representation of
//! everything found in a [`docker-compose.yml` version 2 file][dcv2].
//!
//! Here's an example of one property, `build`, being represented in two
//! different ways, and how we normalize it:
//!
//! ```
//! use std::str::FromStr;
//! use compose_yml::v2 as dc;
//!
//! let yaml = r#"---
//!
//! version: "2"
//! services:
//!   app1:
//!     build:
//!       context: "./app1"
//!       dockerfile: "Dockerfile-alt"
//!
//!   app2:
//!     build: "./app2"
//!
//! "#;
//!
//! let file = dc::File::from_str(yaml).unwrap();
//!
//! let app1 = file.services.get("app1").unwrap();
//! let build1 = app1.build.as_ref().unwrap();
//! assert_eq!(build1.context, dc::value(dc::Context::new("./app1")));
//! assert_eq!(build1.dockerfile.as_ref().unwrap(),
//!            &dc::value("Dockerfile-alt".to_owned()));
//!
//! // We automatically convert all different `build:` syntaxes
//! // to be consistent.
//! let app2 = file.services.get("app2").unwrap();
//! let build2 = app2.build.as_ref().unwrap();
//! assert_eq!(build2.context, dc::value(dc::Context::new("./app2")));
//! ```
//!
//! An interesting place to start browsing this documentation is
//! `docker_compose::v2::Service`.  You can drill down into other fields
//! from there.
//!
//! [dcv2]: https://docs.docker.com/compose/compose-file/

// Enable clippy if our Cargo.toml file asked us to do so.
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
// Enable as many useful Rust and Clippy warnings as we can stand.  We'd
// also enable `trivial_casts`, but we're waiting for
// https://github.com/rust-lang/rust/issues/23416.
#![warn(missing_copy_implementations, missing_debug_implementations, missing_docs,
        trivial_numeric_casts, unsafe_code, unused_import_braces)]
// We disabled `unused_extern_crates` because it's failing on macro-only crates.
// We disabled `unused_qualifications` because it's failing on `try!`.
#![cfg_attr(feature = "clippy", warn(cast_possible_truncation))]
#![cfg_attr(feature = "clippy", warn(cast_possible_wrap))]
#![cfg_attr(feature = "clippy", warn(cast_precision_loss))]
#![cfg_attr(feature = "clippy", warn(cast_sign_loss))]
#![cfg_attr(feature = "clippy", warn(missing_docs_in_private_items))]
#![cfg_attr(feature = "clippy", warn(mut_mut))]
#![cfg_attr(feature = "clippy", warn(print_stdout))]
// This allows us to use `unwrap` on `Option` values (because doing makes
// working with Regex matches much nicer) and when compiling in test mode
// (because using it in tests is idiomatic).
#![cfg_attr(all(not(test), feature = "clippy"), warn(result_unwrap_used))]
#![cfg_attr(feature = "clippy", warn(wrong_pub_self_convention))]
// rustc_macro-based macros only work with Rust nightly builds, not with
// stable compilers.  We want to work with both.
#![cfg_attr(feature = "serde_derive", feature(plugin, proc_macro))]
// The `error_chain` documentation says we need this.
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;
extern crate url;
extern crate valico;
extern crate void;

pub mod errors;
pub mod v2;
