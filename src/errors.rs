//! We provide fancy error-handling support thanks to the [`error_chain`
//! crate][error_chain].  The primary advantage of `error_chain` is that it
//! provides support for backtraces.  The secondary advantage of this crate
//! is that it gives us nice, structured error types.
//!
//! [error_chain]: https://github.com/brson/error-chain

// Sadly, this macro does not generate complete documentation.
#![allow(missing_docs)]
#![cfg_attr(feature="clippy", allow(redundant_closure))]

use serde_yaml;
use std::path::PathBuf;

error_chain! {
    // These are external, non-`error_chain` error types that we can
    // automatically wrap.
    foreign_links {
        // The YAML structure in a `docker-compose.yml` file could not be
        // parsed.
        serde_yaml::Error, Yaml;
    }

    // These are our "native" error types.
    errors {
        /// The interpolation syntax in the specified string was invalid.
        InterpolateInvalidSyntax(s: String) {
            description("invalid interpolation syntax")
            display("invalid interpolation syntax '{}'", &s)
        }

        /// The string contains an undefined environment variable.  This is not
        /// an error for `docker-compose` (which treats undefined variables as
        /// empty), but it is an error for us because we're a
        /// `docker-compose.yml` parsing and transforming library, and we
        /// try not to hide errors.
        InterpolateUndefinedVariable(s: String) {
            description("undefined environment variable in interpolation")
            display("undefined environment variable in interpolation '{}'", &s)
        }

        /// We tried to parse a string that requires environment variable
        /// interpolation, but in a context where we've been asked not to
        /// access the environment.  This is typical when transforming
        /// `docker-compose.yml` files that we want to interpolate at a later
        /// time.
        InterpolationDisabled(s: String) {
            description("cannot parse without interpolating environment variables")
            display("cannot parse without interpolating environment variables '{}'",
                    &s)
        }

        /// A string value in a `docker-compose.yml` file could not be
        /// parsed.
        InvalidValue(wanted: String, input: String) {
            description("invalid value")
            display("invalid {} '{}'", &wanted, &input)
        }

        /// An `.env` file could not be parsed.
        ParseEnv(line: String) {
            description("cannot parse env variable declaration")
            display("cannot parse env variable declaration '{}'", &line)
        }

        /// A Git URL was either invalid or not compatible with
        /// `docker-compose`.
        ParseGitUrl(url: String) {
            description("not a Docker-compatible git URL")
            display("not a Docker-compatible git URL '{}'", &url)
        }

        /// An error occurred reading a file.
        ReadFile(path: PathBuf) {
            description("error reading file")
            display("error reading file '{}'", path.display())
        }

        /// An error occurred writing a file.
        WriteFile(path: PathBuf) {
            description("error writing to file")
            display("error writing to file '{}'", path.display())
        }
    }
}

impl Error {
    /// Create an error reporting an invalid value.
    pub fn invalid_value<S1, S2>(wanted: S1, input: S2) -> Error
        where S1: Into<String>, S2: Into<String> {
        ErrorKind::InvalidValue(wanted.into(), input.into()).into()
    }
}
