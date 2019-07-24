// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// A check that’s run to determine whether or not containers for this service are “healthy”.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Healthcheck {
    /// A command that’s run to determine whether or not containers for this service are “healthy”.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<CommandLine>,

    /// The health check will first run interval seconds after the container is started
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<RawOr<String>>,

    /// If a single run of the check takes longer than timeout seconds then the check is considered to have failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<RawOr<String>>,

    /// It takes retries consecutive failures of the health check for the container to be considered unhealthy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,

    /// start period provides initialization time for containers that need time to bootstrap.
    /// Probe failure during that period will not be counted towards the maximum number of retries.
    /// However, if a health check succeeds during the start period, the container is considered started and
    /// all consecutive failures will be counted towards the maximum number of retries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_period: Option<RawOr<String>>,

    /// To disable any default healthcheck set by the image, you can use disable: true. This is equivalent to specifying test: ["NONE"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable: Option<bool>,

    /// PRIVATE.  Mark this struct as having unknown fields for future
    /// compatibility.  This prevents direct construction and exhaustive
    /// matching.  This needs to be be public because of
    /// http://stackoverflow.com/q/39277157/12089
    #[doc(hidden)]
    #[serde(default, skip_serializing, skip_deserializing)]
    pub _hidden: (),
}

derive_standard_impls_for!(Healthcheck, {
    test, interval, timeout, retries, start_period, disable, _hidden
});

#[test]
fn healtchcheck_regular() {
    let yaml = r#"---
  test: ["CMD", "curl", "-f", "http://localhost"]
  interval: 1m30s
  timeout: 10s
  retries: 3
  start_period: 40s
"#;
    assert_roundtrip!(Healthcheck, yaml);
}

#[test]
fn healtchcheck_disable() {
    let yaml = r#"---
  disable: true
"#;
    assert_roundtrip!(Healthcheck, yaml);
}
