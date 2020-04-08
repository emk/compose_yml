use super::common::*;

/// Specify another service which should be used as the base for this
/// service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Extends {
    /// The name of a service to extend.
    pub service: RawOr<String>,

    /// The file in which the service to extend is defined.  Defaults to
    /// the current file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<RawOr<PathBuf>>,

    /// PRIVATE.  Mark this struct as having unknown fields for future
    /// compatibility.  This prevents direct construction and exhaustive
    /// matching.  This needs to be be public because of
    /// http://stackoverflow.com/q/39277157/12089
    #[doc(hidden)]
    #[serde(default, skip_serializing, skip_deserializing)]
    pub _hidden: (),
}

derive_standard_impls_for!(Extends, {
    service, file, _hidden
});

impl Extends {
    /// Create a new `Extends` by specifying the service name.
    ///
    /// ```
    /// use compose_yml::v2 as dc;
    /// dc::Extends::new("webdefaults");
    /// ```
    pub fn new<S: Into<String>>(service: S) -> Extends {
        Extends {
            service: value(service.into()),
            file: Default::default(),
            _hidden: (),
        }
    }
}

#[test]
fn extends_can_be_roundtripped() {
    let yaml = r#"---
file: "bar/docker-compose.yml"
service: foo
"#;
    assert_roundtrip!(Extends, yaml);
}
