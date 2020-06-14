use super::common::*;

/// Mount modes on volumes that are mapped into the Docker container.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum VolumeModes {
    /// This volume can be read and written (default).
    ReadWrite,
    /// This volume is ready-only.
    ReadOnly,
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
        }
    }
}

impl FromStr for VolumeModes {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "rw" => Ok(VolumeModes::ReadWrite),
            "ro" => Ok(VolumeModes::ReadOnly),
            _ => Err(Error::invalid_value("volume mode", s)),
        }
    }
}

#[test]
fn volume_mode_has_a_string_representation() {
    let pairs = vec![
        (VolumeModes::ReadWrite, "rw"),
        (VolumeModes::ReadOnly, "ro"),
    ];
    for (mode, s) in pairs {
        assert_eq!(mode.to_string(), s);
        assert_eq!(mode, VolumeModes::from_str(s).unwrap());
    }
}

