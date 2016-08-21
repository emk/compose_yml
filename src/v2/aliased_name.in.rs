// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// The name of an external resource, and an optional local alias to which
/// it is mapped inside a container.
///
/// TODO: Not sure I want these fields public; hiding them would simplify
/// validation and allow removing the error result from to_string.
#[derive(Debug, Eq, PartialEq)]
pub struct AliasedName {
    /// The name of the external resouce outside the container.
    pub name: String,

    /// An optional alias for the external resource inside the container.
    /// If not present, the external name should be used.
    pub alias: Option<String>,
}

impl AliasedName {
    /// Create a new AliasedName from a name and option alias.
    pub fn new(name: &str, alias: Option<&str>) ->
        Result<AliasedName, InvalidValueError>
    {
        let result = AliasedName {
            name: name.to_owned(),
            alias: alias.map(|v| v.to_owned()),
        };
        try!(result.validate());
        Ok(result)
    }

    /// (Internal.) Validate an aliased name is safely serializeable.
    fn validate(&self) -> Result<(), InvalidValueError> {
        let bad_name = self.name.contains(":");
        let bad_alias = self.alias.as_ref()
            .map(|a| a.contains(":")).unwrap_or(false);
        if bad_name || bad_alias {
            let val = format!("{:?}", &self);
            return Err(InvalidValueError::new("aliased name", &val));
        }
        Ok(())
    }

    /// Parse an aliased name from a string.
    pub fn from_str(s: &str) -> Result<AliasedName, InvalidValueError> {
        lazy_static! {
            static ref ALIASED_NAME: Regex =
                Regex::new("^([^:]+)(?::([^:]+))?$").unwrap();
        }
        let caps = try!(ALIASED_NAME.captures(s).ok_or_else(|| {
            InvalidValueError::new("aliased name", s)
        }));
        Ok(AliasedName {
            name: caps.at(1).unwrap().to_owned(),
            alias: caps.at(2).map(|v| v.to_owned()),
        })
    }

    /// Convert to a string.
    pub fn to_string(&self) -> Result<String, InvalidValueError> {
        try!(self.validate());
        match &self.alias {
            &Some(ref alias) => Ok(format!("{}:{}", &self.name, alias)),
            &None => Ok(self.name.to_owned()),
        }
    }
}

impl Serialize for AliasedName {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        match &self.to_string() {
            &Ok(ref s) => serializer.serialize_str(s),
            &Err(ref err) => Err(ser::Error::custom(format!("{}", err)))
        }
    }
}

impl Deserialize for AliasedName {
    fn deserialize<D>(deserializer: &mut D) -> Result<AliasedName, D::Error>
        where D: Deserializer
    {
        let string = try!(String::deserialize(deserializer));
        AliasedName::from_str(&string).map_err(|err| {
            de::Error::custom(format!("{}", err))
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

    assert_eq!(AliasedName::new("foo", None).unwrap().to_string().unwrap(),
               "foo");
    assert_eq!(AliasedName::new("foo", Some("bar")).unwrap().to_string().unwrap(),
               "foo:bar");
}
