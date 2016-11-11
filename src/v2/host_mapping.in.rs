// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// A host mapping to add to `/etc/hosts`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostMapping {
    /// The hostname to add to `/etc/hosts`.
    pub hostname: String,
    /// The IPv4 or IPv6 address to map it to.
    pub address: IpAddr,
}

impl HostMapping {
    /// Create a new mapping from `hostname` to `address`.
    pub fn new(hostname: &str, address: &IpAddr) -> HostMapping {
        HostMapping {
            hostname: hostname.to_owned(),
            address: address.to_owned(),
        }
    }
}

impl_interpolatable_value!(HostMapping);

impl fmt::Display for HostMapping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", &self.hostname, &self.address)
    }
}

impl FromStr for HostMapping {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            static ref HOST_ADDRESS: Regex =
                Regex::new("^([^:]+):(.+)$").unwrap();
        }
        let caps = HOST_ADDRESS.captures(s).ok_or_else(|| {
            Error::invalid_value("host mapping", s)
        })?;
        let addr: IpAddr =
            FromStr::from_str(caps.at(2).unwrap()).map_err(|_| {
                Error::invalid_value("IP address", s)
            })?;
        Ok(HostMapping::new(caps.at(1).unwrap(), &addr))
    }
}

#[test]
fn host_mapping_supports_string_serialization() {
    let localhost: IpAddr = FromStr::from_str("127.0.0.1").unwrap();
    assert_eq!(HostMapping::new("foo.example.com", &localhost),
               HostMapping::from_str("foo.example.com:127.0.0.1").unwrap());
    assert_eq!(HostMapping::new("foo.example.com", &localhost).to_string(),
               "foo.example.com:127.0.0.1");
}
