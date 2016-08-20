// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

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

impl Context {
    /// Construct a new Context from a string, identifying it as either a
    /// local path or a remote git repository.
    fn new(s: &str) -> Context {
        // Compile our regex just once.  There's a nice macro for this if
        // we're using nightly Rust, but lazy_static works on stable.
        lazy_static! {
            // See http://stackoverflow.com/a/34120821/12089
            static ref GIT_PREFIX: Regex =
                Regex::new("^(https?://|git://|github.com/|git@)").unwrap();
        }

        if GIT_PREFIX.is_match(&s) {
            Context::GitUrl(s.to_owned())
        } else {
            Context::Dir(Path::new(&s).to_owned())
        }
    }
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
        let s: String = try!(Deserialize::deserialize(deserializer));
        Ok(Context::new(&s))
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
