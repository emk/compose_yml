// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// A server running a Docker registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryHost {
    /// Either a hostname or an IP address.
    pub host: String,

    /// An optional port number.
    pub port: Option<u16>,
}

impl fmt::Display for RegistryHost {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(f, "{}", &self.host));
        if let Some(port) = self.port {
            try!(write!(f, ":{}", port));
        }
        Ok(())
    }
}

/// The name of an external resource, and an optional local alias to which
/// it is mapped inside a container.  Our fields names are based on the
/// `docker` documentation.
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if let Some(ref registry_host) = self.registry_host {
            try!(write!(f, "{}/", registry_host));
        }
        if let Some(ref user_name) = self.user_name {
            try!(write!(f, "{}/", user_name));
        }
        try!(write!(f, "{}", &self.name));
        if let Some(ref tag) = self.tag {
            try!(write!(f, ":{}", tag));
        }
        Ok(())
    }
}

impl FromStr for Image {
    type Err = InvalidValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref IMAGE: Regex =
                Regex::new(r#"^(?:([^/:.]+\.[^/:]+)(?::([0-9]+))?/)?(?:([^/:.]+)/)?([^/:]+)(?::([^/:]+))?$"#).unwrap();
        }
        let caps = try!(IMAGE.captures(s).ok_or_else(|| {
            InvalidValueError::new("image", s)
        }));
        // This could use a good refactoring.
        let registry_host =
            if caps.at(1).is_some() {
                // TODO LOW: Is there a special map function for things
                // which might fail?
                let port =
                    if caps.at(2).is_some() {
                        Some(try!(FromStr::from_str(caps.at(2).unwrap()).map_err(|_| {
                            InvalidValueError::new("image", s)
                        })))
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
