//! Interpolation of shell-style variables into strings.

use regex::{Captures, Regex};
use std::env;
use std::error::{self, Error};
use std::fmt;

/// An error interpolating environment variables in a `docker-compose.yml`
/// file.
#[derive(Debug)]
pub enum InterpolationError {
    /// The interpolation syntax in the specified string was invalid.
    InvalidSyntax(String),
    /// The string contains an undefined environment variable.  This is not
    /// an error for `docker-compose` (which treats undefined variables as
    /// empty), but it is an error for us because we're a
    /// `docker-compose.yml` parsing and transforming library, and we
    /// try not to hide errors.
    UndefinedVariable(String),
    /// We tried to parse a string that requires environment variable
    /// interpolation, but in a context where we've been asked not to
    /// access the environment.  This is typical when transforming
    /// `docker-compose.yml` files that we want to interpolate at a later
    /// time.
    InterpolationDisabled(String),
}

impl fmt::Display for InterpolationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &InterpolationError::InvalidSyntax(ref input) =>
                write!(f, "{}: <{}>", self.description(), input),
            &InterpolationError::UndefinedVariable(ref var) =>
                write!(f, "{}: {}", self.description(), var),
            &InterpolationError::InterpolationDisabled(ref input) =>
                write!(f, "{}: <{}>", self.description(), input),
        }
    }
}

impl error::Error for InterpolationError {
    fn description(&self) -> &str {
        match self {
            &InterpolationError::InvalidSyntax(_) =>
                "invalid interpolation syntax",
            &InterpolationError::UndefinedVariable(_) =>
                "undefined environment variable in interpolation",
            &InterpolationError::InterpolationDisabled(_) =>
                "cannot parse without interpolating environment variables",
        }
    }
}

/// An internal function which handles both interpolations and invalidating
/// interpolation strings.
fn interpolate_helper(input: &str, interpolate: bool) ->
    Result<String, InterpolationError>
{
    lazy_static! {
        static ref VAR: Regex =
            Regex::new(r#"\$(?:([A-Za-z_][A-Za-z0-9_]+)|\{([A-Za-z_][A-Za-z0-9_]+)\}|(\$)|(.))"#).unwrap();
    }
    let mut err = None;
    let result = VAR.replace_all(input, |caps: &Captures| {
        // Our "fallback" group matched, which means that no valid group
        // matched.  Mark as invalid and return an empty replacement.
        if caps.at(4).is_some() {
            err = Some(InterpolationError::InvalidSyntax(input.to_owned()));
            return "".to_owned();
        }
        // If we have `$$`, replace it with a single `$`. 
        if caps.at(3).is_some() {
            return "$".to_owned();
        }
        // If we're not allowed to interpolate, bail now.
        if !interpolate {
            err = Some(InterpolationError::InterpolationDisabled(input.to_owned()));
            return "".to_owned();
        }
        // Handle actual interpolations.
        let var = caps.at(1).or_else(|| caps.at(2)).unwrap();
        match env::var(var) {
            Ok(val) => val,
            Err(_) => {
                err = Some(InterpolationError::UndefinedVariable(var.to_owned()));
                return "".to_owned();
            }
        }
    });
    if let Some(e) = err {
        return Err(e);
    }
    Ok(result)
}

/// Interpolate environment variables into a string using the same rules as
/// `docker-compose.yml`.
///
/// ```
/// use docker_compose::v2::interpolation::interpolate_env;
/// use std::env;
///
/// env::set_var("FOO", "foo");
///
/// assert_eq!("foo", interpolate_env("$FOO").unwrap());
/// assert_eq!("foo", interpolate_env("${FOO}").unwrap());
/// assert_eq!("foo foo", interpolate_env("$FOO $FOO").unwrap());
///
/// assert_eq!("plain", interpolate_env("plain").unwrap());
/// assert_eq!("$escaped", interpolate_env("$$escaped").unwrap());
/// assert_eq!("${escaped}", interpolate_env("$${escaped}").unwrap());
/// ```
pub fn interpolate_env(input: &str) -> Result<String, InterpolationError> {
    interpolate_helper(input, true)
}

#[test]
fn interpolate_env_returns_an_error_if_input_is_invalid() {
    // See https://github.com/docker/compose/blob/85e2fb63b3309280a602f1f76d77d3a82e53b6c2/tests/unit/interpolation_test.py
    assert!(interpolate_env("${").is_err());
    assert!(interpolate_env("$}").is_err());
    assert!(interpolate_env("${}").is_err());
    assert!(interpolate_env("${ }").is_err());
    assert!(interpolate_env("${ foo}").is_err());
    assert!(interpolate_env("${foo }").is_err());
    assert!(interpolate_env("${foo!}").is_err());
}

#[test]
fn interpolate_env_returns_an_error_if_variable_is_undefined() {
    // This behavior differs from `docker-compose`, which treats undefined
    // env variables as empty strings.
    env::remove_var("NOSUCH");
    assert!(interpolate_env("$NOSUCH").is_err());
}

/// Validate an interpolation string and unescape any `$$` sequences to
/// `$`, but fail with an error if we encounter an actual interpolation
/// that would require an environment variable.  This is used for
/// manipulating `docker-compose.yml` files without expanding any
/// environment variables.
///
/// ```
/// use docker_compose::v2::interpolation::interpolate_without_env;
/// use std::env;
///
/// env::set_var("FOO", "foo");
///
/// // Actual interpolation is forbidden.
/// assert!(interpolate_without_env("$FOO").is_err());
///
/// assert_eq!("plain", interpolate_without_env("plain").unwrap());
/// assert_eq!("$escaped", interpolate_without_env("$$escaped").unwrap());
/// assert_eq!("${escaped}", interpolate_without_env("$${escaped}").unwrap());
/// ```
pub fn interpolate_without_env(input: &str) -> Result<String, InterpolationError> {
    interpolate_helper(input, false)
}
