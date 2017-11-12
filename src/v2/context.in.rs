// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// Either a local directory path, or a Git-format "URL" (not necessarily a
/// real URL, alas).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Context {
    /// A regular local directory.
    Dir(PathBuf),
    /// A Git repository, specified using any of the usual git repository
    /// syntaxes.
    GitUrl(GitUrl),
}

impl Context {
    /// Construct a new Context from a string, identifying it as either a
    /// local path or a remote git repository.
    ///
    /// ```
    /// use compose_yml::v2 as dc;
    /// dc::Context::new("https://github.com/docker/docker.git");
    /// dc::Context::new("src/myapp");
    /// ```
    pub fn new<S: AsRef<str>>(s: S) -> Context {
        let s_ref = s.as_ref();
        if GitUrl::should_treat_as_url(s_ref) {
            // unwrap is safe here because of contract on
            // `should_treat_as_url`.
            Context::GitUrl(GitUrl::new(s_ref.to_owned()).unwrap())
        } else {
            Context::Dir(Path::new(&s_ref).to_owned())
        }
    }

    /// Determines if two Contexts are compatible. This is only true
    /// if they are the same directory path, or if they are two git
    /// URLs which can share a checkout with each other.
    pub fn is_compatible_with(&self, other: &Context) -> bool {
        match (self, other) {
            (&Context::Dir(_), &Context::GitUrl(_)) => false,
            (&Context::GitUrl(_), &Context::Dir(_)) => false,
            (&Context::Dir(ref dir_1), &Context::Dir(ref dir_2)) => dir_1 == dir_2,
            (&Context::GitUrl(ref git_url_1), &Context::GitUrl(ref git_url_2)) => {
                git_url_1.can_share_checkout_with(&git_url_2)
            },
        }
    }
}

impl_interpolatable_value!(Context);

impl FromStr for Context {
    type Err = Void;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        Ok(Context::new(s))
    }
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Context::Dir(ref path) => write!(f, "{}", path.display()),
            &Context::GitUrl(ref url) => write!(f, "{}", url),
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
        let context: Context = FromStr::from_str(url).unwrap();
        assert_eq!(context, Context::GitUrl(GitUrl::new(url.to_string()).unwrap()));
        assert_eq!(context.to_string(), url);
    }
}

#[test]
fn context_may_contain_dir_paths() {
    let paths = vec!(".", "./foo", "./foo/bar/");
    for path in paths {
        let context: Context = FromStr::from_str(path).unwrap();
        assert_eq!(context, Context::Dir(Path::new(path).to_owned()));
        assert_eq!(context.to_string(), path);
    }
}

#[test]
fn is_equivalent_if_they_are_the_same_dir_or_the_same_repo_and_branch() {
    let dir_1: Context = FromStr::from_str("./foo").unwrap();
    let dir_2: Context = FromStr::from_str("./bar").unwrap();

    let plain_repo: Context = FromStr::from_str("git@github.com:docker/docker.git").unwrap();
    let repo_with_branch: Context = FromStr::from_str("git@github.com:docker/docker.git#somebranch").unwrap();
    let repo_with_subdir: Context = FromStr::from_str("git@github.com:docker/docker.git#:somedir").unwrap();
    let repo_with_branch_and_subdir: Context = FromStr::from_str("git@github.com:docker/docker.git#somebranch:somedir").unwrap();

    let different_repo: Context = FromStr::from_str("git@github.com:docker/compose.git").unwrap();

    assert!(dir_1.is_compatible_with(&dir_1));
    assert!(!dir_1.is_compatible_with(&dir_2));
    assert!(!dir_1.is_compatible_with(&plain_repo));

    assert!(plain_repo.is_compatible_with(&plain_repo));
    assert!(plain_repo.is_compatible_with(&repo_with_subdir));

    assert!(!plain_repo.is_compatible_with(&dir_1));
    assert!(!plain_repo.is_compatible_with(&different_repo));
    assert!(!plain_repo.is_compatible_with(&repo_with_branch));
    assert!(!plain_repo.is_compatible_with(&repo_with_branch_and_subdir));

    assert!(repo_with_branch.is_compatible_with(&repo_with_branch));
    assert!(repo_with_branch.is_compatible_with(&repo_with_branch_and_subdir));
    assert!(!repo_with_branch.is_compatible_with(&plain_repo));
}
