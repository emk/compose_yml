use super::common::*;

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
            VolumePermissions::ReadWrite => write!(f, "rw"),
            VolumePermissions::ReadOnly => write!(f, "ro"),
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
    let pairs = vec![
        (VolumePermissions::ReadWrite, "rw"),
        (VolumePermissions::ReadOnly, "ro"),
    ];
    for (mode, s) in pairs {
        assert_eq!(mode.to_string(), s);
        assert_eq!(mode, VolumePermissions::from_str(s).unwrap());
    }
}

