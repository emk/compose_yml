// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// How should we configure the container's networking?
#[derive(Debug, PartialEq, Eq)]
enum NetworkMode {
    /// Use the standard Docker networking bridge.
    Bridge,
    /// Use the host's network interface directly.
    Host,
    /// Disable networking in the container.
    None,
    /// Use the networking namespace associated with the named service.
    Service(String),
    /// Use the networking namespace associated with the named container.
    Container(String),
}

impl fmt::Display for NetworkMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &NetworkMode::Bridge => write!(f, "bridge"),
            &NetworkMode::Host => write!(f, "host"),
            &NetworkMode::None => write!(f, "none"),
            &NetworkMode::Service(ref name) => write!(f, "service:{}", name),
            &NetworkMode::Container(ref name) => write!(f, "container:{}", name),
        }
    }
}

impl_serialize_to_string!(NetworkMode);

impl FromStr for NetworkMode {
    type Err = InvalidValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref COMPOUND: Regex =
                Regex::new("^(service|container):(.*)$").unwrap();
        }

        match s {
            "bridge" => Ok(NetworkMode::Bridge),
            "host" => Ok(NetworkMode::Host),
            "none" => Ok(NetworkMode::None),
            _ => {
                let caps = try!(COMPOUND.captures(s).ok_or_else(|| {
                    InvalidValueError::new("network mode", s)
                }));
                let name = caps.at(2).unwrap().to_owned();
                match caps.at(1).unwrap() {
                    "service" => Ok(NetworkMode::Service(name)),
                    "container" => Ok(NetworkMode::Container(name)),
                    _ => unreachable!(),
                }
            }
        }
    }
}

impl_deserialize_from_str!(NetworkMode);

#[test]
fn network_mode_has_a_string_representation() {
    let pairs = vec!(
        (NetworkMode::Bridge, "bridge"),
        (NetworkMode::Host, "host"),
        (NetworkMode::None, "none"),
        (NetworkMode::Service("foo".to_owned()), "service:foo"),
        (NetworkMode::Container("foo".to_owned()), "container:foo"),
    );
    for (mode, s) in pairs {
        assert_eq!(mode.to_string(), s);
        assert_eq!(mode, NetworkMode::from_str(s).unwrap());
    }
}
