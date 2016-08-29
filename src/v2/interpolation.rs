//! Interpolation of shell-style variables into strings.

use regex::{Captures, Regex};
use std::env;
use std::error::{self, Error};
use std::fmt::{self, Display};
use std::str::FromStr;

use super::helpers::InvalidValueError;

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

/// Different modes in which we can run `interpolation_helper`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Interpolate environment variables.
    Interpolate,
    /// Unescape an interpolation string if possible, but fail if we would
    /// need to interpolate a value.
    Unescape,
    /// Validate an interpolation string.
    Validate,
}

/// An internal function which handles interpolating, unescaping and
/// validating interpolation strings.  We use a single function for all
/// three to prevent the risk of divergent code paths.
fn interpolate_helper(input: &str, mode: Mode) ->
    Result<String, InterpolationError>
{
    lazy_static! {
        static ref VAR: Regex =
            Regex::new(r#"\$(?:([A-Za-z_][A-Za-z0-9_]+)|\{([A-Za-z_][A-Za-z0-9_]+)\}|(\$)|(.))"#).unwrap();
    }
    let mut err = None;
    let result = VAR.replace_all(input, |caps: &Captures| {
        if caps.at(4).is_some() {
            // Our "fallback" group matched, which means that no valid
            // group matched.  Mark as invalid and return an empty
            // replacement.
            err = Some(InterpolationError::InvalidSyntax(input.to_owned()));
            "".to_owned()
        } else if caps.at(3).is_some() {
            // If we have `$$`, replace it with a single `$`.
            "$".to_owned()
        } else if mode == Mode::Unescape {
            // If we're not allowed to interpolate, bail now.
            err = Some(InterpolationError::InterpolationDisabled(input.to_owned()));
            return "".to_owned();
        } else {
            // Handle actual interpolations.
            let var = caps.at(1).or_else(|| caps.at(2)).unwrap();
            match env::var(var) {
                _ if mode == Mode::Validate => "".to_owned(),
                Ok(val) => val,
                Err(_) => {
                    err = Some(InterpolationError::UndefinedVariable(var.to_owned()));
                    "".to_owned()
                }
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
fn interpolate_env(input: &str) -> Result<String, InterpolationError> {
    interpolate_helper(input, Mode::Interpolate)
}

#[test]
fn interpolate_env_interpolates_env_vars() {
    env::set_var("FOO", "foo");

    assert_eq!("foo", interpolate_env("$FOO").unwrap());
    assert_eq!("foo", interpolate_env("${FOO}").unwrap());
    assert_eq!("foo foo", interpolate_env("$FOO $FOO").unwrap());
    assert_eq!("plain", interpolate_env("plain").unwrap());
    assert_eq!("$escaped", interpolate_env("$$escaped").unwrap());
    assert_eq!("${escaped}", interpolate_env("$${escaped}").unwrap());
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

/// Escape interpolation sequences in a string literal.
pub fn escape_str(input: &str) -> String {
    input.replace("$", "$$")
}

#[test]
fn escape_str_escapes_dollar_signs() {
    assert_eq!("$$VAR1 $${VAR2} $$", escape_str("$VAR1 ${VAR2} $"));
}

/// Unescape any `$$` sequences to `$` in an interpolation string, but fail
/// with an error if we encounter an actual interpolation that would
/// require an environment variable.  This is used for manipulating
/// `docker-compose.yml` files without expanding any environment variables.
pub fn unescape_str(input: &str) -> Result<String, InterpolationError> {
    interpolate_helper(input, Mode::Unescape)
}

#[test]
fn unescape_str_unescapes_without_interpolating() {
    env::set_var("FOO", "foo");

    // Actual interpolation is forbidden.
    assert!(unescape_str("$FOO").is_err());

    assert_eq!("plain", unescape_str("plain").unwrap());
    assert_eq!("$escaped", unescape_str("$$escaped").unwrap());
    assert_eq!("${escaped}", unescape_str("$${escaped}").unwrap());
}

/// Validate an interpolation string, making sure all interpolations look
/// syntactically valid.
fn validate(input: &str) -> Result<(), InterpolationError> {
    interpolate_helper(input, Mode::Validate).map(|_| ())
}

#[test]
fn validate_tests_interpolation_strings() {
    assert!(validate("plain").is_ok());
    assert!(validate("$$escaped").is_ok());
    assert!(validate("$${escaped}").is_ok());
    assert!(validate("$FOO").is_ok());
    assert!(validate("${FOO}").is_ok());

    // See https://github.com/docker/compose/blob/85e2fb63b3309280a602f1f76d77d3a82e53b6c2/tests/unit/interpolation_test.py
    assert!(validate("${").is_err());
    assert!(validate("$}").is_err());
    assert!(validate("${}").is_err());
    assert!(validate("${ }").is_err());
    assert!(validate("${ foo}").is_err());
    assert!(validate("${foo }").is_err());
    assert!(validate("${foo!}").is_err());
}

/// Either a raw, unparsed string, or a value of the specified type.  This
/// is the internal, private implementation of `RawOr`.
enum RawOrValue<T> 
    where T: FromStr<Err = InvalidValueError>+Display
{
    Raw(String),
    Value(T),
}

/// Either an unparsed interpolation string, or a fully-parsed value.  We
/// use this representation because:
///
/// 1. Almost any string value in `docker-compose.yml` may contain an
///    environment variable interpolation of the form `"$VAR"` or
///    `"${VAR}"`, and we normally want to preserve these values in their
///    uninterpolated form when manipulating `docker-compose.yml` files.
/// 2. When we do actually need to manipate a complex string field of a
///    `docker-compose.yml` file, we prefer to do it using the parsed
///    representation.
///
/// Hence `RawOr<T>`, which can represent both unparsed and parsed values,
/// and switch between them in a controlled fashion.
///
/// ```
/// use std::string::ToString;
/// use docker_compose::v2 as dc;
///
/// // We can call `escape`, `value` and `raw` with explicit type
/// // parameters using the following syntax.
/// assert_eq!("bridge",
///            dc::escape::<dc::NetworkMode, _>("bridge").to_string());
///
/// // But typically, when working with `RawOr`, we'll be passing values
/// // into a context where the type is known, allowing type interference
/// // to supply type parameters to the `value`, `escape` and `raw` functions
/// // automatically.  So let's simulate that using a helper function.
/// fn nm_string(nm: dc::RawOr<dc::NetworkMode>) -> String {
///   nm.to_string()
/// }
///
/// // This is how we'll normally create `RawOr` values.
/// assert_eq!("bridge", nm_string(dc::value(dc::NetworkMode::Bridge)));
/// assert_eq!("bridge", nm_string(dc::escape("bridge")));
/// assert_eq!("container:$$FOO", nm_string(dc::escape("container:$FOO")));
/// assert_eq!("$NETWORK_MODE", nm_string(dc::raw("$NETWORK_MODE").unwrap()));
/// ```
pub struct RawOr<T>(RawOrValue<T>)
    where T: FromStr<Err = InvalidValueError>+Display;

/// Convert a raw string containing variable interpolations into a
/// `RawOr<T>` value.  See `RawOr<T>` for examples of how to use this API.
pub fn raw<T, S>(s: S) -> Result<RawOr<T>, InterpolationError>
    where T: FromStr<Err = InvalidValueError>+Display,
          S: Into<String>
{
    let raw: String = s.into();
    try!(validate(&raw));
    Ok(RawOr(RawOrValue::Raw(raw)))
}

/// Escape a string and convert it into a `RawOr<T>` value.  See `RawOr<T>`
/// for examples of how to use this API.
///
/// TODO HIGH: Always parse?
pub fn escape<T, S>(s: S) -> RawOr<T>
    where T: FromStr<Err = InvalidValueError>+Display,
          S: AsRef<str>
{
    RawOr(RawOrValue::Raw(escape_str(s.as_ref())))
}

/// Convert a value into a `RawOr<T>` value.  See `RawOr<T>` for examples
/// of how to use this API.
pub fn value<T, V>(v: V) -> RawOr<T>
    where T: FromStr<Err = InvalidValueError>+Display,
          V: Into<T>
{
    RawOr(RawOrValue::Value(v.into()))
}

impl<T> RawOr<T>
    where T: FromStr<Err = InvalidValueError>+Display
{
    
}

impl<T> fmt::Display for RawOr<T>
    where T: FromStr<Err = InvalidValueError>+Display
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &RawOr(RawOrValue::Raw(ref raw)) => write!(f, "{}", raw),
            &RawOr(RawOrValue::Value(ref value)) => write!(f, "{}", value),
        }
    }
}
