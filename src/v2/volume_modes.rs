use super::common::*;

/// Mount modes on volumes that are mapped into the Docker container.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum VolumeModes {
    /// This volume can be read and written (default).
    ReadWrite,
    /// This volume is read-only.
    ReadOnly,
    /// This volume and the host are perfectly synchronized.
    Consistent,
    /// Permit delays before updates on the host appear in the container.
    Cached,
    /// Permit delays before updates on the container appear in the host.
    Delegated,
}

impl Default for VolumeModes {
    fn default() -> VolumeModes {
        VolumeModes::ReadWrite
    }
}

impl fmt::Display for VolumeModes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VolumeModes::ReadWrite => write!(f, "rw"),
            VolumeModes::ReadOnly => write!(f, "ro"),
            VolumeModes::Consistent => write!(f, "consistent"),
            VolumeModes::Cached => write!(f, "cached"),
            VolumeModes::Delegated => write!(f, "delegated"),
        }
    }
}

impl FromStr for VolumeModes {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "rw" => Ok(VolumeModes::ReadWrite),
            "ro" => Ok(VolumeModes::ReadOnly),
            "consistent" => Ok(VolumeModes::Consistent),
            "cached" => Ok(VolumeModes::Cached),
            "delegated" => Ok(VolumeModes::Delegated),
            _ => Err(Error::invalid_value("volume mode", s)),
        }
    }
}

#[test]
fn volume_mode_has_a_string_representation() {
    let pairs = vec![
        (VolumeModes::ReadWrite, "rw"),
        (VolumeModes::ReadOnly, "ro"),
        (VolumeModes::Consistent, "consistent"),
        (VolumeModes::Cached, "cached"),
        (VolumeModes::Delegated, "delegated"),
    ];
    for (mode, s) in pairs {
        assert_eq!(mode.to_string(), s);
        assert_eq!(mode, VolumeModes::from_str(s).unwrap());
    }
}
