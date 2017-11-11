//! Utilities for working with git-format "URLs".
//!
//! TODO MED: We may want to promote this upstream to the `docker_compose`
//! crate at some point.

use regex::Regex;
use std::ffi::{OsStr, OsString};
use std::fmt;
use url::Url;

use errors::*;

/// URL of a Git repository.  Git repositories may be specified as either
/// ordinary `http` or `https` URLs, or as `scp`-style remote directory
/// specifiers.
///
/// One of the goals behind this class is to be able to use it as an
/// "enhanced string", much like `PathBuf`, that can be passed to various
/// APIs using conversion via `AsRef` and `From`.  So we implement plenty
/// of conversions, plus `Ord` so we can be used as a key in a `BTreeMap`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GitUrl {
    /// Our URL.
    url: String,
}

impl GitUrl {
    /// Would `docker-compose` interpret this string as a URL?  We check
    /// against a list of known prefixes that trigger URL intepretation in
    /// `docker-compose`.
    pub fn should_treat_as_url<S: AsRef<str>>(s: S) -> bool {
        lazy_static! {
            static ref URL_VALIDATE: Regex =
                Regex::new(r#"^(?:https?://|git://|github\.com/|git@)"#)
                    .unwrap();
        }
        URL_VALIDATE.is_match(s.as_ref())
    }

    /// Create a `GitUrl` from the specified string.  Will only return an
    /// error if `should_treat_as_url` returns false.
    pub fn new<S: Into<String>>(url: S) -> Result<GitUrl> {
        let url = url.into();
        if GitUrl::should_treat_as_url(&url) {
            Ok(GitUrl { url: url })
        } else {
            Err(ErrorKind::ParseGitUrl(url.clone()).into())
        }
    }

    /// Convert a `GitUrl` to a regular `url::Url` object.
    pub fn to_url(&self) -> Result<Url> {
        let mkerr = || ErrorKind::ParseGitUrl(self.url.clone());
        match Url::parse(&self.url) {
            Ok(url) => Ok(url),
            Err(_) => {
                lazy_static! {
                    static ref URL_PARSE: Regex =
                        Regex::new(r#"^(?:git@([^:]+):(.*))|(github\.com/.*)"#)
                            .unwrap();
                }
                let caps = URL_PARSE.captures(&self.url).ok_or_else(&mkerr)?;
                let new = if caps.at(1).is_some() {
                    format!("git://git@{}/{}",
                            caps.at(1).unwrap(),
                            caps.at(2).unwrap())
                } else {
                    format!("https://{}", caps.at(3).unwrap())
                };
                Url::parse(&new).chain_err(&mkerr)
            }
        }
    }

    /// Extract the repository part of the URL
    pub fn repository(&self) -> String {
        lazy_static! {
            static ref REPO_PARSE: Regex = Regex::new(r#"([^#]*)"#).unwrap();
        }
        REPO_PARSE.captures(&self.url).unwrap().at(1).unwrap().to_string()
    }

    /// Extract the optional branch part of the git URL
    pub fn branch(&self) -> Option<String> {
        lazy_static! {
            static ref BRANCH_PARSE: Regex = Regex::new(r#".*#([^:]+)"#).unwrap();
        }
        BRANCH_PARSE.captures(&self.url).map(
            |captures| captures.at(1).unwrap().to_string()
        )
    }

    /// Extract the optional subdirectory part of the git URL
    pub fn subdirectory(&self) -> Option<String> {
        lazy_static! {
            static ref SUBDIR_PARSE: Regex = Regex::new(r#".*#.*:(.+)"#).unwrap();
        }
        SUBDIR_PARSE.captures(&self.url).map(
            |captures| captures.at(1).unwrap().to_string()
        )
    }
}

impl fmt::Display for GitUrl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.url.fmt(f)
    }
}

impl AsRef<str> for GitUrl {
    fn as_ref(&self) -> &str {
        &self.url
    }
}

/// Convert to an `&OsStr`, which makes it easier to use APIs like
/// `std::process::Command` that take `AsRef<OsStr>` for their arguments.
impl AsRef<OsStr> for GitUrl {
    fn as_ref(&self) -> &OsStr {
        self.url.as_ref()
    }
}

impl From<GitUrl> for String {
    fn from(url: GitUrl) -> String {
        From::from(url.url)
    }
}

impl From<GitUrl> for OsString {
    fn from(url: GitUrl) -> OsString {
        From::from(url.url)
    }
}

#[test]
fn to_url_converts_git_urls_to_real_ones() {
    // Example URLs from http://stackoverflow.com/a/34120821/12089,
    // originally from `docker-compose` source code.
    let regular_urls = &["git://github.com/docker/docker",
                         "https://github.com/docker/docker.git",
                         "http://github.com/docker/docker.git"];
    for &url in regular_urls {
        assert_eq!(GitUrl::new(url).unwrap().to_url().unwrap().to_string(), url);
    }

    // According to http://stackoverflow.com/a/34120821/12089, we also need
    // to special-case `git@` and `github.com/` prefixes.
    let fake_urls = &[("git@github.com:docker/docker.git",
                       "git://git@github.com/docker/docker.git"),
                      ("git@bitbucket.org:atlassianlabs/atlassian-docker.git",
                       "git://git@bitbucket.org/atlassianlabs/atlassian-docker.git"),
                      ("github.com/docker/docker.git",
                       "https://github.com/docker/docker.git")];
    for &(fake_url, real_url) in fake_urls {
        assert_eq!(GitUrl::new(fake_url).unwrap().to_url().unwrap().to_string(),
                   real_url);
    }

    let invalid_urls = &["local/path.git"];
    for &url in invalid_urls {
        assert!(GitUrl::new(url).is_err());
    }
}

#[test]
fn it_can_extract_its_repo_branch_and_subdir_parts() {
    let urls = &[
        "git://github.com/docker/docker",
        "https://github.com/docker/docker.git",
        "http://github.com/docker/docker.git",
        "git@github.com:docker/docker.git",
        "git@bitbucket.org:atlassianlabs/atlassian-docker.git",
        "github.com/docker/docker.git",
    ];

    // Refs/folders specified as per:
    // https://docs.docker.com/engine/reference/commandline/build/#git-repositories
    for &url in urls {
        let plain = GitUrl::new(format!("{}{}", url, "")).unwrap();
        assert_eq!(plain.repository(), url);
        assert_eq!(plain.branch(), None);
        assert_eq!(plain.subdirectory(), None);

        let with_ref = GitUrl::new(format!("{}{}", url, "#mybranch")).unwrap();
        assert_eq!(with_ref.repository(), url);
        assert_eq!(with_ref.branch(), Some("mybranch".to_string()));
        assert_eq!(with_ref.subdirectory(), None);

        let with_subdir = GitUrl::new(format!("{}{}", url, "#:myfolder")).unwrap();
        assert_eq!(with_subdir.repository(), url);
        assert_eq!(with_subdir.branch(), None);
        assert_eq!(with_subdir.subdirectory(), Some("myfolder".to_string()));

        let with_ref_and_subdir = GitUrl::new(format!("{}{}", url, "#mybranch:myfolder")).unwrap();
        assert_eq!(with_ref_and_subdir.repository(), url);
        assert_eq!(with_ref_and_subdir.branch(), Some("mybranch".to_string()));
        assert_eq!(with_ref_and_subdir.subdirectory(), Some("myfolder".to_string()));
    }
}
