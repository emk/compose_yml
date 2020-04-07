// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// Permissions on devices that are mapped into the Docker container.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct DevicePermissions {
    /// Can the container read from this device?
    pub read: bool,
    /// Can the container write to this device?
    pub write: bool,
    /// Can the container call `mknod` for this device?
    pub mknod: bool,
}

impl Default for DevicePermissions {
    fn default() -> DevicePermissions {
        DevicePermissions {
            read: true,
            write: true,
            mknod: true,
        }
    }
}

impl fmt::Display for DevicePermissions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.read {
            write!(f, "r")?
        }
        if self.write {
            write!(f, "w")?
        }
        if self.mknod {
            write!(f, "m")?
        }
        Ok(())
    }
}

impl FromStr for DevicePermissions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            static ref PERMS: Regex =
                Regex::new("^(r)?(w)?(m)?").unwrap();
        }
        let caps = PERMS.captures(s).ok_or_else(|| {
            Error::invalid_value("restart-mode", s)
        })?;
        Ok(DevicePermissions {
            read: caps.get(1).is_some(),
            write: caps.get(2).is_some(),
            mknod: caps.get(3).is_some(),
        })
    }
}

#[test]
fn device_permissions_has_a_string_representation() {
    let pairs = vec!(
        (Default::default(), "rwm"),
        (DevicePermissions { read: false, ..Default::default() }, "wm"),
        (DevicePermissions { write: false, ..Default::default() }, "rm"),
        (DevicePermissions { mknod: false, ..Default::default() }, "rw"),
    );
    for (mode, s) in pairs {
        assert_eq!(mode.to_string(), s);
        assert_eq!(mode, DevicePermissions::from_str(s).unwrap());
    }
}

/// Permissions on volumes that are mapped into the Docker container.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum VolumePermissions {
    /// This volume can be read and written (default).
    ReadWrite,
    /// This volume is ready-only.
    ReadOnly,
}

impl Default for VolumePermissions {
    fn default() -> VolumePermissions {
        VolumePermissions::ReadWrite
    }
}

impl fmt::Display for VolumePermissions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &VolumePermissions::ReadWrite => write!(f, "rw"),
            &VolumePermissions::ReadOnly => write!(f, "ro"),
        }
    }
}

impl FromStr for VolumePermissions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "rw" => Ok(VolumePermissions::ReadWrite),
            "ro" => Ok(VolumePermissions::ReadOnly),
            _ => Err(Error::invalid_value("volume permissions", s)),
        }
    }
}

#[test]
fn volume_permissions_has_a_string_representation() {
    let pairs = vec!(
        (VolumePermissions::ReadWrite, "rw"),
        (VolumePermissions::ReadOnly, "ro"),
    );
    for (mode, s) in pairs {
        assert_eq!(mode.to_string(), s);
        assert_eq!(mode, VolumePermissions::from_str(s).unwrap());
    }
}
