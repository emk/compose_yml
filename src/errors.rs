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
use std::{
    error::Error as StdError,
    io::{self, Write},
    path::PathBuf,
};
use thiserror::Error;
use valico::json_schema::ValidationState;

/// A `compose_yml` result.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A `compose_yml` error.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// We could not convert a path mounted inside a Docker container to a
    /// Windows path on the host.
    #[error("could not convert {0:?} to the equivalent Windows path")]
    ConvertMountedPathToWindows(String),

    /// A value did not conform to a JSON schema.
    #[error("data did not confirm to schema: {0}")]
    DoesNotConformToSchema(String),

    /// The interpolation syntax in the specified string was invalid.
    #[error("invalid interpolation syntax {0:?}")]
    InterpolateInvalidSyntax(String),

    /// The string contains an undefined environment variable.  This is not
    /// an error for `docker-compose` (which treats undefined variables as
    /// empty), but it is an error for us because we're a
    /// `docker-compose.yml` parsing and transforming library, and we
    /// try not to hide errors.
    #[error("undefined environment variable in interpolation {0:?}")]
    InterpolateUndefinedVariable(String),

    /// We tried to parse a string that requires environment variable
    /// interpolation, but in a context where we've been asked not to
    /// access the environment.  This is typical when transforming
    /// `docker-compose.yml` files that we want to interpolate at a later
    /// time.
    #[error("cannot parse without interpolating environment variables {0:?}")]
    InterpolationDisabled(String),

    /// A string value in a `docker-compose.yml` file could not be
    /// parsed.
    #[error("invalid {wanted} {input:?}")]
    InvalidValue { wanted: String, input: String },

    #[error("I/O error")]
    IoError(#[source] io::Error),

    /// An `.env` file could not be parsed.
    #[error("cannot parse env variable declaration {line:?}")]
    ParseEnv { line: String },

    /// A Git URL was either invalid or not compatible with
    /// `docker-compose`.
    #[error("not a Docker-compatible git URL {url:?}")]
    ParseGitUrl {
        url: String,
        source: Option<Box<dyn StdError + Send + Sync + 'static>>,
    },

    /// An error occurred reading a file.
    #[error("error reading file {:?}", .path.display())]
    ReadFile {
        path: PathBuf,
        source: Box<dyn StdError + Send + Sync + 'static>,
    },

    /// We don't support the specified version of `docker-compose.yml`.
    #[error("unsupported docker-compose.yml version {0:?}")]
    UnsupportedVersion(String),

    /// We were unable to validate a `docker-compose.yml` file.
    #[error("could not validate `docker-compose.yml` file")]
    ValidationFailed {
        source: Box<dyn StdError + Send + Sync + 'static>,
    },

    /// An error occurred writing a file.
    #[error("error writing to file {:?}", .path.display())]
    WriteFile {
        path: PathBuf,
        source: Box<dyn StdError + Send + Sync + 'static>,
    },

    /// An error occurred parsing a YAML structure.
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
}

impl Error {
    /// Create an error reporting a schema validation error.
    pub(crate) fn does_not_conform_to_schema(state: ValidationState) -> Error {
        assert!(!state.is_strictly_valid());
        let mut out: Vec<u8> = vec![];
        for err in &state.errors {
            write!(&mut out, "\n- validation error: {:?}", err)
                .expect("cannot format validation error");
        }
        for url in &state.missing {
            write!(&mut out, "\n- missing {}", url).expect("cannot format URL");
        }
        Error::DoesNotConformToSchema(String::from_utf8_lossy(&out).into_owned())
    }

    /// Create an error reporting an invalid value.
    pub(crate) fn invalid_value<S1, S2>(wanted: S1, input: S2) -> Error
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Error::InvalidValue {
            wanted: wanted.into(),
            input: input.into(),
        }
    }

    /// Create an `Error::ReadFile`.
    pub(crate) fn parse_git_url<E>(url: String, source: E) -> Error
    where
        E: StdError + Send + Sync + 'static,
    {
        Error::ParseGitUrl {
            url,
            source: Some(Box::new(source)),
        }
    }

    /// Create an `Error::ReadFile`.
    pub(crate) fn read_file<P, E>(path: P, source: E) -> Error
    where
        P: Into<PathBuf>,
        E: StdError + Send + Sync + 'static,
    {
        Error::ReadFile {
            path: path.into(),
            source: Box::new(source),
        }
    }

    pub(crate) fn validation_failed<E>(source: E) -> Error
    where
        E: StdError + Send + Sync + 'static,
    {
        Error::ValidationFailed {
            source: Box::new(source),
        }
    }

    /// Create an `Error::WriteFile`.
    pub(crate) fn write_file<P, E>(path: P, source: E) -> Error
    where
        P: Into<PathBuf>,
        E: StdError + Send + Sync + 'static,
    {
        Error::WriteFile {
            path: path.into(),
            source: Box::new(source),
        }
    }
}
