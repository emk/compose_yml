// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// Where can we find the volume we want to map into a container?
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Volume {
    /// The name of the Docker volume driver to use.  Defaults to
    /// `"local"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    driver: Option<RawOr<String>>,

    /// Key-value options to pass to the volume driver.
    ///
    /// TODO MED: Allow non-string keys.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    driver_opts: BTreeMap<String, RawOr<String>>,

    /// If this is true, then the volume was created outside of
    /// `docker-compose`.  This option is mutually exclusive with the
    /// `driver` options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    external: Option<bool>,
}

derive_standard_impls_for!(Volume, {
    driver, driver_opts, external
});


#[test]
fn empty_volume_can_be_converted_from_and_to_yaml() {
    let yaml = r#"---
{}"#;
    assert_roundtrip!(Volume, yaml);
}

#[test]
fn volume_with_driver_can_be_converted_from_and_to_yaml() {
    let yaml = r#"---
"driver": "sample"
"driver_opts":
  "file_share": "myshare"
"#;
    assert_roundtrip!(Volume, yaml);
}

#[test]
fn external_volume_can_be_converted_from_and_to_yaml() {
    let yaml = r#"---
"external": true
"#;
    assert_roundtrip!(Volume, yaml);
}
