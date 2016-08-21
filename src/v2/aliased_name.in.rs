// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// The name of an external resource, and an optional local alias to which
/// it is mapped inside a container.
#[derive(Debug, Eq, PartialEq)]
pub struct AliasedName {
    /// The name of the external resouce outside the container.
    pub name: String,

    /// An optional alias for the external resource inside the container.
    /// If not present, the external name should be used.
    pub alias: Option<String>,
}

impl AliasedName {
    /// Parse an aliased name from a string.
    pub fn from_str(s: &str) -> Result<AliasedName, ParseError> {
        lazy_static! {
            static ref ALIASED_NAME: Regex =
                Regex::new("^([^:]+)(?::([^:]+))?$").unwrap();
        }

        let caps = try!(ALIASED_NAME.captures(s).ok_or_else(|| {
            ParseError::new("aliased name", s)
        }));

        Ok(AliasedName {
            name: caps.at(1).unwrap().to_owned(),
            alias: caps.at(2).map(|v| v.to_owned()),
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
}
