use super::common::*;

/// A server running a Docker registry.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegistryHost {
    /// Either a hostname or an IP address.
    pub host: String,

    /// An optional port number.
    pub port: Option<u16>,
}

impl fmt::Display for RegistryHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.host)?;
        if let Some(port) = self.port {
            write!(f, ":{}", port)?;
        }
        Ok(())
    }
}

/// The version of a Docker image.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImageVersion {
    /// A tag, serialized as ":{tag}".
    Tag(String),
    /// A unique hash digest, serialized as "@{digest}".
    Digest(String),
}

impl fmt::Display for ImageVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageVersion::Tag(tag) => write!(f, ":{}", tag),
            ImageVersion::Digest(digest) => write!(f, "@{}", digest),
        }
    }
}

/// The name of an external resource, and an optional local alias to which
/// it is mapped inside a container.  Our fields names are based on the
/// `docker` documentation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Image {
    /// The server running our Docker registry.
    pub registry_host: Option<RegistryHost>,

    /// The name of the user account on the Docker registry.
    pub user_name: Option<String>,

    /// The name of this image.
    pub name: String,

    /// A tag identifying a specific version in our image registry.
    pub version: Option<ImageVersion>,
}

impl Image {
    /// Build an image from an image string.
    pub fn new<S: AsRef<str>>(s: S) -> Result<Image> {
        Ok(FromStr::from_str(s.as_ref())?)
    }

    /// Return the `Image` with the tag removed.
    pub fn without_version(&self) -> Image {
        Image {
            version: None,
            ..self.to_owned()
        }
    }
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref registry_host) = self.registry_host {
            write!(f, "{}/", registry_host)?;
        }
        if let Some(ref user_name) = self.user_name {
            write!(f, "{}/", user_name)?;
        }
        write!(f, "{}", &self.name)?;
        if let Some(ref version) = self.version {
            write!(f, "{}", version)?;
        }
        Ok(())
    }
}

impl FromStr for Image {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            static ref IMAGE: Regex =
                Regex::new(r#"^(?:([^/:.@]+\.[^/:@]+)(?::([0-9]+))?/)?(?:([^/:@.]+)/)?([^:@]+)(?::([^/:@]+)|@(.+))?$"#).unwrap();
        }
        let caps = IMAGE
            .captures(s)
            .ok_or_else(|| Error::invalid_value("image", s))?;
        // This could use a good refactoring.
        let registry_host = if caps.get(1).is_some() {
            // TODO LOW: Is there a special map function for things
            // which might fail?
            let port = if caps.get(2).is_some() {
                Some(
                    FromStr::from_str(caps.get(2).unwrap().as_str())
                        .map_err(|_| Error::invalid_value("image", s))?,
                )
            } else {
                None
            };
            Some(RegistryHost {
                host: caps.get(1).unwrap().as_str().to_owned(),
                port,
            })
        } else {
            None
        };
        let version = if caps.get(5).is_some() {
            Some(ImageVersion::Tag(caps.get(5).unwrap().as_str().to_owned()))
        } else if caps.get(6).is_some() {
            Some(ImageVersion::Digest(
                caps.get(6).unwrap().as_str().to_owned(),
            ))
        } else {
            None
        };
        Ok(Image {
            registry_host,
            user_name: caps.get(3).map(|s| s.as_str().to_owned()),
            name: caps.get(4).unwrap().as_str().to_owned(),
            version,
        })
    }
}

impl_interpolatable_value!(Image);

#[test]
fn parses_standard_image_formats() {
    let img1 = Image {
        registry_host: None,
        user_name: None,
        name: "hello".to_owned(),
        version: None,
    };
    let img2 = Image {
        registry_host: None,
        user_name: Some("example".to_owned()),
        name: "hello".to_owned(),
        version: Some(ImageVersion::Tag("4.4-alpine".to_owned())),
    };
    let img3 = Image {
        registry_host: Some(RegistryHost {
            host: "example.com".to_owned(),
            port: Some(123),
        }),
        user_name: None,
        name: "hello".to_owned(),
        version: Some(ImageVersion::Tag("latest".to_owned())),
    };
    let img4 = Image {
        registry_host: Some(RegistryHost {
            host: "example.com".to_owned(),
            port: None,
        }),
        user_name: Some("staff".to_owned()),
        name: "hello".to_owned(),
        version: None,
    };
    let img5 = Image {
        registry_host: None,
        user_name: Some("user".to_owned()),
        name: "foo/bar".to_owned(),
        version: Some(ImageVersion::Tag("latest".to_owned())),
    };
    let img6 = Image {
        registry_host: Some(RegistryHost {
            host: "example.com".to_owned(),
            port: Some(5000),
        }),
        user_name: Some("test".to_owned()),
        name: "busybox".to_owned(),
        version: Some(ImageVersion::Digest(
            "sha256:cbbf2f9a99b47fc460d422812b6a5adff7dfee951d8fa2e4a98caa0382cfbdbf"
                .to_owned(),
        )),
    };
    let pairs = vec![
        (img1, "hello"),
        (img2, "example/hello:4.4-alpine"),
        (img3, "example.com:123/hello:latest"),
        (img4, "example.com/staff/hello"),
        (img5, "user/foo/bar:latest"),
        (img6, "example.com:5000/test/busybox@sha256:cbbf2f9a99b47fc460d422812b6a5adff7dfee951d8fa2e4a98caa0382cfbdbf"),
        // TODO: Handle `localhost:5000`-style hosts without the ".", which seem to be supported now.
    ];
    for (img, s) in pairs {
        assert_eq!(img.to_string(), s);
        assert_eq!(img, Image::from_str(s).unwrap());
    }
}
