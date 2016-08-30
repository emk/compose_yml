// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// Where can we find the volume we want to map into a container?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostVolume {
    /// This volume corresponds to a path on the host.  It may be a
    /// relative or absolute path.
    Path(PathBuf),
    /// A path relative to the current user's home directory on the host.
    /// Must be a relative path.
    UserRelativePath(PathBuf),
    /// This volume corresponds to a volume named in the top-level
    /// `volumes` section.
    Name(String),
}

impl fmt::Display for HostVolume {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &HostVolume::Path(ref path) => {
                let p = try!(path.to_str().ok_or(fmt::Error));
                if path.is_absolute() {
                    write!(f, "{}", p)
                } else if p.starts_with("./") || p.starts_with("../") {
                    write!(f, "{}", p)
                } else {
                    // Relative paths must begin with `./` when serialized.
                    write!(f, "./{}", p)
                }
            }
            &HostVolume::UserRelativePath(ref path) => {
                let p = try!(path.to_str().ok_or(fmt::Error));
                if path.is_absolute() {
                    return Err(fmt::Error);
                }
                write!(f, "~/{}", p)
            }
            &HostVolume::Name(ref name) => {
                write!(f, "{}", name)
            }
        }
    }
}

impl FromStr for HostVolume {
    type Err = InvalidValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref HOST_VOLUME: Regex =
                Regex::new(r#"^(\.{0,2}/.*)|~/(.+)|([^./~].*)$"#).unwrap();
        }
        let caps = try!(HOST_VOLUME.captures(s).ok_or_else(|| {
            InvalidValueError::new("host volume", s)
        }));
        if let Some(path) = caps.at(1) {
            Ok(HostVolume::Path(Path::new(path).to_owned()))
        } else if let Some(path) = caps.at(2) {
            Ok(HostVolume::UserRelativePath(Path::new(path).to_owned()))
        } else if let Some(name) = caps.at(3) {
            Ok(HostVolume::Name(name.to_owned()))
        } else {
            unreachable!()
        }
    }
}

/// A volume associated with a service.
///
/// TODO: Rename to `Mount` or `VolumeMount`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceVolume {
    /// If this volume is external to the container, where should we find
    /// it?  We don't attempt to parse this because the format is
    /// tricky--it can contain variable interpolation, `~/`-relative paths,
    /// and volume names.
    pub host: Option<HostVolume>,
    /// Where should we mount this volume in the container?  This must be
    /// an absolute path.
    pub container: PathBuf,
    /// What should the permissions of this volume be in the container?
    pub permissions: VolumePermissions,
}

impl_interpolatable_value!(ServiceVolume);

impl fmt::Display for ServiceVolume {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // We can't have permissions on a purely internal volume, if I'm
        // reading this correctly.
        if self.host.is_none() && self.permissions != Default::default() {
            return Err(fmt::Error);
        }

        match &self.host {
            &Some(ref host) => try!(write!(f, "{}:", host)),
            &None => {},
        }

        let containerstr = try!(self.container.to_str().ok_or(fmt::Error));
        try!(write!(f, "{}", containerstr));

        if self.permissions != Default::default() {
            try!(write!(f, ":{}", self.permissions))
        }

        Ok(())
    }
}

impl FromStr for ServiceVolume {
    type Err = InvalidValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let items = s.split(":").collect::<Vec<_>>();
        match items.len() {
            1 => {
                Ok(ServiceVolume {
                    host: None,
                    container: Path::new(items[0]).to_owned(),
                    permissions: Default::default(),
                })
            }
            2 => {
                Ok(ServiceVolume {
                    host: Some(try!(FromStr::from_str(items[0]))),
                    container: Path::new(items[1]).to_owned(),
                    permissions: Default::default(),
                })
            }
            3 => {
                Ok(ServiceVolume {
                    host: Some(try!(FromStr::from_str(items[0]))),
                    container: Path::new(items[1]).to_owned(),
                    permissions: try!(FromStr::from_str(items[2])),
                })
            }
            _ => Err(InvalidValueError::new("volume", s)),
        }
    }           
}

#[test]
fn service_volumes_should_have_string_representations() {
    let vol1 = ServiceVolume {
        host: None,
        container: Path::new("/var/lib").to_owned(),
        permissions: Default::default(),
    };
    let vol2 = ServiceVolume {
        host: Some(HostVolume::Name("named".to_owned())),
        container: Path::new("/var/lib").to_owned(),
        permissions: Default::default(),
    };
    let vol3 = ServiceVolume {
        host: Some(HostVolume::Path(Path::new("/etc/foo").to_owned())),
        container: Path::new("/etc/myfoo").to_owned(),
        permissions: VolumePermissions::ReadOnly,
    };

    let pairs = vec!(
        (vol1, "/var/lib"),
        (vol2, "named:/var/lib"),
        (vol3, "/etc/foo:/etc/myfoo:ro"),
    );
    for (mode, s) in pairs {
        assert_eq!(mode.to_string(), s);
        assert_eq!(mode, ServiceVolume::from_str(s).unwrap());
    }
}
