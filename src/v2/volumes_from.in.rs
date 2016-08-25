// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// The name of either a service or a container.
#[derive(Debug, PartialEq, Eq)]
pub enum ServiceOrContainer {
    // TODO: Lots of the mode_enum stuff has these two cases built-in.  Can
    // we re-use this there?

    /// The local name of a service defined in this `docker-compose.yml`
    /// file.
    Service(String),
    /// The global name of a container running under Docker.
    Container(String),
}

/// Mount the volumes defined by another container into this one.
#[derive(Debug, PartialEq, Eq)]
pub struct VolumesFrom {
    /// Where do we get these volumes from?
    pub source: ServiceOrContainer,
    /// What permissions should we apply to these volumes?
    pub permissions: VolumePermissions,
}

impl fmt::Display for VolumesFrom {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // We serialize service names without the `service:` here, but most
        // other places include the label.
        match &self.source {
            &ServiceOrContainer::Service(ref name) =>
                try!(write!(f, "{}", name)),
            &ServiceOrContainer::Container(ref name) =>
                try!(write!(f, "container:{}", name)),
        }
        if self.permissions != Default::default() {
            try!(write!(f, ":{}", self.permissions))
        }
        Ok(())
    }
}

impl_serialize_to_string!(VolumesFrom);

impl FromStr for VolumesFrom {
    type Err = InvalidValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref FROM: Regex =
                Regex::new("^(container:)?([^:]+)(?::([^:]+))?$").unwrap();
        }
        let caps = try!(FROM.captures(s).ok_or_else(|| {
            InvalidValueError::new("volumes_from", s)
        }));

        let name = caps.at(2).unwrap().to_owned();
        let source =
            if caps.at(1).is_some() {
                ServiceOrContainer::Container(name)
            } else {
                ServiceOrContainer::Service(name)
            };
        let permissions =
            match caps.at(3) {
                None => Default::default(),
                Some(permstr) => try!(FromStr::from_str(permstr)),
            };
        Ok(VolumesFrom {
            source: source,
            permissions: permissions,
        })
    }
}

impl_deserialize_from_str!(VolumesFrom);

#[test]
fn volumes_from_should_have_a_string_representation() {
    let vf1 = VolumesFrom {
        source: ServiceOrContainer::Service("foo".to_owned()),
        permissions: Default::default(),
    };
    let vf2 = VolumesFrom {
        source: ServiceOrContainer::Container("foo".to_owned()),
        permissions: VolumePermissions::ReadOnly,
    };

    let pairs = vec!(
        (vf1, "foo"),
        (vf2, "container:foo:ro"),
    );
    for (vf, s) in pairs {
        assert_eq!(vf.to_string(), s);
        assert_eq!(vf, VolumesFrom::from_str(s).unwrap());
    }
}
