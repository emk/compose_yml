// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// A service which will be managed by `docker-compose`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Network {
    /// The name of the network driver to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<RawOr<String>>,

    /// Options to pass to the network driver.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub driver_opts: BTreeMap<String, RawOr<String>>,

    /// Mutually-exclusive with all other options.
    ///
    /// TODO LOW: We could represent `Network` and `ExternalNetwork` as
    /// some kind of enum, but that might break in the future if things get
    /// more complicated.  For now, we're sticking close to the file
    /// format even if it makes things a bit less idiomatic in Rust.
    #[serde(default, skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_opt_true_or_struct",
            deserialize_with = "deserialize_opt_true_or_struct")]
    pub external: Option<ExternalNetwork>,

    // TODO LOW: ipam

    /// PRIVATE.  Mark this struct as having unknown fields for future
    /// compatibility.  This prevents direct construction and exhaustive
    /// matching.  This needs to be be public because of
    /// http://stackoverflow.com/q/39277157/12089
    #[doc(hidden)]
    #[serde(default, skip_serializing, skip_deserializing)]
    pub _hidden: (),
}

derive_standard_impls_for!(Network, {
    driver, driver_opts, external, _hidden
});

#[test]
fn network_handles_driver_correctly() {
    let yaml = r#"---
"driver": "default"
"#;
    assert_roundtrip!(Network, yaml);
}

#[test]
fn network_handles_external_true_correctly() {
    let yaml = r#"---
"external": true
"#;
    assert_roundtrip!(Network, yaml);
}

#[test]
fn network_handles_external_name_correctly() {
    let yaml = r#"---
"external":
  "name": "bridge"
"#;
    assert_roundtrip!(Network, yaml);
}
