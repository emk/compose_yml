// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// A host mapping to add to `/etc/hosts`.
#[derive(Debug, PartialEq, Eq)]
pub struct HostMapping {
    // TODO: Export reader functions?
    hostname: String,
    address: String,
}

impl HostMapping {
    /// Create a new mapping from `hostname` to `address`.
    pub fn new(hostname: &str, address: &str) -> HostMapping {
        // TODO: Check syntax of fields?
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
        Ok(HostMapping::new(caps.at(1).unwrap(), caps.at(2).unwrap()))
    }
}

impl_deserialize_from_str!(HostMapping);

#[test]
fn host_mapping_supports_string_serialization() {
    assert_eq!(HostMapping::new("foo.example.com", "127.0.0.1"),
               HostMapping::from_str("foo.example.com:127.0.0.1").unwrap());
    assert_eq!(HostMapping::new("foo.example.com", "127.0.0.1").to_string(),
               "foo.example.com:127.0.0.1");
}
