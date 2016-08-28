// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// A host mapping to add to `/etc/hosts`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostMapping {
    pub hostname: String,
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

impl fmt::Display for HostMapping {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}:{}", &self.hostname, &self.address)
    }
}

impl_serialize_to_string!(HostMapping);

impl FromStr for HostMapping {
    type Err = InvalidValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref HOST_ADDRESS: Regex =
                Regex::new("^([^:]+):(.+)$").unwrap();
        }
        let caps = try!(HOST_ADDRESS.captures(s).ok_or_else(|| {
            InvalidValueError::new("host mapping", s)
        }));
        let addr: IpAddr =
            try!(FromStr::from_str(caps.at(2).unwrap()).map_err(|_| {
                InvalidValueError::new("IP address", s)
            }));
        Ok(HostMapping::new(caps.at(1).unwrap(), &addr))
    }
}

impl_deserialize_from_str!(HostMapping);

#[test]
fn host_mapping_supports_string_serialization() {
    let localhost: IpAddr = FromStr::from_str("127.0.0.1").unwrap();
    assert_eq!(HostMapping::new("foo.example.com", &localhost),
               HostMapping::from_str("foo.example.com:127.0.0.1").unwrap());
    assert_eq!(HostMapping::new("foo.example.com", &localhost).to_string(),
               "foo.example.com:127.0.0.1");
}
