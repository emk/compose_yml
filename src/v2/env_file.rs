//! Support for parsing the files pointed to by `env_file:`.

use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

use super::Error;

/// A file pointed to by an `env_file:` field.
pub struct EnvFile {
    vars: BTreeMap<String, String>,
}

impl EnvFile {
    /// Read an `EnvFile` from a stream.
    pub fn read<R: io::Read>(input: R) -> Result<EnvFile, Error> {
        let mut vars: BTreeMap<String, String> = BTreeMap::new();
        let reader = io::BufReader::new(input);
        for line_result in reader.lines() {
            let line = try!(line_result);

            lazy_static! {
                static ref BLANK: Regex =
                    Regex::new(r#"^\s*(:?#.*)?$"#).unwrap();
                // We allow lowercase env vars even if POSIX doesn't.
                static ref VAR:  Regex =
                    Regex::new(r#"^([_A-Za-z][_A-Za-z0-9]*)=(.*)"#).unwrap();
            }

            if BLANK.is_match(&line) { continue; }

            let caps = try!(VAR.captures(&line).ok_or_else(|| {
                err!("can't parse env var declaration: <{}>", &line)
            }));
            vars.insert(caps.at(1).unwrap().to_owned(),
                        caps.at(2).unwrap().to_owned());
        }
        Ok(EnvFile { vars: vars })
    }

    /// Load an `EnvFile` from the disk.
    pub fn load(path: &Path) -> Result<EnvFile, Error> {
        EnvFile::read(try!(fs::File::open(path).map_err(|e| {
            err!("can't read {}: {}", path.display(), e)
        })))
    }

    /// The variable mappings as simple BTreeMap.
    pub fn as_map(&self) -> &BTreeMap<String, String> {
        &self.vars
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

# Docker does not currently do anything special with quotes!
WEIRD="quoted"

# TODO LOW: What if an .env file contains a shell variable interpolation?
"#;
    let cursor = io::Cursor::new(input);
    let env_file = EnvFile::read(cursor).unwrap();
    let env = env_file.as_map();
    assert_eq!(env.get("FOO").unwrap(), "foo");
    assert_eq!(env.get("BAR").unwrap(), "2");
    assert_eq!(env.get("WEIRD").unwrap(), "\"quoted\"");
}
