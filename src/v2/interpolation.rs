//! Interpolation of shell-style variables into strings.

use regex::{Captures, Regex};
use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::collections::BTreeMap;
use std::env;
use std::error;
use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::result;
use std::str::FromStr;
use std::string;
use void::Void;

use errors::*;
use super::merge_override::MergeOverride;

/// An error interpolating environment variables in a `docker-compose.yml`
/// file.
#[derive(Debug)]
pub enum InterpolationError {
    /// The interpolation syntax in the specified string was invalid.
    InvalidSyntax(String),
    /// A value was passed to `escape`, but it wasn't parseable as a data
    /// structure of the intended type.
    UnparsableValue(InvalidValueError),
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

impl Display for InterpolationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InterpolationError::InvalidSyntax(ref input) => {
                write!(f, "{}: <{}>", self.description(), input)
            }
            InterpolationError::UnparsableValue(ref err) => {
                write!(f, "{}: {}", self.description(), err)
            }
            InterpolationError::UndefinedVariable(ref var) => {
                write!(f, "{}: {}", self.description(), var)
            }
            InterpolationError::InterpolationDisabled(ref input) => {
                write!(f, "{}: <{}>", self.description(), input)
            }
        }
    }
}

impl error::Error for InterpolationError {
    fn description(&self) -> &str {
        match *self {
            InterpolationError::InvalidSyntax(_) => "invalid interpolation syntax",
            InterpolationError::UnparsableValue(_) => "cannot escape invalid value",
            InterpolationError::UndefinedVariable(_) => {
                "undefined environment variable in interpolation"
            }
            InterpolationError::InterpolationDisabled(_) => {
                "cannot parse without interpolating environment variables"
            }
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            InterpolationError::UnparsableValue(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<InvalidValueError> for InterpolationError {
    fn from(err: InvalidValueError) -> InterpolationError {
        InterpolationError::UnparsableValue(err)
    }
}

/// A source of environment variable values.
pub trait Environment {
    /// Fetch a variable from this environment.  Similar to
    /// `std::env::var`.
    fn var(&self, key: &str) -> result::Result<String, env::VarError>;
}

/// Fetches environment variables from `std::env`.
#[derive(Debug, Default)]
#[allow(missing_copy_implementations)]
pub struct OsEnvironment {
    /// A placeholder to prevent this struct from being directly
    /// constructed.
    _phantom: PhantomData<()>,
}

impl OsEnvironment {
    /// Create a new `OsEnvironment`.
    pub fn new() -> OsEnvironment {
        Default::default()
    }
}

impl Environment for OsEnvironment {
    fn var(&self, key: &str) -> result::Result<String, env::VarError> {
        let result = env::var(key);
        trace!("Read env var {}: {:?}", key, &result);
        result
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
fn interpolate_helper(input: &str,
                      mode: Mode,
                      env: &Environment)
                      -> result::Result<String, InterpolationError> {
    lazy_static! {
        static ref VAR: Regex =
            Regex::new(r#"(?x)
# We found a '$',
\$
# ...but what follows it?
(?:
   # A variable like $FOO
   ([A-Za-z_][A-Za-z0-9_]+)
   |
   # A variable like ${FOO}
   \{([A-Za-z_][A-Za-z0-9_]+)\}
   |
   # An escaped dollar sign?
   (\$)
   |
   # Something else?  In this case, we want to fail.
   (.|$)
)
"#).unwrap();
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
            "".to_owned()
        } else {
            // Handle actual interpolations.
            let var = caps.at(1).or_else(|| caps.at(2)).unwrap();
            match env.var(var) {
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
fn interpolate_env(input: &str,
                   env: &Environment)
                   -> result::Result<String, InterpolationError> {
    interpolate_helper(input, Mode::Interpolate, env)
}

#[test]
fn interpolate_env_interpolates_env_vars() {
    env::set_var("FOO", "foo");
    let env = OsEnvironment::new();

    assert_eq!("foo", interpolate_env("$FOO", &env).unwrap());
    assert_eq!("foo", interpolate_env("${FOO}", &env).unwrap());
    assert_eq!("foo foo", interpolate_env("$FOO $FOO", &env).unwrap());
    assert_eq!("plain", interpolate_env("plain", &env).unwrap());
    assert_eq!("$escaped", interpolate_env("$$escaped", &env).unwrap());
    assert_eq!("${escaped}", interpolate_env("$${escaped}", &env).unwrap());
}

#[test]
fn interpolate_env_returns_an_error_if_input_is_invalid() {
    let env = OsEnvironment::new();

    // See https://github.com/docker/compose/blob/master/
    // tests/unit/interpolation_test.py
    assert!(interpolate_env("$", &env).is_err());
    assert!(interpolate_env("${", &env).is_err());
    assert!(interpolate_env("$}", &env).is_err());
    assert!(interpolate_env("${}", &env).is_err());
    assert!(interpolate_env("${ }", &env).is_err());
    assert!(interpolate_env("${ foo}", &env).is_err());
    assert!(interpolate_env("${foo }", &env).is_err());
    assert!(interpolate_env("${foo!}", &env).is_err());
}

#[test]
fn interpolate_env_returns_an_error_if_variable_is_undefined() {
    let env = OsEnvironment::new();

    // This behavior differs from `docker-compose`, which treats undefined
    // env variables as empty strings.
    env::remove_var("NOSUCH");
    assert!(interpolate_env("$NOSUCH", &env).is_err());
}

/// Escape interpolation sequences in a string literal.
fn escape_str(input: &str) -> String {
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
fn unescape_str(input: &str) -> result::Result<String, InterpolationError> {
    // We can use any `env` we want here; it will be ignored.
    let env = OsEnvironment::new();
    interpolate_helper(input, Mode::Unescape, &env)
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
fn validate(input: &str) -> result::Result<(), InterpolationError> {
    // We can use any `env` we want here; it will be ignored.
    let env = OsEnvironment::new();
    interpolate_helper(input, Mode::Validate, &env).map(|_| ())
}

#[test]
fn validate_tests_interpolation_strings() {
    assert!(validate("plain").is_ok());
    assert!(validate("$$escaped").is_ok());
    assert!(validate("$${escaped}").is_ok());
    assert!(validate("$FOO").is_ok());
    assert!(validate("${FOO}").is_ok());

    // See https://github.com/docker/compose/blob/master/
    // tests/unit/interpolation_test.py
    assert!(validate("${").is_err());
    assert!(validate("$}").is_err());
    assert!(validate("${}").is_err());
    assert!(validate("${ }").is_err());
    assert!(validate("${ foo}").is_err());
    assert!(validate("${foo }").is_err());
    assert!(validate("${foo!}").is_err());
}

/// Local helper trait to convert the different kinds of errors we might
/// receive from `FromStr::Err` into an `InvalidValueError`.  Yeah, this is
/// some abusive template metaprogramming, basically, even though we're not
/// writing C++.
///
/// This will show up as an instance method on all affected types.
pub trait IntoInvalidValueError: error::Error + Sized {
    /// Consume an `Error` and return an `InvalidValueError`.  This is the
    /// default implementation for when an `impl` doesn't override it with
    /// something more specific.
    fn into_invalid_value_error(self, wanted: &str, input: &str) -> InvalidValueError {
        InvalidValueError::new(wanted, input)
    }
}

impl IntoInvalidValueError for InvalidValueError {
    /// We already have the correct type of error, so we override this
    /// function to copy it through.
    fn into_invalid_value_error(self, _: &str, _: &str) -> InvalidValueError {
        self
    }
}

impl IntoInvalidValueError for string::ParseError {
    // Just use the default method in this case.
}

impl IntoInvalidValueError for Void {
    fn into_invalid_value_error(self, _: &str, _: &str) -> InvalidValueError {
        unreachable!()
    }
}

/// A value which can be represented as a string containing environment
/// variable interpolations.  We require a custom `parse` implementation,
/// because we want to handle types that are not necessarily `FromStr`.
pub trait InterpolatableValue: Clone + Eq {
    /// Our equivalent of `from_str`.
    fn iv_from_str(s: &str) -> result::Result<Self, InvalidValueError>;
    /// Our equivalent of `fmt`.
    fn fmt_iv(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

/// Provide a default implementation of InterpolatableValue that works for
/// any type which supports `FromStr<Err = InvalidValueError>` and
/// `Display`.
///
/// Conceptually, this is equivalent to the following, which doesn't work
/// even on nightly Rust with `#[feature(specialization)]` enabled, for
/// some reason that would probably take a long GitHub issues discussion to
/// sort out.
///
/// ```text
/// impl<T, E> InterpolatableValue for T
///     where T: FromStr<Err = E> + Display + Clone + Eq,
///           E: IntoInvalidValueError
/// {
///     default fn iv_from_str(s: &str)
///                            -> std::result::Result<Self, InvalidValueError> {
///         FromStr::from_str(s).map_err(|err| {
///             err.into_invalid_value_error("???", s)
///         })
///     }
///
///     default fn fmt_iv(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
///         self.fmt(f)
///     }
/// }
/// ```
macro_rules! impl_interpolatable_value {
    ($ty:ty) => {
        impl $crate::v2::interpolation::InterpolatableValue for $ty {
            fn iv_from_str(s: &str) ->
                $crate::std::result::Result<Self, $crate::errors::InvalidValueError>
            {
                use $crate::v2::interpolation::IntoInvalidValueError;
                fn convert_err<E>(err: E, input: &str) -> InvalidValueError
                    where E: IntoInvalidValueError
                {
                    err.into_invalid_value_error(stringify!($ty), input)
                }

                FromStr::from_str(s)
                    .map_err(|err| convert_err(err, s))
            }

            fn fmt_iv(&self, f: &mut fmt::Formatter) -> $crate::std::fmt::Result {
                use std::fmt::Display;
                self.fmt(f)
            }
        }
    }
}

impl_interpolatable_value!(String);

/// This can be parsed and formatted, but not using the usual APIs.
impl InterpolatableValue for PathBuf {
    fn iv_from_str(s: &str) -> result::Result<Self, InvalidValueError> {
        Ok(Path::new(s).to_owned())
    }

    fn fmt_iv(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.display().fmt(f)
    }
}

/// A wrapper type to make `format!` call `fmt_iv` instead of `fmt`.
struct DisplayInterpolatableValue<'a, V>(&'a V) where V: 'a + InterpolatableValue;

impl<'a, T> Display for DisplayInterpolatableValue<'a, T>
    where T: InterpolatableValue
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DisplayInterpolatableValue(val) => val.fmt_iv(f),
        }
    }
}

/// Either a raw, unparsed string, or a value of the specified type.  This
/// is the internal, private implementation of `RawOr`.
#[derive(Debug, Clone, PartialEq, Eq)]
enum RawOrValue<T>
    where T: InterpolatableValue
{
    /// A raw value.  Invariant: This is valid, but it contains actual
    /// references to environment variables.  If we can parse a string,
    /// we always do, and we store it as `Value`.
    Raw(String),
    /// A parsed value.
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
/// We normally create `RawOr<T>` values using one of `value`, `escape` or
/// `raw`, as shown below.
///
/// ```
/// use std::string::ToString;
/// use compose_yml::v2 as dc;
///
/// // We can call `escape`, `value` and `raw` with explicit type
/// // parameters using the following syntax.
/// assert_eq!("bridge",
///            dc::escape::<dc::NetworkMode, _>("bridge").unwrap().to_string());
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
/// assert_eq!("bridge", nm_string(dc::escape("bridge").unwrap()));
/// assert_eq!("container:$$FOO", nm_string(dc::escape("container:$FOO").unwrap()));
/// assert_eq!("$NETWORK_MODE", nm_string(dc::raw("$NETWORK_MODE").unwrap()));
///
/// // If we call `escape`, we have to pass it a string which parses to
/// // correct type, or it will return an error.  Similar rules apply to `raw`
/// // if no actual interpolations are present in the string.  This is part of
/// // our "verify as much as possible" philosophy.
/// assert!(dc::escape::<dc::NetworkMode, _>("invalid").is_err());
/// assert!(dc::raw::<dc::NetworkMode, _>("invalid").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawOr<T>(RawOrValue<T>) where T: InterpolatableValue;

/// `InterpolatableValue` is basically just a string that we parse for
/// internal use, so we can merge it as though it were a simple string,
/// without getting into the internal details of whatever it might contain.
/// So go ahead and use the default implementation of `MergeOverride` as if
/// we were a primitive type.
impl<T: InterpolatableValue> MergeOverride for RawOr<T> {}

/// Convert a raw string containing variable interpolations into a
/// `RawOr<T>` value.  See `RawOr<T>` for examples of how to use this API.
pub fn raw<T, S>(s: S) -> result::Result<RawOr<T>, InterpolationError>
    where T: InterpolatableValue,
          S: Into<String>
{
    let raw: String = s.into();
    try!(validate(&raw));
    match unescape_str(&raw) {
        // We can unescape it, so either parse it or fail.
        Ok(unescaped) => {
            let parsed: T = try!(InterpolatableValue::iv_from_str(&unescaped));
            Ok(RawOr(RawOrValue::Value(parsed)))
        }
        // It's valid but we can't unescape it, which means that it contains
        // environment references that we want to leave as raw strings.
        Err(_) => Ok(RawOr(RawOrValue::Raw(raw))),
    }
}

/// Escape a string and convert it into a `RawOr<T>` value.  See `RawOr<T>`
/// for examples of how to use this API.
pub fn escape<T, S>(s: S) -> result::Result<RawOr<T>, InterpolationError>
    where T: InterpolatableValue,
          S: AsRef<str>
{
    let value: T = try!(InterpolatableValue::iv_from_str(s.as_ref()));
    Ok(RawOr(RawOrValue::Value(value)))
}

/// Convert a value into a `RawOr<T>` value, taking ownership of the
/// original value.  See `RawOr<T>` for examples of how to use this API.
pub fn value<T>(v: T) -> RawOr<T>
    where T: InterpolatableValue
{
    RawOr(RawOrValue::Value(v))
}

impl<T> RawOr<T>
    where T: InterpolatableValue
{
    /// Either return a `&T` for this `RawOr<T>`, or return an error if
    /// parsing the value would require performing interpolation.
    ///
    /// ```
    /// use compose_yml::v2 as dc;
    ///
    /// let bridge = dc::value(dc::NetworkMode::Bridge);
    /// assert_eq!(bridge.value().unwrap(), &dc::NetworkMode::Bridge);
    /// ```
    pub fn value(&self) -> result::Result<&T, InterpolationError> {
        match *self {
            RawOr(RawOrValue::Value(ref val)) => Ok(val),
            // Because of invariants on RawOrValue, we know `unescape_str`
            // should always return an error.
            RawOr(RawOrValue::Raw(ref raw)) => Err(unescape_str(raw).unwrap_err()),
        }
    }

    /// Either return a mutable `&mut T` for this `RawOr<T>`, or return an
    /// error if parsing the value would require performing interpolation.
    ///
    /// ```
    /// use compose_yml::v2 as dc;
    ///
    /// let mut mode = dc::value(dc::NetworkMode::Bridge);
    /// *mode.value_mut().unwrap() = dc::NetworkMode::Host;
    /// assert_eq!(mode.value_mut().unwrap(), &dc::NetworkMode::Host);
    /// ```
    pub fn value_mut(&mut self) -> result::Result<&mut T, InterpolationError> {
        match *self {
            RawOr(RawOrValue::Value(ref mut val)) => Ok(val),
            // Because of invariants on RawOrValue, we know `unescape_str`
            // should always return an error.
            RawOr(RawOrValue::Raw(ref raw)) => Err(unescape_str(raw).unwrap_err()),
        }
    }

    /// Return a `&mut T` for this `RawOr<T>`, performing any necessary
    /// environment variable interpolations using the supplied `env` object
    /// and updating the value in place.
    pub fn interpolate_env(&mut self,
                           env: &Environment)
                           -> result::Result<&mut T, InterpolationError> {

        let RawOr(ref mut inner) = *self;

        // We have to very careful about how we destructure this value to
        // avoid winding up with two `mut` references to `self`, and
        // thereby making the borrow checker sad.  This means our code
        // looks very weird.  There may be a way to simplify it.
        //
        // This is one of those fairly rare circumstances where we actually
        // work around the borrow checker in a non-obvious way.
        if let RawOrValue::Value(ref mut val) = *inner {
            // We already have a parsed value, so just return that.
            Ok(val)
        } else {
            let new_val = if let RawOrValue::Raw(ref raw) = *inner {
                let interpolated = try!(interpolate_env(raw, env));
                try!(InterpolatableValue::iv_from_str(&interpolated))
            } else {
                unreachable!()
            };
            *inner = RawOrValue::Value(new_val);
            if let RawOrValue::Value(ref mut val) = *inner {
                Ok(val)
            } else {
                unreachable!()
            }
        }

    }

    /// Return a `&mut T` for this `RawOr<T>`, performing any necessary
    /// environment variable interpolations using the system environment
    /// and updating the value in place.
    ///
    /// ```
    /// use std::env;
    /// use std::str::FromStr;
    /// use compose_yml::v2 as dc;
    ///
    /// env::set_var("NETWORK_MODE", "host");
    /// let mut mode: dc::RawOr<dc::NetworkMode> =
    ///   FromStr::from_str("$NETWORK_MODE").unwrap();
    ///
    /// // Before interpolation.
    /// assert_eq!("$NETWORK_MODE", mode.to_string());
    ///
    /// // Interpolate.
    /// assert_eq!(mode.interpolate().unwrap(), &dc::NetworkMode::Host);
    ///
    /// // After interpolation.
    /// assert_eq!("host", mode.to_string());
    /// ```
    pub fn interpolate(&mut self) -> result::Result<&mut T, InterpolationError> {
        let env = OsEnvironment::new();
        self.interpolate_env(&env)
    }
}

impl<T> Display for RawOr<T>
    where T: InterpolatableValue
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RawOr(RawOrValue::Raw(ref raw)) => write!(f, "{}", raw),
            RawOr(RawOrValue::Value(ref value)) => {
                let s = format!("{}", DisplayInterpolatableValue(value));
                write!(f, "{}", escape_str(&s))
            }
        }
    }
}

impl<T> Serialize for RawOr<T>
    where T: InterpolatableValue
{
    fn serialize<S>(&self, serializer: &mut S) -> result::Result<(), S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<T> FromStr for RawOr<T>
    where T: InterpolatableValue
{
    type Err = InvalidValueError;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        raw(s).map_err(|err| {
            match err {
                // Pass through underlying InvalidValueError.
                InterpolationError::UnparsableValue(err) => err,
                // Otherwise whine about the interpolation.
                //
                // TODO LOW: Add a more descriptive message?
                _ => InvalidValueError::new("interpolation", s),
            }
        })
    }
}

impl<T> Deserialize for RawOr<T>
    where T: InterpolatableValue
{
    fn deserialize<D>(deserializer: &mut D) -> result::Result<Self, D::Error>
        where D: Deserializer
    {
        let string = try!(String::deserialize(deserializer));
        Self::from_str(&string).map_err(|err| de::Error::custom(format!("{}", err)))
    }
}

/// Support for environment variable interpolation.
pub trait InterpolateAll {
    /// Recursively walk over this type, interpolating all `RawOr` values
    /// containing references to the environment.  The default
    /// implementation leaves a value unchanged.
    fn interpolate_all(&mut self) -> result::Result<(), InterpolationError> {
        Ok(())
    }
}

impl InterpolateAll for u16 {}
impl InterpolateAll for u32 {}
impl InterpolateAll for bool {}
impl InterpolateAll for String {}
impl<T> InterpolateAll for PhantomData<T> {}

impl<T: InterpolateAll> InterpolateAll for Option<T> {
    fn interpolate_all(&mut self) -> result::Result<(), InterpolationError> {
        if let Some(ref mut v) = *self {
            try!(v.interpolate_all());
        }
        Ok(())
    }
}

impl<T: InterpolateAll> InterpolateAll for Vec<T> {
    fn interpolate_all(&mut self) -> result::Result<(), InterpolationError> {
        for v in self.iter_mut() {
            try!(v.interpolate_all());
        }
        Ok(())
    }
}

impl<K: Ord + Clone, T: InterpolateAll> InterpolateAll for BTreeMap<K, T> {
    fn interpolate_all(&mut self) -> result::Result<(), InterpolationError> {
        for (_k, v) in self.iter_mut() {
            try!(v.interpolate_all());
        }
        Ok(())
    }
}

impl<T: InterpolatableValue> InterpolateAll for RawOr<T> {
    fn interpolate_all(&mut self) -> result::Result<(), InterpolationError> {
        try!(self.interpolate());
        Ok(())
    }
}

/// Derive `InterpolateAll` for a custom struct type, by recursively
/// interpolating all fields.
macro_rules! derive_interpolate_all_for {
    ($ty:ident, { $( $field:ident ),+ }) => {
        /// Recursive merge all fields in the structure.
        impl $crate::v2::interpolation::InterpolateAll for $ty {
            fn interpolate_all(&mut self) ->
                result::Result<(), $crate::v2::interpolation::InterpolationError>
            {
                $( try!(self.$field.interpolate_all()); )+
                Ok(())
            }
        }
    }
}
