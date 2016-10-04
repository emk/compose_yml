//! We provide fancy error-handling support thanks to the [`error_chain`
//! crate][error_chain].  The primary advantage of `error_chain` is that it
//! provides support for backtraces.  The secondary advantage of this crate
//! is that it gives us nice, structured error types.
//!
//! [error_chain]: https://github.com/brson/error-chain

// Sadly, this macro does not generate complete documentation.
#![allow(missing_docs)]

use serde_yaml;
use std::error;
use std::fmt;
use std::path::PathBuf;

use v2::InterpolationError;

error_chain! {
    // These are external, non-`error_chain` error types that we can
    // automatically wrap.
    foreign_links {
        // Something went wrong interpolationg environment variables into
        // a string.
        InterpolationError, Interpolation;
        // A string value in a `docker-compose.yml` file could not be parsed.
        InvalidValueError, InvalidValue;
        // The YAML structure in a `docker-compose.yml` file could not be
        // parsed.
        serde_yaml::Error, Yaml;
    }

    // These are our "native" error types.
    errors {
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

/// An error parsing a string in a Dockerfile.
#[derive(Debug)]
pub struct InvalidValueError {
    /// A semi-human-readable description of type of data we wanted.
    wanted: String,
    /// The actual input data we received.
    input: String,
}

impl InvalidValueError {
    /// Create an error, specifying the type we wanted, and the value we
    /// actually got.
    pub fn new(wanted: &str, input: &str) -> InvalidValueError {
        InvalidValueError {
            wanted: wanted.to_owned(),
            input: input.to_owned(),
        }
    }
}

impl fmt::Display for InvalidValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid {}: <{}>", &self.wanted, &self.input)
    }
}

impl error::Error for InvalidValueError {
    fn description(&self) -> &str {
        "Invalid value"
    }
}
