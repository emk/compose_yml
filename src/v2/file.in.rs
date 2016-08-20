// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// A `docker-compose.yml` file.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct File {
    /// The individual services which make up this app.
    pub services: BTreeMap<String, Service>,
}

#[test]
fn file_can_be_converted_from_and_to_yaml() {
    let yaml = r#"---
"services":
  "foo":
    "build": "."
"#;

    assert_roundtrip!(File, yaml);

    let file: File = serde_yaml::from_str(&yaml).unwrap();
    let foo = file.services.get("foo").unwrap();
    assert_eq!(foo.build.as_ref().unwrap().context, Context::new("."));
}
