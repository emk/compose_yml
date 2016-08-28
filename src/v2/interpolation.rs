//! Interpolation of shell-style variables into strings.

use regex::{Captures, Regex};
use std::env;

use super::helpers::*;

/// Interpolate environment variables into a string using the same rules as
/// `docker-compose.yml`.
///
/// ```
/// use docker_compose::v2 as dc;
/// use std::env;
///
/// env::set_var("FOO", "foo");
///
/// assert_eq!("foo", dc::interpolate_env("$FOO").unwrap());
/// assert_eq!("foo", dc::interpolate_env("${FOO}").unwrap());
/// assert_eq!("foo foo", dc::interpolate_env("$FOO $FOO").unwrap());
///
/// assert_eq!("plain", dc::interpolate_env("plain").unwrap());
/// assert_eq!("$escaped", dc::interpolate_env("$$escaped").unwrap());
/// assert_eq!("${escaped}", dc::interpolate_env("$${escaped}").unwrap());
/// ```
pub fn interpolate_env(input: &str) -> Result<String, InvalidValueError> {
    lazy_static! {
        static ref VAR: Regex =
            Regex::new(r#"\$(?:([A-Za-z_][A-Za-z0-9_]+)|\{([A-Za-z_][A-Za-z0-9_]+)\}|(\$)|(.))"#).unwrap();
    }
    let mut invalid = false;
    let result = VAR.replace_all(input, |caps: &Captures| {
        // Our "fallback" group matched, which means that no valid group
        // matched.  Mark as invalid and return an empty replacement.
        if caps.at(4).is_some() {
            invalid = true;
            return "".to_owned();
        }
        // If we have `$$`, replace it with a single `$`. 
        if caps.at(3).is_some() {
            return "$".to_owned();
        }
        // Handle actual interpolations.
        let var = caps.at(1).or_else(|| caps.at(2)).unwrap();
        match env::var(var) {
            Ok(val) => val,
            Err(_) => {
                invalid = true;
                return "".to_owned();
            }
        }
    });
    if invalid {
        return Err(InvalidValueError::new("interpolation", input));
    }
    Ok(result)
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
