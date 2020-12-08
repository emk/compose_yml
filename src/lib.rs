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

// Enable as many useful Rust and Clippy warnings as we can stand.  We'd
// also enable `trivial_casts`, but we're waiting for
// https://github.com/rust-lang/rust/issues/23416.
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    trivial_numeric_casts,
    unsafe_code,
    unused_extern_crates,
    unused_import_braces,
    clippy::all
)]

pub mod errors;
pub mod v2;
