// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// Either a port, or a range of ports.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Ports {
    /// A single port.
    Port(u16),
    /// A range of ports.
    Range(u16, u16),
}

impl fmt::Display for Ports {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &Ports::Port(port) => write!(f, "{}", port),
            &Ports::Range(first, last) => write!(f, "{}-{}", first, last),
        }
    }
}

impl FromStr for Ports {
    type Err = InvalidValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref PORTS: Regex =
                Regex::new("^([0-9]+)(?:-([0-9]+))?$").unwrap();
        }
        let caps = try!(PORTS.captures(s).ok_or_else(|| {
            InvalidValueError::new("ports", s)
        }));

        // Convert a regex capture group to a string.  Only call if the
        // specified capture group is known to be valid.
        let port_from_str = |i: usize| -> Result<u16, InvalidValueError> {
            FromStr::from_str(caps.at(i).unwrap()).map_err(|_| {
                InvalidValueError::new("port", s)
            })
        };

        if caps.at(2).is_none() {
            Ok(Ports::Port(try!(port_from_str(1))))
        } else {
            Ok(Ports::Range(try!(port_from_str(1)), try!(port_from_str(2))))
        }
    }
}

/// Specify how to map container ports to the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortMapping {
    /// An optional host address on which to listen.  Defaults to all host
    /// addresses.  If this field is specified, then `host_ports` must also
    /// be specified.
    host_address: Option<IpAddr>,
    /// The host port(s) on which to listen.  Must contain the same number
    /// of ports as `container_ports`.  Defaults to an
    /// automatically-assigned port number.
    host_ports: Option<Ports>,
    /// The container port(s) to export.
    container_ports: Ports,
}

impl_interpolatable_value!(PortMapping);

impl fmt::Display for PortMapping {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // We can't serialize a host_address without host_ports.
        if self.host_address.is_some() && self.host_ports.is_none() {
            return Err(fmt::Error);
        }

        if let Some(ref addr) = self.host_address {
            try!(write!(f, "{}:", addr));
        }
        if let Some(ports) = self.host_ports {
            try!(write!(f, "{}:", ports));
        }
        write!(f, "{}", self.container_ports)
    }
}

impl FromStr for PortMapping {
    type Err = InvalidValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Split backwards from the end of the string, in case the first
        // address field is an IPv6 address with embedded colons.  Hey,
        // it's not specified _never_ to happen.  Note that `fields` will
        // be in reverse order.
        let fields: Vec<_> = s.rsplitn(3, ":").collect();
        match fields.len() {
            1 => {
                Ok(PortMapping {
                    host_address: None,
                    host_ports: None,
                    container_ports: try!(FromStr::from_str(fields[0])),
                })
            }
            2 => {
                Ok(PortMapping {
                    host_address: None,
                    host_ports: Some(try!(FromStr::from_str(fields[1]))),
                    container_ports: try!(FromStr::from_str(fields[0])),
                })
            }
            3 => {
                let addr: IpAddr =
                    try!(FromStr::from_str(fields[2]).map_err(|_| {
                        InvalidValueError::new("IP address", s)
                    }));
                Ok(PortMapping {
                    host_address: Some(addr),
                    host_ports: Some(try!(FromStr::from_str(fields[1]))),
                    container_ports: try!(FromStr::from_str(fields[0])),
                })
            }
            _ => {
                Err(InvalidValueError::new("port mapping", s))
            }
        }
    }
}

#[test]
fn port_mapping_should_have_a_string_representation() {
    let localhost: IpAddr = FromStr::from_str("127.0.0.1").unwrap();

    let map1 = PortMapping {
        host_address: None,
        host_ports: None,
        container_ports: Ports::Port(80),
    };
    let map2 = PortMapping {
        host_address: None,
        host_ports: Some(Ports::Range(8080, 8089)),
        container_ports: Ports::Range(3000, 3009),
    };
    let map3 = PortMapping {
        host_address: Some(localhost),
        host_ports: Some(Ports::Port(80)),
        container_ports: Ports::Port(80),
    };

    let pairs = vec!(
        (map1, "80"),
        (map2, "8080-8089:3000-3009"),
        (map3, "127.0.0.1:80:80"),
    );
    for (map, s) in pairs {
        assert_eq!(map.to_string(), s);
        assert_eq!(map, PortMapping::from_str(s).unwrap());
    }
}
