// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// A volume associated with a service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceVolume {
    /// If this volume is external to the container, where should we find
    /// it?  We don't attempt to parse this because the format is
    /// tricky--it can contain variable interpolation, `~/`-relative paths,
    /// and volume names.
    pub host: Option<String>,
    /// Where should we mount this volume in the container?  This must be
    /// an absolute path.
    pub container: PathBuf,
    /// What should the permissions of this volume be in the container?
    pub permissions: VolumePermissions,
}

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

impl_serialize_to_string!(ServiceVolume);

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
                    host: Some(items[0].to_owned()),
                    container: Path::new(items[1]).to_owned(),
                    permissions: Default::default(),
                })
            }
            3 => {
                Ok(ServiceVolume {
                    host: Some(items[0].to_owned()),
                    container: Path::new(items[1]).to_owned(),
                    permissions: try!(FromStr::from_str(items[2])),
                })
            }
            _ => Err(InvalidValueError::new("volume", s)),
        }
    }           
}

impl_deserialize_from_str!(ServiceVolume);

#[test]
fn service_volumes_should_have_string_representations() {
    let vol1 = ServiceVolume {
        host: None,
        container: Path::new("/var/lib").to_owned(),
        permissions: Default::default(),
    };
    let vol2 = ServiceVolume {
        host: Some("named".to_owned()),
        container: Path::new("/var/lib").to_owned(),
        permissions: Default::default(),
    };
    let vol3 = ServiceVolume {
        host: Some("/etc/foo".to_owned()),
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
