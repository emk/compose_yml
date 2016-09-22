// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// Either a port, or a range of ports.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Ports {
    /// A single port.
    Port(u16),
    /// A range of ports.
    Range(u16, u16),
}

impl From<u16> for Ports {
    /// Convert a raw port number into a `Ports` object.  This is used to
    /// make the `PortMapping` constructors more ergonomic by automatically
    /// promoting a `u16` port number to a `Ports` object in the most
    /// common case.
    fn from(port: u16) -> Ports {
        Ports::Port(port)
    }
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
#[allow(missing_copy_implementations)]
pub struct PortMapping {
    /// An optional host address on which to listen.  Defaults to all host
    /// addresses.  If this field is specified, then `host_ports` must also
    /// be specified.
    pub host_address: Option<IpAddr>,
    /// The host port(s) on which to listen.  Must contain the same number
    /// of ports as `container_ports`.  Defaults to an
    /// automatically-assigned port number.
    pub host_ports: Option<Ports>,
    /// The container port(s) to export.
    pub container_ports: Ports,

    /// PRIVATE.  Mark this struct as having unknown fields for future
    /// compatibility.  This prevents direct construction and exhaustive
    /// matching.  This needs to be be public because of
    /// http://stackoverflow.com/q/39277157/12089
    #[doc(hidden)]
    pub _phantom: PhantomData<()>,
}

impl PortMapping {
    /// Map a specified host port to a container port.  Can also be used to
    /// map port ranges.
    ///
    /// ```
    /// use docker_compose::v2 as dc;
    ///
    /// let mapping = dc::PortMapping::new(80, 3000);
    /// assert_eq!(mapping.host_address, None);
    /// assert_eq!(mapping.host_ports, Some(dc::Ports::Port(80)));
    /// assert_eq!(mapping.container_ports, dc::Ports::Port(3000));
    ///
    /// dc::PortMapping::new(dc::Ports::Range(8080, 8089),
    ///                      dc::Ports::Range(3000, 3009));
    /// ```
    pub fn new<P1, P2>(host_ports: P1, container_ports: P2) -> PortMapping
        where P1: Into<Ports>, P2: Into<Ports>
    {
        PortMapping {
            host_address: Default::default(),
            host_ports: Some(host_ports.into()),
            container_ports: container_ports.into(),
            _phantom: PhantomData,
        }
    }

    /// Allocate a host port and map it to the specified container port.
    /// Can also be used with a port range.
    ///
    /// ```
    /// use docker_compose::v2 as dc;
    ///
    /// let mapping = dc::PortMapping::any_to(3000);
    /// assert_eq!(mapping.host_address, None);
    /// assert_eq!(mapping.host_ports, None);
    /// assert_eq!(mapping.container_ports, dc::Ports::Port(3000));
    /// ```
    pub fn any_to<P>(container_ports: P) -> PortMapping
        where P: Into<Ports>
    {
        PortMapping {
            host_address: Default::default(),
            host_ports: None,
            container_ports: container_ports.into(),
            _phantom: PhantomData,
        }
    }
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
                    _phantom: PhantomData,
                })
            }
            2 => {
                Ok(PortMapping {
                    host_address: None,
                    host_ports: Some(try!(FromStr::from_str(fields[1]))),
                    container_ports: try!(FromStr::from_str(fields[0])),
                    _phantom: PhantomData,
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
                    _phantom: PhantomData,
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

    let map1 = PortMapping::any_to(80);
    let map2 = PortMapping::new(Ports::Range(8080, 8089),
                                Ports::Range(3000, 3009));
    let map3 = PortMapping {
        host_address: Some(localhost),
        ..PortMapping::new(80, 80)
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
