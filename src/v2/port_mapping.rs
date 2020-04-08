use super::common::*;

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &Ports::Port(port) => write!(f, "{}", port),
            &Ports::Range(first, last) => write!(f, "{}-{}", first, last),
        }
    }
}

impl FromStr for Ports {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            static ref PORTS: Regex = Regex::new("^([0-9]+)(?:-([0-9]+))?$").unwrap();
        }
        let caps = PORTS
            .captures(s)
            .ok_or_else(|| Error::invalid_value("ports", s))?;

        // Convert a regex capture group to a string.  Only call if the
        // specified capture group is known to be valid.
        let port_from_str = |i: usize| -> Result<u16> {
            FromStr::from_str(caps.get(i).unwrap().as_str())
                .map_err(|_| Error::invalid_value("port", s))
        };

        if caps.get(2).is_none() {
            Ok(Ports::Port(port_from_str(1)?))
        } else {
            Ok(Ports::Range(port_from_str(1)?, port_from_str(2)?))
        }
    }
}

/// An IP protocol
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Protocol {
    /// Transmission Control Protocol, the default.
    Tcp,
    /// User Datagram Protocol.
    Udp,
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::Tcp
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &Protocol::Tcp => write!(f, "tcp"),
            &Protocol::Udp => write!(f, "udp"),
        }
    }
}

impl FromStr for Protocol {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "tcp" => Ok(Protocol::Tcp),
            "udp" => Ok(Protocol::Udp),
            _ => Err(Error::invalid_value("protocol", s)),
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
    /// The protocol to be used on the given port(s).
    pub protocol: Protocol,

    /// PRIVATE.  Mark this struct as having unknown fields for future
    /// compatibility.  This prevents direct construction and exhaustive
    /// matching.  This needs to be be public because of
    /// http://stackoverflow.com/q/39277157/12089
    #[doc(hidden)]
    pub _hidden: (),
}

impl PortMapping {
    /// Map a specified host port to a container port.  Can also be used to
    /// map port ranges.
    ///
    /// ```
    /// use compose_yml::v2 as dc;
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
    where
        P1: Into<Ports>,
        P2: Into<Ports>,
    {
        PortMapping {
            host_address: Default::default(),
            host_ports: Some(host_ports.into()),
            container_ports: container_ports.into(),
            protocol: Default::default(),
            _hidden: (),
        }
    }

    /// Allocate a host port and map it to the specified container port.
    /// Can also be used with a port range.
    ///
    /// ```
    /// use compose_yml::v2 as dc;
    ///
    /// let mapping = dc::PortMapping::any_to(3000);
    /// assert_eq!(mapping.host_address, None);
    /// assert_eq!(mapping.host_ports, None);
    /// assert_eq!(mapping.container_ports, dc::Ports::Port(3000));
    /// ```
    pub fn any_to<P>(container_ports: P) -> PortMapping
    where
        P: Into<Ports>,
    {
        PortMapping {
            host_address: Default::default(),
            host_ports: None,
            container_ports: container_ports.into(),
            protocol: Default::default(),
            _hidden: (),
        }
    }
}

impl_interpolatable_value!(PortMapping);

impl fmt::Display for PortMapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // We can't serialize a host_address without host_ports.
        if self.host_address.is_some() && self.host_ports.is_none() {
            return Err(fmt::Error);
        }

        if let Some(ref addr) = self.host_address {
            write!(f, "{}:", addr)?;
        }
        if let Some(ports) = self.host_ports {
            write!(f, "{}:", ports)?;
        }
        write!(f, "{}", self.container_ports)?;
        if self.protocol != Protocol::default() {
            write!(f, "/{}", self.protocol)?;
        }
        Ok(())
    }
}

fn consume_protocol(ports_and_protocol: &str) -> Result<(&str, Protocol)> {
    let fields: Vec<_> = ports_and_protocol.split("/").collect();
    match fields.len() {
        1 => Ok((fields[0], Protocol::Tcp)),
        2 => Ok((fields[0], Protocol::from_str(fields[1])?)),
        _ => Err(Error::invalid_value("port mapping", ports_and_protocol)),
    }
}

impl FromStr for PortMapping {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let (s_without_protocol, protocol) = consume_protocol(s)?;
        // Split backwards from the end of the string, in case the first
        // address field is an IPv6 address with embedded colons.  Hey,
        // it's not specified _never_ to happen.  Note that `fields` will
        // be in reverse order.
        let fields: Vec<_> = s_without_protocol.rsplitn(3, ":").collect();
        match fields.len() {
            1 => Ok(PortMapping {
                host_address: None,
                host_ports: None,
                container_ports: FromStr::from_str(fields[0])?,
                protocol,
                _hidden: (),
            }),
            2 => Ok(PortMapping {
                host_address: None,
                host_ports: Some(FromStr::from_str(fields[1])?),
                container_ports: FromStr::from_str(fields[0])?,
                protocol,
                _hidden: (),
            }),
            3 => {
                let addr: IpAddr = FromStr::from_str(fields[2])
                    .map_err(|_| Error::invalid_value("IP address", s))?;
                Ok(PortMapping {
                    host_address: Some(addr),
                    host_ports: Some(FromStr::from_str(fields[1])?),
                    container_ports: FromStr::from_str(fields[0])?,
                    protocol,
                    _hidden: (),
                })
            }
            _ => Err(Error::invalid_value("port mapping", s)),
        }
    }
}

#[test]
fn port_mapping_should_have_a_string_representation() {
    let localhost: IpAddr = FromStr::from_str("127.0.0.1").unwrap();

    let map1 = PortMapping::any_to(80);
    let map2 = PortMapping {
        protocol: Protocol::Udp,
        ..map1
    };
    let map3 = PortMapping::new(Ports::Range(8080, 8089), Ports::Range(3000, 3009));
    let map4 = PortMapping {
        protocol: Protocol::Udp,
        ..map3
    };
    let map5 = PortMapping {
        host_address: Some(localhost),
        ..PortMapping::new(80, 80)
    };
    let map6 = PortMapping {
        protocol: Protocol::Udp,
        ..map5
    };

    let pairs = vec![
        (map1, "80"),
        (map2, "80/udp"),
        (map3, "8080-8089:3000-3009"),
        (map4, "8080-8089:3000-3009/udp"),
        (map5, "127.0.0.1:80:80"),
        (map6, "127.0.0.1:80:80/udp"),
    ];
    for (map, s) in pairs {
        assert_eq!(map.to_string(), s);
        assert_eq!(map, PortMapping::from_str(s).unwrap());
    }
}

#[test]
fn port_mapping_can_be_parsed_from_a_string() {
    // These are just the strings that don't format the same way they parse
    let localhost: IpAddr = FromStr::from_str("127.0.0.1").unwrap();

    let map1 = PortMapping::any_to(80);
    let map2 = PortMapping::new(Ports::Range(8080, 8089), Ports::Range(3000, 3009));
    let map3 = PortMapping {
        host_address: Some(localhost),
        ..PortMapping::new(80, 80)
    };

    {
        let pairs = vec![
            (map1, "80/tcp"),
            (map2, "8080-8089:3000-3009/tcp"),
            (map3, "127.0.0.1:80:80/tcp"),
        ];
        for (map, s) in pairs {
            assert_eq!(map, PortMapping::from_str(s).unwrap());
        }
    }
}
