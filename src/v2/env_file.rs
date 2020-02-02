//! Support for parsing the files pointed to by `env_file:`.

use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

use errors::*;
use super::interpolation::{escape, RawOr};

/// A file pointed to by an `env_file:` field.
pub struct EnvFile {
    /// The variables found in our env file.
    vars: BTreeMap<String, Option<String>>,
}

impl EnvFile {
    /// Read an `EnvFile` from a stream.
    pub fn read<R: io::Read>(input: R) -> Result<EnvFile> {
        let mut vars: BTreeMap<String, Option<String>> = BTreeMap::new();
        let reader = io::BufReader::new(input);
        for line_result in reader.lines() {
            let line = line_result.chain_err(|| "I/O error")?;

            lazy_static! {
                static ref BLANK: Regex =
                    Regex::new(r#"^\s*(:?#.*)?$"#).unwrap();
                // We allow lowercase env vars even if POSIX doesn't.
                static ref VAR:  Regex =
                    Regex::new(r#"^([_A-Za-z][_A-Za-z0-9]*)(=(.*))?"#).unwrap();
            }

            if BLANK.is_match(&line) {
                continue;
            }

            let caps = VAR.captures(&line)
                .ok_or_else(|| ErrorKind::ParseEnv(line.clone()))?;
            vars.insert(
                caps.get(1).unwrap().as_str().to_owned(),
                caps.get(3).map(|v| v.as_str().to_owned()),
            );
        }
        Ok(EnvFile { vars: vars })
    }

    /// Load an `EnvFile` from the disk.
    pub fn load(path: &Path) -> Result<EnvFile> {
        let mkerr = || ErrorKind::ReadFile(path.to_owned());
        let f = fs::File::open(path).chain_err(&mkerr)?;
        EnvFile::read(io::BufReader::new(f)).chain_err(&mkerr)
    }

    /// Convert this `EnvFile` to the format we use for the `environment`
    /// member of `Service`.
    pub fn to_environment(&self) -> Result<BTreeMap<String, Option<RawOr<String>>>> {
        let mut env = BTreeMap::new();
        for (k, v) in &self.vars {
            env.insert(k.to_owned(), match v.as_ref().map(|v| escape(v)) {
                None => None,
                Some(v) => Some(v?),
            });
        }
        Ok(env)
    }

    // TODO MED: We'll need this when we fix the type of
    // `Service::environment` to have values of `RawOr<String>`.
    //
    // /// Convert to a valid `Service::environment` value.
    // pub fn to_env(&self) -> &BTreeMap<String, RawOr<String>> {
    // }
}

#[test]
fn parses_docker_compatible_env_files() {
    let input = r#"
# This is a comment.
# This is a blank line:

# These are environment variables:
FOO=foo
BAR=2
BAZ

# Docker does not currently do anything special with quotes!
WEIRD="quoted"

# TODO LOW: What if an .env file contains a shell variable interpolation?
"#;
    let cursor = io::Cursor::new(input);
    let env_file = EnvFile::read(cursor).unwrap();
    let env = env_file.to_environment().unwrap();
    assert_eq!(env.get("FOO").unwrap().as_ref().unwrap().value().unwrap(), "foo");
    assert_eq!(env.get("BAR").unwrap().as_ref().unwrap().value().unwrap(), "2");
    assert_eq!(*env.get("BAZ").unwrap(), None);
    assert_eq!(env.get("WEIRD").unwrap().as_ref().unwrap().value().unwrap(), "\"quoted\"");
}
