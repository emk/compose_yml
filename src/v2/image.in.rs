// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// A server running a Docker registry.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegistryHost {
    /// Either a hostname or an IP address.
    pub host: String,

    /// An optional port number.
    pub port: Option<u16>,
}

impl fmt::Display for RegistryHost {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.host)?;
        if let Some(port) = self.port {
            write!(f, ":{}", port)?;
        }
        Ok(())
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
    pub tag: Option<String>,
}

impl Image {
    /// Build an image from an image string.
    pub fn new<S: AsRef<str>>(s: S) -> Result<Image> {
        Ok(FromStr::from_str(s.as_ref())?)
    }

    /// Return the `Image` with the tag removed.
    pub fn without_tag(&self) -> Image {
        Image {
            tag: None,
            ..self.to_owned()
        }
    }
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref registry_host) = self.registry_host {
            write!(f, "{}/", registry_host)?;
        }
        if let Some(ref user_name) = self.user_name {
            write!(f, "{}/", user_name)?;
        }
        write!(f, "{}", &self.name)?;
        if let Some(ref tag) = self.tag {
            write!(f, ":{}", tag)?;
        }
        Ok(())
    }
}

impl FromStr for Image {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            static ref IMAGE: Regex =
                Regex::new(r#"^(?:([^/:.]+\.[^/:]+)(?::([0-9]+))?/)?(?:([^/:.]+)/)?([^/:]+)(?::([^/:]+))?$"#).unwrap();
        }
        let caps = IMAGE.captures(s).ok_or_else(|| {
            Error::invalid_value("image", s)
        })?;
        // This could use a good refactoring.
        let registry_host =
            if caps.at(1).is_some() {
                // TODO LOW: Is there a special map function for things
                // which might fail?
                let port =
                    if caps.at(2).is_some() {
                        Some(FromStr::from_str(caps.at(2).unwrap()).map_err(|_| {
                            Error::invalid_value("image", s)
                        })?)
                    } else {
                        None
                    };
                Some(RegistryHost {
                    host: caps.at(1).unwrap().to_owned(),
                    port: port,
                })
            } else {
                None
            };
        Ok(Image {
            registry_host: registry_host,
            user_name: caps.at(3).map(|s| s.to_owned()),
            name: caps.at(4).unwrap().to_owned(),
            tag: caps.at(5).map(|s| s.to_owned()),
        })
    }
}

impl_interpolatable_value!(Image);

#[test]
fn parses_stand_image_formats() {
    let img1 = Image {
        registry_host: None,
        user_name: None,
        name: "hello".to_owned(),
        tag: None,
    };
    let img2 = Image {
        registry_host: None,
        user_name: Some("example".to_owned()),
        name: "hello".to_owned(),
        tag: Some("4.4-alpine".to_owned()),
    };
    let img3 = Image {
        registry_host: Some(RegistryHost {
            host: "example.com".to_owned(),
            port: Some(123),
        }),
        user_name: None,
        name: "hello".to_owned(),
        tag: Some("latest".to_owned()),
    };
    let img4 = Image {
        registry_host: Some(RegistryHost {
            host: "example.com".to_owned(),
            port: None,
        }),
        user_name: Some("staff".to_owned()),
        name: "hello".to_owned(),
        tag: None,
    };
    let pairs = vec!(
        (img1, "hello"),
        (img2, "example/hello:4.4-alpine"),
        (img3, "example.com:123/hello:latest"),
        (img4, "example.com/staff/hello"),
    );
    for (img, s) in pairs {
        assert_eq!(img.to_string(), s);
        assert_eq!(img, Image::from_str(s).unwrap());
    }
}
