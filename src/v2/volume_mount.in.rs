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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeMount {
    /// If this volume is external to the container, where should we find
    /// it?
    pub host: Option<HostVolume>,
    /// Where should we mount this volume in the container?  This must be
    /// an absolute path.
    pub container: PathBuf,
    /// What should the permissions of this volume be in the container?
    pub permissions: VolumePermissions,

    /// PRIVATE.  Mark this struct as having unknown fields for future
    /// compatibility.  This prevents direct construction and exhaustive
    /// matching.  This needs to be be public because of
    /// http://stackoverflow.com/q/39277157/12089
    #[doc(hidden)]
    pub _phantom: PhantomData<()>,
}

impl VolumeMount {
    /// Map a host path to a container path.
    ///
    /// ```
    /// use compose_yml::v2 as dc;
    /// dc::VolumeMount::host("./src", "/app");
    /// ```
    pub fn host<P1, P2>(host: P1, container: P2) -> VolumeMount
        where P1: Into<PathBuf>, P2: Into<PathBuf>
    {
        VolumeMount {
            host: Some(HostVolume::Path(host.into())),
            container: container.into(),
            permissions: Default::default(),
            _phantom: PhantomData,
        }
    }

    /// Map a named volume to a container path.
    ///
    /// ```
    /// use compose_yml::v2 as dc;
    /// dc::VolumeMount::named("pgvolume", "/app");
    /// ```
    pub fn named<S, P>(name: S, container: P) -> VolumeMount
        where S: Into<String>, P: Into<PathBuf>
    {
        VolumeMount {
            host: Some(HostVolume::Name(name.into())),
            container: container.into(),
            permissions: Default::default(),
            _phantom: PhantomData,
        }
    }

    /// An anonymous persistent volume which will remain associated with
    /// this service when it is recreated.
    pub fn anonymous<P>(container: P) -> VolumeMount
        where P: Into<PathBuf>
    {
        VolumeMount {
            host: None,
            container: container.into(),
            permissions: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl_interpolatable_value!(VolumeMount);

impl fmt::Display for VolumeMount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

impl FromStr for VolumeMount {
    type Err = InvalidValueError;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let items = s.split(":").collect::<Vec<_>>();
        match items.len() {
            1 => {
                Ok(VolumeMount {
                    host: None,
                    container: Path::new(items[0]).to_owned(),
                    permissions: Default::default(),
                    _phantom: PhantomData,
                })
            }
            2 => {
                Ok(VolumeMount {
                    host: Some(try!(FromStr::from_str(items[0]))),
                    container: Path::new(items[1]).to_owned(),
                    permissions: Default::default(),
                    _phantom: PhantomData,
                })
            }
            3 => {
                Ok(VolumeMount {
                    host: Some(try!(FromStr::from_str(items[0]))),
                    container: Path::new(items[1]).to_owned(),
                    permissions: try!(FromStr::from_str(items[2])),
                    _phantom: PhantomData,
                })
            }
            _ => Err(InvalidValueError::new("volume", s)),
        }
    }
}

#[test]
fn volume_mounts_should_have_string_representations() {
    let vol1 = VolumeMount::anonymous("/var/lib");
    let vol2 = VolumeMount::named("named", "/var/lib");
    let vol3 = VolumeMount {
        permissions: VolumePermissions::ReadOnly,
        ..VolumeMount::host("/etc/foo", "/etc/myfoo")
    };

    let pairs = vec!(
        (vol1, "/var/lib"),
        (vol2, "named:/var/lib"),
        (vol3, "/etc/foo:/etc/myfoo:ro"),
    );
    for (mode, s) in pairs {
        assert_eq!(mode.to_string(), s);
        assert_eq!(mode, VolumeMount::from_str(s).unwrap());
    }
}
