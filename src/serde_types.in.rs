// Basic datatypes which can be converted to and from YAML.  Processed
// using serde, either in `serde_macros` mode (with nightly Rust) or in
// `serde_codegen` mode and `build.rs` (with stable Rust).
//
// To get better error messages for this file, build it using the nightly
// release of Rust:
//
// ```sh
// rustup toolchain install nightly
// rustup run nightly cargo build --no-default-features --features unstable
// ```

use regex::Regex;
use serde::de::{Deserialize, Deserializer};
use serde::ser::{self, Serialize, Serializer};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A `docker-compose.yml` file.
#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    /// The individual services which make up this app.
    pub services: HashMap<String, Service>,
}

#[test]
fn file_can_be_converted_from_and_to_yaml() {
    let yaml = "---
services:
  foo:
    build:
      context: .
";
    let file: File = serde_yaml::from_str(&yaml).unwrap();
    let foo = file.services.get("foo").unwrap();
    assert_eq!(foo.build.as_ref().unwrap().context,
               Context::Dir(Path::new(".").to_owned()));

    serde_yaml::to_string(&file).unwrap();
}


/// A service which will be managed by `docker-compose`.
#[derive(Serialize, Deserialize, Debug)]
pub struct Service {
    /// How to build an image for this service.
    pub build: Option<Build>,
}

/// Information on how to build
#[derive(Serialize, Deserialize, Debug)]
pub struct Build {
    /// The source directory to use for this build.
    pub context: Context,
}

/// Either a local directory path, or a Git-format "URL" (not necessarily a
/// real URL, alas).
#[derive(Debug, PartialEq, Eq)]
pub enum Context {
    /// A regular local directory.
    Dir(PathBuf),
    /// A Git repository, specified using any of the usual git repository
    /// syntaxes.
    GitUrl(String),
}

// Custom serializer to output both directories and git URLs as strings.
impl Serialize for Context {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        match self {
            &Context::Dir(ref path) => {
                // Rust is very paranoid: Not all OS paths can be converted
                // to UTF-8 strings!  And we need to handle this.
                let s = try!(path.to_str().ok_or_else(|| {
                    let msg = format!("Could not convert path to UTF-8: {:?}", path);
                    ser::Error::invalid_value(&msg)
                }));
                serializer.serialize_str(&s)
            }
            &Context::GitUrl(ref url) => serializer.serialize_str(url),
        }
    }
}

// Custom deserializer to determine whether we have a directory or a git
// URL.
impl Deserialize for Context {
    fn deserialize<D>(deserializer: &mut D) -> Result<Context, D::Error>
        where D: Deserializer
    {
        // Compile our regex just once.  There's a nice macro for this if
        // we're using nightly Rust.
        lazy_static! {
            // See http://stackoverflow.com/a/34120821/12089
            static ref GIT_PREFIX: Regex =
                Regex::new("^(https?://|git://|github.com/|git@)").unwrap();
        }

        let s: String = try!(Deserialize::deserialize(deserializer));
        if GIT_PREFIX.is_match(&s) {
            Ok(Context::GitUrl(s))
        } else {
            Ok(Context::Dir(Path::new(&s).to_owned()))
        }
    }
}

#[test]
fn context_may_contain_git_urls() {
    // See http://stackoverflow.com/a/34120821/12089
    let urls =
        vec!("git://github.com/docker/docker",
             "git@github.com:docker/docker.git",
             "git@bitbucket.org:atlassianlabs/atlassian-docker.git",
             "https://github.com/docker/docker.git",
             "http://github.com/docker/docker.git",
             "github.com/docker/docker.git");

    for url in urls {
        let yaml = format!("---\n\"{}\"", url);
        let context: Context = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(context, Context::GitUrl(url.to_string()));
        assert_eq!(serde_yaml::to_string(&context).unwrap(), yaml);
    }
}

#[test]
fn context_may_contain_dir_paths() {
    let paths = vec!(".", "./foo", "./foo/bar/");
    for path in paths {
        let yaml = format!("---\n\"{}\"", path);
        let context: Context = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(context, Context::Dir(Path::new(path).to_owned()));
        assert_eq!(serde_yaml::to_string(&context).unwrap(), yaml);
    }
}
