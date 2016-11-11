// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// The name of either a service or a container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceOrContainer {
    // TODO MED: Lots of the mode_enum stuff has these two cases built-in.
    // Can we re-use this there?

    /// The local name of a service defined in this `docker-compose.yml`
    /// file.
    Service(String),
    /// The global name of a container running under Docker.
    Container(String),
}

/// Mount the volumes defined by another container into this one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumesFrom {
    /// Where do we get these volumes from?
    pub source: ServiceOrContainer,
    /// What permissions should we apply to these volumes?
    pub permissions: VolumePermissions,
    /// PRIVATE.  Mark this struct as having unknown fields for future
    /// compatibility.  This prevents direct construction and exhaustive
    /// matching.  This needs to be be public because of
    /// http://stackoverflow.com/q/39277157/12089
    #[doc(hidden)]
    pub _hidden: (),
}

impl VolumesFrom {
    /// Construct a `VolumesFrom` object using the name of a service in
    /// this `docker-compose.yml` file.
    ///
    /// ```
    /// use compose_yml::v2 as dc;
    /// let vf = dc::VolumesFrom::service("myservice");
    /// assert_eq!(vf.source,
    ///            dc::ServiceOrContainer::Service("myservice".to_owned()));
    ///
    /// // To override a field, try:
    /// dc::VolumesFrom {
    ///   permissions: dc::VolumePermissions::ReadOnly,
    ///   ..dc::VolumesFrom::service("myservice")
    /// };
    /// ```
    pub fn service<S: Into<String>>(service: S) -> VolumesFrom {
        VolumesFrom {
            source: ServiceOrContainer::Service(service.into()),
            permissions: Default::default(),
            _hidden: (),
        }
    }

    /// Construct a `VolumesFrom` object using the name of a Docker
    /// container defined elsewhere.
    ///
    /// ```
    /// use compose_yml::v2 as dc;
    /// let vf = dc::VolumesFrom::container("mycontainer");
    /// assert_eq!(vf.source,
    ///            dc::ServiceOrContainer::Container("mycontainer".to_owned()));
    /// ```
    pub fn container<S: Into<String>>(container: S) -> VolumesFrom {
        VolumesFrom {
            source: ServiceOrContainer::Container(container.into()),
            permissions: Default::default(),
            _hidden: (),
        }
    }
}

impl_interpolatable_value!(VolumesFrom);

impl fmt::Display for VolumesFrom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // We serialize service names without the `service:` here, but most
        // other places include the label.
        match &self.source {
            &ServiceOrContainer::Service(ref name) =>
                write!(f, "{}", name)?,
            &ServiceOrContainer::Container(ref name) =>
                write!(f, "container:{}", name)?,
        }
        if self.permissions != Default::default() {
            write!(f, ":{}", self.permissions)?
        }
        Ok(())
    }
}

impl FromStr for VolumesFrom {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            static ref FROM: Regex =
                Regex::new("^(container:)?([^:]+)(?::([^:]+))?$").unwrap();
        }
        let caps = FROM.captures(s).ok_or_else(|| {
            Error::invalid_value("volumes_from", s)
        })?;

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
                Some(permstr) => FromStr::from_str(permstr)?,
            };
        Ok(VolumesFrom {
            source: source,
            permissions: permissions,
            _hidden: (),
        })
    }
}

#[test]
fn volumes_from_should_have_a_string_representation() {
    let vf1 = VolumesFrom::service("foo");
    let vf2 = VolumesFrom {
        permissions: VolumePermissions::ReadOnly,
        ..VolumesFrom::container("foo")
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
