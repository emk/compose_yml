// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// Where can we find the volume we want to map into a container?
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Volume {
    /// The name of the Docker volume driver to use.  Defaults to
    /// `"local"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub driver: Option<RawOr<String>>,

    /// Key-value options to pass to the volume driver.
    ///
    /// TODO LOW: We probably shouldn't allow the "list" variant here, but
    /// we're lazy and we want the reset of the machinery provided by
    /// deserialize_map_or_key_value_list.
    ///
    /// TODO LOW: Clear on merge if `driver` changes, like we do for
    /// `Logging` options.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty",
            deserialize_with = "deserialize_map_or_key_value_list")]
    pub driver_opts: BTreeMap<String, RawOr<String>>,

    /// If this is true, then the volume was created outside of
    /// `docker-compose`.  This option is mutually exclusive with the
    /// `driver` options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external: Option<bool>,

    /// Docker labels for this volume, specifying various sorts of
    /// custom metadata.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty",
            deserialize_with = "deserialize_map_or_key_optional_value_list")]
    pub labels: BTreeMap<String, Option<RawOr<String>>>,

    /// PRIVATE.  Mark this struct as having unknown fields for future
    /// compatibility.  This prevents direct construction and exhaustive
    /// matching.  This needs to be be public because of
    /// http://stackoverflow.com/q/39277157/12089
    #[doc(hidden)]
    #[serde(default, skip_serializing, skip_deserializing)]
    pub _hidden: (),
}

derive_standard_impls_for!(Volume, {
    driver, driver_opts, external, labels, _hidden
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
driver: sample
driver_opts:
  file_share: myshare
labels:
  com.example: foo
"#;
    assert_roundtrip!(Volume, yaml);
}

#[test]
fn external_volume_can_be_converted_from_and_to_yaml() {
    let yaml = r#"---
external: true
"#;
    assert_roundtrip!(Volume, yaml);
}
