// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// The name of an external resource, and an optional local alias to which
/// it is mapped inside a container.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AliasedName {
    /// The name of the external resouce outside the container.
    name: String,

    /// An optional alias for the external resource inside the container.
    /// If not present, the external name should be used.
    alias: Option<String>,
}

impl AliasedName {
    /// Create a new AliasedName from a name and option alias.
    pub fn new(name: &str, alias: Option<&str>) -> Result<AliasedName>
    {
        let result = AliasedName {
            name: name.to_owned(),
            alias: alias.map(|v| v.to_owned()),
        };
        result.validate()?;
        Ok(result)
    }

    /// (Internal.) Validate an aliased name is safely serializeable.
    fn validate(&self) -> Result<()> {
        let bad_name = self.name.contains(":");
        let bad_alias = self.alias.as_ref()
            .map(|a| a.contains(":")).unwrap_or(false);
        if bad_name || bad_alias {
            let val = format!("{:?}", &self);
            return Err(Error::invalid_value("aliased name", val));
        }
        Ok(())
    }
}

impl_interpolatable_value!(AliasedName);

impl fmt::Display for AliasedName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.alias {
            &Some(ref alias) => write!(f, "{}:{}", &self.name, alias),
            &None => write!(f, "{}", &self.name),
        }
    }
}

impl FromStr for AliasedName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            static ref ALIASED_NAME: Regex =
                Regex::new("^([^:]+)(?::([^:]+))?$").unwrap();
        }
        let caps = ALIASED_NAME.captures(s).ok_or_else(|| {
            Error::invalid_value("aliased name", s)
        })?;
        Ok(AliasedName {
            name: caps.get(1).unwrap().as_str().to_owned(),
            alias: caps.get(2).map(|v| v.as_str().to_owned()),
        })
    }
}

#[test]
fn aliased_name_can_be_converted_to_and_from_a_string() {
    assert_eq!(AliasedName::from_str("foo").unwrap(),
               AliasedName { name: "foo".to_owned(), alias: None });
    assert_eq!(AliasedName::from_str("foo:bar").unwrap(),
               AliasedName { name: "foo".to_owned(),
                             alias: Some("bar".to_owned()) });
    assert!(AliasedName::from_str("foo:bar:baz").is_err());

    assert_eq!(AliasedName::new("foo", None).unwrap().to_string(),
               "foo");
    assert_eq!(AliasedName::new("foo", Some("bar")).unwrap().to_string(),
               "foo:bar");
}
