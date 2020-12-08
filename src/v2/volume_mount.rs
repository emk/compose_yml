use super::common::*;

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HostVolume::Path(path) => {
                let p = path_str_to_docker(path.to_str().ok_or(fmt::Error)?);
                if path.is_absolute() || p.starts_with("./") || p.starts_with("../") {
                    write!(f, "{}", p)
                } else {
                    // Relative paths must begin with `./` when serialized.
                    write!(f, "./{}", p)
                }
            }
            HostVolume::UserRelativePath(path) => {
                let p = path.to_str().ok_or(fmt::Error)?;
                if path.is_absolute() {
                    return Err(fmt::Error);
                }
                write!(f, "~/{}", p)
            }
            HostVolume::Name(name) => write!(f, "{}", name),
        }
    }
}

/// Leave non-Windows paths unchanged.
#[cfg(not(windows))]
fn path_str_to_docker(s: &str) -> String {
    s.to_owned()
}

/// Fix Windows paths to have the syntax that they'll have inside the Docker
/// container.
#[cfg(windows)]
fn path_str_to_docker(s: &str) -> String {
    lazy_static! {
        static ref DRIVE_LETTER: Regex =
            Regex::new(r#"^(?P<letter>[A-Za-z]):\\"#).unwrap();
    }
    DRIVE_LETTER
        .replace(s, |caps: &Captures| {
            format!("/{}/", caps.name("letter").unwrap().as_str().to_lowercase())
        })
        .replace("\\", "/")
}

impl FromStr for HostVolume {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            static ref HOST_VOLUME: Regex =
                Regex::new(r#"^(\.{0,2}/.*)|~/(.+)|([^./~].*)$"#).unwrap();
        }
        let caps = HOST_VOLUME
            .captures(s)
            .ok_or_else(|| Error::invalid_value("host volume", s))?;
        if let Some(path) = caps.get(1) {
            let fixed_path = path_str_from_docker(path.as_str())?;
            Ok(HostVolume::Path(Path::new(&fixed_path).to_owned()))
        } else if let Some(path) = caps.get(2) {
            let fixed_path = path_str_from_docker(path.as_str())?;
            Ok(HostVolume::UserRelativePath(
                Path::new(&fixed_path).to_owned(),
            ))
        } else if let Some(name) = caps.get(3) {
            Ok(HostVolume::Name(name.as_str().to_owned()))
        } else {
            unreachable!()
        }
    }
}

/// Leave non-Windows paths unchanged.
#[cfg(not(windows))]
fn path_str_from_docker(s: &str) -> Result<String> {
    Ok(s.to_owned())
}

/// Convert from Docker path syntax to Windows path syntax.
#[cfg(windows)]
fn path_str_from_docker(s: &str) -> Result<String> {
    if s.starts_with("/") {
        lazy_static! {
            static ref DRIVE_LETTER: Regex =
                Regex::new(r#"/(?P<letter>[A-Za-z])/"#).unwrap();
        }

        if DRIVE_LETTER.is_match(s) {
            Ok(DRIVE_LETTER.replace(s, "$letter:\\").replace("/", "\\"))
        } else {
            Err(Error::ConvertMountedPathToWindows(s.to_owned()))
        }
    } else {
        Ok(s.replace("/", "\\"))
    }
}

/// A volume associated with a service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeMount {
    /// If this volume is external to the container, where should we find
    /// it?
    pub host: Option<HostVolume>,
    /// Where should we mount this volume in the container?  This must be
    /// an absolute path.  This is a string, because on Windows, it will
    /// use a different path representation than the host OS.
    pub container: String,
    /// What should the mode of this volume be in the container?
    pub mode: VolumeModes,

    /// PRIVATE.  Mark this struct as having unknown fields for future
    /// compatibility.  This prevents direct construction and exhaustive
    /// matching.  This needs to be be public because of
    /// http://stackoverflow.com/q/39277157/12089
    #[doc(hidden)]
    pub _hidden: (),
}

impl VolumeMount {
    /// Map a host path to a container path.
    ///
    /// ```
    /// use compose_yml::v2 as dc;
    /// dc::VolumeMount::host("./src", "/app");
    /// ```
    pub fn host<P1, P2>(host: P1, container: P2) -> VolumeMount
    where
        P1: Into<PathBuf>,
        P2: Into<String>,
    {
        VolumeMount {
            host: Some(HostVolume::Path(host.into())),
            container: container.into(),
            mode: Default::default(),
            _hidden: (),
        }
    }

    /// Map a named volume to a container path.
    ///
    /// ```
    /// use compose_yml::v2 as dc;
    /// dc::VolumeMount::named("pgvolume", "/app");
    /// ```
    pub fn named<S, P>(name: S, container: P) -> VolumeMount
    where
        S: Into<String>,
        P: Into<String>,
    {
        VolumeMount {
            host: Some(HostVolume::Name(name.into())),
            container: container.into(),
            mode: Default::default(),
            _hidden: (),
        }
    }

    /// An anonymous persistent volume which will remain associated with
    /// this service when it is recreated.
    pub fn anonymous<P>(container: P) -> VolumeMount
    where
        P: Into<String>,
    {
        VolumeMount {
            host: None,
            container: container.into(),
            mode: Default::default(),
            _hidden: (),
        }
    }
}

impl_interpolatable_value!(VolumeMount);

impl fmt::Display for VolumeMount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // We can't have mode on a purely internal volume, if I'm
        // reading this correctly.
        if self.host.is_none() && self.mode != Default::default() {
            return Err(fmt::Error);
        }

        match &self.host {
            Some(host) => write!(f, "{}:", host)?,
            None => {}
        }

        write!(f, "{}", &self.container)?;

        if self.mode != Default::default() {
            write!(f, ":{}", self.mode)?
        }

        Ok(())
    }
}

impl FromStr for VolumeMount {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let items = s.split(':').collect::<Vec<_>>();
        match items.len() {
            1 => Ok(VolumeMount {
                host: None,
                container: items[0].to_owned(),
                mode: Default::default(),
                _hidden: (),
            }),
            2 => Ok(VolumeMount {
                host: Some(FromStr::from_str(items[0])?),
                container: items[1].to_owned(),
                mode: Default::default(),
                _hidden: (),
            }),
            3 => Ok(VolumeMount {
                host: Some(FromStr::from_str(items[0])?),
                container: items[1].to_owned(),
                mode: FromStr::from_str(items[2])?,
                _hidden: (),
            }),
            _ => Err(Error::invalid_value("volume", s)),
        }
    }
}

#[test]
fn portable_volume_mounts_should_have_string_representations() {
    let vol1 = VolumeMount::anonymous("/var/lib");
    let vol2 = VolumeMount::named("named", "/var/lib");

    let pairs = vec![(vol1, "/var/lib"), (vol2, "named:/var/lib")];
    for (mode, s) in pairs {
        assert_eq!(mode.to_string(), s);
        assert_eq!(mode, VolumeMount::from_str(s).unwrap());
    }
}

#[test]
#[cfg(not(windows))]
fn unix_windows_volume_mounts_should_have_string_representations() {
    let vol3 = VolumeMount {
        mode: VolumeModes::ReadOnly,
        ..VolumeMount::host("/etc/foo", "/etc/myfoo")
    };

    let pairs = vec![(vol3, "/etc/foo:/etc/myfoo:ro")];
    for (mode, s) in pairs {
        assert_eq!(mode.to_string(), s);
        assert_eq!(mode, VolumeMount::from_str(s).unwrap());
    }
}

#[test]
#[cfg(windows)]
fn windows_volume_mounts_should_have_string_representations() {
    let vol3 = VolumeMount {
        mode: VolumeModes::ReadOnly,
        ..VolumeMount::host("c:\\home\\smith\\foo", "/etc/myfoo")
    };
    let vol4 = VolumeMount::host(".\\foo", "/etc/myfoo");

    let pairs = vec![
        (vol3, "/c/home/smith/foo:/etc/myfoo:ro"),
        (vol4, "./foo:/etc/myfoo"),
    ];
    for (mode, s) in pairs {
        assert_eq!(mode.to_string(), s);
        assert_eq!(mode, VolumeMount::from_str(s).unwrap());
    }
}
