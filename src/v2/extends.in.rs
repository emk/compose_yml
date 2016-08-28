// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// Specify another service which should be used as the base for this
/// service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Extends {
    /// The name of a service to extend.
    pub service: String,

    /// The file in which the service to extend is defined.  Defaults to
    /// the current file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,
}

#[test]
fn extends_can_be_roundtripped() {
    let yaml = r#"---
"file": "bar/docker-compose.yml"
"service": "foo"
"#;
    assert_roundtrip!(Extends, yaml);
}
